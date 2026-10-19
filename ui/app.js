'use strict';
/* WhatsInside UI. Plain JS, no framework, no build step. */

const T = window.__TAURI__;
const invoke = T ? T.core.invoke : (cmd, args) => window.WIMock.invoke(cmd, args);

const $ = (s, el = document) => el.querySelector(s);
const $$ = (s, el = document) => [...el.querySelectorAll(s)];
const esc = (s) => String(s ?? '').replace(/[&<>"']/g, (c) => ({ '&': '&amp;', '<': '&lt;', '>': '&gt;', '"': '&quot;', "'": '&#39;' }[c]));
const sk = (n, d = 0) => n.toLocaleString('sk-SK', { maximumFractionDigits: d, minimumFractionDigits: d });

function bytes(b) {
  if (!b) return '0 B';
  const u = ['B', 'KB', 'MB', 'GB', 'TB'];
  let i = 0;
  while (b >= 1024 && i < u.length - 1) { b /= 1024; i++; }
  return `${sk(b, i === 0 || b >= 100 ? 0 : b >= 10 ? 1 : 1)} ${u[i]}`;
}
const rate = (bps) => (bps < 1024 ? '–' : `${bytes(bps)}/s`);
const plural = (n, one, few, many) => (n === 1 ? one : n >= 2 && n <= 4 ? few : many);

const ICON = {
  dir: '<svg viewBox="0 0 16 16"><path d="M1.8 4.2c0-.7.5-1.2 1.2-1.2h3l1.4 1.6H13c.7 0 1.2.5 1.2 1.2v6c0 .7-.5 1.2-1.2 1.2H3c-.7 0-1.2-.5-1.2-1.2z"/></svg>',
  file: '<svg viewBox="0 0 16 16"><path d="M4 1.8h5l3 3v8.4c0 .6-.4 1-1 1H4c-.6 0-1-.4-1-1V2.8c0-.6.4-1 1-1z"/><path d="M9 1.8v3h3"/></svg>',
  small: '<svg viewBox="0 0 16 16"><rect x="2" y="2" width="5" height="5" rx="1"/><rect x="9" y="2" width="5" height="5" rx="1"/><rect x="2" y="9" width="5" height="5" rx="1"/><rect x="9" y="9" width="5" height="5" rx="1"/></svg>',
  other: '<svg viewBox="0 0 16 16"><path d="M4 8h.01M8 8h.01M12 8h.01"/></svg>',
  reveal: '<svg viewBox="0 0 16 16"><path d="M9 2.5h4.5V7M13.5 2.5L7.5 8.5"/><path d="M12 9.5v3c0 .6-.4 1-1 1H3.5c-.6 0-1-.4-1-1V5c0-.6.4-1 1-1h3"/></svg>',
  drive: '<svg viewBox="0 0 16 16"><rect x="1.8" y="3" width="12.4" height="10" rx="2"/><path d="M1.8 9.5h12.4M11.2 11.3h.01"/></svg>',
  usb: '<svg viewBox="0 0 16 16"><rect x="4.5" y="1.8" width="7" height="5" rx="1"/><path d="M3.5 6.8h9v5.4c0 1.1-.9 2-2 2h-5c-1.1 0-2-.9-2-2z"/></svg>',
  broom: '<svg viewBox="0 0 16 16"><path d="M10.5 1.8L7.2 7.5"/><path d="M5 7.2l4.4 2.5-1.6 4.5c-2-.4-4.3-1.7-5.6-3.4z"/></svg>',
  check: '<svg viewBox="0 0 16 16"><path d="M3.5 8.5l3 3 6-7"/></svg>',
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

/* ---------------------------------------------------------- segmented control */
function seg(el, onChange) {
  const knob = $('.knob', el);
  const place = () => {
    const on = $('button.on', el);
    if (!on) return;
    knob.style.width = `${on.offsetWidth}px`;
    knob.style.transform = `translateX(${on.offsetLeft}px)`;
  };
  el.addEventListener('click', (e) => {
    const b = e.target.closest('button');
    if (!b || b.classList.contains('on')) return;
    $$('button', el).forEach((x) => x.classList.toggle('on', x === b));
    place();
    onChange(b.dataset.k);
  });
  requestAnimationFrame(place);
  return place;
}

/* ---------------------------------------------------------- tooltip */
const tip = $('#tip');
function showTip(html, x, y) {
  tip.innerHTML = html;
  tip.classList.add('on');
  const r = tip.getBoundingClientRect();
  tip.style.left = `${Math.min(x + 14, innerWidth - r.width - 10)}px`;
  tip.style.top = `${Math.min(y + 16, innerHeight - r.height - 10)}px`;
}
const hideTip = () => tip.classList.remove('on');

/* ---------------------------------------------------------- sidebar status */
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
let current = 'storage';
const navButtons = $$('#nav button');
function setView(name) {
  if (name === current && views[name]?.entered) return;
  views[current]?.leave?.();
  current = name;
  navButtons.forEach((b, i) => {
    const on = b.dataset.view === name;
    b.classList.toggle('on', on);
    if (on) $('#nav .ind').style.transform = `translateY(${i * 34}px)`;
  });
  $$('.view').forEach((v) => v.classList.toggle('on', v.id === `v-${name}`));
  $('#main').scrollTop = 0;
  hideTip();
  if (views[name]) { views[name].entered = true; views[name].enter?.(); }
}
navButtons.forEach((b) => (b.onclick = () => setView(b.dataset.view)));

/* ========================================================== STORAGE */
const st = { drives: [], status: null, listing: null, tab: 'dirs', top: [], caches: [], timer: null, drive: null, colors: new Map() };
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
    const notes = [];
    if (s.errors) notes.push(`${sk(s.errors)} ${plural(s.errors, 'priečinok', 'priečinky', 'priečinkov')} som nemohol otvoriť (systémové, bez prístupu).`);
    if (s.cloud_bytes) notes.push(`${bytes(s.cloud_bytes)} v OneDrive je len online a na disku nezaberá miesto, preto to nerátam.`);
    body.innerHTML = `
      <div class="stats">
        <div class="card stat land" style="--i:0"><div class="k">Naskenované</div><div class="v num">${bytes(s.bytes)}</div></div>
        <div class="card stat land" style="--i:1"><div class="k">Súbory</div><div class="v num">${sk(s.files)}</div></div>
        <div class="card stat land" style="--i:2"><div class="k">Trvalo to</div><div class="v num">${sk(s.elapsed_ms / 1000, s.elapsed_ms < 10000 ? 1 : 0)}<small>s</small></div></div>
        <div class="card stat land" style="--i:3"><div class="k">Cache, ktorú môžeš uvoľniť</div><div class="v num">${bytes(cacheTotal)}</div></div>
      </div>
      <div class="crumbs" id="crumbs"></div>
      <div class="card treemap land" id="tmCard" style="--i:4"></div>
      <div style="display:flex;align-items:center;margin:20px 0 10px;gap:12px">
        <div class="seg" id="stab"><div class="knob"></div><button data-k="dirs" class="${st.tab === 'dirs' ? 'on' : ''}">Obsah</button><button data-k="files" class="${st.tab === 'files' ? 'on' : ''}">Najväčšie súbory</button><button data-k="cache" class="${st.tab === 'cache' ? 'on' : ''}">Cache na uvoľnenie</button></div>
      </div>
      <div class="card list" id="slist"></div>
      ${notes.length ? `<div class="note">${notes.map(esc).join('<br>')}</div>` : ''}
      <div class="note">Veľkosti sú logické. Priečinok Windows\\WinSxS môže vyzerať väčší, než naozaj je (pevné odkazy).</div>`;
    seg($('#stab'), (k) => { st.tab = k; renderList(); });
    new ResizeObserver(() => drawTreemap()).observe($('#tmCard'));
  }
  setStatus('Sken hotový', `${s.root.replace(/\\$/, '')} · ${bytes(s.bytes)} za ${sk(s.elapsed_ms / 1000, 0)} s`, 'mint');
  renderCrumbs();
  drawTreemap();
  renderList();
}

async function startScan(root) {
  st.drive = st.drives.find((d) => d.root === root) || null;
  st.listing = null;
  st.colors.clear();
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
    [st.top, st.caches] = await Promise.all([invoke('storage_top_files'), invoke('storage_caches')]);
    $('#storageBody').innerHTML = '';
    await openNode(0, false);
  } else {
    idleStatus();
    renderStorage();
  }
}

