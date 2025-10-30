use anyhow::{anyhow, Context};
use axum::{
    extract::{Multipart, Path, Query, State},
    http::StatusCode,
    response::Json,
};
use calamine::{open_workbook_auto, DataType, Reader};
use chrono::Utc;
use quick_xml::{events::Event as XmlEvent, Reader as XmlReader};
use reqwest::Client;
use serde_json::json;
use sha2::{Digest, Sha256};
use sqlx::{Postgres, QueryBuilder, Row};
use std::collections::hash_map::Entry;
use std::collections::HashMap;
use std::fs::File;
use std::io::Read;
use std::path::{Path as StdPath, PathBuf};
use std::sync::Arc;
use tokio::task;
use uuid::Uuid;
use zip::ZipArchive;

const DEFAULT_CATEGORY_NAME: &str = "General";
const DEFAULT_CATEGORY_DESCRIPTION: &str = "General business documents";
const AUTO_CATEGORY_DESCRIPTION: &str = "Auto-created category";

use crate::{
    dto::{
        ai::{GroqApiRequest, GroqApiResponse, GroqMessage},
        api::ApiResponse,
        rag::{
            DocumentListRequest, DocumentListResponse, DocumentProcessingStatus, DocumentSource,
            DocumentSummary, DocumentTextIngestRequest, DocumentTextIngestResponse,
            DocumentUploadResponse, RagAnswerRequest, RagAnswerResponse, RagConfig,
            RagQueryRequest, RagQueryResponse, UpdateRagConfigRequest,
        },
    },
    models::rag::{Document, DocumentProcessingJob, RagConfiguration},
    AppState,
};

// Upload document endpoint
pub async fn upload_document(
    State(data): State<Arc<AppState>>,
    mut multipart: Multipart,
) -> Result<Json<ApiResponse<DocumentUploadResponse>>, StatusCode> {
    let mut title = String::new();
    let mut description: Option<String> = None;
    let mut category = String::new();
    let mut tags: Vec<String> = Vec::new();
    let mut file_data: Option<Vec<u8>> = None;
    let mut file_name = String::new();
    let mut file_type = String::new();
    let mut mime_type = String::new();

    // Process multipart form data
    while let Some(field) = multipart
        .next_field()
        .await
        .map_err(|_| StatusCode::BAD_REQUEST)?
    {
        let name = field.name().unwrap_or("").to_string();

        match name.as_str() {
            "title" => {
                title = field.text().await.map_err(|_| StatusCode::BAD_REQUEST)?;
            }
            "description" => {
                let desc = field.text().await.map_err(|_| StatusCode::BAD_REQUEST)?;
                if !desc.is_empty() {
                    description = Some(desc);
                }
            }
            "category" => {
                category = field.text().await.map_err(|_| StatusCode::BAD_REQUEST)?;
            }
            "tags" => {
                let tags_str = field.text().await.map_err(|_| StatusCode::BAD_REQUEST)?;
                tags = tags_str.split(',').map(|s| s.trim().to_string()).collect();
            }
            "file" => {
                file_name = field.file_name().unwrap_or("unknown").to_string();
                mime_type = field
                    .content_type()
                    .unwrap_or("application/octet-stream")
                    .to_string();

                // Extract file extension
                if let Some(ext) = file_name.split('.').last() {
                    file_type = ext.to_lowercase();
                }

                file_data = Some(
                    field
                        .bytes()
                        .await
                        .map_err(|_| StatusCode::BAD_REQUEST)?
                        .to_vec(),
                );
            }
            _ => {}
        }
    }

    // Auto-infer title from file name when missing
    if title.trim().is_empty() {
        if file_name.is_empty() || file_name == "unknown" {
            title = "untitled".to_string();
        } else if let Some((stem, _)) = file_name.rsplit_once('.') {
            title = sanitize_filename(stem);
        } else {
            title = sanitize_filename(&file_name);
        }
    }

    // Auto-assign default category when missing
    // Improve robustness: infer file_type from mime_type if extension missing
    if file_type.trim().is_empty() && !mime_type.trim().is_empty() {
        let mt = mime_type.to_lowercase();
        if mt.contains("wordprocessingml.document") {
            file_type = "docx".to_string();
        } else if mt.contains("spreadsheetml.sheet") {
            file_type = "xlsx".to_string();
        } else if mt.contains("pdf") {
            file_type = "pdf".to_string();
        } else if mt.contains("csv") {
            file_type = "csv".to_string();
        } else if mt.starts_with("text/") {
            file_type = "txt".to_string();
        }
    }

    // Auto-tag with file type if no tags provided
    if tags.is_empty() && !file_type.is_empty() {
        tags.push(file_type.clone());
    }

    // Validate required file presence only
    if file_data.is_none() {
        return Err(StatusCode::BAD_REQUEST);
    }

    let file_bytes = file_data.unwrap();
    let file_size = file_bytes.len() as i64;

    // Normalize category against document_categories table
    let category = resolve_document_category(&data.db, &category).await?;

    // Get RAG configuration to validate file
    let config = get_rag_config(&data).await?;

    if !config.is_file_type_supported(&file_type) {
        return Err(StatusCode::UNSUPPORTED_MEDIA_TYPE);
    }

    if !config.is_file_size_allowed(file_size) {
        return Err(StatusCode::PAYLOAD_TOO_LARGE);
    }

    // Check duplicate by file_name (case-insensitive) for non-deleted documents
    if !file_name.is_empty() && file_name.to_lowercase() != "unknown" {
        let existing_opt = sqlx::query_as::<_, Document>(
            r#"
            SELECT *
            FROM documents
            WHERE lower(file_name) = lower($1)
              AND (status IS NULL OR status <> 'deleted')
              AND deleted_at = 0
            ORDER BY created_at DESC
            LIMIT 1
            "#,
        )
        .bind(&file_name)
        .fetch_optional(&data.db)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

        if let Some(existing_document) = existing_opt {
            let response = DocumentUploadResponse {
                id: existing_document.id,
                title: existing_document.title,
                description: existing_document.description,
                category: existing_document.category,
                tags: existing_document.tags,
                file_path: existing_document.file_path,
                file_size: existing_document.file_size,
                file_type: existing_document.file_type,
                status: existing_document.status,
                uploaded_at: existing_document.created_at,
            };

            return Ok(Json(ApiResponse {
                code: 200,
                status: "OK".to_string(),
                message: "File dengan nama yang sama sudah diupload".to_string(),
                data: response,
                errors: json!({}),
            }));
        }
    }

    // Generate unique file path
    let document_id = Uuid::new_v4();
    let file_path = format!("uploads/documents/{}/{}", document_id, file_name);

    // Save file to disk (in production, use cloud storage)
    if let Err(_) = tokio::fs::create_dir_all(format!("uploads/documents/{}", document_id)).await {
        return Err(StatusCode::INTERNAL_SERVER_ERROR);
    }

    if let Err(_) = tokio::fs::write(&file_path, &file_bytes).await {
        return Err(StatusCode::INTERNAL_SERVER_ERROR);
    }

    // Create document record
    let document = sqlx::query_as::<_, Document>(
        r#"
        INSERT INTO documents (
            id, title, description, category, tags, file_path, file_name, 
            file_size, file_type, mime_type, status, created_at, updated_at
        ) VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13)
        RETURNING *
        "#,
    )
    .bind(document_id)
    .bind(&title)
    .bind(&description)
    .bind(&category)
    .bind(&tags)
    .bind(&file_path)
    .bind(&file_name)
    .bind(file_size)
    .bind(&file_type)
    .bind(&mime_type)
    .bind("processing")
    .bind(Utc::now())
    .bind(Utc::now())
    .fetch_one(&data.db)
    .await
    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    // Create processing job entry so status can be tracked
    let job_row = sqlx::query(
        r#"
        INSERT INTO document_processing_jobs (
            document_id, job_type, status, priority, current_step, created_at, updated_at
        ) VALUES ($1, $2, $3, $4, $5, $6, $7)
        RETURNING id
        "#,
    )
    .bind(document_id)
    .bind("extract_text")
    .bind("pending")
    .bind(5)
    .bind("Queued for processing")
    .bind(Utc::now())
    .bind(Utc::now())
    .fetch_one(&data.db)
    .await
    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    let job_id: Uuid = job_row.get("id");

    let started_at = Utc::now();
    let _ = sqlx::query(
        r#"
        UPDATE document_processing_jobs
        SET status = 'processing',
            current_step = $1,
            started_at = $2,
            updated_at = $2
        WHERE id = $3
        "#,
    )
    .bind("Extracting content")
    .bind(started_at)
    .bind(job_id)
    .execute(&data.db)
    .await;

    let ingestion_result =
        process_uploaded_document(&data, document.id, &file_path, &file_type, &config).await;

    match ingestion_result {
        Ok(chunk_count) => {
            let completed_at = Utc::now();

            let _ = sqlx::query(
                r#"
                UPDATE documents
                SET status = 'ready',
                    chunk_count = $1,
                    processing_progress = 100,
                    current_processing_step = 'Completed',
                    error_message = NULL,
                    updated_at = $2
                WHERE id = $3
                "#,
            )
            .bind(chunk_count as i32)
            .bind(completed_at)
            .bind(document.id)
            .execute(&data.db)
            .await;

            let _ = sqlx::query(
                r#"
                UPDATE document_processing_jobs
                SET status = 'completed',
                    progress_percentage = 100,
                    current_step = 'Completed',
                    completed_at = $1,
                    updated_at = $1
                WHERE id = $2
                "#,
            )
            .bind(completed_at)
            .bind(job_id)
            .execute(&data.db)
            .await;
        }
        Err(err) => {
            let failure_time = Utc::now();
            let error_message = err.to_string();

            let _ = sqlx::query(
                r#"
                UPDATE documents
                SET status = 'error',
                    error_message = $1,
                    processing_progress = 0,
                    current_processing_step = 'Failed',
                    updated_at = $2
                WHERE id = $3
                "#,
            )
            .bind(&error_message)
            .bind(failure_time)
            .bind(document.id)
            .execute(&data.db)
            .await;

            let _ = sqlx::query(
                r#"
                UPDATE document_processing_jobs
                SET status = 'failed',
                    current_step = 'Failed',
                    error_message = $1,
                    updated_at = $2
                WHERE id = $3
                "#,
            )
            .bind(&error_message)
            .bind(failure_time)
            .bind(job_id)
            .execute(&data.db)
            .await;

            eprintln!(
                "Failed to ingest document {} ({}): {}",
                document.id, document.file_name, error_message
            );

            return Err(StatusCode::UNPROCESSABLE_ENTITY);
        }
    }

    let updated_document =
        sqlx::query_as::<_, Document>("SELECT * FROM documents WHERE id = $1 AND deleted_at = 0")
            .bind(document.id)
            .fetch_one(&data.db)
            .await
            .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    let response = DocumentUploadResponse {
        id: updated_document.id,
        title: updated_document.title,
        description: updated_document.description,
        category: updated_document.category,
        tags: updated_document.tags,
        file_path: updated_document.file_path,
        file_size: updated_document.file_size,
        file_type: updated_document.file_type,
        status: updated_document.status,
        uploaded_at: updated_document.created_at,
    };

    Ok(Json(ApiResponse {
        code: 201,
        status: "CREATED".to_string(),
        message: "Document uploaded successfully".to_string(),
        data: response,
        errors: json!({}),
    }))
}

