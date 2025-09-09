/**
 * Service Worker for Ruchy Notebook WASM Caching
 * Implements progressive loading and offline support
 */

const CACHE_NAME = 'ruchy-notebook-v1.90.0';
const WASM_CACHE = 'ruchy-wasm-v1.90.0';

// Files to cache for offline operation
const STATIC_FILES = [
    './ruchy-notebook.js',
    './ruchy-worker.js',
    './styles.css',
    './pkg/ruchy_notebook.js',
    './pkg/ruchy_notebook_bg.wasm',
    './pkg/ruchy_notebook_bg.wasm.d.ts',
    './manifest.json'
];

// Large WASM files that need progressive loading
const WASM_FILES = [
    './pkg/ruchy_notebook_bg.wasm'
];

self.addEventListener('install', (event) => {
    console.log('Service Worker: Installing...');
    
    event.waitUntil(
        Promise.all([
            // Cache static files
            caches.open(CACHE_NAME).then((cache) => {
                console.log('Service Worker: Caching static files');
                return cache.addAll(STATIC_FILES.filter(file => !WASM_FILES.includes(file)));
            }),
            
            // Cache WASM files separately with progressive loading
            caches.open(WASM_CACHE).then((cache) => {
                console.log('Service Worker: Caching WASM files');
                return Promise.all(WASM_FILES.map(file => 
                    cacheWasmFile(cache, file)
                ));
            })
        ])
    );
});

self.addEventListener('activate', (event) => {
    console.log('Service Worker: Activating...');
    
    event.waitUntil(
        caches.keys().then((cacheNames) => {
            return Promise.all(
                cacheNames.map((cacheName) => {
                    if (cacheName !== CACHE_NAME && cacheName !== WASM_CACHE) {
                        console.log('Service Worker: Deleting old cache', cacheName);
                        return caches.delete(cacheName);
                    }
                })
            );
        })
    );
});

self.addEventListener('fetch', (event) => {
    const url = new URL(event.request.url);
    
    // Handle WASM files with progressive loading
    if (WASM_FILES.some(file => url.pathname.endsWith(file))) {
        event.respondWith(handleWasmRequest(event.request));
        return;
    }
    
    // Handle other requests with cache-first strategy
    event.respondWith(
        caches.match(event.request).then((response) => {
            if (response) {
                return response;
            }
            
            return fetch(event.request).then((response) => {
                // Don't cache non-successful responses
                if (!response || response.status !== 200 || response.type !== 'basic') {
                    return response;
                }
                
                // Clone response for caching
                const responseToCache = response.clone();
                
                caches.open(CACHE_NAME).then((cache) => {
                    cache.put(event.request, responseToCache);
                });
                
                return response;
            });
        })
    );
});

async function cacheWasmFile(cache, filePath) {
    try {
        const response = await fetch(filePath);
        if (response.ok) {
            await cache.put(filePath, response);
            console.log(`Service Worker: Cached WASM file ${filePath}`);
        }
    } catch (error) {
        console.warn(`Service Worker: Failed to cache WASM file ${filePath}:`, error);
    }
}

async function handleWasmRequest(request) {
    const cache = await caches.open(WASM_CACHE);
    const cachedResponse = await cache.match(request);
    
    if (cachedResponse) {
        console.log('Service Worker: Serving WASM from cache');
        return cachedResponse;
    }
    
    try {
        console.log('Service Worker: Fetching WASM from network');
        const response = await fetch(request);
        
        if (response.ok) {
            // Cache successful response
            const responseToCache = response.clone();
            await cache.put(request, responseToCache);
        }
        
        return response;
    } catch (error) {
        console.error('Service Worker: Failed to fetch WASM:', error);
        
        // Return a fallback response if available
        return new Response('WASM module unavailable', {
            status: 503,
            statusText: 'Service Unavailable'
        });
    }
}

// Handle messages from main thread
self.addEventListener('message', (event) => {
    if (event.data && event.data.type === 'SKIP_WAITING') {
        self.skipWaiting();
    }
    
    if (event.data && event.data.type === 'CACHE_STATS') {
        getCacheStats().then(stats => {
            event.ports[0].postMessage({
                type: 'CACHE_STATS_RESPONSE',
                data: stats
            });
        });
    }
});

async function getCacheStats() {
    const cacheNames = await caches.keys();
    const stats = {};
    
    for (const cacheName of cacheNames) {
        const cache = await caches.open(cacheName);
        const keys = await cache.keys();
        stats[cacheName] = {
            fileCount: keys.length,
            files: keys.map(req => req.url)
        };
    }
    
    return stats;
}

// Periodic cache cleanup
self.addEventListener('periodicsync', (event) => {
    if (event.tag === 'cache-cleanup') {
        event.waitUntil(cleanupOldCaches());
    }
});

async function cleanupOldCaches() {
    const cacheNames = await caches.keys();
    const currentCaches = [CACHE_NAME, WASM_CACHE];
    
    const deletePromises = cacheNames
        .filter(cacheName => !currentCaches.includes(cacheName))
        .map(cacheName => {
            console.log('Cleaning up old cache:', cacheName);
            return caches.delete(cacheName);
        });
    
    await Promise.all(deletePromises);
}