async function openNode(id, rerender = true) {
  const l = await invoke('storage_node', { id });
  if (!l) return;
  st.listing = l;
  if (id === 0) (l.node.children || []).filter((c) => c.kind === 'dir').forEach((c, i) => st.colors.set(c.id, PALETTE[i % PALETTE.length]));
  if (rerender && $('#tmCard')) {
    renderCrumbs();
    drawTreemap(true);
    if (st.tab !== 'dirs') { st.tab = 'dirs'; $$('#stab button').forEach((b) => b.classList.toggle('on', b.dataset.k === 'dirs')); }
    renderList();
    placeSeg('#stab');
  } else {
    renderStorage();
  }
}
function placeSeg(sel) {
  const el = $(sel);
  if (!el) return;
  const on = $('button.on', el), knob = $('.knob', el);
  if (on) { knob.style.width = `${on.offsetWidth}px`; knob.style.transform = `translateX(${on.offsetLeft}px)`; }
}

function renderCrumbs() {
  const l = st.listing;
  if (!l) return;
  const c = $('#crumbs');
  c.innerHTML = l.crumbs
    .map((cr, i) => `${i ? '<span class="sep">›</span>' : ''}<button data-id="${cr.id}">${esc(cr.name || '\\')}</button>`)
    .join('') + `<span class="spacer"></span><span class="num" style="color:var(--muted);font-size:12px">${bytes(l.node.size)} · ${sk(l.node.files)} ${plural(l.node.files, 'súbor', 'súbory', 'súborov')}</span>
    <button class="btn small ghost" id="revealCur" title="Otvoriť v Prieskumníkovi">${ICON.reveal}Otvoriť</button>`;
  $$('button[data-id]', c).forEach((b) => (b.onclick = () => openNode(+b.dataset.id)));
  $('#revealCur').onclick = () => reveal(l.path);
}

