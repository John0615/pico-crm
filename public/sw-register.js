if ('serviceWorker' in navigator) {
  const isLocalDevHost = ['localhost', '127.0.0.1', '[::1]'].includes(window.location.hostname);

  if (isLocalDevHost) {
    navigator.serviceWorker.getRegistrations()
      .then((registrations) => Promise.all(registrations.map((registration) => registration.unregister())))
      .then(() => caches.keys())
      .then((keys) => Promise.all(keys.map((key) => caches.delete(key))))
      .then(() => console.log('PWA: Service Worker disabled on local dev'))
      .catch((e) => console.warn('PWA: failed to disable SW on local dev', e));
  } else {
    navigator.serviceWorker.register('/service-worker.js')
      .then(() => console.log('PWA: Service Worker registered'))
      .catch(e => console.warn('PWA: SW registration failed', e));
  }
}