// Query RAG endpoint
pub async fn query_rag(
    State(data): State<Arc<AppState>>,
    Json(request): Json<RagQueryRequest>,
) -> Result<Json<ApiResponse<RagQueryResponse>>, StatusCode> {
    let start_time = std::time::Instant::now();

    // Validate query
    if request.query.trim().is_empty() {
        return Err(StatusCode::BAD_REQUEST);
    }

    // Get RAG configuration
    let config = get_rag_config(&data).await?;

    let max_results = request.max_results.unwrap_or(config.max_results);
    let similarity_threshold = request
        .similarity_threshold
        .unwrap_or(config.similarity_threshold);

    // Compute query embedding for similarity (use normalized query to be robust to typos)
    let normalized_query_text = tokenize_and_normalize(&request.query).join(" ");
    let query_embedding = embed_texts_local(
        &vec![normalized_query_text.clone()],
        config.embedding_dimensions as usize,
    )
    .into_iter()
    .next()
    .unwrap_or_default();

    // Try Milvus search when available and not in mock mode; otherwise fallback later
    let milvus_client_opt = data.milvus_client.as_ref().cloned();
    let mut milvus_ranked: Vec<(i64, f32)> = Vec::new();
    if milvus_client_opt.is_some() && !data.env.allow_mock_dependencies {
        let milvus_client = milvus_client_opt.unwrap();
        let mut guard = milvus_client.lock().await;
        crate::services::milvus::ensure_rag_collection(&mut guard, &data.milvus_collection)
            .await
            .map_err(|err| {
                tracing::error!("Failed to ensure Milvus collection: {:?}", err);
                StatusCode::INTERNAL_SERVER_ERROR
            })?;

        let results = crate::services::milvus::search_top_k(
            &mut guard,
            &data.milvus_collection,
            &query_embedding,
            max_results as usize,
        )
        .await
        .map_err(|err| {
            tracing::error!("Milvus search_top_k failed: {:?}", err);
            StatusCode::INTERNAL_SERVER_ERROR
        })?;

        milvus_ranked = results;
    }

    let vector_filter_ids: Vec<i64> = milvus_ranked.iter().map(|(id, _)| *id).collect();

    #[derive(Clone)]
    struct ChunkRecord {
        chunk_id: Uuid,
        document_id: Uuid,
        document_title: String,
        content: String,
        chunk_index: i32,
        page_number: Option<i32>,
    }

    let mut chunk_map: HashMap<i64, ChunkRecord> = HashMap::new();

    if !vector_filter_ids.is_empty() {
        let mut query_builder = QueryBuilder::<Postgres>::new(
            "SELECT dc.id, dc.document_id, dc.chunk_index, dc.content, dc.page_number, \
             dc.milvus_vector_id, d.title as document_title \
             FROM document_chunks dc \
             JOIN documents d ON dc.document_id = d.id \
             WHERE d.status = 'ready' AND dc.deleted_at = 0 AND d.deleted_at = 0",
        );

        if let Some(category) = &request.category_filter {
            query_builder.push(" AND d.category = ");
            query_builder.push_bind(category);
        }

        if let Some(doc_ids) = &request.document_ids {
            if !doc_ids.is_empty() {
                query_builder.push(" AND d.id = ANY(");
                query_builder.push_bind(doc_ids);
                query_builder.push(")");
            }
        }

        query_builder.push(" AND dc.milvus_vector_id = ANY(");
        query_builder.push_bind(&vector_filter_ids);
        query_builder.push(")");

        let rows = query_builder
            .build()
            .fetch_all(&data.db)
            .await
            .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

        for row in rows {
            if let Ok(vector_id_opt) = row.try_get::<Option<i64>, _>("milvus_vector_id") {
                if let Some(vector_id) = vector_id_opt {
                    chunk_map.insert(
                        vector_id,
                        ChunkRecord {
                            chunk_id: row.get("id"),
                            document_id: row.get("document_id"),
                            document_title: row.get("document_title"),
                            content: row.get("content"),
                            chunk_index: row.get("chunk_index"),
                            page_number: row.get("page_number"),
                        },
                    );
                }
            }
        }
    }

    // Build RAG response, rank strictly by Milvus similarity
    let mut sources = Vec::new();
    let mut answer = String::from("Based on the available documents, here's what I found:\n\n");

    for (vector_id, similarity_score) in milvus_ranked.iter() {
        if sources.len() >= max_results as usize {
            break;
        }

        if *similarity_score < similarity_threshold {
            continue;
        }

        if let Some(chunk) = chunk_map.get(vector_id) {
            sources.push(DocumentSource {
                document_id: chunk.document_id,
                document_title: chunk.document_title.clone(),
                chunk_text: chunk.content.chars().take(200).collect::<String>() + "...",
                similarity_score: *similarity_score,
                page_number: chunk.page_number,
                chunk_index: chunk.chunk_index,
            });

            answer.push_str(&format!(
                "{}. From '{}': {}\n\n",
                sources.len(),
                chunk.document_title,
                chunk.content.chars().take(150).collect::<String>() + "..."
            ));
        }
    }

    // Fallback: local embedding similarity search from DB when Milvus yields no sources
    if sources.is_empty() {
        let mut qb = QueryBuilder::<Postgres>::new(
            "SELECT dc.id, dc.document_id, dc.chunk_index, dc.content, dc.page_number, dc.embedding, d.title as document_title \n             FROM document_chunks dc \n             JOIN documents d ON dc.document_id = d.id \n             WHERE d.status = 'ready' AND dc.deleted_at = 0 AND d.deleted_at = 0",
        );

        if let Some(category) = &request.category_filter {
            qb.push(" AND d.category = ");
            qb.push_bind(category);
        }

        if let Some(doc_ids) = &request.document_ids {
            if !doc_ids.is_empty() {
                qb.push(" AND dc.document_id IN (");
                let mut separated = qb.separated(", ");
                for id in doc_ids {
                    separated.push_bind(id);
                }
                separated.push_unseparated(")");
            }
        }

        // Prevent scanning too many rows
        qb.push(" ORDER BY dc.created_at DESC LIMIT ");
        qb.push_bind(2000i64);

        let rows = qb
            .build()
            .fetch_all(&data.db)
            .await
            .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

        // Rank by cosine similarity (embeddings are L2-normalized), fallback to text similarity if dim mismatch
        let mut ranked: Vec<(f32, DocumentSource)> = Vec::new();
        for row in rows {
            let emb: Vec<f32> = row.get("embedding");
            let sim = if emb.len() == query_embedding.len() {
                emb.iter()
                    .zip(query_embedding.iter())
                    .map(|(a, b)| a * b)
                    .sum::<f32>()
            } else {
                let content: String = row.get("content");
                calculate_text_similarity(&request.query, &content)
            };

            ranked.push((
                sim,
                DocumentSource {
                    document_id: row.get("document_id"),
                    document_title: row.get("document_title"),
                    chunk_text: {
                        let c: String = row.get("content");
                        c.chars().take(200).collect::<String>() + "..."
                    },
                    similarity_score: sim,
                    page_number: row.try_get("page_number").ok(),
                    chunk_index: row.get("chunk_index"),
                },
            ));
        }

        ranked.sort_by(|a, b| b.0.partial_cmp(&a.0).unwrap_or(std::cmp::Ordering::Equal));
        for (_sim, src) in ranked
            .into_iter()
            .filter(|(s, _)| *s >= similarity_threshold)
            .take(max_results as usize)
        {
            sources.push(src);
        }

        // Compose answer from sources
        if sources.is_empty() {
            answer = "Tidak ditemukan konteks relevan di dokumen untuk pertanyaan ini.".to_string();
        } else {
            answer.clear();
            answer.push_str("Berdasarkan dokumen yang tersedia, berikut rangkuman temuan:\n\n");
            for (i, s) in sources.iter().enumerate() {
                answer.push_str(&format!(
                    "{}. Dari '{}': {}\n\n",
                    i + 1,
                    s.document_title,
                    s.chunk_text.chars().take(150).collect::<String>()
                ));
            }
        }
    }

    let processing_time = start_time.elapsed().as_millis() as i64;
    let confidence_score = if sources.is_empty() {
        0.0
    } else {
        sources.iter().map(|s| s.similarity_score).sum::<f32>() / sources.len() as f32
    };

    // Log query for analytics
    let query_hash = format!("{:x}", Sha256::digest(request.query.as_bytes()));
    let _ = sqlx::query(
        r#"
        INSERT INTO rag_query_history (
            query, query_hash, category_filter, document_ids_filter, 
            max_results, similarity_threshold, results_count, 
            top_similarity_score, response_time_ms, created_at
        ) VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10)
        "#,
    )
    .bind(&request.query)
    .bind(&query_hash)
    .bind(&request.category_filter)
    .bind(&request.document_ids)
    .bind(max_results)
    .bind(similarity_threshold)
    .bind(sources.len() as i32)
    .bind(sources.first().map(|s| s.similarity_score))
    .bind(processing_time)
    .bind(Utc::now())
    .execute(&data.db)
    .await;

    let response = RagQueryResponse {
        answer,
        sources,
        confidence_score,
        processing_time_ms: processing_time,
    };

    let msg = if response.sources.is_empty() {
        "RAG tidak menemukan sumber yang relevan"
    } else {
        "RAG query berhasil"
    };
    Ok(Json(ApiResponse {
        code: 200,
        status: "OK".to_string(),
        message: msg.to_string(),
        data: response,
        errors: json!({}),
    }))
}

