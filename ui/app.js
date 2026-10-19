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
const idleStatus = () => setStatus('Len sa pozerám', 'Nič nemením ani nemažem. Keď ma zavrieš, nebeží nič.');

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
  return `<b>${esc(n.name)}</b><span class="num">${bytes(n.size)}${pct}${files}</span>${when}${tag}`;
}

const EASE = 'cubic-bezier(.2,.9,.25,1)';
function drawTreemap(mode = null, fromRect = null, prev = null) {
  const card = $('#tmCard'), l = st.listing;
  if (!card || !l) return;
  const W = card.clientWidth - 16, H = card.clientHeight - 16;
  const old = $('.tmlayer', card);
  const layer = document.createElement('div');
  layer.className = 'tmlayer';
  const kids = (l.node.children || []).filter((c) => c.size > 0);
  st.rects = new Map();
  if (!kids.length) {
    layer.innerHTML = `<div class="empty">${ICON.dir}Tento priečinok je prázdny.</div>`;
  } else {
    const inherited = l.crumbs.map((c) => st.colors.get(c.id)).find(Boolean);
    const color = (n, i) => (n.kind === 'file' ? 'var(--muted)' : n.kind !== 'dir' ? 'var(--faint)' : st.colors.get(n.id) || inherited || PALETTE[i % PALETTE.length]);
    const rects = squarify(kids.map((k) => k.size), 0, 0, W, H);
    const G = 3;
    kids.forEach((n, i) => {
      let [x, y, w, h] = rects[i];
      x += G / 2; y += G / 2; w -= G; h -= G;
      if (w < 2 || h < 2) return;
      if (n.id != null) st.rects.set(n.id, [x, y, w, h]);
      const el = document.createElement('div');
      el.className = `tm ${n.kind}${n.tag?.kind === 'game' ? ' isgame' : ''}${n.tag?.kind === 'cache' ? ' iscache' : ''}`;
      el.dataset.id = n.id ?? '';
      el.style.cssText = `left:${x}px;top:${y}px;width:${w}px;height:${h}px;--c:${color(n, i)}`;
      const big = w > 90 && h > 46;
      const showTag = n.tag && (n.tag.kind === 'game' || n.tag.kind === 'cache') && w > 120;
      if (big || (w > 54 && h > 30)) el.innerHTML = `<div class="lbl">${showTag ? `<span class="ti">${ICON[TAG[n.tag.kind].icon]}</span>` : ''}${esc(n.name)}<small>${bytes(n.size)}</small></div>`;
      el.onmousemove = (e) => showTip(tmTip(n, l.node.size), e.clientX, e.clientY);
      el.onmouseleave = hideTip;
      if (n.kind === 'dir' && n.id != null) el.onclick = () => { hideTip(); openNode(n.id); };
      const sub = (n.children || []).filter((c) => c.size > 0);
      if (big && h > 90 && w > 120 && sub.length) {
        const sr = squarify(sub.map((c) => c.size), 4, 38, w - 8, h - 42);
        sub.forEach((c, k) => {
          let [sx, sy, sw, sh] = sr[k];
          sx += 1; sy += 1; sw -= 2; sh -= 2;
          if (sw < 3 || sh < 3) return;
          if (c.id != null) st.rects.set(c.id, [x + sx, y + sy, sw, sh]);
          const s = document.createElement('div');
          s.className = `tm sub ${c.kind}`;
          s.dataset.id = c.id ?? '';
          s.style.cssText = `left:${sx}px;top:${sy}px;width:${sw}px;height:${sh}px;--c:${color(n, i)}`;
          if (sw > 64 && sh > 28) s.innerHTML = `<div class="lbl">${esc(c.name)}</div>`;
          s.onmousemove = (e) => { e.stopPropagation(); showTip(tmTip(c, n.size), e.clientX, e.clientY); };
          if (c.kind === 'dir' && c.id != null) s.onclick = (e) => { e.stopPropagation(); hideTip(); openNode(c.id); };
          el.appendChild(s);
        });
      }
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
    const procs = g.procs.slice(0, 40).map((p) => `<div class="proc"><div><div>${esc(p.exe)} <span style="color:var(--faint)">· ${p.pid}</span></div>${p.path ? `<div class="path">${esc(p.path)}</div>` : ''}</div>
      <span class="r">${p.cpu >= 0.1 ? `${sk(p.cpu, 1)} %` : '–'}</span><span class="r">${p.ram ? bytes(p.ram) : '?'}</span><span class="r">${rate(p.io_bps)}</span></div>`).join('');
    return `<div class="pgroup ${open ? 'open' : ''}" data-key="${esc(g.key)}">
      <div class="prow"><div class="who"><svg class="chev" viewBox="0 0 12 12"><path d="M4.5 2.5L8 6l-3.5 3.5"/></svg><span class="nm">${esc(g.name)}</span>${g.verdict === 'system' ? '' : badge(g.verdict)}${n > 1 ? `<span class="cnt">${n} ${plural(n, 'proces', 'procesy', 'procesov')}</span>` : ''}</div>
      <span></span><span class="r ${g.cpu >= 10 ? 'hot' : g.cpu < 0.1 ? 'dim' : ''}">${g.cpu >= 0.1 ? `${sk(g.cpu, 1)} %` : '–'}</span>
      <span class="r">${g.ram ? bytes(g.ram) : '?'}</span><span class="r ${g.io_bps < 1024 ? 'dim' : ''}">${rate(g.io_bps)}</span></div>
      <div class="pdetail">${g.about ? `<div class="about">${esc(g.about)}</div>` : ''}${procs}${n > 40 ? `<div class="proc">… a ďalších ${n - 40}</div>` : ''}</div></div>`;
  }).join('') || '<div class="empty">Nič také nebeží.</div>';
  $$('#plist .prow').forEach((r) => (r.onclick = () => {
    const g = r.parentElement, k = g.dataset.key;
    pr.open.has(k) ? pr.open.delete(k) : pr.open.add(k);
    g.classList.toggle('open');
  }));
}

views.procs = {
  enter(filter) {
    if (filter) pr.filter = filter;
    placeSeg('#psort');
    pollProcs().then(() => { clearTimeout(pr.timer); pr.timer = setTimeout(pollProcs, 600); });
  },
  leave() { clearTimeout(pr.timer); },
  visible(v) { if (v && current === 'procs') pollProcs(); else clearTimeout(pr.timer); },
};

/* ========================================================== STARTUP */
const SECTIONS = [
  ['rgb', 'Výrobcovské RGB appky'],
  ['optional', 'Dá sa vypnúť'],
  ['peripheral', 'Periférie'],
  ['unknown', 'Ostatné'],
  ['needed', 'Potrebné'],
  ['system', 'Windows'],
];
let startupItems = null;
async function loadStartup() {
  $('#startupBody').innerHTML = '<div class="empty">Načítavam…</div>';
  startupItems = await invoke('autostart_list');
  renderStartup();
}
function renderStartup() {
  const items = startupItems || [];
  const actionable = items.filter((i) => i.enabled && (i.verdict === 'rgb' || i.verdict === 'optional')).length;
  $('#startupCount').textContent = actionable || '';
  $('#startupBody').innerHTML = SECTIONS.map(([v, title]) => {
    const list = items.filter((i) => i.verdict === v);
    if (!list.length) return '';
    return `<h2>${title}</h2><div class="card list">${list.map((i) => `<div class="arow">
      <div style="min-width:0"><div class="t">${esc(i.app || i.name)}${i.app && i.app !== i.name ? `<span style="font-weight:400;color:var(--faint)">${esc(i.name)}</span>` : ''}
        ${i.enabled ? '' : '<span class="badge off">Vypnuté</span>'}${i.running === true ? '<span class="badge on">Beží</span>' : ''}</div>
        ${i.about ? `<div class="a">${esc(i.about)}</div>` : ''}<div class="c">${esc(i.command)}</div></div>
      <div class="src">${esc(i.source)}</div></div>`).join('')}</div>`;
  }).join('') || '<div class="card empty">Nič sa nespúšťa automaticky.</div>';
}
$('#startupRefresh').onclick = loadStartup;
views.startup = { enter() { if (!startupItems) loadStartup(); } };

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
    const [facts, quick, snap, auto, drives, specs] = await Promise.all([
      invoke('system_facts'), invoke('quick_check'), invoke('procs_snapshot'),
      startupItems ? Promise.resolve(startupItems) : invoke('autostart_list'),
      invoke('storage_drives'), invoke('pc_specs').catch(() => null),
    ]);
    startupItems = auto;
    renderStartup();
    renderHelper(buildAdvice({ facts, quick, snap, auto, drives, specs }));
    helperDirty = false;
  } catch (e) {
    body.innerHTML = `<div class="card empty">${ICON.warn}Kontrola sa nepodarila: ${esc(e)}</div>`;
  }
  helperBusy = false;
}

