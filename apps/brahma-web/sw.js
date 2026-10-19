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

self.addEventListener('install', () => self.skipWaiting());

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