// RAG + LLM answer endpoint
pub async fn answer_with_rag_and_llm(
    State(data): State<Arc<AppState>>,
    Json(request): Json<RagAnswerRequest>,
) -> Result<Json<ApiResponse<RagAnswerResponse>>, StatusCode> {
    let start_time = std::time::Instant::now();

    if request.query.trim().is_empty() {
        return Err(StatusCode::BAD_REQUEST);
    }

    // Get RAG configuration
    let config = get_rag_config(&data).await?;

    let max_results = request.max_results.unwrap_or(config.max_results);
    let similarity_threshold = request
        .similarity_threshold
        .unwrap_or(config.similarity_threshold);

    // Compute query embedding (normalized query for typo robustness)
    let normalized_query_text = tokenize_and_normalize(&request.query).join(" ");
    let query_embedding = embed_texts_local(
        &vec![normalized_query_text.clone()],
        config.embedding_dimensions as usize,
    )
    .into_iter()
    .next()
    .unwrap_or_default();

    // Optional Milvus search; allow fallback when not available or mock mode
    let milvus_client_opt = data.milvus_client.as_ref().cloned();
    let mut milvus_ranked: Vec<(i64, f32)> = Vec::new();
    let mut candidate_map: HashMap<(Uuid, i32), DocumentSource> = HashMap::new();

    // ID: Hindari meminjam mutable `candidate_map` dalam penutupan yang menangkap lingkungan.
    //     Ini memperbaiki error borrow (E0502) dengan menjadikan peta sebagai parameter eksplisit.
    // EN: Avoid mutably borrowing `candidate_map` via a capturing closure.
    //     This fixes the borrow checker error (E0502) by making the map an explicit parameter.
    let mut upsert_candidate = |map: &mut HashMap<(Uuid, i32), DocumentSource>, candidate: DocumentSource| {
        let key = (candidate.document_id, candidate.chunk_index);
        match map.entry(key) {
            Entry::Vacant(entry) => {
                entry.insert(candidate);
            }
            Entry::Occupied(mut entry) => {
                if candidate.similarity_score > entry.get().similarity_score {
                    entry.insert(candidate);
                }
            }
        }
    };

    if milvus_client_opt.is_some() && !data.env.allow_mock_dependencies {
        let milvus_client = milvus_client_opt.unwrap();
        let mut guard = milvus_client.lock().await;
        crate::services::milvus::ensure_rag_collection(&mut guard, &data.milvus_collection)
            .await
            .map_err(|err| {
                tracing::error!("Failed to ensure Milvus collection: {:?}", err);
                StatusCode::INTERNAL_SERVER_ERROR
            })?;

        let results = crate::services::milvus::search_top_k(
            &mut guard,
            &data.milvus_collection,
            &query_embedding,
            max_results as usize,
        )
        .await
        .map_err(|err| {
            tracing::error!("Milvus search_top_k failed: {:?}", err);
            StatusCode::INTERNAL_SERVER_ERROR
        })?;

        milvus_ranked = results;
    }

    let vector_filter_ids: Vec<i64> = milvus_ranked.iter().map(|(id, _)| *id).collect();

    #[derive(Clone)]
    struct ChunkRecord {
        chunk_id: Uuid,
        document_id: Uuid,
        document_title: String,
        content: String,
        chunk_index: i32,
        page_number: Option<i32>,
    }

    let mut chunk_map: HashMap<i64, ChunkRecord> = HashMap::new();

    if !vector_filter_ids.is_empty() {
        let mut query_builder = QueryBuilder::<Postgres>::new(
            "SELECT dc.id, dc.document_id, dc.chunk_index, dc.content, dc.page_number, \
             dc.milvus_vector_id, d.title as document_title \
             FROM document_chunks dc \
             JOIN documents d ON dc.document_id = d.id \
             WHERE d.status = 'ready' AND dc.deleted_at = 0 AND d.deleted_at = 0",
        );

        if let Some(category) = &request.category_filter {
            query_builder.push(" AND d.category = ");
            query_builder.push_bind(category);
        }

        if let Some(doc_ids) = &request.document_ids {
            if !doc_ids.is_empty() {
                query_builder.push(" AND d.id = ANY(");
                query_builder.push_bind(doc_ids);
                query_builder.push(")");
            }
        }

        query_builder.push(" AND dc.milvus_vector_id = ANY(");
        query_builder.push_bind(&vector_filter_ids);
        query_builder.push(")");

        let rows = query_builder
            .build()
            .fetch_all(&data.db)
            .await
            .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

        for row in rows {
            if let Ok(vector_id_opt) = row.try_get::<Option<i64>, _>("milvus_vector_id") {
                if let Some(vector_id) = vector_id_opt {
                    chunk_map.insert(
                        vector_id,
                        ChunkRecord {
                            chunk_id: row.get("id"),
                            document_id: row.get("document_id"),
                            document_title: row.get("document_title"),
                            content: row.get("content"),
                            chunk_index: row.get("chunk_index"),
                            page_number: row.get("page_number"),
                        },
                    );
                }
            }
        }
    }

    for (vector_id, similarity_score) in milvus_ranked.iter() {
        if let Some(chunk) = chunk_map.get(vector_id) {
            let candidate = DocumentSource {
                document_id: chunk.document_id,
                document_title: chunk.document_title.clone(),
                chunk_text: chunk.content.clone(),
                similarity_score: *similarity_score,
                page_number: chunk.page_number,
                chunk_index: chunk.chunk_index,
            };
            upsert_candidate(&mut candidate_map, candidate);
        }
    }

    // If no sources from Milvus, fallback ke pencarian lokal berbasis embedding
    if candidate_map.is_empty() {
        let mut qb = QueryBuilder::<Postgres>::new(
            "SELECT dc.id, dc.document_id, dc.chunk_index, dc.content, dc.page_number, dc.embedding, d.title as document_title \n             FROM document_chunks dc \n             JOIN documents d ON dc.document_id = d.id \n             WHERE d.status = 'ready' AND dc.deleted_at = 0 AND d.deleted_at = 0",
        );
        if let Some(category) = &request.category_filter {
            qb.push(" AND d.category = ");
            qb.push_bind(category);
        }
        if let Some(doc_ids) = &request.document_ids {
            if !doc_ids.is_empty() {
                qb.push(" AND dc.document_id IN (");
                let mut separated = qb.separated(", ");
                for id in doc_ids {
                    separated.push_bind(id);
                }
                separated.push_unseparated(")");
            }
        }
        qb.push(" ORDER BY dc.created_at DESC LIMIT ");
        qb.push_bind(2000i64);

        let rows = qb
            .build()
            .fetch_all(&data.db)
            .await
            .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
        for row in rows {
            let emb: Vec<f32> = row.get("embedding");
            let sim = if emb.len() == query_embedding.len() {
                emb.iter()
                    .zip(query_embedding.iter())
                    .map(|(a, b)| a * b)
                    .sum::<f32>()
            } else {
                let content: String = row.get("content");
                calculate_text_similarity(&request.query, &content)
            };
            let candidate = DocumentSource {
                document_id: row.get("document_id"),
                document_title: row.get("document_title"),
                chunk_text: row.get("content"),
                similarity_score: sim,
                page_number: row.try_get("page_number").ok(),
                chunk_index: row.get("chunk_index"),
            };
            upsert_candidate(&mut candidate_map, candidate);
        }
    }

    let mut candidates: Vec<DocumentSource> = candidate_map.into_values().collect();
    candidates.sort_by(|a, b| {
        b.similarity_score
            .partial_cmp(&a.similarity_score)
            .unwrap_or(std::cmp::Ordering::Equal)
    });

    let mut sources: Vec<DocumentSource> = candidates
        .iter()
        .filter(|s| s.similarity_score >= similarity_threshold)
        .take(max_results as usize)
        .cloned()
        .collect();

    if sources.is_empty() && !candidates.is_empty() {
        let top_score = candidates[0].similarity_score;
        let adaptive_threshold = (top_score * 0.75).max(0.25);
        sources = candidates
            .iter()
            .filter(|s| s.similarity_score >= adaptive_threshold)
            .take(max_results as usize)
            .cloned()
            .collect();
    }

    if sources.is_empty() && !candidates.is_empty() {
        sources = candidates
            .iter()
            .take(max_results as usize)
            .cloned()
            .collect();
    }

    if sources.is_empty() {
        let processing_time = start_time.elapsed().as_millis() as i64;
        let response = RagAnswerResponse {
            answer: "Tidak ditemukan konteks relevan di dokumen untuk pertanyaan ini.".to_string(),
            sources,
            confidence_score: 0.0,
            processing_time_ms: processing_time,
            llm_model: None,
            tokens_used: None,
        };
        return Ok(Json(ApiResponse {
            code: 200,
            status: "OK".to_string(),
            message: "RAG tidak menemukan sumber yang relevan".to_string(),
            data: response,
            errors: json!({}),
        }));
    }

    let confidence_score =
        sources.iter().map(|s| s.similarity_score).sum::<f32>() / sources.len() as f32;

    // Construct prompts
    let default_system_prompt = "Anda adalah asisten AI untuk bisnis retail.
Jawab hanya berdasarkan KONTEN yang diberikan.
Jika informasi tidak ada di konteks, katakan dengan jujur.
Cantumkan rujukan sumber hanya dalam format: [Rujukan: Judul_Dokumen].
Jangan buat bagian terpisah bernama 'Sumber:' dan jangan tampilkan ID, chunk, atau halaman.
Gunakan bahasa Indonesia yang jelas dan ringkas.";

    let system_prompt = request
        .prompt_instructions
        .as_ref()
        .map(|s| format!("{}\n\nInstruksi tambahan: {}", default_system_prompt, s))
        .unwrap_or_else(|| default_system_prompt.to_string());

    // Build context JSON with sources
    let context_json = serde_json::json!({
        "question": request.query,
        "sources": sources.iter().enumerate().map(|(i, s)| serde_json::json!({
            "index": i,
            "title": s.document_title,
            "similarity": s.similarity_score,
            "text": s.chunk_text,
        })).collect::<Vec<_>>()
    });

    let user_prompt = format!(
        "Pertanyaan pengguna:\n{}\n\nKONTEN untuk menjawab (JSON):\n{}\n\nInstruksi:\n- Jawab ringkas, tepat, dan berbasis KONTEN.\n- Cantumkan SATU baris rujukan di akhir jawaban dengan format: [Rujukan: Judul_Dokumen].\n- Jangan menulis bagian 'Sumber:' terpisah dan jangan tampilkan ID, chunk, atau halaman.\n- Jika konteks tidak memadai, jelaskan keterbatasan.\n",
        request.query, context_json
    );

    // Prepare Groq request
    let model = std::env::var("GROQ_MODEL").unwrap_or_else(|_| "llama-3.1-8b-instant".to_string());
    let api_key =
        std::env::var("GROQ_API_KEY").unwrap_or_else(|_| panic!("GROQ_API_KEY must be set"));
    let api_url = std::env::var("GROQ_API_URL")
        .unwrap_or_else(|_| "https://api.groq.com/openai/v1/chat/completions".to_string());

    let req_body = GroqApiRequest {
        messages: vec![
            GroqMessage {
                role: "system".to_string(),
                content: system_prompt,
            },
            GroqMessage {
                role: "user".to_string(),
                content: user_prompt,
            },
        ],
        model: model.clone(),
        max_tokens: request.max_tokens.or(Some(512)),
        temperature: request.temperature.or(Some(0.2)),
    };

    let client = Client::new();
    let resp = client
        .post(api_url)
        .header("Authorization", format!("Bearer {}", api_key))
        .header("Content-Type", "application/json")
        .json(&req_body)
        .send()
        .await
        .map_err(|_| StatusCode::BAD_GATEWAY)?;

    if !resp.status().is_success() {
        return Err(StatusCode::BAD_GATEWAY);
    }

    let groq_resp: GroqApiResponse = resp.json().await.map_err(|_| StatusCode::BAD_GATEWAY)?;
    let answer = groq_resp
        .choices
        .get(0)
        .map(|c| c.message.content.clone())
        .unwrap_or_else(|| "".to_string());

    let processing_time = start_time.elapsed().as_millis() as i64;
    let response = RagAnswerResponse {
        answer,
        sources,
        confidence_score,
        processing_time_ms: processing_time,
        llm_model: Some(groq_resp.model.clone()),
        tokens_used: Some(groq_resp.usage.total_tokens),
    };

    Ok(Json(ApiResponse {
        code: 200,
        status: "OK".to_string(),
        message: "RAG + LLM answer generated successfully".to_string(),
        data: response,
        errors: json!({}),
    }))
}

