// PicoCRM Service Worker
// Cache strategy:
//   - Static assets (JS/CSS/fonts/images): Cache-first
//   - API calls: Network-first
//   - Vendor scripts: Cache-first

const CACHE_NAME = 'pico-crm-v2';
const STATIC_ASSETS = [
  '/',
  '/vendor/flatpickr.min.css',
  '/vendor/flatpickr.min.js',
  '/vendor/zh.js',
  '/vendor/flyonui.js',
  '/manifest.json',
];

// Install: pre-cache core static assets
self.addEventListener('install', (event) => {
  event.waitUntil(
    caches.open(CACHE_NAME).then((cache) => {
      return cache.addAll(STATIC_ASSETS).catch(() => {
        // Don't fail installation if some assets are unavailable
      });
    }).then(() => {
      return self.skipWaiting();
    })
  );
});

// Activate: clean up old caches
self.addEventListener('activate', (event) => {
  event.waitUntil(
    caches.keys().then((keys) => {
      return Promise.all(
        keys
          .filter((key) => key !== CACHE_NAME)
          .map((key) => caches.delete(key))
      );
    }).then(() => {
      return self.clients.claim();
    })
  );
});

// Fetch: cache-first for static assets, network-first for API requests
self.addEventListener('fetch', (event) => {
  const url = new URL(event.request.url);

  // Skip non-GET requests (API mutations)
  if (event.request.method !== 'GET') {
    return;
  }

  // Skip browser extension requests
  if (!url.protocol.startsWith('http')) {
    return;
  }

  // Leptos build artifacts under /pkg can change between refreshes while keeping
  // stable filenames in dev. Always prefer network here to avoid JS/WASM mismatch.
  if (url.pathname.startsWith('/pkg/')) {
    event.respondWith(networkFirst(event.request));
    return;
  }

  // Cache-first for static assets: CSS, JS, fonts, images, vendor files
  if (
    url.pathname.match(/\.(css|js|woff2?|ttf|eot|svg|png|jpg|jpeg|gif|ico)$/) ||
    url.pathname.startsWith('/vendor/') ||
    url.pathname.startsWith('/icons/')
  ) {
    event.respondWith(cacheFirst(event.request));
    return;
  }

  // Cache-first for navigation requests (HTML)
  if (event.request.mode === 'navigate') {
    event.respondWith(networkFirst(event.request));
    return;
  }

  // Network-first for everything else (API calls)
  event.respondWith(networkFirst(event.request));
});

// Cache-first strategy
async function cacheFirst(request) {
  const cached = await caches.match(request);
  if (cached) {
    return cached;
  }
  try {
    const response = await fetch(request);
    if (response.status === 200) {
      const cache = await caches.open(CACHE_NAME);
      cache.put(request, response.clone());
    }
    return response;
  } catch (e) {
    return new Response('Offline', { status: 503 });
  }
}

// Network-first strategy
async function networkFirst(request) {
  try {
    const response = await fetch(request);
    if (response.status === 200) {
      const cache = await caches.open(CACHE_NAME);
      cache.put(request, response.clone());
    }
    return response;
  } catch (e) {
    const cached = await caches.match(request);
    if (cached) {
      return cached;
    }
    return new Response('Offline', { status: 503 });
  }
}