function reveal(path) {
  invoke('storage_reveal', { path }).catch((e) => setStatus('Nepodarilo sa otvoriť', String(e), 'rec'));
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
  return `<b>${esc(n.name)}</b><span class="num">${bytes(n.size)}${pct}${files}</span>`;
}

function drawTreemap(animate = false) {
  const card = $('#tmCard'), l = st.listing;
  if (!card || !l) return;
  const W = card.clientWidth - 16, H = card.clientHeight - 16;
  const kids = (l.node.children || []).filter((c) => c.size > 0);
  card.innerHTML = '';
  if (!kids.length) { card.innerHTML = `<div class="empty">${ICON.dir}Tento priečinok je prázdny.</div>`; return; }
  const inherited = l.crumbs.map((c) => st.colors.get(c.id)).find(Boolean);
  const color = (n, i) => (n.kind === 'file' ? 'var(--muted)' : n.kind !== 'dir' ? 'var(--faint)' : st.colors.get(n.id) || inherited || PALETTE[i % PALETTE.length]);
  const rects = squarify(kids.map((k) => k.size), 8, 8, W, H);
  const G = 3, frag = document.createDocumentFragment();
  kids.forEach((n, i) => {
    let [x, y, w, h] = rects[i];
    x += G / 2; y += G / 2; w -= G; h -= G;
    if (w < 2 || h < 2) return;
    const el = document.createElement('div');
    el.className = `tm ${n.kind}${animate ? ' land' : ''}`;
    if (animate) el.style.setProperty('--i', Math.min(i, 12));
    el.style.cssText += `left:${x}px;top:${y}px;width:${w}px;height:${h}px;--c:${color(n, i)}`;
    const big = w > 90 && h > 46;
    if (big || (w > 54 && h > 30)) el.innerHTML = `<div class="lbl">${esc(n.name)}<small>${bytes(n.size)}</small></div>`;
    el.onmousemove = (e) => showTip(tmTip(n, l.node.size), e.clientX, e.clientY);
    el.onmouseleave = hideTip;
    if (n.kind === 'dir' && n.id != null) el.onclick = () => { hideTip(); openNode(n.id); };
    // second level inside big folders
    const sub = (n.children || []).filter((c) => c.size > 0);
    if (big && h > 90 && w > 120 && sub.length) {
      const sr = squarify(sub.map((c) => c.size), 4, 38, w - 8, h - 42);
      sub.forEach((c, k) => {
        let [sx, sy, sw, sh] = sr[k];
        sx += 1; sy += 1; sw -= 2; sh -= 2;
        if (sw < 3 || sh < 3) return;
        const s = document.createElement('div');
        s.className = `tm sub ${c.kind}`;
        s.style.cssText = `left:${sx}px;top:${sy}px;width:${sw}px;height:${sh}px;--c:${color(n, i)}`;
        if (sw > 64 && sh > 28) s.innerHTML = `<div class="lbl">${esc(c.name)}</div>`;
        s.onmousemove = (e) => { e.stopPropagation(); showTip(tmTip(c, n.size), e.clientX, e.clientY); };
        if (c.kind === 'dir' && c.id != null) s.onclick = (e) => { e.stopPropagation(); hideTip(); openNode(c.id); };
        el.appendChild(s);
      });
    }
    frag.appendChild(el);
  });
  card.appendChild(frag);
}