// ID: Endpoint sederhana agar user cukup mengirim pertanyaan.
//     Handler ini menyiapkan parameter default yang sesuai untuk pertanyaan spesifik seperti "Andi".
// EN: Simple endpoint so the user only sends the question.
//     This handler prepares sensible defaults tailored for specific queries like "Andi".
pub async fn answer_simple_rag(
    State(data): State<Arc<AppState>>,
    Json(request): Json<crate::dto::rag::SimpleRagRequest>,
) -> Result<Json<ApiResponse<RagAnswerResponse>>, StatusCode> {
    if request.query.trim().is_empty() {
        return Err(StatusCode::BAD_REQUEST);
    }

    // ID: Atur parameter RAG yang konservatif agar lebih mudah menemukan konteks terkait.
    // EN: Set conservative RAG parameters to increase chances of finding relevant context.
    let rag_req = RagAnswerRequest {
        query: request.query.clone(),
        document_ids: None,            // dapat dibatasi kemudian jika diperlukan
        category_filter: None,         // tidak membatasi kategori
        max_results: Some(8),          // ambil hingga 8 sumber
        similarity_threshold: Some(0.20), // turunkan threshold untuk recall lebih tinggi
        prompt_instructions: Some(
            "Ringkas kinerja karyawan bernama Andi selama satu tahun berdasarkan KONTEN. Fokuskan pada KPI utama, pencapaian, kekuatan, area perbaikan, dan rekomendasi singkat. Tambahkan SATU rujukan di akhir dalam format: [Rujukan: Judul_Dokumen]."
                .to_string(),
        ),
        max_tokens: Some(512),
        temperature: Some(0.2),
    };

    // ID: Delegasikan ke handler utama RAG+LLM untuk pemrosesan lengkap.
    // EN: Delegate to the main RAG+LLM handler for full processing.
    answer_with_rag_and_llm(State(data), Json(rag_req)).await
}

