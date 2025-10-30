// static/sw.js
// [ID] No-op service worker untuk dev; aman dihapus saat produksi.
// [EN] No-op SW for dev; safe to remove for production.
self.addEventListener('install', () => self.skipWaiting());
self.addEventListener('activate', () => self.clients.claim());