const SYSFILES = {
  'pagefile.sys': 'Stránkovací súbor Windowsu. Nemaž ho.',
  'hiberfil.sys': 'Hibernácia. Ak ju nepoužívaš, vypneš ju príkazom powercfg /h off.',
  'swapfile.sys': 'Súčasť Windowsu. Nemaž ho.',
  'memory.dmp': 'Výpis po páde systému. Dá sa zmazať.',
};

function renderList() {
  const box = $('#slist'), l = st.listing;
  if (!box || !l) return;
  if (st.tab === 'dirs') {
    const kids = l.node.children || [];
    if (!kids.length) { box.innerHTML = `<div class="empty">${ICON.dir}Prázdny priečinok.</div>`; return; }
    box.innerHTML = kids.map((c) => {
      const share = l.node.size ? c.size / l.node.size : 0;
      const canReveal = c.kind === 'dir' || c.kind === 'file';
      const extra = c.kind === 'dir' ? `<small>${sk(c.files)} ${plural(c.files, 'súbor', 'súbory', 'súborov')}</small>` : '';
      return `<div class="lrow ${c.kind === 'dir' && c.id != null ? 'click' : ''}" data-id="${c.id ?? ''}" data-kind="${c.kind}" data-name="${esc(c.name)}">
        <span class="ic ${c.kind === 'dir' ? 'dir' : ''}">${ICON[c.kind] || ICON.file}</span>
        <span class="nm">${esc(c.name)}${extra}</span>
        <div class="bar"><i style="width:${(share * 100).toFixed(1)}%;background:${c.kind === 'dir' ? 'var(--accent)' : 'var(--faint)'}"></i></div>
        <span class="sz num">${bytes(c.size)}</span><span class="pct num">${sk(share * 100, share < 0.1 ? 1 : 0)} %</span>
        <span>${canReveal ? `<button class="btn icon ghost rev" title="Otvoriť v Prieskumníkovi">${ICON.reveal}</button>` : ''}</span></div>`;
    }).join('');
    $$('.lrow', box).forEach((r) => {
      if (r.dataset.kind === 'dir' && r.dataset.id) r.onclick = () => openNode(+r.dataset.id);
      const b = $('.rev', r);
      if (b) b.onclick = (e) => { e.stopPropagation(); reveal(`${l.path.replace(/\\$/, '')}\\${r.dataset.name}`); };
    });
  } else if (st.tab === 'files') {
    if (!st.top.length) { box.innerHTML = `<div class="empty">${ICON.file}Žiadne veľké súbory.</div>`; return; }
    const max = st.top[0].size;
    box.innerHTML = st.top.map((f, i) => {
      const cut = f.path.lastIndexOf('\\');
      const name = f.path.slice(cut + 1), dir = f.path.slice(0, cut);
      const hint = SYSFILES[name.toLowerCase()];
      return `<div class="lrow path" data-i="${i}"><span class="ic">${ICON.file}</span>
        <span class="nm">${esc(name)} <span class="dim">${esc(hint || dir)}</span></span>
        <div class="bar"><i style="width:${((f.size / max) * 100).toFixed(1)}%;background:var(--blue)"></i></div>
        <span class="sz num">${bytes(f.size)}</span><span></span>
        <span><button class="btn icon ghost rev" title="Ukázať v Prieskumníkovi">${ICON.reveal}</button></span></div>`;
    }).join('');
    $$('.lrow', box).forEach((r) => ($('.rev', r).onclick = () => reveal(st.top[+r.dataset.i].path)));
  } else {
    if (!st.caches.length) {
      box.innerHTML = `<div class="empty">${ICON.check}Nenašiel som žiadnu známu cache, ktorá by sa oplatila mazať.</div>`;
      return;
    }
    const total = st.caches.reduce((a, c) => a + c.size, 0);
    box.innerHTML = `<div class="cache" style="padding-bottom:6px"><div><div class="t">Spolu sa dá uvoľniť ${bytes(total)}</div>
      <div class="h">Zatiaľ nič nemažem. Ku každej položke ti napíšem, ako ju bezpečne vyčistiť.</div></div></div>` +
      st.caches.map((c, i) => `<div class="cache" data-i="${i}"><div><div class="t">${esc(c.label)}</div><div class="p">${esc(c.path)}</div><div class="h">${esc(c.hint)}</div></div>
        <span class="num">${bytes(c.size)}</span><button class="btn icon ghost" title="Otvoriť v Prieskumníkovi">${ICON.reveal}</button></div>`).join('');
    $$('.cache[data-i]', box).forEach((r) => ($('button', r).onclick = () => reveal(st.caches[+r.dataset.i].path)));
  }
}