// List documents endpoint
pub async fn list_documents(
    State(data): State<Arc<AppState>>,
    Query(params): Query<DocumentListRequest>,
) -> Result<Json<ApiResponse<DocumentListResponse>>, StatusCode> {
    let page = params.page.unwrap_or(1).max(1);
    let limit = params.limit.unwrap_or(20).min(100);
    let offset = (page - 1) * limit;

    // Simple query without dynamic building for now
    let documents_query = r#"
        SELECT d.*, COALESCE(COUNT(dc.id), 0) as chunk_count 
        FROM documents d 
        LEFT JOIN document_chunks dc ON d.id = dc.document_id AND dc.deleted_at = 0
        WHERE d.status != 'deleted' AND d.deleted_at = 0
        GROUP BY d.id 
        ORDER BY d.created_at DESC
        LIMIT $1 OFFSET $2
    "#;

    let count_query = "SELECT COUNT(*) FROM documents WHERE status != 'deleted' AND deleted_at = 0";

    // Get total count
    let total_count: i64 = sqlx::query_scalar(count_query)
        .fetch_one(&data.db)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    // Get documents
    let rows = sqlx::query(documents_query)
        .bind(limit)
        .bind(offset)
        .fetch_all(&data.db)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    let mut documents = Vec::new();
    for row in rows {
        documents.push(DocumentSummary {
            id: row.get("id"),
            title: row.get("title"),
            description: row.get("description"),
            category: row.get("category"),
            tags: row.get("tags"),
            file_type: row.get("file_type"),
            file_size: row.get("file_size"),
            status: row.get("status"),
            chunk_count: row.get::<i64, _>("chunk_count") as i32,
            uploaded_at: row.get("created_at"),
            last_accessed: row.get("last_accessed"),
        });
    }

    let total_pages = ((total_count as f64) / (limit as f64)).ceil() as i32;

    let response = DocumentListResponse {
        documents,
        total_count,
        page,
        limit,
        total_pages,
    };

    Ok(Json(ApiResponse {
        code: 200,
        status: "OK".to_string(),
        message: "Documents retrieved successfully".to_string(),
        data: response,
        errors: json!({}),
    }))
}

// Get document processing status
pub async fn get_document_status(
    State(data): State<Arc<AppState>>,
    Path(document_id): Path<Uuid>,
) -> Result<Json<ApiResponse<DocumentProcessingStatus>>, StatusCode> {
    let document =
        sqlx::query_as::<_, Document>("SELECT * FROM documents WHERE id = $1 AND deleted_at = 0")
            .bind(document_id)
            .fetch_optional(&data.db)
            .await
            .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
            .ok_or(StatusCode::NOT_FOUND)?;

    let job = sqlx::query_as::<_, DocumentProcessingJob>(
        "SELECT * FROM document_processing_jobs WHERE document_id = $1 AND deleted_at = 0 ORDER BY created_at DESC LIMIT 1"
    )
    .bind(document_id)
    .fetch_optional(&data.db)
    .await
    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    let status = DocumentProcessingStatus {
        document_id,
        status: document.status.clone(),
        progress_percentage: document.processing_progress,
        current_step: document
            .current_processing_step
            .unwrap_or("Unknown".to_string()),
        error_message: document.error_message,
        chunks_processed: document.chunk_count,
        total_chunks: document.chunk_count, // Simplified
        estimated_completion: job.and_then(|j| j.completed_at),
    };

    Ok(Json(ApiResponse {
        code: 200,
        status: "OK".to_string(),
        message: "Document status retrieved successfully".to_string(),
        data: status,
        errors: json!({}),
    }))
}

// Delete document
pub async fn delete_document(
    State(data): State<Arc<AppState>>,
    Path(document_id): Path<Uuid>,
) -> Result<Json<ApiResponse<()>>, StatusCode> {
    let result = sqlx::query(
        "UPDATE documents SET status = 'deleted', updated_at = $1, deleted_at = $2 WHERE id = $3",
    )
    .bind(Utc::now())
    .bind(Utc::now().timestamp_millis())
    .bind(document_id)
    .execute(&data.db)
    .await
    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    if result.rows_affected() == 0 {
        return Err(StatusCode::NOT_FOUND);
    }

    Ok(Json(ApiResponse {
        code: 200,
        status: "OK".to_string(),
        message: "Document deleted successfully".to_string(),
        data: (),
        errors: json!({}),
    }))
}

