# Cara menjalankan aplikasi backend

# Prequisaite
1. Install Rust Programming Language https://www.rust-lang.org/tools/install
2. Install Podman https://podman.io/getting-started/installation
3. Install Postman https://www.postman.com/

# Keterangan
Makefile adalah daftar perintah yang sudah disederhanakan

# Step by step jika baru downlaod dari github
1. make install
2. make podman-dev-up
3. make migrate-up
4. make start-server
5. Done and test in postman

# Step by step jika ada update
1. make install
2. make podman-dev-up
3. make migrate-down
4. make migrate-up
5. make start-server
6. Done and test in postman

## Seed Data (setelah migrate)

Setelah `sqlx migrate run`, beberapa data master perlu di-seed agar fitur berjalan lengkap.

### 1) Regions master (province, regency, district, village)
- Env: gunakan `REGION_CSV_PATH` (default `data/regions.csv`).
- Local:
  - `cargo run --bin seed_regions`
- Container (Podman): otomatis dijalankan oleh entrypoint jika `RUN_SEED=true`.
  - Variabel bawaan image: `RUN_SEED=true`, `REGION_CSV_PATH=/app/data/regions.csv`.
- Verifikasi:
  - `GET /api/v1/regions/provinces`
  - `GET /api/v1/regions/regencies?province_code=<kode_provinsi>`
  - `GET /api/v1/regions/districts?regency_code=<kode_kabkot>`
  - `GET /api/v1/regions/villages?district_code=<kode_kecamatan>`

### 2) BMKG area + priorities
- Pilihan A (dari master DB regions):
  - Jalankan: `cargo run --bin seed_bmkg_area` (butuh data regions sudah ter-seed).
- Pilihan B (dari CSV):
  - Env: `BMKG_CSV_PATH` (default `data/regions.csv`), `BMKG_PRIORITIES_CSV_PATH` (default `data/priorities.csv`).
  - Jalankan: `cargo run --bin seed_bmkg_area_from_csv` (mengisi `bmkg_area` dan `bmkg_area_priority`).
  - Jika `data/priorities.csv` belum ada, bisa dibuat: `cargo run --bin generate_priorities_csv`.
- Container (Podman): entrypoint mendukung auto seeding jika `RUN_BMKG_SEED=true`.
  - Variabel bawaan image: `RUN_BMKG_SEED=true`, `BMKG_CSV_PATH=/app/data/regions.csv`, `BMKG_PRIORITIES_CSV_PATH=/app/data/priorities.csv`, `RUN_GENERATE_PRIORITIES=false`.
- Verifikasi:
  - `GET /api/v1/weather_bmkg/regions` (daftar area BMKG/adm4)
  - `GET /api/v1/weather_bmkg/priorities` (daftar prioritas aktif)
  - Lihat juga `PRIORITIES.md` untuk contoh penggunaan dan CLI helper `cargo run --bin show_bmkg_priority`.

### 3) Roles (wajib untuk relasi `profiles.roles_number`)
- Jika tabel `roles` masih kosong, isi data awal berikut (contoh menggunakan psql):
```
INSERT INTO roles (number, name) VALUES
  (1, 'Super Admin'),
  (2, 'Admin'),
  (3, 'Chief Operating Officer (COO)'),
  (4, 'Supervisor'),
  (5, 'Manager'),
  (6, 'Staff'),
  (7, 'Owner'),
  (8, 'Finance');
```
- Verifikasi:
  - `GET /api/v1/roles` (perlu token akses karena endpoint dilindungi JWT)

### Catatan
- Pastikan `.env` berisi `DATABASE_URL` yang benar.
- Jalankan seeding setelah migrasi selesai dan service database siap.
- Di environment container, seeding dapat dikontrol via variabel: `RUN_SEED`, `RUN_BMKG_SEED`, `RUN_GENERATE_PRIORITIES`.

## RAG API + Milvus

> Fitur ini memanfaatkan Milvus (via `compose.yml`) untuk menyimpan embedding `document_chunks`. Pastikan service `vcai-etcd`, `vcai-minio`, `vcai-pulsar`, dan `vcai-milvus` sedang berjalan.

## Trend News API (Serper.dev)

