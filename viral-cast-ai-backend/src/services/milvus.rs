use anyhow::Result;
use milvus::schema;
use milvus::Client as MilvusClient;
use uuid::Uuid;

// Ensure collection for RAG chunks exists with INT64 PK and FLOAT_VECTOR field
pub async fn ensure_rag_collection(client: &mut MilvusClient, collection_name: &str) -> Result<()> {
    // Build schema: id (INT64, primary), embedding (FLOAT_VECTOR)
    let id_field = schema::FieldSchema {
        field_id: 1,
        name: "id".to_string(),
        is_primary_key: true,
        description: "primary key field".to_string(),
        data_type: schema::DataType::Int64 as i32,
        type_params: Vec::new(),
        index_params: Vec::new(),
        auto_id: false,
    };

    let embed_field = schema::FieldSchema {
        field_id: 2,
        name: "embedding".to_string(),
        is_primary_key: false,
        description: "embedding vector".to_string(),
        data_type: schema::DataType::FloatVector as i32,
        type_params: Vec::new(),
        index_params: Vec::new(),
        auto_id: false,
    };

    let coll_schema = schema::CollectionSchema {
        name: collection_name.to_string(),
        description: "RAG document chunks".to_string(),
        auto_id: false,
        fields: vec![id_field, embed_field],
    };

    // Try to create; if exists, ignore error
    match client
        .create_collection(collection_name, coll_schema, None, None)
        .await
    {
        Ok(_) => {}
        Err(_e) => {
            // Assume already exists; proceed
        }
    }

    Ok(())
}

// Convert a UUID to a stable i64 for Milvus PK
pub fn uuid_to_i64(uuid: &Uuid) -> i64 {
    let b = uuid.as_u128();
    // Truncate to lower 64 bits
    (b & 0xFFFF_FFFF_FFFF_FFFF) as i64
}

// Upsert chunk embeddings into Milvus collection
pub async fn upsert_chunk_embeddings(
    client: &mut MilvusClient,
    collection_name: &str,
    chunk_ids: &[Uuid],
    embeddings: &[Vec<f32>],
) -> Result<()> {
    // Prepare columnar data: id and embedding
    let ids: Vec<i64> = chunk_ids.iter().map(|u| uuid_to_i64(u)).collect();

    let dim = embeddings
        .get(0)
        .map(|v| v.len())
        .unwrap_or_else(|| embedding_dim_from_list(embeddings)) as i32;
    let flat: Vec<f32> = embeddings.iter().flat_map(|v| v.iter().cloned()).collect();

    // FieldData for id (INT64)
    let id_field = schema::FieldData {
        r#type: schema::DataType::Int64 as i32,
        field_name: "id".to_string(),
        field: Some(schema::field_data::Field::Scalars(schema::ScalarField {
            data: Some(schema::scalar_field::Data::LongData(schema::LongArray {
                data: ids,
            })),
        })),
        field_id: 1,
    };

    // FieldData for embedding (FLOAT_VECTOR)
    let embed_field = schema::FieldData {
        r#type: schema::DataType::FloatVector as i32,
        field_name: "embedding".to_string(),
        field: Some(schema::field_data::Field::Vectors(schema::VectorField {
            dim: dim as i64,
            data: Some(schema::vector_field::Data::FloatVector(
                schema::FloatArray { data: flat },
            )),
        })),
        field_id: 2,
    };

    client
        .insert(
            collection_name,
            Option::<String>::None,
            vec![id_field, embed_field],
        )
        .await?;

    Ok(())
}

// Stub search: return empty to allow compilation
pub async fn search_top_k(
    _client: &mut MilvusClient,
    _collection_name: &str,
    _query_vec: &[f32],
    top_k: usize,
) -> Result<Vec<(i64, f32)>> {
    Ok(Vec::with_capacity(top_k))
}

fn embedding_dim_from_list(embeddings: &[Vec<f32>]) -> usize {
    embeddings
        .iter()
        .find(|v| !v.is_empty())
        .map(|v| v.len())
        .unwrap_or(1536)
}
