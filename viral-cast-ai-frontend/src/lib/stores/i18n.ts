import { browser } from '$app/environment';
import { invalidateAll } from '$app/navigation';
import { writable, get } from 'svelte/store';

export const supportedLocales = ['en', 'id'] as const;
export type Locale = (typeof supportedLocales)[number];

const DEFAULT_LOCALE: Locale = 'id';

// Global locale store (SSR-safe)
export const locale = writable<Locale>(DEFAULT_LOCALE);

// Initialize from document.lang or localStorage
if (browser) {
  try {
    const docLang = (globalThis as any)?.document?.documentElement?.lang as string | undefined;
    const saved = (globalThis as any)?.localStorage?.getItem('locale') as Locale | null;
    const initial = (saved && supportedLocales.includes(saved))
      ? saved
      : (supportedLocales.includes(docLang as Locale) ? (docLang as Locale) : DEFAULT_LOCALE);
    locale.set(initial);
  } catch {
    locale.set(DEFAULT_LOCALE);
  }

  locale.subscribe((l) => {
    try {
      (globalThis as any)?.localStorage?.setItem('locale', l);
      const d = (globalThis as any)?.document?.documentElement;
      if (d) d.lang = l;
    } catch {}
  });
}

export function setLocale(l: Locale) {
  locale.set(l);
  // Optional: sync <html lang> for a11y/SEO and consistency
  try {
    (globalThis as any)?.document?.documentElement &&
      ((globalThis as any).document.documentElement.lang = l);
  } catch {}

  // Note: we keep the current URL structure and only update text, because
  // localized path prefixes (e.g. /id/...) are not used in this project.
  if (browser) {
    // Re-run current page load so translated strings update without changing the URL.
    invalidateAll().catch(() => {});
  }
}

