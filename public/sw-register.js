if ('serviceWorker' in navigator) {
  navigator.serviceWorker.register('/service-worker.js')
    .then(() => console.log('PWA: Service Worker registered'))
    .catch(e => console.warn('PWA: SW registration failed', e));
}
