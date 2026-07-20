const SHELL_ASSETS = ['/', '/index.html', '/manifest.webmanifest', '/offline.html'];

export function registerServiceWorker() {
  if (!import.meta.env.PROD || !('serviceWorker' in navigator)) {
    return;
  }

  window.addEventListener('load', () => {
    void navigator.serviceWorker
      .register('/service-worker.js', { scope: '/' })
      .then(async () => {
        await navigator.serviceWorker.ready;
        postCurrentShellAssets();
      })
      .catch((error: unknown) => {
        console.warn('Service worker registration failed', error);
      });
  });
}

function postCurrentShellAssets() {
  const assets = currentShellAssets();
  const controller = navigator.serviceWorker.controller;

  if (controller) {
    controller.postMessage({ type: 'CACHE_APP_SHELL', assets });
    return;
  }

  navigator.serviceWorker.addEventListener(
    'controllerchange',
    () => {
      navigator.serviceWorker.controller?.postMessage({
        type: 'CACHE_APP_SHELL',
        assets
      });
    },
    { once: true }
  );
}

function currentShellAssets() {
  const assets = new Set(SHELL_ASSETS);

  for (const entry of performance.getEntriesByType('resource')) {
    if (!('name' in entry)) {
      continue;
    }

    const url = new URL(entry.name, window.location.origin);
    if (url.origin === window.location.origin && url.pathname.startsWith('/assets/')) {
      assets.add(url.pathname);
    }
  }

  return Array.from(assets);
}