/** Turns raw facts into plain-language advice. Every item says why and how. */
function buildAdvice({ facts, quick, snap, auto, drives, specs }) {
  const tips = [], ok = [];
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
  const extra = auto.filter((i) => i.enabled && i.verdict === 'optional');
  if (extra.length) {
    tips.push({ level: extra.length > 4 ? 'mid' : 'low', icon: 'rocket', title: `${extra.length} ${plural(extra.length, 'appka sa spúšťa', 'appky sa spúšťajú', 'appiek sa spúšťa')} s Windowsom zbytočne`,
      text: 'Spomaľujú štart a potom bežia na pozadí. Keď ich budeš potrebovať, jednoducho ich otvoríš.',
      list: extra.slice(0, 6).map((i) => [i.app || i.name, i.source]),
      steps: ['Ctrl + Shift + Esc → Aplikácie pri spustení → vyber appku → Vypnúť.', 'Discord, Steam či Spotify majú túto voľbu aj vo vlastných nastaveniach.'],
      action: { label: 'Ukázať všetko', run: () => setView('startup') } });
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
      steps: ['Nastavenia → Systém → Napájanie → Režim napájania → Vyvážený alebo Najlepší výkon.'] });
  } else if (facts.power_name) ok.push(`Plán napájania: ${facts.power_name}`);

  // 10. Game Mode
  if (facts.game_mode === false) {
    tips.push({ level: 'low', icon: 'game', title: 'Herný režim je vypnutý',
      text: 'Pri hraní dáva hre prednosť pred aktualizáciami a appkami na pozadí. Na výkonnom PC je rozdiel malý, ale škodiť nebude.',
      steps: ['Nastavenia → Hry → Herný režim → Zapnúť.'] });
  } else if (facts.game_mode) ok.push('Herný režim je zapnutý');

  // 11. HAGS for RTX 40/50 frame generation
  const gpuName = (specs?.gpus || []).map((g) => g.name).join(' ');
  const rtxFg = /RTX\s*(40|50)\d\d/i.test(gpuName);
  if (facts.hags === false && rtxFg) {
    tips.push({ level: 'mid', icon: 'gpu', title: 'Plánovanie GPU s hardvérovou akceleráciou je vypnuté',
      text: 'Tvoja grafika podporuje DLSS Frame Generation, ktoré bez neho nefunguje.',
      steps: ['Nastavenia → Systém → Obrazovka → Grafika → Zmeniť predvolené nastavenia grafiky → zapnúť.', 'Potom reštartuj PC.'] });
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
  const level = { high: ['rec', 'Dôležité'], mid: ['amber', 'Odporúčam'], low: ['blue', 'Tip'] };
  $('#helperBody').innerHTML =
    (r.tips.length ? `<div class="tips">${r.tips.map((t, i) => {
      if (t.action) helperActions[i] = t.action.run;
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
    }).join('')}</div>` : `<div class="card empty">${ICON.check}Nenašiel som nič, čo by stálo za zmenu. Pekne udržiavaný počítač.</div>`) +
    (r.ok.length ? `<h2>V poriadku</h2><div class="card oklist">${r.ok.map((o) => `<div>${ICON.check}<span>${esc(o)}</span></div>`).join('')}</div>` : '') +
    (r.scanned ? '' : '<div class="note">Tip: po skene disku v Úložisku nájdem aj cache DaVinci Resolve, staré súbory a hry, ktoré už nehráš.</div>') +
    '<div class="note">Nič z toho za teba nemením. Ku každej veci máš kroky, ako to spraviť sám.</div>';
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
  return c || { title: 'Môj PC', subtitle: '', theme: 'dark', accent: '#ff9f0a', layout: 'list', footer: true, rows: [] };
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
      <div class="seg" id="cLayout"><div class="knob"></div><button data-k="list" class="${card.layout === 'list' ? 'on' : ''}">Zoznam</button><button data-k="grid" class="${card.layout === 'grid' ? 'on' : ''}">Mriežka</button></div>
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
  seg($('#cLayout'), (k) => { card.layout = k; redraw(); });
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

/** Draws the share card. Same function for the preview and the export (scale 2). */
function drawCard(canvas = $('#pcCanvas'), scale = 2) {
  const rows = card.rows.filter((r) => r.on && r.value.trim());
  const dark = card.theme === 'dark';
  const C = dark
    ? { bg: '#1c1c1e', panel: '#2c2c2e', text: '#f5f5f7', muted: '#a1a1a6', faint: '#6e6e73', line: 'rgba(255,255,255,.08)' }
    : { bg: '#f5f5f7', panel: '#ffffff', text: '#1d1d1f', muted: '#6e6e73', faint: '#a1a1a6', line: 'rgba(0,0,0,.07)' };
  const W = 960, P = 48, grid = card.layout === 'grid';
  const rowH = grid ? 112 : 92, gap = 14, cols = grid ? 2 : 1;
  const headH = card.subtitle ? 150 : 124;
  const nRows = Math.ceil(rows.length / cols);
  const H = P + headH + Math.max(1, nRows) * (rowH + gap) - gap + (card.footer ? 72 : P);
  canvas.width = W * scale;
  canvas.height = H * scale;
  const ctx = canvas.getContext('2d');
  ctx.scale(scale, scale);
  // background
  roundRect(ctx, 0, 0, W, H, 40);
  ctx.save();
  ctx.clip();
  ctx.fillStyle = C.bg;
  ctx.fillRect(0, 0, W, H);
  const glow = ctx.createRadialGradient(W - 110, 30, 0, W - 110, 30, 640);
  const ga = dark ? 0.34 : 0.22;
  [[0, 1], [0.25, 0.62], [0.5, 0.3], [0.75, 0.09], [1, 0]].forEach(([o, k]) => glow.addColorStop(o, hexA(card.accent, ga * k)));
  ctx.fillStyle = glow;
  ctx.fillRect(0, 0, W, H);
  // header
  ctx.fillStyle = C.text;
  ctx.font = '700 46px "Inter Variable", "Segoe UI", sans-serif';
  ctx.textBaseline = 'alphabetic';
  ctx.fillText(fitText(ctx, card.title || 'Môj PC', W - P * 2), P, P + 52);
  if (card.subtitle) {
    ctx.fillStyle = C.muted;
    ctx.font = '500 24px "Inter Variable", "Segoe UI", sans-serif';
    ctx.fillText(fitText(ctx, card.subtitle, W - P * 2), P, P + 92);
  }
  // rows
  const colW = (W - P * 2 - gap * (cols - 1)) / cols;
  rows.forEach((r, i) => {
    const cx = P + (i % cols) * (colW + gap), cy = P + headH + Math.floor(i / cols) * (rowH + gap);
    roundRect(ctx, cx, cy, colW, rowH, 22);
    ctx.fillStyle = C.panel;
    ctx.fill();
    ctx.strokeStyle = C.line;
    ctx.lineWidth = 1;
    ctx.stroke();
    // icon tile
    const it = 52, ix = cx + 20, iy = cy + (rowH - it) / 2;
    roundRect(ctx, ix, iy, it, it, 15);
    ctx.fillStyle = hexA(card.accent, dark ? 0.16 : 0.13);
    ctx.fill();
    ctx.save();
    ctx.translate(ix + 10, iy + 10);
    ctx.scale(2, 2);
    ctx.strokeStyle = card.accent;
    ctx.lineWidth = 1.5;
    ctx.lineCap = 'round';
    ctx.lineJoin = 'round';
    ctx.stroke(new Path2D(CARD_ICONS[r.icon] || CARD_ICONS.star));
    ctx.restore();
    // text
    const tx = ix + it + 18, tw = colW - (tx - cx) - 20;
    ctx.fillStyle = C.muted;
    ctx.font = '600 17px "Inter Variable", "Segoe UI", sans-serif';
    ctx.fillText(fitText(ctx, r.label.toUpperCase(), tw), tx, cy + (grid ? 42 : 38));
    ctx.fillStyle = C.text;
    ctx.font = `600 ${grid ? 23 : 25}px "Inter Variable", "Segoe UI", sans-serif`;
    if (grid) {
      // two lines in grid mode
      const words = r.value.split(' ');
      let l1 = '', k = 0;
      while (k < words.length && ctx.measureText((l1 + ' ' + words[k]).trim()).width <= tw) l1 = (l1 + ' ' + words[k++]).trim();
      ctx.fillText(l1 || fitText(ctx, r.value, tw), tx, cy + 72);
      if (k < words.length && l1) ctx.fillText(fitText(ctx, words.slice(k).join(' '), tw), tx, cy + 100);
    } else {
      ctx.fillText(fitText(ctx, r.value, tw), tx, cy + 70);
    }
  });
  if (!rows.length) {
    ctx.fillStyle = C.faint;
    ctx.font = '500 22px "Inter Variable", "Segoe UI", sans-serif';
    ctx.fillText('Zapni aspoň jeden riadok', P, P + headH + 50);
  }
  // footer
  if (card.footer) {
    const fy = H - 34;
    ctx.save();
    ctx.translate(P, fy - 19);
    ctx.strokeStyle = C.faint;
    ctx.lineWidth = 1.6;
    roundRect(ctx, 0, 0, 24, 24, 7);
    ctx.stroke();
    ctx.beginPath();
    ctx.ellipse(12, 12, 8, 5, 0, 0, Math.PI * 2);
    ctx.stroke();
    ctx.beginPath();
    ctx.arc(12, 12, 2.6, 0, Math.PI * 2);
    ctx.fillStyle = card.accent;
    ctx.fill();
    ctx.restore();
    ctx.fillStyle = C.faint;
    ctx.font = '600 17px "Inter Variable", "Segoe UI", sans-serif';
    ctx.fillText('WhatsInside', P + 36, fy);
    const w1 = ctx.measureText('WhatsInside').width;
    ctx.font = '500 17px "Inter Variable", "Segoe UI", sans-serif';
    ctx.fillText('  od WheresTheApp', P + 36 + w1, fy);
  }
  ctx.restore();
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
      $('#hwList').innerHTML = hwReport.devices.map((d, i) => `<div class="card dev land" style="--i:${i}">
        <div class="k"><span class="dot ${d.supported ? 'mint' : ''}"></span>${esc(d.kind)}${d.supported ? ' · podporované' : ''}</div>
        <div class="n">${esc(d.name)}</div><div class="i">${esc(d.id)}</div><div class="d">${esc(d.note)}</div></div>`).join('') +
        (hwReport.devices.length ? '' : '<div class="card empty">Nenašiel som RGB zariadenie, ktoré poznám.</div>') +
        hwReport.errors.map((e) => `<div class="card dev"><div class="k"><span class="dot rec"></span>Chyba</div><div class="d">${esc(e)}</div></div>`).join('');
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

/* ---------------------------------------------------------- start */
idleStatus();
setView('helper');
setIdle();