- Migrasi: jalankan `sqlx migrate run` untuk membuat tabel `trend_news_sources` dan `trend_news_articles`.
- Konfigurasi: set `SERPER_API_KEY` pada `.env` (opsional: `SERPER_BASE_URL`, `SERPER_DEFAULT_GL`, `SERPER_DEFAULT_HL`).
- Endpoint:
  - `GET /api/trend-news` → ambil daftar berita F&B terbaru dari database (query opsional: `page`, `limit`, `source`, `category`, `country`, `language`, `has_image`, `refresh=true`, `refresh_with_images=true`).
  - `POST /api/trend-news/sync` → paksa fetch berita baru dari Serper.dev lalu simpan ke database.
- Response mencakup judul, ringkasan, isi singkat, tautan sumber, gambar (jika tersedia), dan metadata sumber API.

1. Jalankan `podman compose up vcai-milvus` (atau `make podman-dev-up` untuk seluruh stack).
2. Isi variabel `.env` berikut:
   - `MILVUS_URI=http://127.0.0.1:19530` (ganti ke `http://vcai-milvus:19530` bila dijalankan di dalam container).
   - `MILVUS_COLLECTION=rag_chunks` (default sudah cocok dengan kode).
   - `MILVUS_TOKEN=` hanya diperlukan jika Milvus diamankan.
3. Jalankan `sqlx migrate run` untuk menerapkan migrasi terbaru (`20251027131500_update_rag_supported_file_types` menambahkan dukungan Excel).
4. Start server (`make start-server`). Saat boot, aplikasi akan otomatis:
   - Membaca konfigurasi RAG dari tabel `rag_configurations`.
   - Membuat koleksi Milvus jika belum ada.
   - Mengirim embedding chunk ke Milvus setiap kali dokumen baru diproses.

### Upload dokumen (Word / Excel)

Endpoint: `POST /api/rag/documents/upload` (multipart).

Form fields yang wajib:

| Field       | Keterangan                                               |
|-------------|----------------------------------------------------------|
| `title`     | Judul dokumen                                            |
| `category`  | Salah satu kategori yang ada (misal `Sales`)             |
| `tags`      | Dipisahkan koma, contoh `sales,q3`                       |
| `file`      | File `.docx`, `.xlsx`, `.xls`, `.xlsm`, juga `pdf/txt/md/csv` |

Contoh cURL:

```bash
curl -X POST http://localhost:4000/api/rag/documents/upload \
  -H "Authorization: Bearer <token>" \
  -F "title=Laporan Penjualan" \
  -F "category=Sales" \
  -F "tags=sales,q3,laporan" \
  -F "file=@data/laporan-q3.xlsx"
```

Aplikasi akan:
1. Mengekstrak teks dari Word (`docx`) menggunakan parser XML.
2. Mengekstrak seluruh sel dari Excel (`xlsx/xls/xlsm`) lewat `calamine`.
3. Melakukan chunking sesuai konfigurasi (`chunk_size`, `chunk_overlap`).
4. Membuat embedding lokal lalu menyimpan:
   - Metadata chunk ke tabel `document_chunks` (kolom `embedding` sengaja dikosongkan agar Postgres tidak menyimpan vektor).
   - Seluruh vektor ke koleksi Milvus (`rag_chunks`) sehingga hanya vector DB yang menyimpan embedding.

> Catatan: Jika Milvus tidak tersedia, proses unggah/ingest akan gagal. Hal ini memastikan semua embedding hanya hidup di vector database.

Status proses dapat dicek via `GET /api/rag/documents/{document_id}/status`.

### Query RAG

Endpoint: `POST /api/rag/query`

Payload contoh:

```json
{
  "query": "Bagaimana performa penjualan Q3?",
  "category_filter": "Sales",
  "max_results": 5
}
```

Respon akan memuat (seluruh perhitungan kemiripan diambil langsung dari Milvus):
- `answer`: ringkasan hasil dengan potongan konten dokumen.
- `sources`: daftar chunk yang relevan (termasuk judul dokumen, skor kemiripan, dan metadata).
- `confidence_score`: rata-rata skor kemiripan.

Gunakan informasi ini untuk membangun fitur RAG di UI atau integrasi internal.