views.storage = {
  async enter() {
    if (!st.status) {
      await loadDrives();
      st.status = await invoke('storage_status');
      if (st.status.phase === 'scanning') pollScan();
      else if (st.status.phase === 'done') pollScan();
      else renderStorage();
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
  enter() {
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

/* ========================================================== HARDWARE */
let hwLoaded = false;
views.hw = {
  async enter() {
    if (!hwLoaded) {
      $('#hwList').innerHTML = '<div class="card empty">Hľadám…</div>';
      const r = await invoke('hw_scan');
      hwLoaded = true;
      $('#hwList').innerHTML = r.devices.map((d, i) => `<div class="card dev land" style="--i:${i}">
        <div class="k"><span class="dot ${d.supported ? 'mint' : ''}"></span>${esc(d.kind)}${d.supported ? ' · podporované' : ''}</div>
        <div class="n">${esc(d.name)}</div><div class="i">${esc(d.id)}</div><div class="d">${esc(d.note)}</div></div>`).join('') +
        (r.nvapi ? '' : '') + (r.devices.length ? '' : '<div class="card empty">Nenašiel som nič, čo poznám.</div>') +
        r.errors.map((e) => `<div class="card dev"><div class="k"><span class="dot rec"></span>Chyba</div><div class="d">${esc(e)}</div></div>`).join('');
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
setView('storage');
setIdle();
// Count for the sidebar without opening the view.
invoke('autostart_list').then((items) => { if (!startupItems) { startupItems = items; renderStartup(); } }).catch(() => {});
