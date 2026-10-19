// Theme before first paint: saved choice, else follow the system.
(function () {
  var s = null;
  try { s = localStorage.getItem('scheme'); } catch (e) {}
  var sys = matchMedia('(prefers-color-scheme: dark)').matches ? 'dark' : 'light';
  document.documentElement.dataset.scheme = (s === 'dark' || s === 'light') ? s : sys;
})();