// Get RAG configuration
pub async fn get_rag_configuration(
    State(data): State<Arc<AppState>>,
) -> Result<Json<ApiResponse<RagConfig>>, StatusCode> {
    let config = get_rag_config(&data).await?;

    let response = RagConfig {
        id: config.id,
        chunk_size: config.chunk_size,
        chunk_overlap: config.chunk_overlap,
        embedding_model: config.embedding_model,
        similarity_threshold: config.similarity_threshold,
        max_results: config.max_results,
        enable_reranking: config.enable_reranking,
        created_at: config.created_at,
        updated_at: config.updated_at,
    };

    Ok(Json(ApiResponse {
        code: 200,
        status: "OK".to_string(),
        message: "Configuration retrieved successfully".to_string(),
        data: response,
        errors: json!({}),
    }))
}

// Update RAG configuration
pub async fn update_rag_configuration(
    State(data): State<Arc<AppState>>,
    Json(request): Json<UpdateRagConfigRequest>,
) -> Result<Json<ApiResponse<RagConfig>>, StatusCode> {
    // For simplicity, let's update all fields at once
    let query = r#"
        UPDATE rag_configurations 
        SET 
            chunk_size = COALESCE($1, chunk_size),
            chunk_overlap = COALESCE($2, chunk_overlap),
            embedding_model = COALESCE($3, embedding_model),
            similarity_threshold = COALESCE($4, similarity_threshold),
            max_results = COALESCE($5, max_results),
            enable_reranking = COALESCE($6, enable_reranking),
            updated_at = $7
        WHERE id = (SELECT id FROM rag_configurations WHERE deleted_at = 0 ORDER BY created_at DESC LIMIT 1) 
        RETURNING *
    "#;

    let config = sqlx::query_as::<_, RagConfiguration>(query)
        .bind(request.chunk_size)
        .bind(request.chunk_overlap)
        .bind(request.embedding_model)
        .bind(request.similarity_threshold)
        .bind(request.max_results)
        .bind(request.enable_reranking)
        .bind(Utc::now())
        .fetch_one(&data.db)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    let response = RagConfig {
        id: config.id,
        chunk_size: config.chunk_size,
        chunk_overlap: config.chunk_overlap,
        embedding_model: config.embedding_model,
        similarity_threshold: config.similarity_threshold,
        max_results: config.max_results,
        enable_reranking: config.enable_reranking,
        created_at: config.created_at,
        updated_at: config.updated_at,
    };

    Ok(Json(ApiResponse {
        code: 200,
        status: "OK".to_string(),
        message: "Configuration updated successfully".to_string(),
        data: response,
        errors: json!({}),
    }))
}

async fn process_uploaded_document(
    data: &Arc<AppState>,
    document_id: Uuid,
    file_path: &str,
    file_type: &str,
    config: &RagConfiguration,
) -> anyhow::Result<usize> {
    let resolved_path = PathBuf::from(file_path);
    if !resolved_path.exists() {
        return Err(anyhow!("Uploaded file is missing on disk"));
    }

    let file_type_lower = file_type.to_lowercase();
    let content = match file_type_lower.as_str() {
        "docx" => {
            let path_clone = resolved_path.clone();
            task::spawn_blocking(move || extract_text_from_docx(&path_clone))
                .await
                .map_err(|e| anyhow!("Failed to join docx extraction task: {}", e))??
        }
        "xlsx" | "xls" | "xlsm" => {
            let path_clone = resolved_path.clone();
            task::spawn_blocking(move || extract_text_from_xlsx(&path_clone))
                .await
                .map_err(|e| anyhow!("Failed to join xlsx extraction task: {}", e))??
        }
        "csv" | "txt" | "md" | "json" => read_plain_text_file(&resolved_path).await?,
        other => {
            return Err(anyhow!(
                "Unsupported file type '{}' for RAG ingestion",
                other
            ))
        }
    };

    if content.trim().is_empty() {
        return Err(anyhow!("Document does not contain extractable text"));
    }

    let base_metadata = json!({
        "source": "upload",
        "file_type": file_type_lower,
        "file_path": file_path,
    });

    ingest_document_content(data, document_id, &content, config, base_metadata).await
}

async fn ingest_document_content(
    data: &Arc<AppState>,
    document_id: Uuid,
    content: &str,
    config: &RagConfiguration,
    base_metadata: serde_json::Value,
) -> anyhow::Result<usize> {
    let chunk_size = config.chunk_size.max(1) as usize;
    let chunk_overlap = config.chunk_overlap.max(0) as usize;
    let embed_dim = config.embedding_dimensions.max(8) as usize;

    let chunks = chunk_text(content, chunk_size, chunk_overlap);
    if chunks.is_empty() {
        return Err(anyhow!("No chunks generated from document content"));
    }

    let texts: Vec<String> = chunks.iter().map(|c| c.0.clone()).collect();
    let embeddings = embed_texts_local(&texts, embed_dim);

    let mut chunk_ids = Vec::with_capacity(chunks.len());
    let inserted_at = Utc::now();

    for (idx, (chunk_text, start_char, end_char)) in chunks.iter().enumerate() {
        let chunk_id = Uuid::new_v4();
        chunk_ids.push(chunk_id);

        let content_hash = format!("{:x}", Sha256::digest(chunk_text.as_bytes()));
        let vector_id = crate::services::milvus::uuid_to_i64(&chunk_id);

        let mut metadata = base_metadata.clone();
        if let Some(map) = metadata.as_object_mut() {
            map.insert("chunk_index".to_string(), json!(idx));
        }

        sqlx::query(
            r#"
            INSERT INTO document_chunks (
                id, document_id, chunk_index, content, content_hash, embedding,
                page_number, start_char, end_char, metadata, created_at, milvus_vector_id
            ) VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12)
            "#,
        )
        .bind(chunk_id)
        .bind(document_id)
        .bind(idx as i32)
        .bind(chunk_text)
        .bind(content_hash)
        .bind(&embeddings[idx])
        .bind(None::<i32>)
        .bind(*start_char as i32)
        .bind(*end_char as i32)
        .bind(metadata)
        .bind(inserted_at)
        .bind(vector_id)
        .execute(&data.db)
        .await
        .context("Failed to insert document chunk")?;
    }
    // In mock dependency mode, skip Milvus operations and finish successfully
    if data.env.allow_mock_dependencies {
        return Ok(chunks.len());
    }

    let milvus_client = data
        .milvus_client
        .as_ref()
        .ok_or_else(|| {
            anyhow!("Milvus client not configured; set MILVUS_URI before ingesting documents")
        })?
        .clone();

    let mut guard = milvus_client.lock().await;
    crate::services::milvus::ensure_rag_collection(&mut guard, &data.milvus_collection)
        .await
        .context("Ensuring Milvus collection failed")?;
    crate::services::milvus::upsert_chunk_embeddings(
        &mut guard,
        &data.milvus_collection,
        &chunk_ids,
        &embeddings,
    )
    .await
    .context("Failed to upsert embeddings to Milvus")?;

    Ok(chunks.len())
}

async fn resolve_document_category(
    db: &sqlx::Pool<Postgres>,
    requested: &str,
) -> Result<String, StatusCode> {
    let trimmed = requested.trim();

    if let Some(existing) = fetch_category_case_insensitive(db, trimmed).await? {
        return Ok(existing);
    }

    if trimmed.is_empty() {
        return ensure_default_category(db).await;
    }

    let normalized = trimmed.to_string();
    sqlx::query(
        r#"
        INSERT INTO document_categories (name, description)
        VALUES ($1, $2)
        ON CONFLICT (name) DO NOTHING
        "#,
    )
    .bind(&normalized)
    .bind(Some(format!(
        "{} {}",
        AUTO_CATEGORY_DESCRIPTION, normalized
    )))
    .execute(db)
    .await
    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    Ok(normalized)
}

async fn ensure_default_category(db: &sqlx::Pool<Postgres>) -> Result<String, StatusCode> {
    if let Some(existing) = fetch_category_case_insensitive(db, DEFAULT_CATEGORY_NAME).await? {
        return Ok(existing);
    }

    sqlx::query(
        r#"
        INSERT INTO document_categories (name, description)
        VALUES ($1, $2)
        ON CONFLICT (name) DO NOTHING
        "#,
    )
    .bind(DEFAULT_CATEGORY_NAME)
    .bind(Some(DEFAULT_CATEGORY_DESCRIPTION.to_string()))
    .execute(db)
    .await
    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    Ok(DEFAULT_CATEGORY_NAME.to_string())
}

