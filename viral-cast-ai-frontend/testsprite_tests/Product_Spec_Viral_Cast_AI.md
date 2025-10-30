# Spesifikasi Produk — Viral Cast AI Frontend

Versi: 1.0  
Tanggal: 2025-10-19  
Status: Draft

## 1. Ringkasan
Viral Cast AI Frontend adalah aplikasi SvelteKit untuk manajemen toko, konten, resep, dan integrasi AI/otomasi. Fokus fitur meliputi autentikasi, pengelolaan profil dan pengaturan, katalog produk, bahan, unit satuan (UOM), resep, pesanan, cuaca BMKG, serta UI/UX modern yang responsif.

## 2. Tujuan
- Memudahkan tim operasional dan konten mengelola inventori, resep, pesanan, dan promosi.
- Menghadirkan antarmuka cepat, aman, dan konsisten di desktop serta seluler.
- Menyediakan fondasi untuk otomasi dan integrasi AI pada konten dan rekomendasi.

## 3. Persona Pengguna
- Admin: mengelola konfigurasi, pengguna, hak akses, dan data master.
- Editor/Konten: mengelola resep, bahan, produk, dan publikasi konten.
- Operasional/Toko: menangani pesanan, stok, dan informasi cuaca operasional.

## 4. Lingkup
- Frontend SvelteKit yang terhubung ke Backend API (`API_BASE_URL`) default: `http://localhost:12000`.
- Preview lokal produksi tersedia di `http://localhost:4173` untuk QA/UI testing.

## 5. Fitur Utama
- Autentikasi
  - Login, logout; validasi melalui API `/api/v1/auth/login` dengan sesi berbasis token.
- Profil & Pengaturan
  - Halaman “Settings” dengan field non-edit awal, dapat diubah setelah klik tombol “Edit”.
- Manajemen Toko
  - Daftar toko, detail, pengaturan jam buka, dan status aktif/nonaktif.
- Produk
  - CRUD produk, unggah gambar, kategorisasi, dan penetapan harga.
- Bahan (Ingredients)
  - CRUD bahan, stok, supplier, dan keterkaitan dengan resep.
- UOM (Units of Measurement)
  - CRUD satuan, konversi, dan integrasi ke bahan/produk.
- Resep
  - CRUD resep, langkah, bahan terkait, porsi, dan biaya.
- Pesanan
  - Daftar pesanan, status, detail item, dan tindakan operasional.
- Cuaca BMKG
  - Pemilihan wilayah hingga desa/kelurahan; penarikan prakiraan cuaca operasional.
- UI/UX
  - Navigasi sidebar, header responsif, mode gelap/terang, aksesibilitas dasar.

## 6. Alur Fungsional Utama
- Login
  1. Pengguna menuju `/login`.
  2. Input kredensial, submit ke backend.
  3. Simpan token/sesi, redirect ke halaman utama atau `from` URL jika ada.
- Settings
  1. Field non-edit saat pertama kali dibuka.
  2. Klik “Edit” mengaktifkan input, simpan mengirim PATCH/PUT ke API.
- Produk
  1. Tambah produk: nama, kategori, harga, deskripsi, gambar (preview).
  2. Simpan; tampil di daftar, dapat diubah/hapus.
- Resep
  1. Buat resep, pilih bahan (link ke ingredients), set porsi & langkah.
  2. Simpan; tampil di daftar; integrasi biaya/perhitungan.
- Pesanan
  1. Lihat daftar; buka detail item.
  2. Tindakan: ubah status, catatan operasional.
- Cuaca
  1. Pilih provinsi → kota/kab → kecamatan → desa/kelurahan.
  2. Tampilkan prakiraan dari API BMKG terintegrasi.

## 7. Persyaratan Teknis
- Platform: SvelteKit, Node.js 20+, pnpm.
- Build/Preview:
  - `pnpm run build` untuk produksi.
  - `pnpm run preview -- --port 4173` untuk preview lokal.
- Lingkungan:
  - `API_BASE_URL` default `http://localhost:12000`.
  - `FRONTEND_URL` disesuaikan (contoh `http://localhost:4173` saat QA).
- Keamanan:
  - Token-based auth, proteksi rute, sanitasi input, dan CORS sesuai backend.
- Performa:
  - Lazy-loading komponen, caching ringan, dan minim render ulang.

## 8. Integrasi API (Contoh)
- Auth: `POST /api/v1/auth/login` → token/sesi.
- UOM: `GET/POST/PATCH /api/v1/uoms` dan `GET/PUT/PATCH /api/v1/uoms/{uuid}`.
- Cuaca BMKG: `GET /api/v1/weather_bmkg/prediction?region_code=...`.
- Resep/Produk/Bahan/Pesanan: endpoint CRUD sesuai skema backend.

## 9. Testing & Validasi
- Manual QA
  - Login dengan `admin_vcai_2` / `admin` di `http://localhost:4173/login`.
  - Buka Settings di `http://localhost:4173/settings`: verifikasi field non-edit, klik “Edit” menjadi editable, simpan perubahan.
- Otomatis (rencana)
  - Playwright E2E (login, navigasi, Settings toggle).
  - Laporan dikirim ke TestSprite MCP setelah stabil.
- Unit & Integration
  - Unit untuk util/stores; integration pada service API dan halaman kunci.

## 10. Error Handling
- Tampilkan pesan kesalahan ringkas, actionable, dan tidak mengekspos detail sensitif.
- Logging klien untuk debug (non-PII), fallback UI bila API down.

## 11. Kriteria Sukses
- Login berfungsi dan redirect berjalan.
- Halaman Settings sesuai perilaku: non-edit → klik “Edit” → bisa ubah → simpan.
- CRUD inti (produk, bahan, UOM, resep) bekerja tanpa error kritis.
- Pratinjau cuaca BMKG tampil sesuai pilihan wilayah.

## 12. Rencana Rilis
- Beta QA lokal (preview 4173).
- Penyesuaian UX & stabilisasi API.
- Integrasi laporan otomatis ke TestSprite MCP.

## 13. Catatan Implementasi
- `.dockerignore` mengecualikan file `.md` dari konteks build container—OK untuk dokumentasi repo.
- Gunakan `pnpm dlx md-to-pdf` untuk konversi PDF lokal; Chromium akan diunduh oleh Puppeteer bila diperlukan.

---
Dokumen ini ditempatkan di `testsprite_tests/Product_Spec_Viral_Cast_AI.md` agar mudah diambil oleh pipeline TestSprite MCP.