// Minimal in-app messages for EN/ID
type Messages = Record<string, string>;
const dict: Record<Locale, Messages> = {
  en: {
    menu: 'Menu',
    theme_light: 'Light',
    theme_dark: 'Dark',
    language: 'Language',

    // Navigation
    nav_dashboard: 'Dashboard',
    nav_products: 'Products',
    nav_categories: 'Categories',
    nav_recipes: 'Recipes',
    nav_ingredients: 'Ingredients',
    nav_uoms: 'UOMs',
    master: 'Master',

    // Dashboard
    dashboard_overview: 'Dashboard Overview',
    revenue_trend: 'Revenue Trend',

    // Common columns & actions
    col_image: 'Image',
    col_name: 'Name',
    col_sku: 'SKU',
    col_price: 'Price',
    col_status: 'Status',
    col_stock: 'Stock',
    col_minimum_stock: 'Minimum Stock',
    col_code: 'Code',
    col_created_at: 'Created At',
    col_updated_at: 'Updated At',
    col_actions: 'Actions',
    edit: 'Edit',
    delete: 'Delete',
    view: 'View',
    detail: 'Detail',
    yes: 'Yes',
    no: 'No',
    clear: 'Clear',
    no_image: 'No Image',
    not_available: 'Not available',
    back: 'Back',
    press_enter_to_search: 'Press Enter to search',

    // Generic form labels & placeholders
    form_label_name: 'Name',
    form_label_code: 'Code',
	form_label_base_uom: 'Base UOM',
	form_label_stock: 'Stock',
	form_label_shelf_life_days: 'Shelf Life (Days)',
	form_label_description: 'Description',
	placeholder_name: 'Enter name',
	placeholder_code: 'Enter code',
	select_uom: 'Select UOM',
	placeholder_stock: 'e.g., 5.0, 10.5',
	placeholder_shelf_life_days: 'e.g., 7, 30, 365',
	placeholder_description: 'Enter description (optional)',
	placeholder_notes_optional: 'Enter notes (optional)',

    // Products page
    products_title: 'Products',
    add_product: 'Add Product',
    showing_products: 'Showing {count} products',
    showing_filtered_products: 'Showing {shown} of {total} products for "{query}"',

    // Ingredients page
    ingredients_title: 'Ingredients Catalog',
    add_ingredient: 'Add Ingredient',
    new_ingredient_title: 'New Ingredient',
    edit_ingredient_title: 'Edit Ingredient',
    col_base_uom: 'Base UOM',
    col_min_stock: 'Stock',
    col_shelf_life_days: 'Shelf Life (Days)',
    showing_ingredients: 'Showing {count} ingredients',
    showing_filtered_ingredients: 'Showing {shown} of {total} ingredients for "{query}"',
    no_ingredients_available: 'No ingredients available',
    no_ingredients_match_search: 'No ingredients match your search',

    // Ingredient Stocks page
    ingredient_stocks_title: 'Ingredient Stocks',
    add_ingredient_stock: 'Add Ingredient Stock',
    new_ingredient_stock_title: 'New Ingredient Stock',
    edit_ingredient_stock_title: 'Edit Ingredient Stock',
    showing_ingredient_stocks: 'Showing {count} ingredient stocks',
    showing_filtered_ingredient_stocks: 'Showing {shown} of {total} ingredient stocks for "{query}"',
    no_ingredient_stocks_available: 'No ingredient stocks available',
    no_ingredient_stocks_match_search: 'No ingredient stocks match your search',
    delete_failed_stock: 'Failed to delete ingredient stock',
    delete_error_stock: 'Error deleting ingredient stock',
    col_total_quantity: 'Total Quantity',
    col_total_value: 'Total Value',
    col_current_cost: 'Current Cost',
    col_avg_cost: 'Average Cost',

    // Ingredient Stock Moves page
    ingredient_stock_moves_title: 'Ingredient Stock Moves',
    ingredient_stock_moves_subtitle: 'Manage ingredient stock movements',
    new_ingredient_stock_move_title: 'New Ingredient Stock Move',
    add_ingredient_stock_move: 'Add Stock Move',
    edit_ingredient_stock_move_title: 'Edit Ingredient Stock Move',
    detail_ingredient_stock_move_title: 'Ingredient Stock Move Detail',
    stock_move_search_label: 'Search',
    stock_move_search_placeholder: 'Enter keywords...',
    filter_all_types: 'All Types',
    filter_all_ingredients: 'All Ingredients',
    filter_all_stores: 'All Stores',
    move_type: 'Move Type',
    ingredient: 'Ingredient',
    move_type_production: 'Production',
    move_type_adjustment: 'Adjustment',
    move_type_waste: 'Waste',
    move_type_return: 'Return',
    move_type_purchase: 'Purchase',
    stock_move_from_date: 'From date',
    stock_move_to_date: 'To date',
    select_store: 'Select Store',
    select_ingredient: 'Select Ingredient',
    select_move_type: 'Select Type',
    quantity: 'Quantity',
    apply_filters: 'Apply Filters',
    no_stock_moves_found: 'No stock moves found',

    // Generic actions
    create: 'Create',
    update: 'Update',
    save: 'Save',
    saving: 'Saving...',

    // Delete confirmations
    delete_confirm_title_ingredient: 'Delete Ingredient',
    delete_confirm_message_ingredient: 'Are you sure you want to delete this ingredient? This action cannot be undone.',
    delete_confirm_title_uom: 'Delete Unit of Measurement',
    delete_confirm_message_uom: 'Are you sure you want to delete this unit of measurement? This action cannot be undone.',
    delete_confirm_title_stock: 'Delete Ingredient Stock',
    delete_confirm_message_stock: 'Are you sure you want to delete this ingredient stock? This action cannot be undone.',
    delete_confirm_title_stock_move: 'Delete Stock Move',
    delete_confirm_message_stock_move: 'Are you sure you want to delete this stock move? This action cannot be undone.',

    // UOMs page
    uoms_title: 'Units of Measurements',
    add_unit: 'Add Unit',
    new_uom_title: 'New Unit of Measurement',
    edit_uom_title: 'Edit Unit of Measurement',
    showing_uoms: 'Showing {count} units of measurements',
    showing_filtered_uoms: 'Showing {shown} of {total} units for "{query}"',
    no_uoms_available: 'No units of measurements available',
    no_uoms_match_search: 'No units of measurements match your search',

    // Recipes page
    recipes_title: 'Recipes',
    no_recipes_found: 'No recipes found',
    previous: 'Previous',
    next: 'Next',

    // Weather page
    weather: 'Weather',
    weather_intro:
      'Please select region: province → regency/city → district → village/subdistrict. Then click the button to view BMKG weather.',

    province: 'Province',
    regency_city: 'Regency/City',
    district: 'District',
    village_subdistrict: 'Village/Subdistrict',

    select_province: 'Select Province',
    select_regency_city: 'Select Regency/City',
    select_district: 'Select District',
    select_village_subdistrict: 'Select Village/Subdistrict',

    loading_provinces: 'Loading provinces...',
    loading_regencies: 'Loading regencies/cities...',
    loading_districts: 'Loading districts...',
    loading_villages: 'Loading villages/subdistricts...',

    failed_provinces: 'Failed to load provinces',
    failed_regencies: 'Failed to load regencies/cities',
    failed_districts: 'Failed to load districts',
    failed_villages: 'Failed to load villages/subdistricts',

    select_province_first: 'Select a province first',
    select_regency_first: 'Select a regency/city first',
    select_district_first: 'Select a district first',

    view_prediction: 'View Prediction',
    prediction_results_below: 'Weather prediction results will appear below.',
    results: 'Results',
    loading_prediction: 'Loading weather prediction...',
    failed_prediction: 'Failed to load weather prediction',
    region: 'Region',
    last_updated: 'Last updated',
    temperature: 'Temperature',
    humidity: 'Humidity',
    rainfall: 'Rainfall',
    wind_direction: 'Wind Direction',
    wind_speed: 'Wind Speed',
    total_cloud_cover: 'Total Cloud Cover',
    no_weather_forecast: 'No weather forecast data.',
    no_results_yet: 'No results yet. Select a region and press the button.',
    please_complete_selections: 'Please complete the region selections first.',
    more_details: 'More details',
    less_details: 'Hide details',
    essential_metrics: 'Essentials'
    ,
    risk: 'Risk',
    risk_rain: 'Rain ≥ {mm} mm',
    risk_heat: 'Heat ≥ {c}°C',
    risk_humidity: 'Humidity ≥ {percent}%'
    ,
    // Operational recommendation badges
    ops_canopy_queue: 'Provide umbrellas/canopy for queue area',
    ops_push_delivery: 'Promote delivery & takeaway',
    ops_add_ice: 'Add ice stock & cold drinks',
    ops_cooling_display: 'Strengthen cooling for food display',
    ops_protect_dry_goods: 'Protect dry goods from humidity',
    ops_secure_signage: 'Secure outdoor signage/canopy',
    // Settings page
    profile_title: 'Profile',
    profile_settings: 'Profile Settings',
    change_cover: 'Change Cover',
    change_photo: 'Change Photo',
    first_name: 'First Name',
    last_name: 'Last Name',
    gender: 'Gender',
    phone: 'Phone',
    birth_date: 'Birth Date',
    birth_place: 'Birth Place',
    save_profile: 'Save Profile',
    cancel: 'Cancel',
    store: 'Store',
    store_title: 'Store',
    store_name: 'Store Name',
    whatsapp: 'WhatsApp',
    instagram: 'Instagram',
    rt: 'RT',
    rw: 'RW',
    postal_code: 'Postal Code',
    brand_image: 'Brand Image',
    no_store_data: 'No store data',
    background: 'Background',
    user_avatar: 'User avatar',
    brand: 'Brand',
    male: 'Male',
    female: 'Female',
    other: 'Other',

    // AI Chat page
    ai_chat_page_title: 'AI Chat - Viral Cast AI',
    ai_chat_heading: '🤖 AI Chat',
    ai_chat_subheading: 'Chat with our AI assistant powered by Llama 3.1',
    ai_chat_rag_notice: 'RAG on – answers will use your selected documents.',
    ai_chat_toggle_on: 'RAG: ON',
    ai_chat_toggle_off: 'Use RAG',
    ai_chat_upload_label: 'Upload Document',
    ai_chat_uploading: 'Uploading...',
    ai_chat_settings_button: 'Settings',
    ai_chat_clear_button: 'Clear Chat',
    ai_chat_settings_title: 'Chat Settings',
    ai_chat_max_tokens_label: 'Max Tokens: {value}',
    ai_chat_max_tokens_help: 'Controls response length',
    ai_chat_temperature_label: 'Temperature: {value}',
    ai_chat_temperature_help: 'Controls creativity (0.1 = focused, 1.0 = creative)',
    ai_chat_welcome_title: 'Welcome to AI Chat',
    ai_chat_welcome_subtitle: 'Start a conversation by typing a message below.',
    ai_chat_loading: 'AI is thinking...',
    ai_chat_token_counter: '• {count} tokens',
    ai_chat_copy_tooltip: 'Copy message',
    ai_chat_input_placeholder:
      'Type your message here... (Press Enter to send, Shift+Enter for new line)',
    ai_chat_sending: 'Sending...',
    ai_chat_send: 'Send',
    ai_chat_footer_status: 'Max tokens: {tokens} | Temperature: {temperature}',
    ai_chat_message_count: '{count} messages',
    ai_chat_upload_description: 'Uploaded from AI chat interface',
    ai_chat_upload_failed: 'Document upload failed',
    ai_chat_upload_success: 'Document uploaded successfully.',
    ai_chat_upload_success_with_id: 'Document uploaded successfully (ID: {id}).',
    ai_chat_upload_error: 'Failed to upload document. Please try again.',
    ai_chat_no_rag_response: 'No response returned from RAG.',
    ai_chat_no_answer: 'No answer was generated.',
    ai_chat_error_rag_generate: 'Failed to generate a RAG answer',
    ai_chat_error_rag_fetch: 'Failed to get a RAG response',
    ai_chat_error_ai_fetch: 'Failed to get an AI response',
    ai_chat_error_send: 'Failed to send message. Please try again.',
    ai_chat_error_with_reason: 'Sorry, I ran into an error: {message}',

    // Error pages
    error_500_title: 'Internal Server Error',
    error_500_desc:
      'Sorry, something went wrong on our server. Our team has been notified and is fixing it.',
    error_details: 'Error Details',
    try_again: 'Try Again',
    go_home: 'Go Home'
  },
  id: {
    menu: 'Menu',
    theme_light: 'Terang',
    theme_dark: 'Gelap',
    language: 'Bahasa',

    // Navigasi
    nav_dashboard: 'Dasbor',
    nav_products: 'Produk',
    nav_categories: 'Kategori',
    nav_recipes: 'Resep',
    nav_ingredients: 'Bahan',
    nav_uoms: 'Satuan',
    master: 'Master',

    // Dashboard
    dashboard_overview: 'Ringkasan Dasbor',
    revenue_trend: 'Tren Pendapatan',

    // Kolom & aksi umum
    col_image: 'Gambar',
    col_name: 'Nama',
    col_sku: 'SKU',
    col_price: 'Harga',
    col_status: 'Status',
    col_stock: 'Stok',
    col_minimum_stock: 'Stok Minimum',
    col_code: 'Kode',
    col_created_at: 'Dibuat',
    col_updated_at: 'Diperbarui',
    col_actions: 'Aksi',
    edit: 'Ubah',
    delete: 'Hapus',
    view: 'Lihat',
    detail: 'Detail',
    yes: 'Ya',
    no: 'Tidak',
    clear: 'Bersihkan',
    no_image: 'Tidak ada gambar',
    not_available: 'Tidak tersedia',
    back: 'Kembali',
    press_enter_to_search: 'Tekan Enter untuk mencari',

    // Label & placeholder form umum
    form_label_name: 'Nama',
    form_label_code: 'Kode',
	form_label_base_uom: 'Satuan Dasar',
	form_label_stock: 'Stok',
	form_label_shelf_life_days: 'Masa Simpan (Hari)',
	form_label_description: 'Deskripsi',
	placeholder_name: 'Masukkan nama',
	placeholder_code: 'Masukkan kode',
	select_uom: 'Pilih UOM',
	placeholder_stock: 'contoh: 5.0, 10.5',
	placeholder_shelf_life_days: 'contoh: 7, 30, 365',
	placeholder_description: 'Masukkan deskripsi (opsional)',
	placeholder_notes_optional: 'Masukkan catatan (opsional)',

    // Halaman Produk
    products_title: 'Produk',
    add_product: 'Tambah Produk',
    showing_products: 'Menampilkan {count} produk',
    showing_filtered_products: 'Menampilkan {shown} dari {total} produk untuk "{query}"',

    // Halaman Bahan
    ingredients_title: 'Bahan',
    add_ingredient: 'Tambah Bahan',
    new_ingredient_title: 'Bahan Baru',
    edit_ingredient_title: 'Ubah Bahan',
    col_base_uom: 'Satuan Dasar',
    col_min_stock: 'Stok Min',
    col_shelf_life_days: 'Masa Simpan (Hari)',
    showing_ingredients: 'Menampilkan {count} bahan',
    showing_filtered_ingredients: 'Menampilkan {shown} dari {total} bahan untuk "{query}"',
    no_ingredients_available: 'Tidak ada bahan',
    no_ingredients_match_search: 'Tidak ada bahan yang cocok',

    // Halaman Stok Bahan
    ingredient_stocks_title: 'Stok Bahan',
    add_ingredient_stock: 'Tambah Stok Bahan',
    new_ingredient_stock_title: 'Stok Bahan Baru',
    edit_ingredient_stock_title: 'Ubah Stok Bahan',
    showing_ingredient_stocks: 'Menampilkan {count} stok bahan',
    showing_filtered_ingredient_stocks: 'Menampilkan {shown} dari {total} stok bahan untuk "{query}"',
    no_ingredient_stocks_available: 'Tidak ada stok bahan',
    no_ingredient_stocks_match_search: 'Tidak ada stok bahan yang cocok',
    delete_failed_stock: 'Gagal menghapus stok bahan',
    delete_error_stock: 'Terjadi kesalahan saat menghapus stok bahan',
    col_total_quantity: 'Total Kuantitas',
    col_total_value: 'Total Nilai',
    col_current_cost: 'Biaya Saat Ini',
    col_avg_cost: 'Biaya Rata-rata',

    // Halaman Pergerakan Stok Bahan
    ingredient_stock_moves_title: 'Pergerakan Stok Bahan',
    ingredient_stock_moves_subtitle: 'Kelola pergerakan stok bahan',
    new_ingredient_stock_move_title: 'Pergerakan Stok Bahan Baru',
    add_ingredient_stock_move: 'Tambah Pergerakan Stok',
    edit_ingredient_stock_move_title: 'Ubah Pergerakan Stok Bahan',
    detail_ingredient_stock_move_title: 'Detail Pergerakan Stok Bahan',
    stock_move_search_label: 'Pencarian',
    stock_move_search_placeholder: 'Masukkan kata kunci...',
    filter_all_types: 'Semua Tipe',
    filter_all_ingredients: 'Semua Bahan',
    filter_all_stores: 'Semua Toko',
    move_type: 'Tipe Pergerakan',
    ingredient: 'Bahan',
    move_type_production: 'Produksi',
    move_type_adjustment: 'Penyesuaian',
    move_type_waste: 'Limbah',
    move_type_return: 'Retur',
    move_type_purchase: 'Pembelian',
    stock_move_from_date: 'Tanggal mulai',
    stock_move_to_date: 'Tanggal akhir',
    select_store: 'Pilih Toko',
    select_ingredient: 'Pilih Bahan',
    select_move_type: 'Pilih Tipe',
    quantity: 'Jumlah',
    apply_filters: 'Terapkan Filter',
    no_stock_moves_found: 'Tidak ada data stock move yang ditemukan',

    // Aksi umum
    create: 'Buat',
    update: 'Perbarui',
    save: 'Simpan',
    saving: 'Menyimpan...',

    // Dialog konfirmasi hapus
    delete_confirm_title_ingredient: 'Hapus Bahan',
    delete_confirm_message_ingredient: 'Apakah Anda yakin ingin menghapus bahan ini? Tindakan ini tidak dapat dibatalkan.',
    delete_confirm_title_uom: 'Hapus Satuan',
    delete_confirm_message_uom: 'Apakah Anda yakin ingin menghapus satuan ini? Tindakan ini tidak dapat dibatalkan.',
    delete_confirm_title_stock: 'Hapus Stok Bahan',
    delete_confirm_message_stock: 'Apakah Anda yakin ingin menghapus stok bahan ini? Tindakan ini tidak dapat dibatalkan.',
    delete_confirm_title_stock_move: 'Hapus Pergerakan Stok',
    delete_confirm_message_stock_move: 'Apakah Anda yakin ingin menghapus pergerakan stok ini? Tindakan ini tidak dapat dibatalkan.',

    // Halaman Satuan
    uoms_title: 'Satuan Ukur',
    add_unit: 'Tambah Satuan',
    new_uom_title: 'Satuan Ukur Baru',
    edit_uom_title: 'Ubah Satuan Ukur',
    showing_uoms: 'Menampilkan {count} satuan',
    showing_filtered_uoms: 'Menampilkan {shown} dari {total} satuan untuk "{query}"',
    no_uoms_available: 'Tidak ada satuan',
    no_uoms_match_search: 'Tidak ada satuan yang cocok',

    // Halaman Resep
    recipes_title: 'Resep',
    no_recipes_found: 'Tidak ada resep',
    previous: 'Sebelumnya',
    next: 'Berikutnya',

    // Weather page
    weather: 'Cuaca',
    weather_intro:
      'Silakan pilih wilayah: provinsi → kabupaten/kota → kecamatan → desa/kelurahan. Setelah itu, klik tombol untuk melihat cuaca dari BMKG.',

    province: 'Provinsi',
    regency_city: 'Kabupaten/Kota',
    district: 'Kecamatan',
    village_subdistrict: 'Desa/Kelurahan',

    select_province: 'Pilih Provinsi',
    select_regency_city: 'Pilih Kabupaten/Kota',
    select_district: 'Pilih Kecamatan',
    select_village_subdistrict: 'Pilih Desa/Kelurahan',

    loading_provinces: 'Memuat provinsi...',
    loading_regencies: 'Memuat kabupaten/kota...',
    loading_districts: 'Memuat kecamatan...',
    loading_villages: 'Memuat desa/kelurahan...',

    failed_provinces: 'Gagal memuat provinsi',
    failed_regencies: 'Gagal memuat kabupaten/kota',
    failed_districts: 'Gagal memuat kecamatan',
    failed_villages: 'Gagal memuat desa/kelurahan',

    select_province_first: 'Pilih provinsi terlebih dahulu',
    select_regency_first: 'Pilih kabupaten/kota terlebih dahulu',
    select_district_first: 'Pilih kecamatan terlebih dahulu',

    view_prediction: 'Lihat Prakiraan',
    prediction_results_below: 'Hasil prediksi cuaca akan muncul di bawah.',
    results: 'Hasil',
    loading_prediction: 'Memuat prakiraan cuaca...',
    failed_prediction: 'Gagal memuat prakiraan cuaca',
    region: 'Wilayah',
    last_updated: 'Terakhir diperbarui',
    temperature: 'Suhu',
    humidity: 'Kelembapan',
    rainfall: 'Curah Hujan',
    wind_direction: 'Arah Angin',
    wind_speed: 'Kecepatan Angin',
    total_cloud_cover: 'Total Tutupan Awan',
    no_weather_forecast: 'Tidak ada data prakiraan cuaca.',
    no_results_yet: 'Belum ada hasil. Pilih wilayah dan tekan tombol.',
    please_complete_selections: 'Silakan lengkapi pilihan wilayah terlebih dahulu.',
    more_details: 'Lihat detail',
    less_details: 'Tutup detail',
    essential_metrics: 'Esensial'
    ,
    risk: 'Risiko',
    risk_rain: 'Hujan ≥ {mm} mm',
    risk_heat: 'Panas ≥ {c}°C',
    risk_humidity: 'Kelembapan ≥ {percent}%'
    ,
    // Lencana rekomendasi operasional
    ops_canopy_queue: 'Siapkan payung/kanopi area antre',
    ops_push_delivery: 'Dorong promo delivery & takeaway',
    ops_add_ice: 'Tambahkan stok es & minuman dingin',
    ops_cooling_display: 'Perkuat pendinginan area display makanan',
    ops_protect_dry_goods: 'Lindungi bahan kering dari lembap',
    ops_secure_signage: 'Amankan signage/kanopi outdoor',
    // Halaman Settings
    profile_title: 'Profil',
    profile_settings: 'Pengaturan Profil',
    change_cover: 'Ganti Cover',
    change_photo: 'Ganti Foto',
    first_name: 'Nama Depan',
    last_name: 'Nama Belakang',
    gender: 'Jenis Kelamin',
    phone: 'Telepon',
    birth_date: 'Tanggal Lahir',
    birth_place: 'Tempat Lahir',
    save_profile: 'Simpan Profil',
    cancel: 'Batal',
    store: 'Toko',
    store_title: 'Toko',
    store_name: 'Nama Toko',
    whatsapp: 'WhatsApp',
    instagram: 'Instagram',
    rt: 'RT',
    rw: 'RW',
    postal_code: 'Kode Pos',
    brand_image: 'Gambar Brand',
    no_store_data: 'Belum ada data toko.',
    background: 'Latar',
    user_avatar: 'Foto Profil',
    brand: 'Brand',
    male: 'Pria',
    female: 'Wanita',
    other: 'Lainnya',

    // Halaman AI Chat
    ai_chat_page_title: 'AI Chat - Viral Cast AI',
    ai_chat_heading: '🤖 AI Chat',
    ai_chat_subheading: 'Ngobrol dengan asisten AI berbasis Llama 3.1',
    ai_chat_rag_notice: 'RAG aktif – jawaban akan disesuaikan dengan dokumen pilihanmu.',
    ai_chat_toggle_on: 'RAG: AKTIF',
    ai_chat_toggle_off: 'Pakai RAG',
    ai_chat_upload_label: 'Unggah Dokumen',
    ai_chat_uploading: 'Mengunggah...',
    ai_chat_settings_button: 'Pengaturan',
    ai_chat_clear_button: 'Hapus Riwayat',
    ai_chat_settings_title: 'Pengaturan Chat',
    ai_chat_max_tokens_label: 'Maksimal Token: {value}',
    ai_chat_max_tokens_help: 'Atur panjang jawaban AI',
    ai_chat_temperature_label: 'Temperatur: {value}',
    ai_chat_temperature_help: 'Atur kreativitas (0.1 = fokus, 1.0 = kreatif)',
    ai_chat_welcome_title: 'Selamat datang di AI Chat',
    ai_chat_welcome_subtitle: 'Mulai percakapan dengan mengetik pesan di bawah ini.',
    ai_chat_loading: 'AI sedang berpikir...',
    ai_chat_token_counter: '• {count} token',
    ai_chat_copy_tooltip: 'Salin pesan',
    ai_chat_input_placeholder:
      'Tulis pesanmu di sini... (Enter untuk kirim, Shift+Enter untuk baris baru)',
    ai_chat_sending: 'Mengirim...',
    ai_chat_send: 'Kirim',
    ai_chat_footer_status: 'Maksimal token: {tokens} | Temperatur: {temperature}',
    ai_chat_message_count: '{count} pesan',
    ai_chat_upload_description: 'Diunggah dari antarmuka AI Chat',
    ai_chat_upload_failed: 'Gagal mengunggah dokumen',
    ai_chat_upload_success: 'Dokumen berhasil diunggah.',
    ai_chat_upload_success_with_id: 'Dokumen berhasil diunggah (ID: {id}).',
    ai_chat_upload_error: 'Gagal mengunggah dokumen. Silakan coba lagi.',
    ai_chat_no_rag_response: 'Tidak ada respons dari RAG.',
    ai_chat_no_answer: 'Tidak ada jawaban yang dihasilkan.',
    ai_chat_error_rag_generate: 'Gagal menghasilkan jawaban RAG',
    ai_chat_error_rag_fetch: 'Gagal mendapatkan respons RAG',
    ai_chat_error_ai_fetch: 'Gagal mendapatkan respons AI',
    ai_chat_error_send: 'Gagal mengirim pesan. Silakan coba lagi.',
    ai_chat_error_with_reason: 'Maaf, aku mengalami kendala: {message}',

    // Halaman error
    error_500_title: 'Kesalahan Server',
    error_500_desc:
      'Maaf, terjadi kesalahan pada server. Tim kami telah diberitahu dan sedang memperbaiki masalah ini.',
    error_details: 'Detail Error',
    try_again: 'Coba Lagi',
    go_home: 'Kembali ke Beranda'
  }
};

export function t(key: string, replacements?: Record<string, any>): string {
  const l = get(locale);
  const template = dict[l]?.[key] ?? dict[DEFAULT_LOCALE]?.[key] ?? key;
  return template.replace(/\{(\w+)\}/g, (_, k) => String(replacements?.[k] ?? ''));
}
