// brahma's service worker: the app works offline after its first visit.
//
// Trunk names the wasm and js by content hash, so they are immutable: cache-first. The page
// itself (index.html, the manifest, the font manifest) is small and changes with every
// deployment: network-first, falling back to the cached copy when offline. Nothing else
// (fonts the host serves, user data) is touched: fonts are cached on first use like the wasm;
// documents never go through the network.
//
// After a new deployment the new index.html references new hashed files; the old ones are
// dropped from the cache on the next activation so the cache holds one version.
const CACHE = 'brahma-site-v1';
const IMMUTABLE = /\.(wasm|js|woff2?|ttf|otf|png)$/;

// On install, cache the page and everything it references (the hashed js and wasm, the
// manifest, the icons), so the app starts offline after the very first visit, not the second.
self.addEventListener('install', (event) => {
  event.waitUntil((async () => {
    try {
      const cache = await caches.open(CACHE);
      const index = new URL('./index.html', self.location).href;
      const res = await fetch(index, { cache: 'no-cache' });
      if (!res.ok) return;
      await cache.put(index, res.clone());
      const html = await res.text();
      const urls = new Set();
      for (const m of html.matchAll(/(?:href|src)="([^"]+)"/g)) {
        const u = new URL(m[1], self.location);
        if (u.origin === self.location.origin && /\.(js|wasm|webmanifest|png)$/.test(u.pathname)) urls.add(u.href);
      }
      await Promise.all([...urls].map(async (u) => {
        try { const r = await fetch(u); if (r.ok) await cache.put(u, r); } catch (e) { /* cached on first use instead */ }
      }));
    } catch (e) {
      // Offline during install: the fetch handler fills the cache on first use.
    }
    await self.skipWaiting();
  })());
});

self.addEventListener('activate', (event) => {
  event.waitUntil((async () => {
    await self.clients.claim();
    try {
      const cache = await caches.open(CACHE);
      const page = await cache.match('./index.html') || await cache.match(new URL('./', self.location).href);
      if (!page) return;
      const html = await page.text();
      const referenced = new Set();
      for (const m of html.matchAll(/(?:href|src)="([^"]+)"/g)) referenced.add(new URL(m[1], self.location).href);
      for (const req of await cache.keys()) {
        const url = new URL(req.url);
        if (/\.(wasm|js)$/.test(url.pathname) && !referenced.has(req.url)) await cache.delete(req);
      }
    } catch (e) {
      // A pruning failure only costs disk space.
    }
  })());
});

self.addEventListener('fetch', (event) => {
  const req = event.request;
  if (req.method !== 'GET') return;
  const url = new URL(req.url);
  if (url.origin !== self.location.origin) return;
  if (IMMUTABLE.test(url.pathname)) {
    event.respondWith(cacheFirst(req));
  } else {
    event.respondWith(networkFirst(req));
  }
});

async function cacheFirst(req) {
  const cache = await caches.open(CACHE);
  const hit = await cache.match(req);
  if (hit) return hit;
  const res = await fetch(req);
  if (res.ok) cache.put(req, res.clone()).catch(() => {});
  return res;
}

async function networkFirst(req) {
  const cache = await caches.open(CACHE);
  try {
    const res = await fetch(req);
    if (res.ok) cache.put(req, res.clone()).catch(() => {});
    return res;
  } catch (e) {
    const hit = await cache.match(req);
    if (hit) return hit;
    // A navigation while offline: serve the app shell.
    if (req.mode === 'navigate') {
      const shell = await cache.match('./index.html') || await cache.match(new URL('./', self.location).href);
      if (shell) return shell;
    }
    return Response.error();
  }
}
