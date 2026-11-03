'use strict';
/* WhatsInside UI. Plain JS, no framework, no build step. */

const T = window.__TAURI__;
const invoke = T ? T.core.invoke : (cmd, args) => window.WIMock.invoke(cmd, args);
const REDUCED = matchMedia('(prefers-reduced-motion: reduce)').matches;

const $ = (s, el = document) => el.querySelector(s);
const $$ = (s, el = document) => [...el.querySelectorAll(s)];
const esc = (s) => String(s ?? '').replace(/[&<>"']/g, (c) => ({ '&': '&amp;', '<': '&lt;', '>': '&gt;', '"': '&quot;', "'": '&#39;' }[c]));
const sk = (n, d = 0) => n.toLocaleString('sk-SK', { maximumFractionDigits: d, minimumFractionDigits: d });

function bytes(b) {
  if (!b) return '0 B';
  const u = ['B', 'KB', 'MB', 'GB', 'TB'];
  let i = 0;
  while (b >= 1024 && i < u.length - 1) { b /= 1024; i++; }
  return `${sk(b, i === 0 || b >= 100 ? 0 : 1)} ${u[i]}`;
}
const rate = (bps) => (bps < 1024 ? '–' : `${bytes(bps)}/s`);
const plural = (n, one, few, many) => (n === 1 ? one : n >= 2 && n <= 4 ? few : many);
function age(days) {
  if (!days) return '';
  if (days < 1) return 'dnes';
  if (days < 31) return `pred ${days} ${plural(days, 'dňom', 'dňami', 'dňami')}`;
  const m = Math.round(days / 30.4);
  if (m < 12) return `pred ${m} ${plural(m, 'mesiacom', 'mesiacmi', 'mesiacmi')}`;
  const y = Math.floor(days / 365);
  return `pred ${y} ${plural(y, 'rokom', 'rokmi', 'rokmi')}`;
}
const debounce = (fn, ms) => { let t; return (...a) => { clearTimeout(t); t = setTimeout(() => fn(...a), ms); }; };

const ICON = {
  dir: '<svg viewBox="0 0 16 16"><path d="M1.8 4.2c0-.7.5-1.2 1.2-1.2h3l1.4 1.6H13c.7 0 1.2.5 1.2 1.2v6c0 .7-.5 1.2-1.2 1.2H3c-.7 0-1.2-.5-1.2-1.2z"/></svg>',
  file: '<svg viewBox="0 0 16 16"><path d="M4 1.8h5l3 3v8.4c0 .6-.4 1-1 1H4c-.6 0-1-.4-1-1V2.8c0-.6.4-1 1-1z"/><path d="M9 1.8v3h3"/></svg>',
  small: '<svg viewBox="0 0 16 16"><rect x="2" y="2" width="5" height="5" rx="1"/><rect x="9" y="2" width="5" height="5" rx="1"/><rect x="2" y="9" width="5" height="5" rx="1"/><rect x="9" y="9" width="5" height="5" rx="1"/></svg>',
  other: '<svg viewBox="0 0 16 16"><path d="M4 8h.01M8 8h.01M12 8h.01"/></svg>',
  reveal: '<svg viewBox="0 0 16 16"><path d="M9 2.5h4.5V7M13.5 2.5L7.5 8.5"/><path d="M12 9.5v3c0 .6-.4 1-1 1H3.5c-.6 0-1-.4-1-1V5c0-.6.4-1 1-1h3"/></svg>',
  drive: '<svg viewBox="0 0 16 16"><rect x="1.8" y="3" width="12.4" height="10" rx="2"/><path d="M1.8 9.5h12.4M11.2 11.3h.01"/></svg>',
  usb: '<svg viewBox="0 0 16 16"><rect x="4.5" y="1.8" width="7" height="5" rx="1"/><path d="M3.5 6.8h9v5.4c0 1.1-.9 2-2 2h-5c-1.1 0-2-.9-2-2z"/></svg>',
  check: '<svg viewBox="0 0 16 16"><path d="M3.5 8.5l3 3 6-7"/></svg>',
  game: '<svg viewBox="0 0 16 16"><path d="M5 5h6a3.5 3.5 0 0 1 3.4 4.3l-.5 2a1.8 1.8 0 0 1-3.1.8L9.5 10.6h-3l-1.3 1.5a1.8 1.8 0 0 1-3.1-.8l-.5-2A3.5 3.5 0 0 1 5 5z"/><path d="M5 7.2v2M4 8.2h2M10.8 7.6h.01M12 8.8h.01"/></svg>',
  app: '<svg viewBox="0 0 16 16"><rect x="2" y="2" width="12" height="12" rx="3"/><path d="M5.5 8h5M8 5.5v5"/></svg>',
  clock: '<svg viewBox="0 0 16 16"><circle cx="8" cy="8" r="6"/><path d="M8 4.5V8l2.3 1.5"/></svg>',
  broom: '<svg viewBox="0 0 16 16"><path d="M10.5 1.8L7.2 7.5"/><path d="M5 7.2l4.4 2.5-1.6 4.5c-2-.4-4.3-1.7-5.6-3.4z"/></svg>',
  search: '<svg viewBox="0 0 16 16"><circle cx="7" cy="7" r="4.6"/><path d="M10.5 10.5l3.5 3.5"/></svg>',
  x: '<svg viewBox="0 0 16 16"><path d="M4.5 4.5l7 7M11.5 4.5l-7 7"/></svg>',
  arrow: '<svg viewBox="0 0 16 16"><path d="M6 3.5L10.5 8 6 12.5"/></svg>',
  bolt: '<svg viewBox="0 0 16 16"><path d="M9 1.5L3.5 9h4l-1 5.5L12.5 7h-4z"/></svg>',
  ram: '<svg viewBox="0 0 16 16"><path d="M1.5 5h13v5h-13zM4 10v2M7 10v2M10 10v2M13 10v2"/></svg>',
  rocket: '<svg viewBox="0 0 16 16"><path d="M8 1.8v5.4"/><path d="M4.4 4.1a5.3 5.3 0 1 0 7.2 0"/></svg>',
  gpu: '<svg viewBox="0 0 16 16"><rect x="1.5" y="4" width="13" height="7.5" rx="1.5"/><circle cx="5.2" cy="7.8" r="1.9"/><circle cx="10.8" cy="7.8" r="1.9"/><path d="M3 11.5v2"/></svg>',
  warn: '<svg viewBox="0 0 16 16"><path d="M8 2.2l6.2 11H1.8z"/><path d="M8 6.5v3M8 11.5h.01"/></svg>',
  cpu: '<svg viewBox="0 0 16 16"><rect x="4" y="4" width="8" height="8" rx="1.6"/><path d="M6.5 1.5V4M9.5 1.5V4M6.5 12v2.5M9.5 12v2.5M1.5 6.5H4M1.5 9.5H4M12 6.5h2.5M12 9.5h2.5"/></svg>',
  board: '<svg viewBox="0 0 16 16"><rect x="2" y="2" width="12" height="12" rx="1.6"/><rect x="5" y="5" width="4" height="4" rx=".6"/><path d="M11 5v2M11 9v2M5 11h4"/></svg>',
  os: '<svg viewBox="0 0 16 16"><path d="M2 3.5l5-.7v4.7H2zM8 2.7l6-.9v5.7H8zM2 8.5h5v4.7l-5-.7zM8 8.5h6v5.7l-6-.9z"/></svg>',
  monitor: '<svg viewBox="0 0 16 16"><rect x="1.5" y="2.5" width="13" height="8.5" rx="1.5"/><path d="M6 14h4M8 11v3"/></svg>',
  video: '<svg viewBox="0 0 16 16"><rect x="1.5" y="3.5" width="9.5" height="9" rx="1.8"/><path d="M11 7l3.5-2v6L11 9"/></svg>',
  image: '<svg viewBox="0 0 16 16"><rect x="1.8" y="2.5" width="12.4" height="11" rx="1.8"/><circle cx="5.6" cy="6.2" r="1.2"/><path d="M2 12l3.8-3.5 2.6 2.3 2.4-2.1 3.4 3"/></svg>',
  audio: '<svg viewBox="0 0 16 16"><path d="M6 12.2V3.5l7-1.5v8.7"/><circle cx="4.3" cy="12.2" r="1.8"/><circle cx="11.3" cy="10.7" r="1.8"/></svg>',
  archive: '<svg viewBox="0 0 16 16"><rect x="2" y="2" width="12" height="12" rx="2"/><path d="M8 2v2M8 5v1.5M8 7.5V9"/><rect x="6.8" y="9.5" width="2.4" height="2.4" rx=".6"/></svg>',
  disc: '<svg viewBox="0 0 16 16"><circle cx="8" cy="8" r="6.2"/><circle cx="8" cy="8" r="1.6"/></svg>',
  exe: '<svg viewBox="0 0 16 16"><rect x="1.8" y="2.5" width="12.4" height="11" rx="1.8"/><path d="M1.8 5.5h12.4M4.5 8.5l2 1.5-2 1.5M8 11.5h3"/></svg>',
  gear: '<svg viewBox="0 0 16 16"><circle cx="8" cy="8" r="2.2"/><path d="M8 1.8v1.8M8 12.4v1.8M1.8 8h1.8M12.4 8h1.8M3.6 3.6l1.3 1.3M11.1 11.1l1.3 1.3M3.6 12.4l1.3-1.3M11.1 4.9l1.3-1.3"/></svg>',
  doc: '<svg viewBox="0 0 16 16"><path d="M4 1.8h5l3 3v8.4c0 .6-.4 1-1 1H4c-.6 0-1-.4-1-1V2.8c0-.6.4-1 1-1z"/><path d="M5.5 8h5M5.5 10.5h5M5.5 5.5h2"/></svg>',
  user: '<svg viewBox="0 0 16 16"><circle cx="8" cy="5.5" r="2.8"/><path d="M2.8 14c.6-2.7 2.7-4.3 5.2-4.3s4.6 1.6 5.2 4.3"/></svg>',
};
const VERDICT = {
  rgb: { label: 'RGB appka', cls: 'rgb' },
  optional: { label: 'Dá sa vypnúť', cls: 'optional' },
  peripheral: { label: 'Periféria', cls: 'peripheral' },
  needed: { label: 'Potrebné', cls: 'needed' },
  system: { label: 'Windows', cls: 'system' },
  unknown: null,
};
const badge = (v) => (VERDICT[v] ? `<span class="badge ${VERDICT[v].cls}">${VERDICT[v].label}</span>` : '');
const TAG = {
  game: { cls: 'game', icon: 'game' },
  app: { cls: 'app', icon: 'app' },
  app_data: { cls: 'appdata', icon: null },
  system: { cls: 'system', icon: null },
  cache: { cls: 'cache', icon: 'broom' },
};
function tagPill(tag, withLabel = false) {
  if (!tag) return '';
  const t = TAG[tag.kind] || TAG.system;
  const text = {
    game: tag.source || 'Hra',
    app: 'Program',
    app_data: 'Dáta appky',
    system: 'Windows',
    cache: 'Dá sa uvoľniť',
  }[tag.kind];
  return `<span class="badge tag-${t.cls}">${t.icon ? ICON[t.icon] : ''}${esc(withLabel ? tag.label : text)}</span>`;
}

/* ---------------------------------------------------------- window */
if (T) {
  const w = T.window.getCurrentWindow();
  $('#wmin').onclick = () => w.minimize();
  $('#wmax').onclick = () => w.toggleMaximize();
  $('#wclose').onclick = () => w.close();
} else {
  $('.winbtns').style.visibility = 'hidden';
}
const setIdle = () => document.body.classList.toggle('idle', document.hidden || !document.hasFocus());
addEventListener('blur', setIdle);
addEventListener('focus', setIdle);
document.addEventListener('visibilitychange', () => { setIdle(); views[current]?.visible?.(!document.hidden); });

// Logo pupil follows the cursor a little.
const pupil = $('.logo .pupil');
addEventListener('mousemove', (e) => {
  const r = pupil.ownerSVGElement.getBoundingClientRect();
  const dx = e.clientX - (r.left + r.width / 2), dy = e.clientY - (r.top + r.height / 2);
  const d = Math.hypot(dx, dy) || 1, k = Math.min(1.6, d / 60);
  pupil.style.transform = `translate(${(dx / d) * k}px, ${(dy / d) * k * 0.6}px)`;
}, { passive: true });

/* ---------------------------------------------------------- theme */
const themeOrder = ['auto', 'light', 'dark'];
const themeName = { auto: 'Téma: podľa systému', light: 'Téma: svetlá', dark: 'Téma: tmavá' };
let themeMode = (() => { try { return localStorage.getItem('scheme') || 'auto'; } catch { return 'auto'; } })();
function applyTheme() {
  const sys = matchMedia('(prefers-color-scheme: dark)').matches ? 'dark' : 'light';
  document.documentElement.dataset.scheme = themeMode === 'auto' ? sys : themeMode;
  $('#themeLabel').textContent = themeName[themeMode];
}
$('#theme').onclick = () => {
  themeMode = themeOrder[(themeOrder.indexOf(themeMode) + 1) % 3];
  try { themeMode === 'auto' ? localStorage.removeItem('scheme') : localStorage.setItem('scheme', themeMode); } catch {}
  applyTheme();
};
matchMedia('(prefers-color-scheme: dark)').addEventListener('change', applyTheme);
applyTheme();

/* ---------------------------------------------------------- small components */
function seg(el, onChange) {
  el.addEventListener('click', (e) => {
    const b = e.target.closest('button');
    if (!b || b.classList.contains('on')) return;
    $$('button', el).forEach((x) => x.classList.toggle('on', x === b));
    placeSeg(el);
    onChange(b.dataset.k);
  });
  requestAnimationFrame(() => placeSeg(el));
}
function placeSeg(el) {
  if (typeof el === 'string') el = $(el);
  if (!el) return;
  const on = $('button.on', el), knob = $('.knob', el);
  if (on && knob) { knob.style.width = `${on.offsetWidth}px`; knob.style.transform = `translateX(${on.offsetLeft}px)`; }
}

const tip = $('#tip');
function showTip(html, x, y) {
  tip.innerHTML = html;
  tip.classList.add('on');
  const r = tip.getBoundingClientRect();
  tip.style.left = `${Math.min(x + 14, innerWidth - r.width - 10)}px`;
  tip.style.top = `${Math.min(y + 16, innerHeight - r.height - 10)}px`;
}
const hideTip = () => tip.classList.remove('on');

function toast(text, kind = 'ok') {
  const el = document.createElement('div');
  el.className = `toast ${kind}`;
  el.innerHTML = `<span class="dot ${kind === 'ok' ? 'mint' : 'rec'}"></span>${esc(text)}`;
  $('#toasts').appendChild(el);
  setTimeout(() => { el.classList.add('out'); setTimeout(() => el.remove(), 260); }, 3200);
}

function setStatus(title, text, dot = 'mint', progress = null) {
  $('#statusTitle').textContent = title;
  $('#statusText').textContent = text;
  const d = $('#status .dot');
  d.className = `dot ${dot}${dot === 'amber' ? ' pulse' : ''}`;
  const bar = $('#statusBar');
  bar.hidden = progress === null;
  if (progress !== null) $('i', bar).style.width = `${Math.round(progress * 100)}%`;
}
const idleStatus = () => setStatus('Pripravený', 'Nič nemením bez opýtania a všetko sa dá vrátiť. Keď ma zavrieš, nebeží nič.');

/* ---------------------------------------------------------- navigation */
const views = {};
let current = null;
const navButtons = $$('#nav button');
function setView(name) {
  if (name === current) return;
  if (current) views[current]?.leave?.();
  current = name;
  navButtons.forEach((b, i) => {
    const on = b.dataset.view === name;
    b.classList.toggle('on', on);
    if (on) $('#nav .ind').style.transform = `translateY(${i * 34}px)`;
  });
  $$('.view').forEach((v) => v.classList.toggle('on', v.id === `v-${name}`));
  $('#main').scrollTop = 0;
  hideTip();
  views[name]?.enter?.();
}
navButtons.forEach((b) => (b.onclick = () => setView(b.dataset.view)));

/* ========================================================== STORAGE */
const st = {
  drives: [], status: null, listing: null, tab: 'dirs', top: [], caches: [], owners: null, old: [],
  timer: null, drive: null, colors: new Map(), rects: new Map(), query: '', results: null, flash: null,
};
const PALETTE = ['var(--blue)', 'var(--mint)', 'var(--lilac)', 'var(--amber)', 'var(--accent)'];

async function loadDrives() {
  st.drives = await invoke('storage_drives');
}

function driveCard(d, i) {
  const used = d.total - d.free;
  const pct = d.total ? used / d.total : 0;
  const cls = pct > 0.9 ? 'rec' : pct > 0.75 ? 'amber' : '';
  const name = d.label ? `${d.label} (${d.root.replace(/\\$/, '')})` : `Disk ${d.root.replace(/\\$/, '')}`;
  return `<div class="card drive land" style="--i:${i}" data-root="${esc(d.root)}" tabindex="0">
    <div class="top">${d.removable ? ICON.usb : ICON.drive}<div><div class="name">${esc(name)}</div><div class="meta">${esc(d.fs || 'disk')}${d.removable ? ' · vymeniteľný' : ''}</div></div></div>
    <div class="bar ${cls}"><i style="width:${(pct * 100).toFixed(1)}%"></i></div>
    <div class="free"><span>Obsadené ${bytes(used)}</span><span>Voľné ${bytes(d.free)} z ${bytes(d.total)}</span></div>
  </div>`;
}

const TABS = [
  ['dirs', 'Obsah'],
  ['owners', 'Hry a programy'],
  ['files', 'Najväčšie súbory'],
  ['old', 'Staré súbory'],
  ['cache', 'Na uvoľnenie'],
];

function renderStorage() {
  const body = $('#storageBody'), actions = $('#storageActions');
  const s = st.status;
  if (!s || s.phase === 'idle' || s.phase === 'cancelled') {
    actions.innerHTML = '';
    body.innerHTML = `<h2 style="margin-top:0">Vyber disk, ktorý chceš prezrieť</h2>
      <div class="drives">${st.drives.map(driveCard).join('') || '<div class="card empty">Nenašiel som žiadny disk.</div>'}</div>
      <div class="note">Skenovanie len číta, nič nemaže. Systémové priečinky, ku ktorým nemám prístup, preskočím.</div>`;
    $$('.drive', body).forEach((el) => {
      const go = () => startScan(el.dataset.root);
      el.onclick = go;
      el.onkeydown = (e) => { if (e.key === 'Enter' || e.key === ' ') go(); };
    });
    return;
  }
  if (s.phase === 'scanning') {
    actions.innerHTML = '<button class="btn" id="scanCancel">Zrušiť</button>';
    $('#scanCancel').onclick = async () => { await invoke('storage_cancel'); pollScan(); };
    const used = st.drive ? st.drive.total - st.drive.free : 0;
    const p = used ? Math.min(0.99, s.bytes / used) : null;
    if (!$('#scanCard')) {
      body.innerHTML = `<div class="card scanning land" id="scanCard">
        <div class="title"><span class="dot amber pulse"></span>Skenujem ${esc(s.root)}</div>
        <div class="nums"><div>Súbory<b id="scF">0</b></div><div>Priečinky<b id="scD">0</b></div><div>Veľkosť<b id="scB">0 B</b></div><div>Čas<b id="scT">0 s</b></div></div>
        <div class="bar" id="scBar"><i></i></div></div>`;
    }
    $('#scF').textContent = sk(s.files);
    $('#scD').textContent = sk(s.dirs);
    $('#scB').textContent = bytes(s.bytes);
    $('#scT').textContent = `${sk(s.elapsed_ms / 1000, 0)} s`;
    const bar = $('#scBar');
    bar.classList.toggle('indet', p === null);
    if (p !== null) $('i', bar).style.width = `${(p * 100).toFixed(1)}%`;
    setStatus(`Skenujem ${s.root.replace(/\\$/, '')}`, `${sk(s.files)} súborov, ${bytes(s.bytes)}`, 'amber', p);
    return;
  }
  // done
  actions.innerHTML = `<button class="btn" id="scanOther">${ICON.drive}Iný disk</button><button class="btn primary" id="scanAgain">Skenovať znova</button>`;
  $('#scanOther').onclick = () => { st.status = { phase: 'idle' }; st.listing = null; idleStatus(); loadDrives().then(renderStorage); };
  $('#scanAgain').onclick = () => startScan(s.root);
  if (!$('#tmCard')) {
    const cacheTotal = st.caches.reduce((a, c) => a + c.size, 0);
    const oldTotal = st.old.reduce((a, c) => a + c.size, 0);
    const notes = [];
    if (s.errors) notes.push(`${sk(s.errors)} ${plural(s.errors, 'priečinok', 'priečinky', 'priečinkov')} som nemohol otvoriť (systémové, bez prístupu).`);
    if (s.cloud_bytes) notes.push(`${bytes(s.cloud_bytes)} v OneDrive je len online a na disku nezaberá miesto, preto to nerátam.`);
    body.innerHTML = `
      <div class="stats">
        <div class="card stat land" style="--i:0"><div class="k">Naskenované za ${sk(s.elapsed_ms / 1000, s.elapsed_ms < 10000 ? 1 : 0)} s</div><div class="v num">${bytes(s.bytes)}</div></div>
        <div class="card stat land click" style="--i:1" data-tab="owners"><div class="k">Hry</div><div class="v num">${bytes(st.owners?.games_total || 0)}<small>${st.owners?.games.length || 0} ${plural(st.owners?.games.length || 0, 'hra', 'hry', 'hier')}</small></div></div>
        <div class="card stat land click" style="--i:2" data-tab="cache"><div class="k">Môžeš uvoľniť</div><div class="v num">${bytes(cacheTotal)}</div></div>
        <div class="card stat land click" style="--i:3" data-tab="old"><div class="k">Staré a zabudnuté</div><div class="v num">${bytes(oldTotal)}</div></div>
      </div>
      <div class="toolbar">
        <div class="crumbs" id="crumbs"></div>
        <label class="search wide">${ICON.search}<input id="ssearch" placeholder="Hľadať súbor alebo priečinok" spellcheck="false" value="${esc(st.query)}"><button class="clear" id="sclear" hidden>${ICON.x}</button></label>
      </div>
      <div class="owner" id="ownerBar" hidden></div>
      <div class="card treemap land" id="tmCard" style="--i:4"></div>
      <div class="legend"><span style="--c:var(--lilac)">Hry</span><span style="--c:var(--blue)">Programy</span><span style="--c:var(--amber)">Dá sa uvoľniť</span><span style="--c:var(--faint)">Systém</span><span style="--c:var(--teal)">Ostatné priečinky</span><span style="--c:var(--muted)">Súbory</span></div>
      <div class="tabrow"><div class="seg" id="stab"><div class="knob"></div>${TABS.map(([k, l]) => `<button data-k="${k}" class="${st.tab === k ? 'on' : ''}">${l}</button>`).join('')}</div><span id="resultsInfo"></span></div>
      <div class="card list" id="slist"></div>
      ${notes.length ? `<div class="note">${notes.map(esc).join('<br>')}</div>` : ''}
      <div class="note">Veľkosti sú logické. Priečinok Windows\\WinSxS môže vyzerať väčší, než naozaj je (pevné odkazy). Hľadám medzi priečinkami a súbormi nad 1 MB.</div>`;
    seg($('#stab'), (k) => { st.tab = k; clearSearch(false); renderList(true); });
    $$('.stat.click', body).forEach((c) => (c.onclick = () => selectTab(c.dataset.tab)));
    const input = $('#ssearch');
    input.addEventListener('input', debounce(() => runSearch(input.value), 180));
    input.addEventListener('keydown', (e) => { if (e.key === 'Escape') clearSearch(); });
    $('#sclear').onclick = () => clearSearch();
    let lastW = 0;
    new ResizeObserver(() => { const w = $('#tmCard')?.clientWidth; if (w && w !== lastW) { lastW = w; drawTreemap(); } }).observe($('#tmCard'));
  }
  setStatus('Sken hotový', `${s.root.replace(/\\$/, '')} · ${bytes(s.bytes)} za ${sk(s.elapsed_ms / 1000, 0)} s`, 'mint');
  renderCrumbs();
  drawTreemap();
  renderList();
}

function selectTab(k) {
  st.tab = k;
  clearSearch(false);
  $$('#stab button').forEach((b) => b.classList.toggle('on', b.dataset.k === k));
  placeSeg('#stab');
  renderList(true);
  $('#stab').scrollIntoView({ behavior: REDUCED ? 'auto' : 'smooth', block: 'start' });
}

async function startScan(root) {
  st.drive = st.drives.find((d) => d.root === root) || null;
  st.listing = null;
  st.colors.clear();
  st.query = '';
  st.results = null;
  try {
    await invoke('storage_start', { root });
  } catch (e) {
    setStatus('Nepodarilo sa', String(e), 'rec');
    return;
  }
  $('#storageBody').innerHTML = '';
  pollScan();
}

async function pollScan() {
  clearTimeout(st.timer);
  st.status = await invoke('storage_status');
  if (st.status.phase === 'scanning') {
    renderStorage();
    st.timer = setTimeout(pollScan, document.hidden ? 1000 : 250);
  } else if (st.status.phase === 'done') {
    [st.top, st.caches, st.owners, st.old] = await Promise.all([
      invoke('storage_top_files'), invoke('storage_caches'), invoke('storage_owners'), invoke('storage_old'),
    ]);
    $('#storageBody').innerHTML = '';
    await openNode(0, { first: true });
    helperDirty = true;
  } else {
    idleStatus();
    renderStorage();
  }
}

/** Opens a folder. `from` is the tile rect we zoom out of, so it looks like diving in. */
async function openNode(id, opts = {}) {
  const prev = st.listing;
  const l = await invoke('storage_node', { id });
  if (!l) return;
  st.listing = l;
  if (id === 0) (l.node.children || []).filter((c) => c.kind === 'dir').forEach((c, i) => st.colors.set(c.id, PALETTE[i % PALETTE.length]));
  if (opts.first || !$('#tmCard')) { renderStorage(); return; }

  // Which way are we going?
  let mode = null, rect = null;
  if (prev && !REDUCED) {
    const goingIn = l.crumbs.some((c) => c.id === prev.node.id) && l.node.id !== prev.node.id;
    const goingOut = prev.crumbs.some((c) => c.id === l.node.id) && l.node.id !== prev.node.id;
    if (goingIn) {
      // Tile of the direct child we entered (or its ancestor if we jumped deeper).
      const step = l.crumbs[l.crumbs.findIndex((c) => c.id === prev.node.id) + 1];
      rect = step && st.rects.get(step.id);
      mode = rect ? 'in' : 'fade';
    } else if (goingOut) {
      mode = 'out';
    } else {
      mode = 'fade';
    }
  }
  renderCrumbs();
  drawTreemap(mode, rect, prev);
  if (st.tab !== 'dirs' && !opts.keepTab) { st.tab = 'dirs'; $$('#stab button').forEach((b) => b.classList.toggle('on', b.dataset.k === 'dirs')); placeSeg('#stab'); }
  if (!opts.keepList) { clearSearch(false); renderList(true); }
}

function renderCrumbs() {
  const l = st.listing;
  if (!l) return;
  const c = $('#crumbs');
  c.innerHTML = l.crumbs
    .map((cr, i) => `${i ? '<span class="sep">›</span>' : ''}<button data-id="${cr.id}">${esc(cr.name || '\\')}</button>`)
    .join('') + `<button class="btn small ghost rv" id="revealCur" title="Otvoriť v Prieskumníkovi">${ICON.reveal}</button>`;
  $$('button[data-id]', c).forEach((b) => (b.onclick = () => openNode(+b.dataset.id)));
  $('#revealCur').onclick = () => reveal(l.path);
  const ob = $('#ownerBar');
  const o = l.owner;
  if (o && (o.kind === 'game' || o.kind === 'app' || o.kind === 'cache')) {
    const what = { game: `Patrí hre <b>${esc(o.label)}</b>${o.source ? ` (${esc(o.source)})` : ''}`, app: `Patrí programu <b>${esc(o.label)}</b>`, cache: `<b>${esc(o.label)}</b> – dá sa bezpečne uvoľniť` }[o.kind];
    ob.innerHTML = `${ICON[TAG[o.kind]?.icon] || ''}<span>${what}</span><span class="num">${bytes(l.node.size)} · ${sk(l.node.files)} ${plural(l.node.files, 'súbor', 'súbory', 'súborov')}</span>`;
    ob.className = `owner tag-${TAG[o.kind].cls}`;
    ob.hidden = false;
  } else {
    ob.hidden = true;
  }
}

function reveal(path) {
  invoke('storage_reveal', { path }).catch((e) => toast(String(e), 'err'));
}

/* squarified treemap: returns [x,y,w,h] per value (values sorted desc) */
function squarify(values, x, y, w, h) {
  const total = values.reduce((a, b) => a + b, 0);
  const out = new Array(values.length);
  if (total <= 0 || w <= 0 || h <= 0) return out.fill([x, y, 0, 0]);
  const area = values.map((v) => (v / total) * w * h);
  const worst = (row, side) => {
    const s = row.reduce((a, b) => a + b, 0), mx = Math.max(...row), mn = Math.min(...row);
    return Math.max((side * side * mx) / (s * s), (s * s) / (side * side * mn));
  };
  let i = 0, row = [], start = 0;
  const flush = () => {
    const s = row.reduce((a, b) => a + b, 0);
    if (w >= h) {
      const cw = s / h; let yy = y;
      row.forEach((a, k) => { const hh = a / cw; out[start + k] = [x, yy, cw, hh]; yy += hh; });
      x += cw; w -= cw;
    } else {
      const rh = s / w; let xx = x;
      row.forEach((a, k) => { const ww = a / rh; out[start + k] = [xx, y, ww, rh]; xx += ww; });
      y += rh; h -= rh;
    }
    start += row.length; row = [];
  };
  while (i < area.length) {
    const side = Math.min(w, h);
    if (!row.length || worst(row, side) >= worst([...row, area[i]], side)) { row.push(area[i]); i++; }
    else flush();
  }
  if (row.length) flush();
  return out;
}

function tmTip(n, parentSize) {
  const pct = parentSize ? ` · ${sk((n.size / parentSize) * 100, 1)} %` : '';
  const files = n.kind === 'dir' ? ` · ${sk(n.files)} ${plural(n.files, 'súbor', 'súbory', 'súborov')}` : '';
  const when = n.age_days ? `<br><span>Zmenené ${age(n.age_days)}</span>` : '';
  const tag = n.tag && n.tag.kind !== 'system' && n.tag.kind !== 'app_data' ? `<div style="margin-top:5px">${tagPill(n.tag)}</div>` : '';
  const sys = n.kind === 'file' && SYSFILES[n.name.toLowerCase()] ? `<div class="tipsub">${esc(SYSFILES[n.name.toLowerCase()])}</div>` : '';
  const inside = n.kind === 'dir' ? subItems(n) : null;
  const top = inside ? inside.items.slice(0, 5) : [];
  const list = top.length ? `<div class="tiplist">${inside.path.length ? `<div class="tipsub">› ${esc(inside.path.join(' › '))}</div>` : ''}${top.map((c) => `<div><span>${esc(c.name)}</span><span class="num">${bytes(c.size)}</span></div>`).join('')}</div>` : '';
  return `<b>${esc(n.name)}</b><span class="num">${bytes(n.size)}${pct}${files}</span>${when}${tag}${sys}${list}`;
}

const EXT_ICON = [
  [/\.(mp4|mov|mkv|avi|webm|m4v|braw|r3d|mxf|prores|ts)$/i, 'video'],
  [/\.(jpe?g|png|gif|webp|heic|psd|tiff?|raw|cr2|cr3|nef|arw|dng|exr)$/i, 'image'],
  [/\.(wav|mp3|flac|aac|ogg|m4a|aiff?)$/i, 'audio'],
  [/\.(zip|rar|7z|tar|gz|bz2|xz)$/i, 'archive'],
  [/\.(iso|img|vhdx?|vmdk|qcow2|wim|esd)$/i, 'disc'],
  [/\.(exe|msi|msix|appx)$/i, 'exe'],
  [/\.(sys|dll|dmp|etl|log|db|dat|bin|pak|cab)$/i, 'gear'],
  [/\.(pdf|docx?|xlsx?|pptx?|txt|md|rtf|odt)$/i, 'doc'],
];
/** Icon for a tile: what it belongs to, else what kind of file it is. */
function tileIcon(n) {
  const t = n.tag?.kind;
  if (t === 'game') return 'game';
  if (t === 'cache') return 'broom';
  if (t === 'app') return 'app';
  if (n.kind === 'small' || n.kind === 'other') return 'small';
  if (n.kind === 'file') return (EXT_ICON.find(([re]) => re.test(n.name)) || [0, 'file'])[1];
  if (t === 'system' || /^(windows|program ?data|\$recycle\.bin|system volume information)$/i.test(n.name)) return 'gear';
  if (/^(users|používatelia)$/i.test(n.name)) return 'user';
  if (/^(videos?|videá|klipy|captures|footage|render)$/i.test(n.name)) return 'video';
  if (/^(pictures|obrázky|photos|fotky|screenshots)$/i.test(n.name)) return 'image';
  if (/^(music|hudba)$/i.test(n.name)) return 'audio';
  if (/^(downloads|stiahnuté)$/i.test(n.name)) return 'archive';
  if (/^(documents|dokumenty)$/i.test(n.name)) return 'doc';
  if (/^program files/i.test(n.name)) return 'app';
  return 'dir';
}

/** Merges specks into one "menších" tile so the map shows what matters. */
function foldSmall(items, total, minFrac) {
  const keep = [], small = [];
  items.forEach((c) => (c.size / total < minFrac || c.id == null ? small : keep).push(c));
  if (small.length > 1 || (small.length === 1 && small[0].id == null)) {
    keep.push({ id: null, kind: 'other', name: `${sk(small.length)} menších položiek`, size: small.reduce((a, c) => a + c.size, 0), files: 0, children: null });
  } else keep.push(...small);
  return keep;
}

/** Skips chains like Steam › steamapps › common that hold one big thing. */
function subItems(n) {
  if (n.via?.length) return { items: (n.children || []).filter((c) => c.size > 0), path: n.via };
  let cur = n;
  const path = [];
  for (let d = 0; d < 3; d++) {
    const ks = (cur.children || []).filter((c) => c.size > 0);
    const top = ks[0];
    if (!top || top.kind !== 'dir' || !top.children || top.size / cur.size < 0.8) break;
    path.push(top.name);
    cur = top;
  }
  return { items: (cur.children || []).filter((c) => c.size > 0), path };
}

const EASE = 'cubic-bezier(.2,.9,.25,1)';
function drawTreemap(mode = null, fromRect = null, prev = null) {
  const card = $('#tmCard'), l = st.listing;
  if (!card || !l) return;
  const W = card.clientWidth - 16, H = card.clientHeight - 16;
  // Only the layer currently on screen may animate out. Anything older (left over
  // from fast clicking) is dropped, otherwise a ghost folder stays behind.
  const old = st.layer && st.layer.isConnected ? st.layer : null;
  $$('.tmlayer', card).forEach((x) => { if (x !== old) x.remove(); });
  if (old) old.getAnimations().forEach((an) => an.cancel());
  const layer = document.createElement('div');
  st.layer = layer;
  layer.className = 'tmlayer';
  const kids = foldSmall((l.node.children || []).filter((c) => c.size > 0), l.node.size, 0.006);
  st.rects = new Map();
  if (!kids.length) {
    layer.innerHTML = `<div class="empty">${ICON.dir}Tento priečinok je prázdny.</div>`;
  } else {
    const KCOL = { game: 'var(--lilac)', app: 'var(--blue)', cache: 'var(--amber)', system: 'var(--faint)' };
    // Colors carry meaning (see the legend); everything else is one calm color.
    const color = (n) => (n.kind === 'file' ? 'var(--muted)' : n.kind !== 'dir' ? 'var(--faint)' : KCOL[n.tag?.kind] || 'var(--teal)');
    const rects = squarify(kids.map((k) => k.size), 0, 0, W, H);
    const G = 4;
    kids.forEach((n, i) => {
      let [x, y, w, h] = rects[i];
      x += G / 2; y += G / 2; w -= G; h -= G;
      if (w < 2 || h < 2) return;
      if (n.id != null) st.rects.set(n.id, [x, y, w, h]);
      const tier = w > 170 && h > 120 ? 'xl' : w > 100 && h > 62 ? 'lg' : w > 60 && h > 36 ? 'md' : w > 24 && h > 24 ? 'sm' : 'xs';
      const el = document.createElement('div');
      el.className = `tm ${n.kind} t-${tier}${n.tag?.kind === 'game' ? ' isgame' : ''}${n.tag?.kind === 'cache' ? ' iscache' : ''}${n.kind === 'dir' && n.id != null ? ' open' : ''}`;
      el.dataset.id = n.id ?? '';
      el.style.cssText = `left:${x}px;top:${y}px;width:${w}px;height:${h}px;--c:${color(n, i)}`;
      const pct = l.node.size ? (n.size / l.node.size) * 100 : 0;
      const pctTxt = pct >= 1 ? `${sk(pct, 0)} %` : '<1 %';
      const chip = `<span class="chip">${ICON[tileIcon(n)] || ICON.file}</span>`;
      const si = n.kind === 'dir' ? subItems(n) : { items: [], path: [] };
      let html = '';
      if (tier === 'xl' || tier === 'lg') {
        const via = si.path.length && tier === 'xl' ? `<span class="via">› ${esc(si.path.join(' › '))}</span>` : '';
        const prev = tier === 'xl' ? foldSmall(si.items, n.size, 0.03).filter((c) => c.id != null).slice(0, Math.max(0, Math.floor((h - 118) / 24))) : [];
        html = `<div class="tc">${chip}<span class="nm">${esc(n.name)}</span>${via}<span class="sz">${bytes(n.size)} <em>${pctTxt}</em></span>
          ${prev.length ? `<div class="pv">${prev.map((c) => `<a data-id="${c.id}" style="--w:${Math.max(4, (c.size / n.size) * 100)}%"><span>${esc(c.name)}</span><span class="num">${bytes(c.size)}</span></a>`).join('')}</div>` : ''}</div>`;
      } else if (tier === 'md') {
        html = `<div class="tc">${w > 110 ? chip : ''}<span class="nm">${esc(n.name)}</span><span class="sz">${bytes(n.size)}</span></div>`;
      } else if (tier === 'sm') {
        html = `<div class="tc">${chip}</div>`;
      }
      el.innerHTML = html;
      el.onmousemove = (e) => showTip(tmTip(n, l.node.size), e.clientX, e.clientY);
      el.onmouseleave = hideTip;
      if (n.kind === 'dir' && n.id != null) el.onclick = () => { hideTip(); openNode(n.id); };
      $$('.pv a', el).forEach((a) => (a.onclick = (e) => { e.stopPropagation(); hideTip(); openNode(+a.dataset.id); }));
      layer.appendChild(el);
    });
  }
  card.appendChild(layer);
  if (st.flash != null) {
    const f = $(`.tm[data-id="${st.flash}"]`, layer);
    if (f) { f.classList.add('flash'); setTimeout(() => f.classList.remove('flash'), 2400); }
    st.flash = null;
  }

  if (!old) return;
  if (!mode || !layer.animate) { old.remove(); return; }
  const opts = { duration: 520, easing: EASE, fill: 'both' };
  const fit = ([x, y, w, h]) => `translate(${x}px, ${y}px) scale(${w / W}, ${h / H})`;
  const zoom = ([x, y, w, h]) => `translate(${(-x * W) / w}px, ${(-y * H) / h}px) scale(${W / w}, ${H / h})`;
  layer.classList.add('morph');
  let a, b;
  if (mode === 'in') {
    a = layer.animate([{ transform: fit(fromRect), opacity: 0.35 }, { transform: 'none', opacity: 1 }], opts);
    b = old.animate([{ transform: 'none', opacity: 1 }, { transform: zoom(fromRect), opacity: 0 }], opts);
  } else if (mode === 'out') {
    // The folder we came from (or its ancestor) is a tile in the new layer.
    const back = prev?.crumbs.map((c) => st.rects.get(c.id)).find(Boolean);
    if (back) {
      a = layer.animate([{ transform: zoom(back), opacity: 0 }, { transform: 'none', opacity: 1 }], opts);
      b = old.animate([{ transform: 'none', opacity: 1 }, { transform: fit(back), opacity: 0 }], opts);
    }
  }
  if (!a) {
    a = layer.animate([{ opacity: 0, transform: 'scale(.985)' }, { opacity: 1, transform: 'none' }], { ...opts, duration: 320 });
    b = old.animate([{ opacity: 1 }, { opacity: 0 }], { ...opts, duration: 200 });
  }
  b.onfinish = () => old.remove();
  setTimeout(() => { if (old.isConnected && old !== st.layer) old.remove(); }, opts.duration + 150);
  a.onfinish = () => { a.cancel(); layer.classList.remove('morph'); };
}

const SYSFILES = {
  'pagefile.sys': 'Stránkovací súbor Windowsu. Nemaž ho.',
  'hiberfil.sys': 'Hibernácia. Ak ju nepoužívaš, vypneš ju príkazom powercfg /h off.',
  'swapfile.sys': 'Súčasť Windowsu. Nemaž ho.',
  'memory.dmp': 'Výpis po páde systému. Dá sa zmazať.',
};

function splitPath(p) {
  const cut = p.lastIndexOf('\\');
  return [p.slice(cut + 1), p.slice(0, cut)];
}

function fileRow(path, size, max, i, extra = '') {
  const [name, dir] = splitPath(path);
  const hint = SYSFILES[name.toLowerCase()];
  return `<div class="lrow path land" style="--i:${Math.min(i, 14)}" data-path="${esc(path)}"><span class="ic">${ICON.file}</span>
    <span class="nm">${esc(name)} <span class="dim">${esc(hint || dir)}</span></span>
    <div class="bar"><i style="width:${((size / max) * 100).toFixed(1)}%;background:var(--blue)"></i></div>
    <span class="sz num">${bytes(size)}</span><span class="pct">${extra}</span>
    <span><button class="btn icon ghost rev" title="Ukázať v Prieskumníkovi">${ICON.reveal}</button></span></div>`;
}

function bindReveal(box) {
  $$('.lrow[data-path]', box).forEach((r) => { const b = $('.rev', r); if (b) b.onclick = (e) => { e.stopPropagation(); reveal(r.dataset.path); }; });
}

function renderList(animate = false) {
  const box = $('#slist'), l = st.listing;
  if (!box || !l) return;
  const info = $('#resultsInfo');
  info.innerHTML = '';
  const land = (i) => (animate ? ` land" style="--i:${Math.min(i, 14)}` : '');

  if (st.results) {
    info.innerHTML = `<span class="muted">${st.results.length ? `${st.results.length >= 200 ? '200+' : st.results.length} ${plural(st.results.length, 'výsledok', 'výsledky', 'výsledkov')}` : ''}</span>`;
    if (!st.results.length) { box.innerHTML = `<div class="empty">${ICON.search}Nič som nenašiel. Hľadám medzi priečinkami a súbormi nad 1 MB.</div>`; return; }
    const max = st.results[0].size || 1;
    box.innerHTML = st.results.map((h, i) => {
      const [, dir] = splitPath(h.path);
      return `<div class="lrow path click${land(i)}" data-i="${i}" data-path="${esc(h.path)}"><span class="ic ${h.kind === 'dir' ? 'dir' : ''}">${ICON[h.kind] || ICON.file}</span>
        <span class="nm">${esc(h.name)} ${h.owner && h.owner !== h.name ? `<span class="badge tag-app sm">${esc(h.owner)}</span>` : ''}<span class="dim">${esc(dir)}</span></span>
        <div class="bar"><i style="width:${((h.size / max) * 100).toFixed(1)}%;background:${h.kind === 'dir' ? 'var(--accent)' : 'var(--blue)'}"></i></div>
        <span class="sz num">${bytes(h.size)}</span><span class="pct">${h.age_days ? age(h.age_days) : ''}</span>
        <span><button class="btn icon ghost rev" title="Ukázať v Prieskumníkovi">${ICON.reveal}</button></span></div>`;
    }).join('');
    $$('.lrow[data-i]', box).forEach((r) => (r.onclick = () => {
      const h = st.results[+r.dataset.i];
      if (h.kind === 'dir') openNode(h.id, { keepList: true });
      else { st.flash = h.id; openNode(h.parent, { keepList: true }); }
      $('#tmCard').scrollIntoView({ behavior: REDUCED ? 'auto' : 'smooth', block: 'center' });
    }));
    bindReveal(box);
    return;
  }

  if (st.tab === 'dirs') {
    const kids = l.node.children || [];
    if (!kids.length) { box.innerHTML = `<div class="empty">${ICON.dir}Prázdny priečinok.</div>`; return; }
    box.innerHTML = kids.map((c, i) => {
      const share = l.node.size ? c.size / l.node.size : 0;
      const canReveal = c.kind === 'dir' || c.kind === 'file';
      const extra = c.kind === 'dir' ? `<small>${sk(c.files)} ${plural(c.files, 'súbor', 'súbory', 'súborov')}</small>` : '';
      const tag = c.tag && c.tag.kind !== 'app_data' && !(c.tag.kind === 'system' && l.node.id !== 0) ? tagPill(c.tag) : '';
      return `<div class="lrow ${c.kind === 'dir' && c.id != null ? 'click' : ''}${land(i)}" data-id="${c.id ?? ''}" data-kind="${c.kind}" data-name="${esc(c.name)}">
        <span class="ic ${c.kind === 'dir' ? 'dir' : ''}">${ICON[c.kind] || ICON.file}</span>
        <span class="nm">${esc(c.name)}${tag}${extra}</span>
        <div class="bar"><i style="width:${(share * 100).toFixed(1)}%;background:${c.kind === 'dir' ? 'var(--accent)' : 'var(--faint)'}"></i></div>
        <span class="sz num">${bytes(c.size)}</span><span class="pct num">${sk(share * 100, share < 0.1 ? 1 : 0)} %</span>
        <span>${canReveal ? `<button class="btn icon ghost rev" title="Otvoriť v Prieskumníkovi">${ICON.reveal}</button>` : ''}</span></div>`;
    }).join('');
    $$('.lrow', box).forEach((r) => {
      if (r.dataset.kind === 'dir' && r.dataset.id) r.onclick = () => openNode(+r.dataset.id);
      const b = $('.rev', r);
      if (b) b.onclick = (e) => { e.stopPropagation(); reveal(`${l.path.replace(/\\$/, '')}\\${r.dataset.name}`); };
    });
  } else if (st.tab === 'owners') {
    const o = st.owners || { games: [], apps: [] };
    const block = (title, items, icon, empty) => {
      if (!items.length) return `<div class="lsec">${title}</div><div class="empty small">${empty}</div>`;
      const max = items[0].size;
      return `<div class="lsec">${title}<span class="num">${bytes(items.reduce((a, x) => a + x.size, 0))}</span></div>` + items.map((g, i) => `
        <div class="lrow click${land(i)}" data-id="${g.id}" data-path="${esc(g.path)}"><span class="ic ${icon === 'game' ? 'game' : 'dir'}">${ICON[icon]}</span>
        <span class="nm">${esc(g.label)}${g.source ? `<small>${esc(g.source)}</small>` : ''}</span>
        <div class="bar"><i style="width:${((g.size / max) * 100).toFixed(1)}%;background:${icon === 'game' ? 'var(--lilac)' : 'var(--blue)'}"></i></div>
        <span class="sz num">${bytes(g.size)}</span><span class="pct">${g.age_days > 180 ? `<span class="badge off" title="Odvtedy sa nezmenil žiadny súbor">${age(g.age_days)}</span>` : ''}</span>
        <span><button class="btn icon ghost rev" title="Otvoriť v Prieskumníkovi">${ICON.reveal}</button></span></div>`).join('');
    };
    box.innerHTML = block('Hry', o.games, 'game', 'Na tomto disku som nenašiel hry zo Steamu, Epicu, Xboxu, GOG, Riotu, Ubisoftu ani EA.') +
      block('Programy', o.apps.slice(0, 30), 'app', 'Žiadne programy v Program Files.') +
      '<div class="note" style="padding:0 16px 10px">Hra, pri ktorej sa dlho nič nezmenilo, sa zrejme ani neaktualizovala, takže ju asi nehráš. Odinštaluj ju cez jej launcher.</div>';
    $$('.lrow[data-id]', box).forEach((r) => (r.onclick = () => openNode(+r.dataset.id, { keepTab: false })));
    bindReveal(box);
  } else if (st.tab === 'files') {
    if (!st.top.length) { box.innerHTML = `<div class="empty">${ICON.file}Žiadne veľké súbory.</div>`; return; }
    box.innerHTML = st.top.map((f, i) => fileRow(f.path, f.size, st.top[0].size, animate ? i : 99)).join('');
    bindReveal(box);
  } else if (st.tab === 'old') {
    const inst = st.old.filter((o) => o.reason === 'installer'), old = st.old.filter((o) => o.reason === 'old');
    if (!st.old.length) { box.innerHTML = `<div class="empty">${ICON.check}Nenašiel som nič starého ani zabudnutého.</div>`; return; }
    const max = st.old[0].size;
    let i = 0;
    box.innerHTML =
      (inst.length ? `<div class="lsec">Inštalačky a archívy v Stiahnutých<span class="num">${bytes(inst.reduce((a, x) => a + x.size, 0))}</span></div>
        <div class="lhint">Program je už nainštalovaný, inštalačku väčšinou netreba. Archív skontroluj, či ho máš rozbalený.</div>` +
        inst.map((o) => fileRow(o.path, o.size, max, animate ? i++ : 99, age(o.age_days))).join('') : '') +
      (old.length ? `<div class="lsec">Veľké súbory, na ktoré si dlho nesiahol<span class="num">${bytes(old.reduce((a, x) => a + x.size, 0))}</span></div>
        <div class="lhint">Nezmenené viac ako rok. Neznamená to, že sú zbytočné: skontroluj, či ich ešte potrebuješ, prípadne ich presuň na iný disk.</div>` +
        old.map((o) => fileRow(o.path, o.size, max, animate ? i++ : 99, age(o.age_days))).join('') : '');
    bindReveal(box);
  } else {
    if (!st.caches.length) { box.innerHTML = `<div class="empty">${ICON.check}Nenašiel som žiadnu známu cache, ktorá by sa oplatila mazať.</div>`; return; }
    const total = st.caches.reduce((a, c) => a + c.size, 0);
    box.innerHTML = `<div class="cache" style="padding-bottom:6px"><div><div class="t">Spolu sa dá uvoľniť ${bytes(total)}</div>
      <div class="h">Zatiaľ nič nemažem. Ku každej položke ti napíšem, ako ju bezpečne vyčistiť.</div></div></div>` +
      st.caches.map((c, i) => `<div class="cache${land(i)}" data-i="${i}"><div><div class="t">${esc(c.label)}</div><div class="p">${esc(c.path)}</div><div class="h">${esc(c.hint)}</div></div>
        <span class="num">${bytes(c.size)}</span><button class="btn icon ghost" title="Otvoriť v Prieskumníkovi">${ICON.reveal}</button></div>`).join('');
    $$('.cache[data-i]', box).forEach((r) => ($('button', r).onclick = () => reveal(st.caches[+r.dataset.i].path)));
  }
}

async function runSearch(q) {
  st.query = q.trim();
  $('#sclear').hidden = !st.query;
  if (!st.query) { st.results = null; renderList(true); return; }
  st.results = await invoke('storage_search', { query: st.query });
  renderList(true);
}
function clearSearch(rerender = true) {
  if (!st.results && !st.query) return;
  st.query = '';
  st.results = null;
  const i = $('#ssearch');
  if (i) i.value = '';
  if ($('#sclear')) $('#sclear').hidden = true;
  if (rerender) renderList(true);
}

views.storage = {
  async enter() {
    if (!st.status) {
      await loadDrives();
      st.status = await invoke('storage_status');
      if (st.status.phase === 'scanning' || st.status.phase === 'done') pollScan();
      else renderStorage();
    } else if (st.listing) {
      requestAnimationFrame(() => { placeSeg('#stab'); drawTreemap(); });
    }
  },
};

/* ========================================================== PROCESSES */
const pr = { snap: null, timer: null, sort: 'ram', filter: 'all', q: '', open: new Set() };
const FILTERS = [
  ['all', 'Všetko', () => true],
  ['rgb', 'Výrobcovské RGB', (g) => g.verdict === 'rgb'],
  ['optional', 'Dá sa vypnúť', (g) => g.verdict === 'optional'],
  ['peripheral', 'Periférie', (g) => g.verdict === 'peripheral'],
  ['system', 'Windows', (g) => g.verdict === 'system'],
];
seg($('#psort'), (k) => { pr.sort = k; renderProcs(); });
$('#psearch').addEventListener('input', (e) => { pr.q = e.target.value.trim().toLowerCase(); renderProcs(); });

async function pollProcs() {
  clearTimeout(pr.timer);
  try { pr.snap = await invoke('procs_snapshot'); } catch (e) { console.error(e); }
  renderProcs();
  if (current === 'procs' && !document.hidden) pr.timer = setTimeout(pollProcs, 2000);
}

function renderProcs() {
  const s = pr.snap;
  if (!s) return;
  const procCount = s.groups.reduce((a, g) => a + g.procs.length, 0);
  $('#sCpu').textContent = `${sk(s.cpu_total, 0)} %`;
  $('#sCpuBar').className = `bar ${s.cpu_total > 80 ? 'rec' : s.cpu_total > 50 ? 'amber' : 'mint'}`;
  $('#sCpuBar i').style.width = `${s.cpu_total}%`;
  const rp = s.ram_total ? s.ram_used / s.ram_total : 0;
  $('#sRam').innerHTML = `${bytes(s.ram_used)}<small>z ${bytes(s.ram_total)}</small>`;
  $('#sRamBar').className = `bar ${rp > 0.9 ? 'rec' : rp > 0.75 ? 'amber' : 'mint'}`;
  $('#sRamBar i').style.width = `${(rp * 100).toFixed(1)}%`;
  $('#sCount').textContent = sk(procCount);
  const rgb = s.groups.filter((g) => g.verdict === 'rgb');
  $('#sRgb').innerHTML = rgb.length ? `${bytes(rgb.reduce((a, g) => a + g.ram, 0))}<small>${rgb.length} ${plural(rgb.length, 'appka', 'appky', 'appiek')}</small>` : 'žiadne';

  $('#pchips').innerHTML = FILTERS.map(([k, label, f]) => {
    const n = s.groups.filter(f).length;
    return k !== 'all' && !n ? '' : `<button class="chip ${pr.filter === k ? 'on' : ''}" data-k="${k}">${label}<span class="n">${n}</span></button>`;
  }).join('');
  $$('#pchips .chip').forEach((c) => (c.onclick = () => { pr.filter = c.dataset.k; renderProcs(); }));

  const f = (FILTERS.find((x) => x[0] === pr.filter) || FILTERS[0])[2];
  const key = { ram: (g) => g.ram, cpu: (g) => g.cpu, io: (g) => g.io_bps }[pr.sort];
  const groups = s.groups
    .filter(f)
    .filter((g) => !pr.q || g.name.toLowerCase().includes(pr.q) || g.procs.some((p) => p.exe.toLowerCase().includes(pr.q)))
    .sort((a, b) => key(b) - key(a) || b.ram - a.ram);

  $('#plist').innerHTML = groups.map((g) => {
    const open = pr.open.has(g.key);
    const n = g.procs.length;
    const killable = g.verdict !== 'system' && g.key !== 'WhatsInside';
    const procs = g.procs.slice(0, 40).map((p) => `<div class="proc"><div><div>${esc(p.exe)} <span style="color:var(--faint)">· ${p.pid}</span>${killable ? `<button class="btn small ghost kill" data-pid="${p.pid}" data-exe="${esc(p.exe)}">Ukončiť</button>` : ''}</div>${p.path ? `<div class="path">${esc(p.path)}</div>` : ''}${p.services?.length ? `<div class="svcs">Služby: ${p.services.map(esc).join(', ')}</div>` : ''}</div>
      <span class="r">${p.cpu >= 0.1 ? `${sk(p.cpu, 1)} %` : '–'}</span><span class="r">${p.ram ? bytes(p.ram) : '?'}</span><span class="r">${rate(p.io_bps)}</span></div>`).join('');
    const auto = autoOf(g);
    return `<div class="pgroup ${open ? 'open' : ''}" data-key="${esc(g.key)}">
      <div class="prow"><div class="who"><svg class="chev" viewBox="0 0 12 12"><path d="M4.5 2.5L8 6l-3.5 3.5"/></svg><span class="nm">${esc(g.name)}</span>${g.verdict === 'system' ? '' : badge(g.verdict)}${auto ? `<span class="badge auto" data-tip="Spúšťa sa so systémom: ${esc(auto.when)} (${esc(auto.source)})">${ICON.rocket}So systémom</span>` : ''}${n > 1 ? `<span class="cnt">${n} ${plural(n, 'proces', 'procesy', 'procesov')}</span>` : ''}</div>
      <span></span><span class="r ${g.cpu >= 10 ? 'hot' : g.cpu < 0.1 ? 'dim' : ''}">${g.cpu >= 0.1 ? `${sk(g.cpu, 1)} %` : '–'}</span>
      <span class="r">${g.ram ? bytes(g.ram) : '?'}</span><span class="r ${g.io_bps < 1024 ? 'dim' : ''}">${rate(g.io_bps)}</span></div>
      <div class="pdetail">${g.about ? `<div class="about">${esc(g.about)}</div>` : ''}${killable || auto ? `<div class="gact">${killable ? `<button class="btn small killall">Ukončiť${n > 1 ? ` všetky (${n})` : ''}</button>` : ''}${auto ? '<button class="btn small ghost toauto">Vypnúť zo spúšťania…</button>' : ''}</div>` : ''}${procs}${n > 40 ? `<div class="proc">… a ďalších ${n - 40}</div>` : ''}</div></div>`;
  }).join('') || '<div class="empty">Nič také nebeží.</div>';
  $$('#plist .prow').forEach((r) => (r.onclick = () => {
    const g = r.parentElement, k = g.dataset.key;
    pr.open.has(k) ? pr.open.delete(k) : pr.open.add(k);
    g.classList.toggle('open');
  }));
  const groupOf = (el) => s.groups.find((g) => g.key === el.closest('.pgroup').dataset.key);
  $$('#plist .kill').forEach((b) => (b.onclick = (e) => { e.stopPropagation(); killProcs([{ pid: +b.dataset.pid, exe: b.dataset.exe }], b.dataset.exe); }));
  $$('#plist .killall').forEach((b) => (b.onclick = () => { const g = groupOf(b); killProcs(g.procs.map((p) => ({ pid: p.pid, exe: p.exe })), g.name); }));
  $$('#plist .toauto').forEach((b) => (b.onclick = () => { const name = groupOf(b).name; su.q = name.toLowerCase(); su.kind = 'all'; $('#startupSearch').value = name; setView('startup'); }));
  bindTips($('#plist'));
}

/** The enabled autostart entry a process group comes from, if any. */
function autoOf(g) {
  if (!startupItems) return null;
  const exes = new Set(g.procs.map((p) => p.exe.toLowerCase()));
  return startupItems.find((i) => i.enabled && i.verdict !== 'system' && ((i.exe && exes.has(i.exe)) || (i.app && i.app === g.name))) || null;
}

function killProcs(list, name) {
  const m = modal(`<h3>Ukončiť ${esc(name)}?</h3>
    <p class="mp">Neuložená práca v ${list.length > 1 ? 'týchto procesoch' : 'tomto procese'} sa stratí. Ak sa appka spúšťa so systémom, po reštarte sa znova zapne.</p>
    <div class="mact"><button class="btn" data-close>Zrušiť</button><button class="btn danger" id="killGo">Ukončiť</button></div>`);
  $('#killGo', m).onclick = async () => {
    closeModal();
    const res = await invoke('actions_apply', { actions: list.map((p) => ({ op: 'kill', pid: p.pid, exe: p.exe })) }).catch((e) => list.map(() => ({ ok: false, error: String(e) })));
    const ok = res.filter((r) => r.ok).length, bad = res.find((r) => !r.ok);
    if (ok) toast(`Ukončené: ${ok}`);
    if (bad) toast(bad.error || 'Nepodarilo sa.', 'err');
    pollProcs();
  };
}

views.procs = {
  enter(filter) {
    if (filter) pr.filter = filter;
    placeSeg('#psort');
    if (!startupItems) invoke('autostart_list').then((x) => { startupItems = x; renderProcs(); }).catch(() => {});
    pollProcs().then(() => { clearTimeout(pr.timer); pr.timer = setTimeout(pollProcs, 600); });
  },
  leave() { clearTimeout(pr.timer); },
  visible(v) { if (v && current === 'procs') pollProcs(); else clearTimeout(pr.timer); },
};

/* ========================================================== STARTUP */
const SECTIONS = [
  ['rgb', 'Výrobcovské RGB appky', 'Ak ich vypneš, svetlá sa môžu po reštarte vrátiť na predvolené farby. Doska si väčšinou pamätá posledný efekt, grafika a vetráky nie vždy.'],
  ['optional', 'Netreba spúšťať so systémom', 'Keď ich budeš potrebovať, jednoducho ich otvoríš.'],
  ['peripheral', 'Periférie', 'Ak má myš alebo klávesnica profil uložený v sebe, appka nemusí bežať stále.'],
  ['unknown', 'Ostatné', 'Tieto nepoznám. Pozri si cestu k programu, väčšinou prezradí, čo to je.'],
  ['needed', 'Potrebné', null],
  ['system', 'Súčasti Windowsu', null],
];
const KINDS = [
  ['all', 'Všetko', () => true],
  ['apps', 'Appky', (i) => i.kind === 'run' || i.kind === 'folder'],
  ['task', 'Naplánované úlohy', (i) => i.kind === 'task'],
  ['service', 'Služby', (i) => i.kind === 'service'],
  ['other', 'Iné', (i) => i.kind === 'runonce' || i.kind === 'winlogon'],
];
const su = { kind: 'all', showSystem: false, q: '', busy: new Set() };
let startupItems = null;
let changesLog = [];

async function loadStartup() {
  if (!startupItems) $('#startupBody').innerHTML = '<div class="card empty">Prechádzam register, naplánované úlohy a služby…</div>';
  [startupItems, changesLog] = await Promise.all([invoke('autostart_list'), invoke('changes_list').catch(() => [])]);
  renderStartup();
}

/** What we suggest to switch off: optional things, not Windows "maybe" ones. */
const recommended = (i) => i.enabled && i.toggle && i.verdict === 'optional' && (!i.advice || i.advice === 'off');

const SHIELD = '<svg class="shield" viewBox="0 0 16 16"><path d="M8 1.6l5 2v4c0 3.2-2.1 5.7-5 6.8-2.9-1.1-5-3.6-5-6.8v-4z"/></svg>';
const EYE_OFF = '<svg viewBox="0 0 16 16"><path d="M2 2l12 12M6.6 6.7a2 2 0 0 0 2.7 2.7M4.3 4.5C2.9 5.4 1.9 6.7 1.5 8c1 2.9 3.6 4.8 6.5 4.8 1.2 0 2.4-.3 3.4-.9M7 3.3c.3 0 .7-.1 1-.1 2.9 0 5.5 1.9 6.5 4.8-.3.9-.8 1.7-1.4 2.4"/></svg>';

function startupRow(i) {
  const busy = su.busy.has(i.id);
  const adv = i.advice === 'maybe' ? '<span class="badge maybe">Záleží, čo používaš</span>' : i.advice === 'keep' ? '<span class="badge needed">Nechaj</span>' : '';
  return `<div class="arow srow ${i.enabled ? '' : 'isoff'}" data-id="${esc(i.id)}">
    <div style="min-width:0">
      <div class="t">${esc(i.app || i.name)}${i.app && i.app !== i.name ? `<span class="sub">${esc(i.name)}</span>` : ''}
        ${i.hidden ? `<span class="badge ghost" data-tip="Správca úloh túto položku v zozname Aplikácie pri spustení neukazuje.">${EYE_OFF}Skryté</span>` : ''}
        ${i.running === true ? '<span class="badge on">Beží</span>' : ''}${adv}
        ${recommended(i) ? '<span class="badge optional">Odporúčam vypnúť</span>' : ''}</div>
      ${i.about ? `<div class="a">${esc(i.about)}</div>` : ''}
      <div class="meta"><span>${esc(i.when)}</span><span>·</span><span>${esc(i.source)}</span></div>
      <div class="c" title="${esc(i.command)}">${esc(i.command)}</div>
    </div>
    <div class="sw">
      ${i.toggle ? `${i.needs_admin ? `<span data-tip="Zmena si vypýta povolenie správcu (UAC).">${SHIELD}</span>` : ''}
        <label class="switch ${busy ? 'busy' : ''}"><input type="checkbox" ${i.enabled ? 'checked' : ''} ${busy ? 'disabled' : ''}><i></i></label>`
        : '<span class="na" data-tip="Toto sa nedá vypnúť prepínačom.">–</span>'}
    </div></div>`;
}

function renderStartup() {
  const items = startupItems || [];
  const rec = items.filter(recommended);
  $('#startupCount').textContent = rec.length || '';
  const on = items.filter((i) => i.enabled);
  const hidden = on.filter((i) => i.hidden && i.verdict !== 'system');
  const pending = changesLog.filter((c) => !c.undone).length;
  $('#changesBtn').innerHTML = `${ICON.clock}Zmeny${pending ? `<span class="pill">${pending}</span>` : ''}`;

  const kf = (KINDS.find((k) => k[0] === su.kind) || KINDS[0])[2];
  const q = su.q;
  const shown = items.filter(kf).filter((i) => !q || `${i.app || ''} ${i.name} ${i.command} ${i.source}`.toLowerCase().includes(q));
  const sysCount = shown.filter((i) => i.verdict === 'system').length;

  $('#startupBody').innerHTML = `
    <div class="stats">
      <div class="card stat"><div class="k">Spúšťa sa so systémom</div><div class="v num">${sk(on.length)}</div></div>
      <div class="card stat"><div class="k">Nevidno v Správcovi úloh</div><div class="v num">${sk(hidden.length)}<small>bez Windowsu</small></div></div>
      <div class="card stat"><div class="k">Odporúčam vypnúť</div><div class="v num">${sk(rec.length)}</div></div>
      <div class="card stat"><div class="k">Vypnuté</div><div class="v num">${sk(items.length - on.length)}</div></div>
    </div>
    ${rec.length ? `<div class="card recbar"><div>${ICON.rocket}<div><b>${rec.length} ${plural(rec.length, 'vec sa spúšťa', 'veci sa spúšťajú', 'vecí sa spúšťa')} zbytočne.</b><span>Pozrieš si zoznam, vyberieš a vypneš naraz. Všetko sa dá vrátiť.</span></div></div>
      <button class="btn primary" id="recBtn">Vybrať a vypnúť</button></div>` : ''}
    <div class="chips">${KINDS.map(([k, l, f]) => { const n = items.filter(f).filter((i) => su.showSystem || i.verdict !== 'system').length; return k !== 'all' && !n ? '' : `<button class="chip ${su.kind === k ? 'on' : ''}" data-k="${k}">${l}<span class="n">${n}</span></button>`; }).join('')}
      <label class="swl"><span>Ukázať súčasti Windowsu (${sk(sysCount)})</span><label class="switch"><input type="checkbox" id="showSys" ${su.showSystem ? 'checked' : ''}><i></i></label></label></div>
    ${SECTIONS.map(([v, title, hint]) => {
      if (v === 'system' && !su.showSystem) return '';
      const list = shown.filter((i) => i.verdict === v);
      if (!list.length) return '';
      return `<h2>${title}<span class="hcount">${list.length}</span></h2>${hint ? `<div class="hint">${hint}</div>` : ''}<div class="card list">${list.map(startupRow).join('')}</div>`;
    }).join('') || '<div class="card empty">Nič také sa nespúšťa.</div>'}`;

  $$('#startupBody .chip').forEach((c) => (c.onclick = () => { su.kind = c.dataset.k; renderStartup(); }));
  $('#showSys').onchange = (e) => { su.showSystem = e.target.checked; renderStartup(); };
  if ($('#recBtn')) $('#recBtn').onclick = openRecommend;
  $$('#startupBody .srow').forEach((row) => {
    const cb = $('input[type=checkbox]', row);
    if (cb) cb.onchange = () => toggleItems([items.find((i) => i.id === row.dataset.id)], cb.checked);
  });
  bindTips($('#startupBody'));
}

/** Switches items on/off. One UAC prompt for everything that needs admin. */
async function toggleItems(list, enable) {
  list = list.filter((i) => i && i.toggle);
  if (!list.length) return;
  list.forEach((i) => su.busy.add(i.id));
  renderStartup();
  let res = [];
  try {
    res = await invoke('actions_apply', { actions: list.map((i) => ({ op: 'startup', id: i.id, name: i.app || i.name, toggle: i.toggle, enable })) });
  } catch (e) { res = list.map(() => ({ ok: false, error: String(e) })); }
  list.forEach((i) => su.busy.delete(i.id));
  const ok = res.filter((r) => r.ok).length, bad = res.filter((r) => !r.ok);
  if (ok) toast(enable ? `Zapnuté: ${ok}` : `Vypnuté: ${ok}. Prejaví sa pri ďalšom štarte.`);
  if (bad.length) toast(bad[0].error || 'Nepodarilo sa.', 'err');
  await loadStartup();
  helperDirty = true;
}

function openRecommend() {
  const rec = (startupItems || []).filter(recommended);
  const m = modal(`<h3>Vypnúť zo spúšťania</h3>
    <p class="mp">Nič sa nemaže. Položky sa len prestanú spúšťať so systémom a v Zmenách ich jedným klikom vrátiš.</p>
    <div class="mlist">${rec.map((i, k) => `<label class="mrow"><input type="checkbox" data-k="${k}" ${i.verdict === 'rgb' ? '' : 'checked'}>
      <div><b>${esc(i.app || i.name)}</b>${i.hidden ? ' <span class="badge ghost">Skryté</span>' : ''}<span>${esc(i.when)} · ${esc(i.source)}</span></div>${i.needs_admin ? SHIELD : ''}</label>`).join('')}</div>
    ${rec.some((i) => i.needs_admin) ? `<p class="mnote">${SHIELD}Pri službách a položkách pre všetkých používateľov sa raz objaví okno UAC.</p>` : ''}
    <div class="mact"><button class="btn" data-close>Zrušiť</button><button class="btn primary" id="recGo">Vypnúť vybrané</button></div>`);
  const count = () => { const n = $$('input:checked', m).length; $('#recGo', m).textContent = n ? `Vypnúť vybrané (${n})` : 'Vypnúť vybrané'; $('#recGo', m).disabled = !n; };
  $$('input', m).forEach((c) => (c.onchange = count));
  count();
  $('#recGo', m).onclick = () => {
    const pick = $$('input:checked', m).map((c) => rec[+c.dataset.k]);
    closeModal();
    toggleItems(pick, false);
  };
}

$('#startupRefresh').onclick = loadStartup;
$('#changesBtn').onclick = () => setView('history');
$('#startupSearch').addEventListener('input', debounce((e) => { su.q = e.target.value.trim().toLowerCase(); renderStartup(); }, 120));
views.startup = { enter() { loadStartup(); } };

/* ---------------------------------------------------------- modal + tips */
function modal(html) {
  closeModal();
  const back = document.createElement('div');
  back.className = 'modal-back';
  back.innerHTML = `<div class="modal" role="dialog">${html}</div>`;
  document.body.appendChild(back);
  back.onclick = (e) => { if (e.target === back || e.target.closest('[data-close]')) closeModal(); };
  return $('.modal', back);
}
function closeModal() { $$('.modal-back').forEach((b) => b.remove()); }
document.addEventListener('keydown', (e) => { if (e.key === 'Escape') closeModal(); });
function bindTips(root) {
  $$('[data-tip]', root).forEach((el) => {
    el.onmousemove = (e) => showTip(esc(el.dataset.tip), e.clientX, e.clientY);
    el.onmouseleave = hideTip;
  });
}

/* ========================================================== PERFORMANCE */
const DGPU = /nvidia|geforce|quadro|radeon\s*(rx|pro)|radeon\s*\d{3,4}|arc\s*a\d|arc\s*b\d/i;
const IGPU = /intel.*(uhd|iris|hd graphics)|radeon\(tm\)\s*graphics|^amd radeon graphics|radeon\s*\d{3}m|vega\s*\d+ graphics/i;
let displays = null, tweakStates = null;

function monthsSince(driverDate) {
  const m = /^(\d{1,2})-(\d{1,2})-(\d{4})$/.exec(driverDate || '');
  if (!m) return null;
  const d = new Date(+m[3], +m[1] - 1, +m[2]);
  return Math.floor((Date.now() - d) / (30.4 * 864e5));
}

/** Hardware checks that matter on any PC. Returns tips (helper format) and ok lines. */
function hwChecks(specs, displays, facts) {
  const tips = [], ok = [];
  if (!specs) return { tips, ok };
  const mods = specs.ram_modules || [];

  // RAM below its rated (XMP/EXPO) speed
  const slow = mods.filter((m) => m.rated_speed && m.configured_speed && m.configured_speed < m.rated_speed - 150);
  if (slow.length) {
    const m = slow[0];
    tips.push({ level: 'high', icon: 'ram', key: 'xmp', title: `RAM beží na ${sk(m.configured_speed)} MT/s, ale je predávaná ako ${sk(m.rated_speed)} MT/s`,
      text: 'Pamäť sa bez zapnutia profilu v BIOSe spustí na bezpečnej základnej rýchlosti. V hrách, kde brzdí procesor, býva rozdiel 5–15 %. Rýchlosť odhadujem z označenia modulu, ak si ju znížil zámerne kvôli stabilite, ignoruj to.',
      list: slow.map((x) => [`${x.slot || 'Modul'} · ${x.part}`, `${sk(x.configured_speed)} / ${sk(x.rated_speed)} MT/s`]),
      steps: ['Reštartuj PC a stláčaj Delete (alebo F2), otvorí sa BIOS.', 'Zapni profil pamäte: XMP (Intel), EXPO alebo DOCP (AMD). Býva v sekcii pamäte alebo pretaktovania, niektorí výrobcovia ho volajú inak (napr. A-XMP).', 'Ulož (F10). Prvý štart môže trvať dlhšie, pamäť sa „trénuje“.'] });
  } else if (mods.some((m) => m.rated_speed && m.configured_speed)) {
    ok.push(`RAM beží na plnej rýchlosti (${sk(mods[0].configured_speed)} MT/s)`);
  }

  // Single channel
  if (mods.length === 1) {
    tips.push({ level: 'mid', icon: 'ram', key: 'channel', title: 'Máš len jeden modul RAM, beží v jednom kanáli',
      text: 'Pamäť má polovičnú priepustnosť. Druhý rovnaký modul často pridá v hrách 10–20 %, pri integrovanej grafike aj viac. Pri notebooku s pamäťou napevno to ale nezmeníš.',
      steps: ['Dokúp rovnaký modul (rovnaké označenie, ' + (mods[0].part || 'pozri v Hardvéri') + ').', 'Podľa manuálu dosky ho daj do druhého kanála, väčšinou sloty A2 a B2.'] });
  } else if (mods.length >= 2) {
    const ch = mods.map((m) => m.channel);
    if (ch.every(Boolean) && new Set(ch).size === 1) {
      tips.push({ level: 'mid', icon: 'ram', key: 'channel', title: 'Moduly RAM sú v rovnakom kanáli',
        text: 'Podľa názvov slotov sú oba moduly v tom istom kanáli, takže pamäť beží len jednokanálovo.',
        list: mods.map((m) => [m.slot, `${sk(m.size_mb / 1024)} GB`]),
        steps: ['Pozri manuál dosky, ktoré sloty použiť pre dva moduly (väčšinou A2 a B2).'] });
    } else if (ch.every(Boolean)) ok.push('RAM beží vo viacerých kanáloch');
  }

  // Monitors
  for (const d of displays || []) {
    const name = `${d.width}×${d.height}`;
    if (d.max_hz > d.hz + 5) {
      tips.push({ level: 'high', icon: 'monitor', key: 'hz', title: `Monitor beží na ${d.hz} Hz, ale vie ${d.max_hz} Hz`,
        text: `Displej ${name} máš nastavený na nižšiu obnovovaciu frekvenciu, než zvláda. Obraz je trhanejší a myš má väčšie oneskorenie. Častá chyba pri novom monitore.`,
        steps: ['Nastavenia → Systém → Obrazovka → Rozšírené nastavenia obrazovky.', `Pri „Zvoliť obnovovaciu frekvenciu“ vyber ${d.max_hz} Hz.`, 'Ak tam vyššia nie je, použi DisplayPort alebo kvalitnejší HDMI kábel.'] });
    } else if (d.hz > 0) ok.push(`Monitor ${name} beží na plných ${d.hz} Hz`);
  }

  // Monitor plugged into the motherboard while a graphics card is installed (desktops only)
  const hasDgpu = (specs.gpus || []).some((g) => DGPU.test(g.name));
  const onIgpu = (displays || []).filter((d) => IGPU.test(d.adapter) && !DGPU.test(d.adapter));
  if (hasDgpu && onIgpu.length && !facts?.has_battery) {
    tips.push({ level: 'high', icon: 'gpu', key: 'cable', title: 'Monitor je zapojený do základnej dosky, nie do grafiky',
      text: `${onIgpu.length > 1 ? 'Niektoré monitory idú' : 'Monitor ide'} cez grafiku v procesore (${onIgpu[0].adapter}). Hry potom bežia pomalšie alebo s oneskorením, hoci máš výkonnú grafickú kartu.`,
      steps: ['Prepoj kábel monitora do portu na grafickej karte. Je nižšie na zadnej strane PC, porty sú v rade vodorovne.', 'Porty hore pri USB patria základnej doske, tie nepoužívaj.'] });
  } else if (hasDgpu && (displays || []).length) ok.push('Monitor je zapojený do grafickej karty');

  // Old GPU driver
  for (const g of (specs.gpus || []).filter((g) => DGPU.test(g.name))) {
    const mo = monthsSince(g.driver_date);
    if (mo == null) continue;
    const where = /nvidia|geforce|quadro/i.test(g.name) ? 'NVIDIA app alebo nvidia.com/drivers' : /radeon/i.test(g.name) ? 'AMD Software: Adrenalin Edition' : 'Intel Graphics Software';
    if (mo >= 6) {
      tips.push({ level: 'low', icon: 'gpu', key: 'driver', title: `Ovládač grafiky je ${mo} ${plural(mo, 'mesiac', 'mesiace', 'mesiacov')} starý`,
        text: `${g.name}, verzia ${g.driver_version || '?'}. Nové ovládače často prinášajú opravy a vyšší výkon v nových hrách.`,
        steps: [`Aktualizuj cez ${where}.`] });
    } else ok.push(`Ovládač grafiky je aktuálny (${g.driver_date.replace(/-/g, '. ')})`);
  }
  return { tips, ok };
}

const TWEAK_CATS = [
  ['responsive', 'Odozva systému'],
  ['games', 'Hry'],
  ['background', 'Na pozadí'],
  ['looks', 'Pre slabšie PC'],
];
const RECOMMENDED_TWEAKS = ['startup_delay', 'menu_delay', 'game_mode', 'hags', 'tips'];
const PLACEBO = [
  ['Vypnutie Nagleho algoritmu a TCP úpravy', 'Hry posielajú dáta po svojom (väčšinou cez UDP), na ping to nemá vplyv.'],
  ['SystemResponsiveness a NetworkThrottlingIndex', 'Týkajú sa len starej služby na prehrávanie médií. Hry z toho nič nemajú.'],
  ['Win32PrioritySeparation a priority procesov', 'Plánovač Windowsu 10 a 11 rieši priority sám, zmena nemá merateľný efekt.'],
  ['Vypnutie HPET a dynamic tick (bcdedit)', 'Na moderných PC môže odozvu skôr zhoršiť.'],
  ['Čističe RAM', 'Voľná RAM je nevyužitá RAM. Windows si ju po vyčistení znova naplní a appky sa načítavajú pomalšie.'],
  ['Vypnutie stránkovacieho súboru', 'Niektoré hry a appky padajú, keď chýba, aj keď máš RAM dosť.'],
  ['Vypnutie SysMain (Superfetch)', 'Na SSD skoro bez efektu, skôr spomalí spúšťanie appiek.'],
  ['Plán „Ultimate Performance“', 'Oproti Vysokému výkonu na stolnom PC nič nepridá.'],
];

async function loadPerf() {
  const body = $('#perfBody');
  if (!tweakStates) body.innerHTML = '<div class="card empty">Kontrolujem hardvér a nastavenia…</div>';
  const [f, sp, ds, tw] = await Promise.all([
    invoke('system_facts'), specs ? Promise.resolve(specs) : invoke('pc_specs'),
    invoke('displays_list').catch(() => []), invoke('tweaks_list'),
  ]);
  specs = sp; displays = ds; tweakStates = tw; perfFacts = f;
  renderPerf();
}
let perfFacts = null;

function tipCards(tips, store) {
  const level = { high: ['rec', 'Dôležité'], mid: ['amber', 'Odporúčam'], low: ['blue', 'Tip'] };
  return `<div class="tips">${tips.map((t, i) => {
    if (t.action) store[i] = t.action.run;
    return `<div class="card tipcard land lvl-${t.level}" style="--i:${i}">
      <div class="ti">${ICON[t.icon] || ICON.check}</div>
      <div class="tb">
        <div class="tt"><span>${esc(t.title)}</span><span class="badge lv-${level[t.level][0]}">${level[t.level][1]}</span></div>
        <div class="tx">${esc(t.text)}</div>
        ${t.list ? `<div class="tl">${t.list.map(([a, b]) => `<div><span>${esc(a)}</span><span class="num">${esc(b)}</span></div>`).join('')}</div>` : ''}
        ${t.steps ? `<ol class="ts">${t.steps.map((s) => `<li>${esc(s)}</li>`).join('')}</ol>` : ''}
      </div>
      ${t.action ? `<button class="btn small" data-a="${i}">${esc(t.action.label)}${ICON.arrow}</button>` : ''}
    </div>`;
  }).join('')}</div>`;
}

const tweakBusy = new Set();
function renderPerf() {
  const hw = hwChecks(specs, displays, perfFacts);
  const tw = (tweakStates || []).filter((t) => t.available);
  const off = tw.filter((t) => !t.on && RECOMMENDED_TWEAKS.includes(t.key));
  $('#perfCount').textContent = hw.tips.filter((t) => t.level === 'high').length || '';
  const restart = { logoff: 'Po odhlásení', reboot: 'Po reštarte' };
  const actions = [];
  $('#perfBody').innerHTML = `
    <h2>Kontrola hardvéru</h2>
    <div class="hint">Najväčšie rozdiely vo výkone bývajú v nastavení hardvéru, nie vo Windowse.</div>
    ${hw.tips.length ? tipCards(hw.tips, actions) : `<div class="card empty">${ICON.check}Hardvér je nastavený správne.</div>`}
    ${hw.ok.length ? `<div class="card oklist" style="margin-top:12px">${hw.ok.map((o) => `<div>${ICON.check}<span>${esc(o)}</span></div>`).join('')}</div>` : ''}

    <div class="h2row"><h2>Úpravy systému</h2>${off.length ? `<button class="btn primary small" id="twRec">Zapnúť odporúčané (${off.length})</button>` : ''}</div>
    <div class="hint">Len úpravy, ktoré naozaj niečo robia. Každú vypneš rovnakým prepínačom a nájdeš ju v Histórii.</div>
    ${TWEAK_CATS.map(([cat, title]) => {
      const list = tw.filter((t) => t.category === cat);
      if (!list.length) return '';
      return `<h3 class="cat">${title}</h3><div class="card list">${list.map((t) => {
        const busy = tweakBusy.has(t.key);
        const laptop = t.key === 'power_plan' && perfFacts?.has_battery;
        return `<div class="arow srow ${t.on ? '' : 'isoff'}" data-k="${t.key}">
          <div style="min-width:0"><div class="t">${esc(t.title)}
            <span class="badge ${t.effect === 'big' ? 'needed' : 'maybe'}">${t.effect === 'big' ? 'Citeľné' : 'Malý efekt'}</span>
            ${restart[t.restart] ? `<span class="badge maybe">${restart[t.restart]}</span>` : ''}
            ${RECOMMENDED_TWEAKS.includes(t.key) && !t.on ? '<span class="badge optional">Odporúčam</span>' : ''}</div>
            <div class="a">${esc(t.about)}${laptop ? ' <b>Máš notebook, na batérii to neodporúčam.</b>' : ''}</div></div>
          <div class="sw">${t.admin ? `<span data-tip="Zmena si vypýta povolenie správcu (UAC).">${SHIELD}</span>` : ''}
            <label class="switch ${busy ? 'busy' : ''}"><input type="checkbox" ${t.on ? 'checked' : ''} ${busy ? 'disabled' : ''}><i></i></label></div></div>`;
      }).join('')}</div>`;
    }).join('')}

    <details class="card placebo"><summary>${ICON.warn}<span><b>Čo zámerne nerobím</b><small>Obľúbené „optimalizácie“, ktoré na Windowse 10 a 11 nepomáhajú alebo škodia</small></span></summary>
      <div class="plist">${PLACEBO.map(([a, b]) => `<div><b>${esc(a)}</b><span>${esc(b)}</span></div>`).join('')}</div></details>`;

  $$('#perfBody [data-a]').forEach((b) => (b.onclick = () => actions[+b.dataset.a]?.()));
  $$('#perfBody .srow').forEach((row) => {
    const t = tw.find((x) => x.key === row.dataset.k);
    $('input', row).onchange = (e) => applyTweaks([t], e.target.checked);
  });
  if ($('#twRec')) $('#twRec').onclick = () => {
    const m = modal(`<h3>Zapnúť odporúčané úpravy</h3>
      <p class="mp">Každú úpravu neskôr vypneš rovnakým prepínačom alebo ju vrátiš v Histórii.</p>
      <div class="mlist">${off.map((t, k) => `<label class="mrow"><input type="checkbox" data-k="${k}" checked><div><b>${esc(t.title)}</b><span>${esc(t.about)}</span></div>${t.admin ? SHIELD : ''}</label>`).join('')}</div>
      <div class="mact"><button class="btn" data-close>Zrušiť</button><button class="btn primary" id="twGo">Zapnúť</button></div>`);
    $('#twGo', m).onclick = () => { const pick = $$('input:checked', m).map((c) => off[+c.dataset.k]); closeModal(); applyTweaks(pick, true); };
  };
  bindTips($('#perfBody'));
}

async function applyTweaks(list, enable) {
  if (!list.length) return;
  list.forEach((t) => tweakBusy.add(t.key));
  renderPerf();
  const res = await invoke('actions_apply', { actions: list.map((t) => ({ op: 'startup', id: `tweak:${t.key}`, name: t.title, toggle: { t: 'tweak', key: t.key }, enable })) })
    .catch((e) => list.map(() => ({ ok: false, error: String(e) })));
  list.forEach((t) => tweakBusy.delete(t.key));
  const ok = list.filter((_, i) => res[i]?.ok), bad = res.find((r) => !r.ok);
  if (ok.length) {
    const r = ok.find((t) => t.restart === 'reboot') ? ' Prejaví sa po reštarte.' : ok.find((t) => t.restart === 'logoff') ? ' Prejaví sa po odhlásení.' : '';
    toast(`${enable ? 'Zapnuté' : 'Vypnuté'}: ${ok.length}.${r}`);
  }
  if (bad) toast(bad.error || 'Nepodarilo sa.', 'err');
  tweakStates = await invoke('tweaks_list');
  renderPerf();
  helperDirty = true;
}

$('#perfRefresh').onclick = () => { tweakStates = null; loadPerf(); };
views.perf = { enter() { loadPerf(); } };

/* ========================================================== HISTORY */
const hi = { filter: 'all' };
const HKINDS = [
  ['all', 'Všetko', () => true],
  ['startup', 'Spúšťanie', (c) => c.toggle && c.toggle.t !== 'tweak'],
  ['tweak', 'Úpravy', (c) => c.toggle?.t === 'tweak'],
  ['kill', 'Ukončené procesy', (c) => !c.toggle],
];
function changeText(c) {
  const t = c.toggle;
  if (!t) return ['Proces ukončený', 'rec'];
  if (t.t === 'tweak') return [c.enabled ? 'Úprava zapnutá' : 'Úprava vypnutá', c.enabled ? 'mint' : 'amber'];
  const where = { approved: 'zo spúšťania', task: 'naplánovaná úloha', service: 'služba' }[t.t] || '';
  if (t.t === 'service') return [c.enabled ? 'Služba sa znova spúšťa automaticky' : 'Služba prepnutá na ručné spúšťanie', c.enabled ? 'mint' : 'amber'];
  return [`${c.enabled ? 'Zapnuté' : 'Vypnuté'} · ${where}`, c.enabled ? 'mint' : 'amber'];
}
function dayLabel(d) {
  const today = new Date(); today.setHours(0, 0, 0, 0);
  const x = new Date(d); x.setHours(0, 0, 0, 0);
  const diff = Math.round((today - x) / 864e5);
  if (diff === 0) return 'Dnes';
  if (diff === 1) return 'Včera';
  return x.toLocaleDateString('sk-SK', { weekday: 'long', day: 'numeric', month: 'long', year: x.getFullYear() === today.getFullYear() ? undefined : 'numeric' });
}
async function loadHistory() {
  changesLog = await invoke('changes_list').catch(() => []);
  renderHistory();
}
function renderHistory() {
  const all = changesLog.map((c, i) => ({ ...c, i }));
  // Only the newest change of an item can be undone; older ones are history.
  const latest = new Map();
  all.forEach((c) => { if (c.toggle && !c.undone) latest.set(c.id, c.i); });
  const f = (HKINDS.find((k) => k[0] === hi.filter) || HKINDS[0])[2];
  const list = all.filter(f).reverse();
  const active = all.filter((c) => c.toggle && !c.undone && latest.get(c.id) === c.i);
  $('#historyBody').innerHTML = `
    <div class="stats">
      <div class="card stat"><div class="k">Zmien celkom</div><div class="v num">${sk(all.length)}</div></div>
      <div class="card stat"><div class="k">Platí teraz</div><div class="v num">${sk(active.length)}</div></div>
      <div class="card stat"><div class="k">Vrátené</div><div class="v num">${sk(all.filter((c) => c.undone).length)}</div></div>
      <div class="card stat"><div class="k">Ukončené procesy</div><div class="v num">${sk(all.filter((c) => !c.toggle).length)}</div></div>
    </div>
    <div class="chips">${HKINDS.map(([k, l, fn]) => { const n = all.filter(fn).length; return k !== 'all' && !n ? '' : `<button class="chip ${hi.filter === k ? 'on' : ''}" data-k="${k}">${l}<span class="n">${n}</span></button>`; }).join('')}</div>
    ${list.length ? (() => {
      let out = '', day = '';
      for (const c of list) {
        const dl = dayLabel(c.at * 1000);
        if (dl !== day) { if (day) out += '</div>'; out += `<h3 class="cat">${esc(dl)}</h3><div class="card list">`; day = dl; }
        const [txt, dot] = changeText(c);
        const canUndo = c.toggle && !c.undone && latest.get(c.id) === c.i;
        out += `<div class="arow hrow ${c.undone ? 'isoff' : ''}"><span class="dot ${c.undone ? '' : dot}"></span>
          <div style="min-width:0"><div class="t">${esc(c.name)}${c.undone ? '<span class="badge off">Vrátené</span>' : ''}</div><div class="a">${esc(txt)}</div></div>
          <span class="time num">${new Date(c.at * 1000).toLocaleTimeString('sk-SK', { hour: '2-digit', minute: '2-digit' })}</span>
          ${canUndo ? `<button class="btn small" data-undo="${c.i}">Vrátiť</button>` : '<span></span>'}</div>`;
      }
      return out + '</div>';
    })() : `<div class="card empty">${ICON.clock}Zatiaľ si cez WhatsInside nič nezmenil.</div>`}
    <div class="note">História sa ukladá v priečinku %LOCALAPPDATA%\\WhatsInside. Ukončený proces sa vrátiť nedá, stačí appku znova spustiť.</div>`;
  $$('#historyBody .chip').forEach((b) => (b.onclick = () => { hi.filter = b.dataset.k; renderHistory(); }));
  $$('#historyBody [data-undo]').forEach((b) => (b.onclick = async () => {
    b.disabled = true;
    try { await invoke('changes_undo', { index: +b.dataset.undo }); toast('Vrátené'); }
    catch (e) { toast(String(e), 'err'); }
    startupItems = null; tweakStates = null; helperDirty = true;
    loadHistory();
  }));
  $('#historyCount').textContent = '';
}
views.history = { enter() { loadHistory(); } };

/* ========================================================== HELPER */
let helperDirty = true, helperBusy = false;
const POWER_SAVER = 'a1841308-3541-4fab-bc81-f71556f20b4a';

async function runHelper() {
  if (helperBusy) return;
  helperBusy = true;
  const body = $('#helperBody');
  if (!body.children.length) {
    body.innerHTML = `<div class="card scanning land"><div class="title"><span class="dot amber pulse"></span>Kontrolujem tvoj systém</div>
      <div class="note" style="margin-top:6px">Disky, dočasné súbory, procesy, automatické spúšťanie a nastavenia. Chvíľu to trvá.</div><div class="bar indet" style="margin-top:14px"><i></i></div></div>`;
  }
  try {
    const [facts, quick, snap, auto, drives, sp, ds] = await Promise.all([
      invoke('system_facts'), invoke('quick_check'), invoke('procs_snapshot'),
      startupItems ? Promise.resolve(startupItems) : invoke('autostart_list'),
      invoke('storage_drives'), specs ? Promise.resolve(specs) : invoke('pc_specs').catch(() => null),
      displays ? Promise.resolve(displays) : invoke('displays_list').catch(() => []),
    ]);
    startupItems = auto;
    if (sp) specs = sp;
    displays = ds;
    renderStartup();
    renderHelper(buildAdvice({ facts, quick, snap, auto, drives, specs: sp, displays: ds }));
    helperDirty = false;
  } catch (e) {
    body.innerHTML = `<div class="card empty">${ICON.warn}Kontrola sa nepodarila: ${esc(e)}</div>`;
  }
  helperBusy = false;
}

/** Turns raw facts into plain-language advice. Every item says why and how. */
function buildAdvice({ facts, quick, snap, auto, drives, specs, displays }) {
  const tips = [], ok = [];
  // 0. Hardware set up wrong (RAM profile, refresh rate, cable, driver)
  const hw = hwChecks(specs, displays, facts);
  hw.tips.forEach((t) => tips.push({ ...t, action: t.action || { label: 'Výkon', run: () => setView('perf') } }));
  ok.push(...hw.ok);
  // 1. Disk space
  for (const d of drives.filter((d) => !d.removable && d.total)) {
    const pct = d.free / d.total;
    const letter = d.root.replace(/\\$/, '');
    if (pct < 0.1 || d.free < 25 * 2 ** 30) {
      tips.push({ level: pct < 0.05 ? 'high' : 'mid', icon: 'drive', title: `Na disku ${letter} ti dochádza miesto`,
        text: `Voľných zostáva ${bytes(d.free)} z ${bytes(d.total)}. Keď je disk takmer plný, Windows sa spomaľuje a aktualizácie môžu zlyhať.`,
        action: { label: 'Prezrieť disk', run: () => { setView('storage'); startScan(d.root); } } });
    }
  }
  if (!tips.some((t) => t.icon === 'drive')) ok.push('Na diskoch máš dosť miesta');

  // 2. Caches – full scan results win (they also find Resolve caches), else the quick check.
  const scanCaches = st.status?.phase === 'done' ? st.caches : null;
  const caches = scanCaches || quick.caches;
  const cacheTotal = caches.reduce((a, c) => a + c.size, 0);
  if (cacheTotal > 1 * 2 ** 30) {
    const top = [...caches].sort((a, b) => b.size - a.size).slice(0, 4);
    tips.push({ level: cacheTotal > 10 * 2 ** 30 ? 'mid' : 'low', icon: 'broom', gain: cacheTotal, title: `Môžeš uvoľniť ${bytes(cacheTotal)} cache`,
      text: 'Dočasné súbory a cache, ktoré si appky vytvoria znova. Ich zmazanie nič nepokazí.',
      list: top.map((c) => [c.label, bytes(c.size)]),
      steps: ['Väčšinu vyčistíš v Nastavenia → Systém → Úložisko → Dočasné súbory.', 'Cache Resolve a Adobe maž radšej v samotnej appke (návod je v Úložisku pri každej položke).'],
      action: scanCaches ? { label: 'Ukázať', run: () => { setView('storage'); selectTab('cache'); } } : { label: 'Nájsť viac (sken disku)', run: () => setView('storage') } });
  } else ok.push('Cache a dočasné súbory sú v norme');

  // 3. Old installers
  const inst = st.status?.phase === 'done' ? st.old.filter((o) => o.reason === 'installer') : quick.installers;
  const instTotal = inst.reduce((a, x) => a + x.size, 0);
  if (instTotal > 500 * 2 ** 20) {
    tips.push({ level: 'low', icon: 'file', gain: instTotal, title: `Staré inštalačky v Stiahnutých: ${bytes(instTotal)}`,
      text: `${inst.length} ${plural(inst.length, 'súbor', 'súbory', 'súborov')} (.exe, .msi, .iso, .zip…) starších ako mesiac. Program už máš nainštalovaný, inštalačku netreba.`,
      list: inst.slice(0, 4).map((o) => [splitPath(o.path)[0], bytes(o.size)]) });
  }

  // 4. Hibernation file
  if (facts.hibernation && facts.hiberfil_bytes > 4 * 2 ** 30) {
    tips.push({ level: 'low', icon: 'bolt', gain: facts.hiberfil_bytes, title: `Hibernácia zaberá ${bytes(facts.hiberfil_bytes)}`,
      text: 'Ak PC nedávaš do hibernácie (bežné uspanie nie je hibernácia), tento súbor netreba.',
      steps: ['Otvor Terminál ako správca a spusti: powercfg /h off', 'Vypne to aj „Rýchle spustenie“. Na NVMe disku to nespoznáš.'] });
  }

  // 5. RAM pressure
  const rp = snap.ram_total ? snap.ram_used / snap.ram_total : 0;
  if (rp > 0.85) {
    const heavy = snap.groups.filter((g) => g.verdict !== 'system').slice(0, 3);
    tips.push({ level: 'high', icon: 'ram', title: 'Pamäť je takmer plná',
      text: `Používaš ${bytes(snap.ram_used)} z ${bytes(snap.ram_total)}. Keď dôjde, Windows začne odkladať na disk a všetko sa spomalí.`,
      list: heavy.map((g) => [g.name, bytes(g.ram)]), action: { label: 'Procesy', run: () => setView('procs') } });
  } else ok.push(`Pamäť je v pohode (${sk(rp * 100, 0)} % využitých)`);

  // 6. Vendor RGB apps
  const rgb = snap.groups.filter((g) => g.verdict === 'rgb');
  if (rgb.length) {
    const ram = rgb.reduce((a, g) => a + g.ram, 0);
    tips.push({ level: 'low', icon: 'gpu', title: `Appky na svetlá ťa stoja ${bytes(ram)} RAM`,
      text: 'Bežia stále, aj keď na svetlá nesiahaš. Niektoré zariadenia si ale farbu bez nich nepamätajú, preto ich zatiaľ nevypínaj naslepo. Ovládanie RGB priamo vo WhatsInside pripravujem.',
      list: rgb.map((g) => [g.name, `${bytes(g.ram)} · ${g.procs.length} ${plural(g.procs.length, 'proces', 'procesy', 'procesov')}`]),
      action: { label: 'Ukázať procesy', run: () => { pr.filter = 'rgb'; setView('procs'); } } });
  }

  // 7. Autostart
  const extra = auto.filter(recommended);
  if (extra.length) {
    tips.push({ level: extra.length > 4 ? 'mid' : 'low', icon: 'rocket', title: `${extra.length} ${plural(extra.length, 'vec sa spúšťa', 'veci sa spúšťajú', 'vecí sa spúšťa')} s Windowsom zbytočne`,
      text: `Spomaľujú štart a potom bežia na pozadí.${extra.some((i) => i.hidden) ? ` ${extra.filter((i) => i.hidden).length} z nich v Správcovi úloh ani neuvidíš (naplánované úlohy a služby).` : ''} Vypneš ich tu jedným klikom a v Zmenách ich kedykoľvek vrátiš.`,
      list: extra.slice(0, 6).map((i) => [i.app || i.name, i.when]),
      action: { label: 'Vybrať a vypnúť', run: () => { setView('startup'); setTimeout(() => startupItems && openRecommend(), 400); } } });
  } else ok.push('Po spustení sa nespúšťa nič zbytočné');

  // 8. Heavy optional apps running right now
  const heavyOpt = snap.groups.filter((g) => g.verdict === 'optional' && g.ram > 150 * 2 ** 20);
  if (heavyOpt.length) {
    tips.push({ level: 'low', icon: 'ram', title: 'Na pozadí beží niečo, čo nepotrebuješ stále',
      text: 'Tieto appky práve zaberajú pamäť. Ak ich teraz nepoužívaš, zavri ich.',
      list: heavyOpt.map((g) => [g.name, bytes(g.ram)]) });
  }

  // 9. Power plan
  if (facts.power_guid === POWER_SAVER) {
    tips.push({ level: 'mid', icon: 'bolt', title: 'Máš zapnutý úsporný režim napájania',
      text: 'Procesor sa v ňom drží na nízkych taktoch, hry aj export videa sú pomalšie.',
      steps: ['Nastavenia → Systém → Napájanie → Režim napájania → Vyvážený alebo Najlepší výkon.'],
      action: { label: 'Výkon', run: () => setView('perf') } });
  } else if (facts.power_name) ok.push(`Plán napájania: ${facts.power_name}`);

  // 10. Game Mode, HAGS: one click in Výkon
  if (facts.game_mode === false) {
    tips.push({ level: 'low', icon: 'game', title: 'Herný režim je vypnutý',
      text: 'Pri hraní dáva hre prednosť pred aktualizáciami a appkami na pozadí. Na výkonnom PC je rozdiel malý, ale škodiť nebude.',
      action: { label: 'Zapnúť vo Výkone', run: () => setView('perf') } });
  } else if (facts.game_mode) ok.push('Herný režim je zapnutý');
  const gpuName = (specs?.gpus || []).map((g) => g.name).join(' ');
  const rtxFg = /RTX\s*(40|50)\d\d/i.test(gpuName);
  if (facts.hags === false && rtxFg) {
    tips.push({ level: 'mid', icon: 'gpu', title: 'Plánovanie GPU s hardvérovou akceleráciou je vypnuté',
      text: 'Tvoja grafika podporuje DLSS Frame Generation, ktoré bez neho nefunguje.',
      action: { label: 'Zapnúť vo Výkone', run: () => setView('perf') } });
  } else if (facts.hags) ok.push('Plánovanie GPU s hardvérovou akceleráciou je zapnuté');

  const order = { high: 0, mid: 1, low: 2 };
  tips.sort((a, b) => order[a.level] - order[b.level]);
  return { tips, ok, gain: tips.reduce((a, t) => a + (t.gain || 0), 0), scanned: st.status?.phase === 'done' };
}

let helperActions = [];
function renderHelper(r) {
  helperActions = [];
  $('#helperCount').textContent = r.tips.filter((t) => t.level !== 'low').length || '';
  $('#helperSub').textContent = r.tips.length
    ? `Našiel som ${r.tips.length} ${plural(r.tips.length, 'vec', 'veci', 'vecí')}, ktoré stoja za pozornosť${r.gain ? ` · môžeš uvoľniť ${bytes(r.gain)}` : ''}`
    : 'Všetko vyzerá dobre';
  $('#helperBody').innerHTML =
    (r.tips.length ? tipCards(r.tips, helperActions) : `<div class="card empty">${ICON.check}Nenašiel som nič, čo by stálo za zmenu. Pekne udržiavaný počítač.</div>`) +
    (r.ok.length ? `<h2>V poriadku</h2><div class="card oklist">${r.ok.map((o) => `<div>${ICON.check}<span>${esc(o)}</span></div>`).join('')}</div>` : '') +
    (r.scanned ? '' : '<div class="note">Tip: po skene disku v Úložisku nájdem aj cache strihových a grafických programov, staré súbory a hry, ktoré už nehráš.</div>') +
    '<div class="note">Sám nič nemením. Kde sa dá, zmeníš to jedným klikom a v Histórii to vrátiš.</div>';
  $$('#helperBody [data-a]').forEach((b) => (b.onclick = () => helperActions[+b.dataset.a]?.()));
}
$('#helperRefresh').onclick = () => { helperDirty = true; $('#helperBody').innerHTML = ''; runHelper(); };
views.helper = { enter() { if (helperDirty) runHelper(); } };

/* ========================================================== HARDWARE + PC CARD */
let hwLoaded = false, specs = null, hwReport = null;
const CARD_ICONS = {
  cpu: 'M4 4h8v8H4z M6.5 1.5V4M9.5 1.5V4M6.5 12v2.5M9.5 12v2.5M1.5 6.5H4M1.5 9.5H4M12 6.5h2.5M12 9.5h2.5',
  gpu: 'M1.5 4h13v7.5h-13z M3 11.5v2 M7.1 7.8a1.9 1.9 0 1 1-3.8 0a1.9 1.9 0 1 1 3.8 0 M12.7 7.8a1.9 1.9 0 1 1-3.8 0a1.9 1.9 0 1 1 3.8 0',
  board: 'M2 2h12v12H2z M4.5 4.5h4v4h-4z M11 4.5v1 M11 7.5v1 M4.5 11h7',
  ram: 'M1.5 5h13v5h-13z M4 10v2 M7 10v2 M10 10v2 M13 10v2 M4 7h1.5 M7.2 7h1.5 M10.5 7h1.5',
  storage: 'M1.8 3h12.4v10H1.8z M1.8 9.5h12.4 M11.2 11.3h.01',
  cooler: 'M14 8a6 6 0 1 1-12 0a6 6 0 1 1 12 0 M8 8c0-2 1-3.5 3-3.5 M8 8c-2 0-3.5-1-3.5-3 M8 8c0 2-1 3.5-3 3.5 M8 8c2 0 3.5 1 3.5 3',
  case: 'M4.5 1.5h7v13h-7z M6.5 4h3 M8 11.5h.01',
  psu: 'M9 1.5L3.5 9h4l-1 5.5L12.5 7h-4z',
  monitor: 'M1.5 2.5h13v8.5h-13z M6 14h4 M8 11v3',
  mouse: 'M8 1.5c2.5 0 4 1.8 4 4.5v4c0 2.5-1.6 4.5-4 4.5s-4-2-4-4.5V6c0-2.7 1.5-4.5 4-4.5z M8 4v2.5',
  os: 'M2 3.5l5-.7v4.7H2z M8 2.7l6-.9v5.7H8z M2 8.5h5v4.7l-5-.7z M8 8.5h6v5.7l-6-.9z',
  star: 'M8 1.8l1.8 3.9 4.2.5-3.1 2.9.8 4.2L8 11.2l-3.7 2.1.8-4.2L2 6.2l4.2-.5z',
};
const ROW_DEFS = [
  ['cpu', 'Procesor', 'cpu'], ['gpu', 'Grafika', 'gpu'], ['board', 'Doska', 'board'], ['ram', 'Pamäť', 'ram'],
  ['storage', 'Úložisko', 'storage'], ['cooler', 'Chladenie', 'cooler'], ['case', 'Skrinka', 'case'], ['psu', 'Zdroj', 'psu'],
  ['monitor', 'Monitor', 'monitor'], ['peripherals', 'Periférie', 'mouse'], ['os', 'Systém', 'os'],
];
const ACCENTS = ['#ff9f0a', '#0a84ff', '#bf5af2', '#30d158', '#ff375f', '#ff453a', '#64d2ff', '#ffd60a'];
const card = loadCard();

function loadCard() {
  let c = null;
  try { c = JSON.parse(localStorage.getItem('pccard') || 'null'); } catch {}
  return c || { title: 'Môj PC', subtitle: '', theme: 'dark', accent: '#ff9f0a', finish: 'rainbow', footer: true, rows: [] };
}
function saveCard() { try { localStorage.setItem('pccard', JSON.stringify(card)); } catch {} }

function detectedValues() {
  const v = {};
  if (!specs) return v;
  v.cpu = specs.cpu.replace(/\((R|TM)\)/gi, '').replace(/\s+CPU\s+@.*$/i, '').replace(/\s{2,}/g, ' ').trim();
  const known = hwReport?.devices.find((d) => d.kind === 'Grafika' && d.supported);
  const g = specs.gpus[0];
  if (known || g) v.gpu = `${known ? known.name.replace('GeForce ', '') : g.name}${g?.vram ? ` · ${Math.round(g.vram / 2 ** 30)} GB` : ''}`;
  v.board = specs.board;
  const mods = specs.ram_modules;
  if (mods.length) {
    const total = mods.reduce((a, m) => a + m.size_mb, 0) / 1024;
    const speed = mods[0].configured_speed || mods[0].speed;
    const vendor = mods[0].manufacturer && !/unknown|undefined/i.test(mods[0].manufacturer) ? ` · ${prettyVendor(mods[0].manufacturer)}` : '';
    v.ram = `${sk(total, 0)} GB ${mods[0].kind || ''} ${speed ? `${speed} MT/s` : ''}`.replace(/\s+/g, ' ').trim() + vendor;
  } else if (specs.ram_total) v.ram = `${Math.round(specs.ram_total / 2 ** 30)} GB`;
  v.storage = specs.disks.join(' + ');
  v.os = specs.os;
  return v;
}
function prettyVendor(s) {
  const m = [['g skill', 'G.Skill'], ['kingston', 'Kingston'], ['corsair', 'Corsair'], ['crucial', 'Crucial'], ['samsung', 'Samsung'], ['hynix', 'SK hynix'], ['micron', 'Micron'], ['team', 'TeamGroup'], ['patriot', 'Patriot'], ['adata', 'ADATA']];
  const l = s.toLowerCase();
  return (m.find(([k]) => l.includes(k)) || [0, s.trim()])[1];
}

/** Fills rows with detected values without overwriting what the user typed. */
function syncRows(force = false) {
  const det = detectedValues();
  const have = new Map(card.rows.map((r) => [r.key, r]));
  for (const [key, label, icon] of ROW_DEFS) {
    const r = have.get(key);
    if (!r) {
      card.rows.push({ key, label, icon, value: det[key] || '', on: !!det[key], edited: false });
    } else if ((force || !r.edited) && det[key]) {
      r.value = det[key];
      if (force) r.on = true;
    }
  }
  saveCard();
}

function renderEditor() {
  const ed = $('#cardEditor');
  ed.innerHTML = `
    <div class="erow2"><label class="field"><span>Nadpis</span><input id="cTitle" value="${esc(card.title)}" maxlength="40"></label>
      <label class="field"><span>Podnadpis</span><input id="cSub" value="${esc(card.subtitle)}" placeholder="napr. tvoj nick" maxlength="50"></label></div>
    <div class="erow">
      <div class="seg" id="cTheme"><div class="knob"></div><button data-k="dark" class="${card.theme === 'dark' ? 'on' : ''}">Tmavá</button><button data-k="light" class="${card.theme === 'light' ? 'on' : ''}">Svetlá</button></div>
      <div class="seg" id="cLayout"><div class="knob"></div><button data-k="rainbow" class="${card.finish !== 'gold' ? 'on' : ''}">Dúha</button><button data-k="gold" class="${card.finish === 'gold' ? 'on' : ''}">Zlato</button></div>
    </div>
    <div class="swatches" id="cAccent">${ACCENTS.map((a) => `<button style="--s:${a}" class="${a === card.accent ? 'on' : ''}" data-a="${a}" title="${a}"></button>`).join('')}</div>
    <div class="rows" id="cRows">${card.rows.map((r, i) => `
      <div class="crow" data-i="${i}">
        <label class="switch"><input type="checkbox" ${r.on ? 'checked' : ''}><i></i></label>
        <input class="lab" value="${esc(r.label)}" maxlength="20">
        <input class="val" value="${esc(r.value)}" placeholder="${r.key.startsWith('custom') ? 'Hodnota' : 'Doplň sám'}" maxlength="80">
        <div class="mv"><button class="btn icon" data-m="-1" title="Vyššie"><svg viewBox="0 0 16 16"><path d="M4 10l4-4 4 4"/></svg></button><button class="btn icon" data-m="1" title="Nižšie"><svg viewBox="0 0 16 16"><path d="M4 6l4 4 4-4"/></svg></button>${r.key.startsWith('custom') ? `<button class="btn icon" data-del title="Odstrániť">${ICON.x}</button>` : ''}</div>
      </div>`).join('')}</div>
    <div class="erow">
      <button class="btn small" id="cAdd">+ Pridať riadok</button>
      <button class="btn small ghost" id="cReset">Načítať zistené hodnoty</button>
      <label class="swl"><span>Podpis WhatsInside</span><label class="switch"><input type="checkbox" id="cFooter" ${card.footer ? 'checked' : ''}><i></i></label></label>
    </div>`;
  const redraw = () => { saveCard(); drawCard(); };
  $('#cTitle').oninput = (e) => { card.title = e.target.value; redraw(); };
  $('#cSub').oninput = (e) => { card.subtitle = e.target.value; redraw(); };
  seg($('#cTheme'), (k) => { card.theme = k; redraw(); });
  seg($('#cLayout'), (k) => { card.finish = k; $('#holo').classList.toggle('gold', k === 'gold'); redraw(); });
  $$('#cAccent button').forEach((b) => (b.onclick = () => { card.accent = b.dataset.a; $$('#cAccent button').forEach((x) => x.classList.toggle('on', x === b)); redraw(); }));
  $$('#cRows .crow').forEach((row) => {
    const r = card.rows[+row.dataset.i];
    $('input[type=checkbox]', row).onchange = (e) => { r.on = e.target.checked; redraw(); };
    $('.lab', row).oninput = (e) => { r.label = e.target.value; redraw(); };
    $('.val', row).oninput = (e) => { r.value = e.target.value; r.edited = true; if (r.value && !r.on) { r.on = true; $('input[type=checkbox]', row).checked = true; } redraw(); };
    $$('[data-m]', row).forEach((b) => (b.onclick = () => {
      const i = +row.dataset.i, j = i + +b.dataset.m;
      if (j < 0 || j >= card.rows.length) return;
      [card.rows[i], card.rows[j]] = [card.rows[j], card.rows[i]];
      renderEditor(); redraw();
    }));
    const del = $('[data-del]', row);
    if (del) del.onclick = () => { card.rows.splice(+row.dataset.i, 1); renderEditor(); redraw(); };
  });
  $('#cAdd').onclick = () => { card.rows.push({ key: `custom${Date.now()}`, label: 'Vlastné', icon: 'star', value: '', on: true, edited: true }); renderEditor(); redraw(); $$('#cRows .val').at(-1)?.focus(); };
  $('#cReset').onclick = () => { card.rows.forEach((r) => (r.edited = false)); syncRows(true); renderEditor(); redraw(); toast('Načítal som zistené hodnoty'); };
  $('#cFooter').onchange = (e) => { card.footer = e.target.checked; redraw(); };
}

function roundRect(ctx, x, y, w, h, r) {
  ctx.beginPath();
  ctx.moveTo(x + r, y);
  ctx.arcTo(x + w, y, x + w, y + h, r);
  ctx.arcTo(x + w, y + h, x, y + h, r);
  ctx.arcTo(x, y + h, x, y, r);
  ctx.arcTo(x, y, x + w, y, r);
  ctx.closePath();
}
function fitText(ctx, text, maxW) {
  if (ctx.measureText(text).width <= maxW) return text;
  let t = text;
  while (t.length > 1 && ctx.measureText(t + '…').width > maxW) t = t.slice(0, -1);
  return t + '…';
}
function hexA(hex, a) {
  const n = parseInt(hex.slice(1), 16);
  return `rgba(${n >> 16},${(n >> 8) & 255},${n & 255},${a})`;
}

/* ---- Pokémon-style PC card: drawn on a canvas (preview, PNG export), shine is CSS on top ---- */
function hashStr(s) { let h = 2166136261; for (let i = 0; i < s.length; i++) { h ^= s.charCodeAt(i); h = Math.imul(h, 16777619); } return h >>> 0; }
function rng(seed) { let s = seed || 1; return () => ((s = Math.imul(s ^ (s >>> 15), 2246822507) ^ Math.imul(s ^ (s >>> 13), 3266489909)) >>> 0) / 4294967296; }
function mixHex(hex, to, t) {
  const a = parseInt(hex.slice(1), 16), b = parseInt(to.slice(1), 16);
  const c = (s) => Math.round(((a >> s) & 255) * (1 - t) + ((b >> s) & 255) * t);
  return `rgb(${c(16)},${c(8)},${c(0)})`;
}
function fitFont(ctx, text, maxW, size, min, weight) {
  let s = size;
  for (; s > min; s -= 1) { ctx.font = `${weight} ${s}px "Inter Variable", "Segoe UI", sans-serif`; if (ctx.measureText(text).width <= maxW) return text; }
  ctx.font = `${weight} ${min}px "Inter Variable", "Segoe UI", sans-serif`;
  return fitText(ctx, text, maxW);
}
function star4(ctx, x, y, r) {
  ctx.beginPath();
  ctx.moveTo(x, y - r); ctx.quadraticCurveTo(x, y, x + r, y); ctx.quadraticCurveTo(x, y, x, y + r);
  ctx.quadraticCurveTo(x, y, x - r, y); ctx.quadraticCurveTo(x, y, x, y - r); ctx.fill();
}
function rarity() {
  const ram = specs ? specs.ram_total / 2 ** 30 : 0;
  const vram = specs?.gpus?.[0]?.vram ? specs.gpus[0].vram / 2 ** 30 : 0;
  if (vram >= 16 && ram >= 30) return '★ ULTRA RARE';
  if (vram >= 8 || ram >= 15) return '◆ RARE';
  return '● COMMON';
}

/** Draws the card. Same function for the preview and the export (scale 2). */
function drawCard(canvas = $('#pcCanvas'), scale = 2) {
  const rows = card.rows.filter((r) => r.on && r.value.trim());
  const dark = card.theme === 'dark', A = card.accent, gold = card.finish === 'gold';
  const F = (w, s) => `${w} ${s}px "Inter Variable", "Segoe UI", sans-serif`;
  const C = dark
    ? { inner: '#18181b', text: '#f5f5f7', muted: '#a1a1a6', line: 'rgba(255,255,255,.1)' }
    : { inner: '#fbf7ec', text: '#1d1d1f', muted: '#6e6e73', line: 'rgba(0,0,0,.1)' };
  const W = 750, H = 1050;
  canvas.width = W * scale;
  canvas.height = H * scale;
  const ctx = canvas.getContext('2d');
  ctx.setTransform(scale, 0, 0, scale, 0, 0);
  ctx.clearRect(0, 0, W, H);
  ctx.textBaseline = 'alphabetic';
  const seed = hashStr(`${card.title}|${A}`);

  // frame
  roundRect(ctx, 0, 0, W, H, 40);
  ctx.save(); ctx.clip();
  const fg = ctx.createLinearGradient(0, 0, W, H);
  const base = gold ? '#e9b949' : A;
  [[0, mixHex(base, '#000000', 0.15)], [0.22, mixHex(base, '#ffffff', 0.55)], [0.5, base], [0.74, mixHex(base, '#ffffff', 0.4)], [1, mixHex(base, '#000000', 0.45)]].forEach(([o, c]) => fg.addColorStop(o, c));
  ctx.fillStyle = fg; ctx.fillRect(0, 0, W, H);
  // inner
  roundRect(ctx, 24, 24, W - 48, H - 48, 22);
  ctx.fillStyle = C.inner; ctx.fill();
  ctx.save(); ctx.clip();
  const glow = ctx.createRadialGradient(W - 80, 60, 0, W - 80, 60, 600);
  [[0, 0.30], [0.3, 0.14], [0.6, 0.05], [1, 0]].forEach(([o, k]) => glow.addColorStop(o, hexA(A, k * (dark ? 1 : 0.7))));
  ctx.fillStyle = glow; ctx.fillRect(0, 0, W, H);
  ctx.restore();

  // header: tag, name, HP
  const ram = specs ? Math.round(specs.ram_total / 2 ** 30) : 0;
  ctx.font = F(700, 15);
  const tag = 'ZÁKLADNÝ PC';
  const tw = ctx.measureText(tag).width + 22;
  roundRect(ctx, 52, 44, tw, 26, 13);
  ctx.fillStyle = hexA(A, 0.18); ctx.fill();
  ctx.fillStyle = A; ctx.fillText(tag, 63, 62);
  let rightX = W - 52;
  if (ram) {
    ctx.textAlign = 'right';
    ctx.font = F(800, 52); ctx.fillStyle = C.text;
    ctx.fillText(String(ram), rightX, 104);
    const nw = ctx.measureText(String(ram)).width;
    ctx.font = F(800, 20); ctx.fillStyle = A;
    ctx.fillText('HP', rightX - nw - 8, 104);
    ctx.textAlign = 'left';
    rightX -= nw + 60;
  }
  ctx.fillStyle = C.text;
  const name = fitFont(ctx, card.title || 'Môj PC', rightX - 52 - 14, 50, 26, 800);
  ctx.fillText(name, 52, 108);

  // art window
  const ax = 52, ay = 128, aw = W - 104, ah = 350;
  roundRect(ctx, ax - 5, ay - 5, aw + 10, ah + 10, 18);
  const wf = ctx.createLinearGradient(ax, ay, ax + aw, ay + ah);
  wf.addColorStop(0, mixHex(base, '#ffffff', 0.6)); wf.addColorStop(0.5, mixHex(base, '#000000', 0.2)); wf.addColorStop(1, mixHex(base, '#ffffff', 0.45));
  ctx.fillStyle = wf; ctx.fill();
  roundRect(ctx, ax, ay, aw, ah, 14);
  ctx.save(); ctx.clip();
  const ag = ctx.createLinearGradient(ax, ay, ax + aw * 0.6, ay + ah);
  ag.addColorStop(0, mixHex(A, '#000000', 0.55)); ag.addColorStop(0.55, mixHex(A, '#000000', 0.15)); ag.addColorStop(1, mixHex(A, '#ffffff', 0.18));
  ctx.fillStyle = ag; ctx.fillRect(ax, ay, aw, ah);
  const cx = ax + aw / 2, cy = ay + ah * 0.46;
  // rays
  for (let i = 0; i < 28; i++) {
    const a0 = (i / 28) * Math.PI * 2, a1 = a0 + Math.PI / 28;
    ctx.beginPath(); ctx.moveTo(cx, cy);
    ctx.lineTo(cx + Math.cos(a0) * 600, cy + Math.sin(a0) * 600); ctx.lineTo(cx + Math.cos(a1) * 600, cy + Math.sin(a1) * 600);
    ctx.fillStyle = 'rgba(255,255,255,.055)'; ctx.fill();
  }
  // circuit traces
  const R = rng(seed);
  ctx.strokeStyle = 'rgba(255,255,255,.16)'; ctx.lineWidth = 2; ctx.lineJoin = 'round';
  for (let i = 0; i < 9; i++) {
    let x = ax + 20 + R() * (aw - 40), y = ay + 14 + R() * 60;
    ctx.beginPath(); ctx.moveTo(x, y);
    for (let k = 0; k < 4; k++) { y += 30 + R() * 50; ctx.lineTo(x, y); x += (R() - 0.5) * 120; ctx.lineTo(x, y); }
    ctx.stroke();
    ctx.fillStyle = 'rgba(255,255,255,.3)'; ctx.beginPath(); ctx.arc(x, y, 4, 0, 7); ctx.fill();
  }
  // halo + logo
  const halo = ctx.createRadialGradient(cx, cy, 10, cx, cy, 190);
  [[0, 0.55], [0.4, 0.22], [1, 0]].forEach(([o, k]) => halo.addColorStop(o, `rgba(255,255,255,${k})`));
  ctx.fillStyle = halo; ctx.fillRect(ax, ay, aw, ah);
  ctx.save(); ctx.translate(cx - 70, cy - 70); ctx.lineWidth = 9; ctx.strokeStyle = '#fff'; ctx.shadowColor = 'rgba(0,0,0,.35)'; ctx.shadowBlur = 24; ctx.shadowOffsetY = 8;
  roundRect(ctx, 0, 0, 140, 140, 40); ctx.stroke();
  ctx.beginPath(); ctx.ellipse(70, 70, 46, 29, 0, 0, Math.PI * 2); ctx.stroke();
  ctx.shadowColor = 'transparent'; ctx.fillStyle = gold ? '#e9b949' : '#ff9f0a'; ctx.beginPath(); ctx.arc(70, 70, 15, 0, Math.PI * 2); ctx.fill();
  ctx.restore();
  // sparkles
  ctx.fillStyle = 'rgba(255,255,255,.85)';
  for (let i = 0; i < 14; i++) star4(ctx, ax + 16 + R() * (aw - 32), ay + 16 + R() * (ah - 32), 4 + R() * 9);
  // GPU ribbon
  const gpu = rows.find((r) => r.key === 'gpu')?.value || rows.find((r) => r.key === 'cpu')?.value || '';
  if (gpu) {
    const t = fitFont(ctx, gpu, aw - 90, 30, 18, 800);
    const gw = ctx.measureText(t).width + 44;
    roundRect(ctx, cx - gw / 2, ay + ah - 62, gw, 42, 21);
    ctx.fillStyle = 'rgba(0,0,0,.42)'; ctx.fill();
    ctx.textAlign = 'center'; ctx.fillStyle = '#fff'; ctx.fillText(t, cx, ay + ah - 32); ctx.textAlign = 'left';
  }
  // baked-in sheen so the PNG still looks foil
  ctx.globalCompositeOperation = 'overlay';
  const sh = ctx.createLinearGradient(ax, ay + ah, ax + aw, ay);
  (gold ? ['#fff3b0', '#e9b949', '#fff3b0', '#c98a1a'] : ['#ff5e5e', '#ffd25e', '#5eff9a', '#5ecbff', '#b45eff']).forEach((c, i, arr) => sh.addColorStop(i / (arr.length - 1), c));
  ctx.globalAlpha = 0.38; ctx.fillStyle = sh; ctx.fillRect(ax, ay, aw, ah);
  ctx.restore();

  // info strip
  const os = rows.find((r) => r.key === 'os')?.value;
  const info = card.subtitle || os || 'Počítač';
  ctx.textAlign = 'center'; ctx.fillStyle = C.muted; ctx.font = 'italic 500 17px "Inter Variable", "Segoe UI", sans-serif';
  ctx.fillText(fitText(ctx, info, aw - 40), W / 2, ay + ah + 38);
  ctx.textAlign = 'left';

  // attacks
  const spec = rows.filter((r) => r.key !== 'os' || card.subtitle);
  const top = ay + ah + 56, bottom = H - 106, avail = bottom - top;
  const n = spec.length;
  const rh = Math.min(88, avail / Math.max(n, 1));
  spec.forEach((r, i) => {
    const y = top + i * rh, mid = y + rh / 2;
    if (i) { ctx.fillStyle = C.line; ctx.fillRect(52, y, W - 104, 1.2); }
    const rad = Math.min(24, rh * 0.34), ix = 52 + rad + 4;
    const ig = ctx.createRadialGradient(ix - rad * 0.3, mid - rad * 0.4, 1, ix, mid, rad);
    ig.addColorStop(0, mixHex(A, '#ffffff', 0.4)); ig.addColorStop(1, A);
    ctx.beginPath(); ctx.arc(ix, mid, rad, 0, Math.PI * 2); ctx.fillStyle = ig; ctx.fill();
    ctx.save(); const s = (rad * 1.15) / 16; ctx.translate(ix - 8 * s, mid - 8 * s); ctx.scale(s, s);
    ctx.strokeStyle = '#fff'; ctx.lineWidth = 1.6; ctx.lineCap = 'round'; ctx.lineJoin = 'round';
    ctx.stroke(new Path2D(CARD_ICONS[r.icon] || CARD_ICONS.star)); ctx.restore();
    const tx = ix + rad + 18, maxW = W - 52 - tx;
    if (rh >= 58) {
      ctx.fillStyle = C.muted; ctx.font = F(700, 14); ctx.fillText(fitText(ctx, r.label.toUpperCase(), maxW), tx, mid - 8);
      ctx.fillStyle = C.text; ctx.fillText(fitFont(ctx, r.value, maxW, 27, 15, 700), tx, mid + 21);
    } else {
      const lw = 110;
      ctx.fillStyle = C.text; ctx.fillText(fitFont(ctx, r.value, maxW - lw - 12, Math.min(22, rh * 0.52), 13, 700), tx, mid + 6);
      ctx.textAlign = 'right'; ctx.fillStyle = C.muted; ctx.font = F(700, 12); ctx.fillText(fitText(ctx, r.label.toUpperCase(), lw), W - 52, mid + 4); ctx.textAlign = 'left';
    }
  });
  if (!n) { ctx.fillStyle = C.muted; ctx.font = F(500, 22); ctx.fillText('Zapni aspoň jeden riadok', 52, top + 50); }

  // footer
  const fy = H - 52;
  ctx.fillStyle = C.line; ctx.fillRect(52, fy - 36, W - 104, 1.2);
  if (card.footer) {
    ctx.save(); ctx.translate(52, fy - 18); ctx.strokeStyle = C.muted; ctx.lineWidth = 1.6;
    roundRect(ctx, 0, 0, 24, 24, 7); ctx.stroke();
    ctx.beginPath(); ctx.ellipse(12, 12, 8, 5, 0, 0, Math.PI * 2); ctx.stroke();
    ctx.beginPath(); ctx.arc(12, 12, 2.6, 0, Math.PI * 2); ctx.fillStyle = A; ctx.fill(); ctx.restore();
    ctx.fillStyle = C.muted; ctx.font = F(600, 16); ctx.fillText('WhatsInside', 88, fy);
  }
  ctx.textAlign = 'right'; ctx.fillStyle = A; ctx.font = F(800, 16);
  ctx.fillText(`${rarity()}  ·  №${String(seed % 900 + 100)}`, W - 52, fy); ctx.textAlign = 'left';
  ctx.restore();
}

/** Tilt + foil shine that follow the pointer (CSS does the painting). */
function initHolo() {
  const h = $('#holo');
  if (!h || h.dataset.on) return;
  h.dataset.on = '1';
  let raf = 0;
  const set = (px, py, on) => {
    cancelAnimationFrame(raf);
    raf = requestAnimationFrame(() => {
      h.style.setProperty('--mx', `${px * 100}%`);
      h.style.setProperty('--my', `${py * 100}%`);
      h.style.setProperty('--rx', `${on ? (0.5 - py) * 16 : 0}deg`);
      h.style.setProperty('--ry', `${on ? (px - 0.5) * 20 : 0}deg`);
      h.classList.toggle('hot', on);
    });
  };
  h.addEventListener('pointermove', (e) => { const r = h.getBoundingClientRect(); set((e.clientX - r.left) / r.width, (e.clientY - r.top) / r.height, !REDUCED); });
  h.addEventListener('pointerleave', () => set(0.5, 0.5, false));
}

function cardText() {
  const rows = card.rows.filter((r) => r.on && r.value.trim());
  return `**${card.title || 'Môj PC'}**${card.subtitle ? ` · ${card.subtitle}` : ''}\n` + rows.map((r) => `> **${r.label}:** ${r.value}`).join('\n');
}

$('#cardCopy').onclick = async () => {
  try {
    const c = document.createElement('canvas');
    drawCard(c, 2);
    const blob = await new Promise((res) => c.toBlob(res, 'image/png'));
    await navigator.clipboard.write([new ClipboardItem({ 'image/png': blob })]);
    toast('Skopírované. Vlož ho do Discordu cez Ctrl + V.');
  } catch (e) {
    toast('Do schránky sa nepodarilo, skús Uložiť PNG.', 'err');
  }
};
$('#cardSave').onclick = async () => {
  try {
    const c = document.createElement('canvas');
    drawCard(c, 2);
    const path = await invoke('save_card', { pngBase64: c.toDataURL('image/png') });
    toast(`Uložené: ${path}`);
  } catch (e) {
    toast(`Nepodarilo sa uložiť: ${e}`, 'err');
  }
};
$('#cardText').onclick = async () => {
  try { await navigator.clipboard.writeText(cardText()); toast('Text skopírovaný, vlož ho do Discordu.'); } catch { toast('Nepodarilo sa skopírovať.', 'err'); }
};


/** Plain list of what's inside, for any PC. */
function renderComponents() {
  const sp = specs;
  if (!sp) return;
  const card = (k, icon, name, lines, i) => `<div class="card dev comp land" style="--i:${i}"><div class="k">${ICON[icon] || ''}${esc(k)}</div>
    <div class="n">${esc(name || 'Nezistené')}</div>${lines.filter(Boolean).map((l) => `<div class="d">${l}</div>`).join('')}</div>`;
  const mods = sp.ram_modules || [];
  const ramName = mods.length ? `${sk(mods.reduce((a, m) => a + m.size_mb, 0) / 1024)} GB ${mods[0].kind || ''}`.trim() : sp.ram_total ? `${Math.round(sp.ram_total / 2 ** 30)} GB` : '';
  const out = [
    card('Procesor', 'cpu', sp.cpu.replace(/\((R|TM)\)/gi, '').replace(/\s{2,}/g, ' '), [sp.threads ? `${sp.threads} vlákien` : ''], 0),
    ...(sp.gpus.length ? sp.gpus : [{ name: '' }]).map((g, i) => card('Grafika', 'gpu', g.name, [
      g.vram ? `${Math.round(g.vram / 2 ** 30)} GB pamäte` : '',
      g.driver_version ? `Ovládač ${esc(g.driver_version)}${g.driver_date ? ` · ${esc(g.driver_date.replace(/-/g, '. '))}` : ''}` : '',
    ], 1 + i)),
    card('Pamäť', 'ram', ramName, mods.map((m) => `<span class="num">${esc(m.slot || '?')}</span> · ${sk(m.size_mb / 1024)} GB · ${m.configured_speed ? `${sk(m.configured_speed)} MT/s` : ''}${m.rated_speed && m.rated_speed > m.configured_speed + 150 ? ` <span class="badge lv-amber">vie ${sk(m.rated_speed)}</span>` : ''}<br><span class="faint">${esc(prettyVendor(m.manufacturer || ''))} ${esc(m.part || '')}</span>`), 3),
    card('Základná doska', 'board', sp.board, [], 4),
    card('Úložisko', 'drive', sp.disks.length ? `${sp.disks.length} ${plural(sp.disks.length, 'disk', 'disky', 'diskov')}` : '', sp.disks.map(esc), 5),
    card('Monitory', 'monitor', (displays || []).length ? `${displays.length} ${plural(displays.length, 'monitor', 'monitory', 'monitorov')}` : '', (displays || []).map((d) => `${d.width} × ${d.height} · ${d.hz} Hz${d.max_hz > d.hz + 5 ? ` <span class="badge lv-amber">vie ${d.max_hz} Hz</span>` : ''}<br><span class="faint">${esc(d.adapter)}${d.primary ? ' · hlavný' : ''}</span>`), 6),
    card('Systém', 'os', sp.os, [], 7),
  ];
  $('#hwComp').innerHTML = out.join('');
}

views.hw = {
  async enter() {
    if (!hwLoaded) {
      $('#hwList').innerHTML = '<div class="card empty">Hľadám…</div>';
      [hwReport, specs] = await Promise.all([invoke('hw_scan'), invoke('pc_specs')]);
      hwLoaded = true;
      syncRows();
      renderEditor();
      await document.fonts.load('700 46px "Inter Variable"').catch(() => {});
      drawCard();
      initHolo();
      $('#holo').classList.toggle('gold', card.finish === 'gold');
      $('#hwList').innerHTML = hwReport.devices.map((d, i) => `<div class="card dev land" style="--i:${i}">
        <div class="k"><span class="dot ${d.supported ? 'mint' : ''}"></span>${esc(d.kind)}${d.supported ? ' · rozpoznané' : ''}</div>
        <div class="n">${esc(d.name)}</div><div class="i">${esc(d.id)}</div><div class="d">${esc(d.note)}</div></div>`).join('') +
        (hwReport.devices.length ? '' : '<div class="card empty">Nenašiel som zariadenie so svetlami, ktoré by som poznal.</div>') +
        hwReport.errors.map((e) => `<div class="card dev"><div class="k"><span class="dot rec"></span>Chyba</div><div class="d">${esc(e)}</div></div>`).join('');
      if (!displays) displays = await invoke('displays_list').catch(() => []);
      renderComponents();
    } else {
      requestAnimationFrame(() => { ['#cTheme', '#cLayout'].forEach(placeSeg); });
    }
    const snap = await invoke('procs_snapshot');
    const rgb = snap.groups.filter((g) => g.verdict === 'rgb');
    $('#hwRgbApps').innerHTML = rgb.length
      ? `<div class="cache"><div><div class="t">Tieto appky ti teraz kvôli svetlám držia ${bytes(rgb.reduce((a, g) => a + g.ram, 0))} RAM</div></div></div>` +
        rgb.map((g) => `<div class="cache"><div><div class="t">${esc(g.name)}</div><div class="h">${esc(g.about || '')}</div></div>
          <span class="num">${bytes(g.ram)}</span><span class="badge">${g.procs.length} ${plural(g.procs.length, 'proces', 'procesy', 'procesov')}</span></div>`).join('')
      : `<div class="empty">${ICON.check}Nebeží žiadna výrobcovská RGB appka.</div>`;
  },
};

/* ========================================================== UPDATES */
let updateInfo = null;
async function checkUpdate(manual = false) {
  try { updateInfo = await invoke('update_check'); } catch (e) { updateInfo = null; }
  const u = updateInfo;
  $('#appVer').textContent = u ? `v${u.current}` : '';
  const b = $('#updateBtn');
  if (u && u.newer && u.latest) {
    b.innerHTML = `<span class="dot amber"></span><div><b>Nová verzia ${esc(u.latest.version)}</b><span>Klikni a aktualizuj</span></div>`;
    b.hidden = false;
  } else {
    b.hidden = true;
    if (manual) toast(u?.error ? u.error : 'Máš najnovšiu verziu.', u?.error ? 'err' : 'ok');
  }
}
$('#updateBtn').onclick = () => {
  const u = updateInfo;
  if (!u?.latest) return;
  const m = modal(`<h3>WhatsInside ${esc(u.latest.version)}</h3>
    <p class="mp">Teraz máš ${esc(u.current)}. Stiahnem inštalátor z GitHubu, overím jeho kontrolný súčet a nainštalujem ho. Appka sa na chvíľu zavrie a sama sa znova otvorí.</p>
    ${u.latest.notes ? `<div class="notes">${esc(u.latest.notes).replace(/^### (.*)$/gm, '<b>$1</b>').replace(/^- (.*)$/gm, '• $1').replace(/\n/g, '<br>')}</div>` : ''}
    ${u.installed ? '' : '<p class="mnote">Spúšťaš prenosnú verziu, preto sa otvorí bežný inštalátor. Po inštalácii spúšťaj WhatsInside zo Štartu.</p>'}
    <div class="mact"><a class="btn ghost" href="https://github.com/Skyramp420/WhatsInside/releases" target="_blank" rel="noreferrer">Všetky verzie</a><button class="btn" data-close>Neskôr</button><button class="btn primary" id="updGo">Aktualizovať</button></div>`);
  $('#updGo', m).onclick = async () => {
    const b = $('#updGo', m);
    b.disabled = true;
    b.textContent = 'Sťahujem…';
    try { await invoke('update_install', { latest: u.latest }); }
    catch (e) { toast(String(e), 'err'); b.disabled = false; b.textContent = 'Skúsiť znova'; }
  };
};
$('#appVer').onclick = () => checkUpdate(true);

/* ---------------------------------------------------------- start */
idleStatus();
setView('helper');
setIdle();
setTimeout(checkUpdate, 2500);