async fn fetch_category_case_insensitive(
    db: &sqlx::Pool<Postgres>,
    name: &str,
) -> Result<Option<String>, StatusCode> {
    if name.trim().is_empty() {
        return Ok(None);
    }

    sqlx::query_scalar::<_, String>(
        r#"
        SELECT name
        FROM document_categories
        WHERE lower(name) = lower($1)
        LIMIT 1
        "#,
    )
    .bind(name)
    .fetch_optional(db)
    .await
    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)
}

fn extract_text_from_docx(path: &StdPath) -> anyhow::Result<String> {
    let file = File::open(path).with_context(|| format!("Opening DOCX file {:?}", path))?;
    let mut archive = ZipArchive::new(file).context("Reading DOCX zip archive")?;
    let mut document_xml = archive
        .by_name("word/document.xml")
        .context("DOCX missing word/document.xml")?;

    let mut xml_content = String::new();
    document_xml
        .read_to_string(&mut xml_content)
        .context("Reading DOCX XML content")?;

    // ID: Gunakan quick_xml reader untuk membaca struktur DOCX termasuk paragraf, tab, dan tabel.
    // EN: Use quick_xml reader to parse DOCX structure including paragraphs, tabs, and tables.
    let mut reader = XmlReader::from_str(&xml_content);
    reader.trim_text(true);
    let mut buf = Vec::new();
    let mut text = String::new();
    // ID: Tracking status tabel sehingga setiap cell dipisah tab, dan baris berakhir newline.
    // EN: Track table status so each cell is separated by tabs and rows end with newline.
    let mut in_row = false;
    let mut first_cell_in_row = true;

    loop {
        match reader.read_event_into(&mut buf) {
            Ok(XmlEvent::Start(ref e)) if e.name().as_ref() == b"w:p" => {
                // ID: Setiap paragraf baru diberi newline.
                // EN: Add newline at the start of each new paragraph.
                if !text.is_empty() && !text.ends_with('\n') {
                    text.push('\n');
                }
            }
            Ok(XmlEvent::Start(ref e)) if e.name().as_ref() == b"w:br" => {
                // ID: Break baris eksplisit.
                // EN: Explicit line break.
                text.push('\n');
            }
            Ok(XmlEvent::Start(ref e)) if e.name().as_ref() == b"w:tab" => {
                // ID: Tanda tab.
                // EN: Tab marker.
                text.push('\t');
            }
            Ok(XmlEvent::Start(ref e)) if e.name().as_ref() == b"w:tr" => {
                // ID: Mulai baris tabel.
                // EN: Begin table row.
                in_row = true;
                first_cell_in_row = true;
                if !text.is_empty() && !text.ends_with('\n') {
                    text.push('\n');
                }
            }
            Ok(XmlEvent::End(ref e)) if e.name().as_ref() == b"w:tr" => {
                // ID: Akhiri baris tabel.
                // EN: End table row.
                in_row = false;
                first_cell_in_row = true;
                if !text.ends_with('\n') {
                    text.push('\n');
                }
            }
            Ok(XmlEvent::Start(ref e)) if e.name().as_ref() == b"w:tc" => {
                // ID: Pisahkan cell dengan tab, kecuali cell pertama.
                // EN: Separate table cells with tabs, except the first cell.
                if in_row {
                    if first_cell_in_row {
                        first_cell_in_row = false;
                    } else {
                        text.push('\t');
                    }
                }
            }
            Ok(XmlEvent::Text(e)) => {
                let value = e.unescape()?.to_string();
                if !value.trim().is_empty() {
                    // ID: Tambahkan teks; untuk tabel hindari spasi ekstra di akhir cell.
                    // EN: Append text; for tables avoid extra trailing spaces per cell.
                    text.push_str(&value);
                    if !in_row {
                        text.push(' ');
                    }
                }
            }
            Ok(XmlEvent::Eof) => break,
            Ok(_) => {}
            Err(err) => return Err(anyhow!("Failed to parse DOCX XML: {}", err)),
        }
        buf.clear();
    }
    // ID: Rapikan spasi akhir per baris agar output bersih.
    // EN: Trim trailing spaces per line for clean output.
    let cleaned = text
        .lines()
        .map(|l| l.trim_end())
        .collect::<Vec<_>>()
        .join("\n");
    Ok(cleaned.trim().to_string())
}

fn extract_text_from_xlsx(path: &StdPath) -> anyhow::Result<String> {
    let mut workbook =
        open_workbook_auto(path).with_context(|| format!("Opening workbook {:?}", path))?;
    let mut output = String::new();

    let sheet_names = workbook.sheet_names().to_owned();
    for sheet in sheet_names {
        if let Some(Ok(range)) = workbook.worksheet_range(&sheet) {
            output.push_str(&format!("Sheet: {}\n", sheet));
            for row in range.rows() {
                let line = row
                    .iter()
                    .map(data_type_to_string)
                    .collect::<Vec<_>>()
                    .join("\t");
                // ID: Lewati baris yang seluruhnya kosong/tab agar hasil lebih ringkas.
                // EN: Skip rows that are entirely empty/tabs to keep output concise.
                if !line.trim_matches(['\t', ' '].as_ref()).is_empty() {
                    output.push_str(&line);
                    output.push('\n');
                }
            }
            output.push('\n');
        }
    }

    Ok(output.trim().to_string())
}

async fn read_plain_text_file(path: &StdPath) -> anyhow::Result<String> {
    let bytes = tokio::fs::read(path)
        .await
        .with_context(|| format!("Reading text file {:?}", path))?;
    Ok(match String::from_utf8(bytes) {
        Ok(s) => s,
        Err(err) => String::from_utf8_lossy(&err.into_bytes()).into_owned(),
    })
}

fn data_type_to_string(value: &DataType) -> String {
    match value {
        DataType::Empty => String::new(),
        DataType::String(s) => s.clone(),
        DataType::Float(f) => format!("{}", f),
        DataType::Int(i) => i.to_string(),
        DataType::Bool(b) => b.to_string(),
        DataType::DateTime(dt) => dt.to_string(),
        DataType::Duration(d) => d.to_string(),
        DataType::Error(e) => format!("ERR({})", e),
        other => other.to_string(),
    }
}

// Helper functions
async fn get_rag_config(data: &Arc<AppState>) -> Result<RagConfiguration, StatusCode> {
    sqlx::query_as::<_, RagConfiguration>(
        "SELECT * FROM rag_configurations WHERE deleted_at = 0 ORDER BY created_at DESC LIMIT 1",
    )
    .fetch_optional(&data.db)
    .await
    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
    .unwrap_or_else(|| RagConfiguration::default())
    .pipe(Ok)
}

fn calculate_text_similarity(query: &str, content: &str) -> f32 {
    // Robust text similarity with Indonesian-friendly normalization and fuzzy matching
    let q_tokens = tokenize_and_normalize(query);
    let c_tokens = tokenize_and_normalize(content);

    if q_tokens.is_empty() || c_tokens.is_empty() {
        return 0.0;
    }

    // Recall-style match: for each query token, check exact/fuzzy match in content tokens
    let mut matched = 0usize;
    for q in &q_tokens {
        if c_tokens.contains(q) {
            matched += 1;
            continue;
        }
        let found_fuzzy = c_tokens.iter().any(|c| {
            if c == q {
                return true;
            }
            // substring containment (handles inflectional variants)
            if c.contains(q) || q.contains(c) {
                return true;
            }
            // small edit distance tolerates typos like "meleyani" vs "melayani"
            levenshtein_distance(q, c) <= 1
        });
        if found_fuzzy {
            matched += 1;
        }
    }
    let recall = matched as f32 / (q_tokens.len() as f32);

    // Jaccard over sets for stability
    use std::collections::HashSet;
    let q_set: HashSet<&String> = q_tokens.iter().collect();
    let c_set: HashSet<&String> = c_tokens.iter().collect();
    let intersection = q_set.intersection(&c_set).count();
    let union = q_set.union(&c_set).count();
    let jaccard = if union == 0 {
        0.0
    } else {
        intersection as f32 / union as f32
    };

    // Weighted combo; emphasize recall to ensure retrieval under typos/synonyms
    0.7 * recall + 0.3 * jaccard
}

fn tokenize_and_normalize(text: &str) -> Vec<String> {
    text.to_lowercase()
        .split(|c: char| c.is_whitespace() || !c.is_alphanumeric())
        .filter(|t| !t.is_empty())
        .map(|t| normalize_token_id(t))
        .filter(|t| !is_stopword_id(t))
        .collect()
}

fn normalize_token_id(token: &str) -> String {
    // Remove common Indonesian prefixes/suffixes and normalize simple variants
    let mut t = token.to_string();
    // Common prefixes
    for p in [
        "meng", "meny", "men", "mem", "me", "ber", "ter", "se", "pe", "di", "ke",
    ]
    .iter()
    {
        if t.starts_with(p) && t.len() > (p.len() + 2) {
            t = t.strip_prefix(p).unwrap().to_string();
            break;
        }
    }
    // Common suffixes
    for s in ["kan", "nya", "lah", "kah", "pun", "an", "i"].iter() {
        if t.ends_with(s) && t.len() > (s.len() + 2) {
            t = t.strip_suffix(s).unwrap().to_string();
            break;
        }
    }
    // Normalize vowel variants (e.g., "meleyani" -> "melayani" handled by edit distance, but also stabilize)
    t = t.replace("ey", "ay");
    t
}

fn is_stopword_id(token: &str) -> bool {
    matches!(
        token,
        "yang"
            | "dan"
            | "atau"
            | "untuk"
            | "dengan"
            | "di"
            | "ke"
            | "dari"
            | "pada"
            | "ini"
            | "itu"
            | "sebuah"
            | "para"
    )
}

fn levenshtein_distance(a: &str, b: &str) -> usize {
    // Classic DP implementation; optimized for small tokens
    let (m, n) = (a.len(), b.len());
    if m == 0 {
        return n;
    }
    if n == 0 {
        return m;
    }
    let mut prev: Vec<usize> = (0..=n).collect();
    let mut curr: Vec<usize> = vec![0; n + 1];
    for (i, ca) in a.chars().enumerate() {
        curr[0] = i + 1;
        for (j, cb) in b.chars().enumerate() {
            let cost = if ca == cb { 0 } else { 1 };
            curr[j + 1] =
                std::cmp::min(std::cmp::min(curr[j] + 1, prev[j + 1] + 1), prev[j] + cost);
        }
        prev.copy_from_slice(&curr);
    }
    prev[n]
}

// Extension trait for pipe operation
trait Pipe<T> {
    fn pipe<U, F>(self, f: F) -> U
    where
        F: FnOnce(T) -> U;
}

impl<T> Pipe<T> for T {
    fn pipe<U, F>(self, f: F) -> U
    where
        F: FnOnce(T) -> U,
    {
        f(self)
    }
}

// Ingest a text document without file upload: chunk, embed, store
pub async fn ingest_text_document(
    State(data): State<Arc<AppState>>,
    Json(body): Json<DocumentTextIngestRequest>,
) -> Result<Json<ApiResponse<DocumentTextIngestResponse>>, StatusCode> {
    if body.title.trim().is_empty()
        || body.category.trim().is_empty()
        || body.content.trim().is_empty()
    {
        return Err(StatusCode::BAD_REQUEST);
    }

    // Load configuration for chunking and embeddings
    let rag = get_rag_config(&data).await?;

    // Create document record
    let document_id = Uuid::new_v4();
    let file_name = format!("{}.txt", sanitize_filename(&body.title));
    let file_path = format!("inline://{}", document_id);
    let file_bytes = body.content.as_bytes();

    let document = sqlx::query_as::<_, Document>(
        r#"
        INSERT INTO documents (
            id, title, description, category, tags, file_path, file_name,
            file_size, file_type, mime_type, status, created_at, updated_at
        ) VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13)
        RETURNING *
        "#,
    )
    .bind(document_id)
    .bind(&body.title)
    .bind(&body.description)
    .bind(&body.category)
    .bind(body.tags.unwrap_or_default())
    .bind(&file_path)
    .bind(&file_name)
    .bind(file_bytes.len() as i64)
    .bind("txt")
    .bind("text/plain")
    .bind("processing")
    .bind(Utc::now())
    .bind(Utc::now())
    .fetch_one(&data.db)
    .await
    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    let metadata = json!({"source": "inline"});
    let ingest_result =
        ingest_document_content(&data, document_id, &body.content, &rag, metadata).await;

    let chunk_count = match ingest_result {
        Ok(count) => count,
        Err(err) => {
            let now = Utc::now();
            let err_msg = err.to_string();

            let _ = sqlx::query(
                r#"
                UPDATE documents
                SET status = 'error',
                    error_message = $1,
                    processing_progress = 0,
                    current_processing_step = 'Failed',
                    updated_at = $2
                WHERE id = $3
                "#,
            )
            .bind(&err_msg)
            .bind(now)
            .bind(document_id)
            .execute(&data.db)
            .await;

            eprintln!(
                "Failed to ingest inline document {}: {}",
                document_id, err_msg
            );

            return Err(StatusCode::UNPROCESSABLE_ENTITY);
        }
    };

    let _ = sqlx::query(
        r#"
        UPDATE documents
        SET status = 'ready',
            chunk_count = $1,
            processing_progress = 100,
            current_processing_step = 'Completed',
            error_message = NULL,
            updated_at = $2
        WHERE id = $3
        "#,
    )
    .bind(chunk_count as i32)
    .bind(Utc::now())
    .bind(document_id)
    .execute(&data.db)
    .await
    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    let response = DocumentTextIngestResponse {
        document_id,
        chunk_count: chunk_count as i32,
        milvus_collection: data.milvus_collection.clone(),
        status: "ready".to_string(),
    };

    Ok(Json(ApiResponse {
        code: 201,
        status: "CREATED".to_string(),
        message: "Text document ingested successfully".to_string(),
        data: response,
        errors: json!({}),
    }))
}

// Helper: naive char-based chunking with overlap
fn chunk_text(content: &str, chunk_size: usize, chunk_overlap: usize) -> Vec<(String, i32, i32)> {
    if chunk_size == 0 {
        return vec![];
    }
    let chars: Vec<char> = content.chars().collect();
    let mut start = 0usize;
    let mut result = Vec::new();
    let len = chars.len();
    while start < len {
        let end = (start + chunk_size).min(len);
        let slice: String = chars[start..end].iter().collect();
        result.push((slice, start as i32, end as i32));
        if end == len {
            break;
        }
        let next_start = end.saturating_sub(chunk_overlap);
        if next_start <= start {
            break;
        }
        start = next_start;
    }
    result
}

// Helper: simple hashing-based embedding generator as local fallback
fn embed_texts_local(texts: &[String], dim: usize) -> Vec<Vec<f32>> {
    let dim = dim.max(8); // safety
    texts
        .iter()
        .map(|t| {
            let mut vec = vec![0f32; dim];
            for token in t.split_whitespace() {
                let mut hasher = sha2::Sha256::new();
                hasher.update(token.as_bytes());
                let hash = hasher.finalize();
                // Use first 8 bytes to index and weight
                let idx = (u64::from_be_bytes([
                    hash[0], hash[1], hash[2], hash[3], hash[4], hash[5], hash[6], hash[7],
                ]) % dim as u64) as usize;
                vec[idx] += 1.0;
            }
            // L2 normalize
            let norm = vec.iter().map(|x| x * x).sum::<f32>().sqrt();
            if norm > 0.0 {
                vec.iter_mut().for_each(|x| *x /= norm);
            }
            vec
        })
        .collect()
}

fn sanitize_filename(name: &str) -> String {
    let mut s = name.trim().to_string();
    s = s.replace('/', "-").replace('\\', "-");
    s.chars().filter(|c| c.is_ascii()).collect()
}
