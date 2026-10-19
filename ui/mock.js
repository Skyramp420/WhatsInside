// Fake backend so the UI can be previewed in a normal browser.
// Never used inside the app (window.__TAURI__ exists there).
(function () {
  if (window.__TAURI__) return;
  const GB = 1 << 30, MB = 1 << 20;
  let scanStart = 0, scanning = false, done = false, snapN = 0;

  // A small fake tree.
  const nodes = [];
  const TAGS = {
    'Cyberpunk 2077': { kind: 'game', label: 'Cyberpunk 2077', source: 'Steam' },
    'Baldurs Gate 3': { kind: 'game', label: 'Baldurs Gate 3', source: 'Steam' },
    'Counter-Strike 2': { kind: 'game', label: 'Counter-Strike 2', source: 'Steam' },
    'Fortnite': { kind: 'game', label: 'Fortnite', source: 'Epic Games' },
    'Adobe': { kind: 'app', label: 'Adobe', source: null },
    'Blackmagic Design': { kind: 'app', label: 'Blackmagic Design', source: null },
    'NVIDIA Corporation': { kind: 'app', label: 'NVIDIA Corporation', source: null },
    'Windows': { kind: 'system', label: 'Windows', source: null },
    'CacheClip': { kind: 'cache', label: 'DaVinci Resolve – render cache', source: null },
  };
  function add(name, size, files, kind, parent, kids) {
    const id = nodes.length;
    nodes.push({ id, name, size, files, kind, parent, children: [], tag: TAGS[name] || null, age: 3 + (id * 37) % 900 });
    if (parent != null) nodes[parent].children.push(id);
    (kids || []).forEach((k) => add(k[0], k[1], k[2], k[3], id, k[4]));
    return id;
  }
  add('C:\\', 0, 0, 'dir', null, [
    ['Users', 0, 0, 'dir', [
      ['Alex', 0, 0, 'dir', [
        ['Videos', 220 * GB, 1840, 'dir', [['ShapeFX', 160 * GB, 900, 'dir', [['Klient X', 98 * GB, 400, 'dir'], ['CacheClip', 41 * GB, 3200, 'dir'], ['Archív', 21 * GB, 300, 'dir']]], ['Klipy', 60 * GB, 940, 'dir']]],
        ['AppData', 64 * GB, 210000, 'dir', [['Local', 48 * GB, 150000, 'dir'], ['Roaming', 16 * GB, 60000, 'dir']]],
        ['Downloads', 38 * GB, 1200, 'dir'],
        ['Documents', 12 * GB, 5400, 'dir'],
      ]],
      ['Public', 1 * GB, 40, 'dir'],
    ]],
    ['Program Files', 0, 0, 'dir', [['Steam', 0, 0, 'dir', [['steamapps', 0, 0, 'dir', [['common', 0, 0, 'dir', [['Cyberpunk 2077', 92 * GB, 30000, 'dir'], ['Baldurs Gate 3', 128 * GB, 20000, 'dir'], ['Counter-Strike 2', 38 * GB, 12000, 'dir']]]]]]], ['Epic Games', 0, 0, 'dir', [['Fortnite', 46 * GB, 9000, 'dir']]], ['Adobe', 24 * GB, 40000, 'dir'], ['Blackmagic Design', 6 * GB, 9000, 'dir'], ['NVIDIA Corporation', 3 * GB, 2000, 'dir']]],
    ['Windows', 0, 0, 'dir', [['WinSxS', 14 * GB, 80000, 'dir'], ['System32', 7 * GB, 30000, 'dir'], ['Installer', 5 * GB, 600, 'dir']]],
    ['ProgramData', 9 * GB, 30000, 'dir'],
    ['pagefile.sys', 24 * GB, 1, 'file'],
    ['hiberfil.sys', 12 * GB, 1, 'file'],
    ['18 menších súborov', 2 * MB, 18, 'small'],
  ]);
  (function sum(id) {
    const n = nodes[id];
    if (n.children.length) { n.size = 0; n.files = 0; n.children.forEach((c) => { sum(c); n.size += nodes[c].size; n.files += nodes[c].files; }); }
    n.children.sort((a, b) => nodes[b].size - nodes[a].size);
  })(0);
  const view = (id, depth) => {
    const n = nodes[id];
    return { id, name: n.name, size: n.size, files: n.files, kind: n.kind, age_days: n.age, tag: n.tag, children: depth && n.kind === 'dir' ? n.children.map((c) => view(c, depth - 1)) : null };
  };
  const pathOf = (id) => { const p = []; let c = id; while (nodes[c].parent != null) { p.unshift(nodes[c].name); c = nodes[c].parent; } return 'C:' + (p.length ? '\\' + p.join('\\') : '\\'); };

  const R = (n) => n * MB;
  function procs() {
    snapN++;
    const j = (x) => x * (0.9 + 0.2 * Math.random());
    const g = (key, name, verdict, about, list) => ({ key, name, verdict, about, procs: list, cpu: list.reduce((a, p) => a + p.cpu, 0), ram: list.reduce((a, p) => a + (p.ram || 0), 0), io_bps: list.reduce((a, p) => a + p.io_bps, 0) });
    const p = (pid, exe, ram, cpu = 0, io = 0, path = null) => ({ pid, exe, ram: R(ram), cpu: snapN > 1 ? j(cpu) : 0, io_bps: snapN > 1 ? j(io) : 0, path });
    return {
      cpu_count: 24, cpu_total: snapN > 1 ? j(14) : 0, ram_total: 64 * GB, ram_used: 23.4 * GB,
      groups: [
        g('exe:chrome.exe', 'chrome', 'unknown', null, [p(1, 'chrome.exe', 610, 3.1, 120000, 'C:\\Program Files\\Google\\Chrome\\Application\\chrome.exe'), p(2, 'chrome.exe', 380, 0.4), p(3, 'chrome.exe', 210, 0.2)]),
        g('windows', 'Windows', 'system', 'Súčasti Windowsu. Nechaj ich tak.', [p(4, 'dwm.exe', 240, 1.2), p(5, 'explorer.exe', 180, 0.3), p(6, 'svchost.exe', null, 0.5)]),
        g('Logitech G HUB', 'Logitech G HUB', 'peripheral', 'Nastavenia myši a klávesnice Logitech. Ak má myš profil uložený v sebe, G HUB nemusí bežať stále.', [p(7, 'lghub_system_tray.exe', 77), p(8, 'lghub_agent.exe', 75, 0.2)]),
        g('MSI Center / Mystic Light', 'MSI Center / Mystic Light', 'rgb', 'Ovláda svetlá dosky a všetkého, čo je na nej zapojené (vetráky, AIO). Beží stále, aj keď ju práve nepoužívaš.', [p(9, 'LEDKeeper2.exe', 92, 0.6, 0, 'C:\\Program Files (x86)\\MSI\\MSI Center\\Mystic Light\\LEDKeeper2.exe')]),
        g('GIGABYTE Control Center', 'GIGABYTE Control Center', 'rgb', 'Ovláda svetlá a displej grafiky. Má niekoľko pomocných procesov (meranie FPS, PresentMon), ktoré bežia stále.', [p(10, 'GvFvSDKCSharp.exe', 42, 0.3), p(11, 'GCC.exe', 21, 0.1), p(12, 'GvFPS.exe', 13, 0.8), p(13, 'Gv.PresentMon.exe', 9, 0.4)]),
        g('Discord', 'Discord', 'optional', 'Spúšťa sa s Windowsom. Dá sa vypnúť v Discorde → Nastavenia → Nastavenia systému Windows.', [p(14, 'Discord.exe', 310, 0.9, 3000)]),
        g('exe:lookatthis.exe', 'LookAtThis', 'unknown', null, [p(15, 'LookAtThis.exe', 140, 0.5, 2400000)]),
        g('WhatsInside', 'WhatsInside', 'needed', 'To som ja. Keď ma zatvoríš, nebeží nič.', [p(16, 'whatsinside.exe', 38, 0.3)]),
      ],
    };
  }

  window.WIMock = {
    async invoke(cmd, args) {
      await new Promise((r) => setTimeout(r, 40));
      switch (cmd) {
        case 'storage_drives': return [
          { root: 'C:\\', label: 'Systém', fs: 'NTFS', total: 1863 * GB, free: 1150 * GB, removable: false },
          { root: 'D:\\', label: 'Projekty', fs: 'NTFS', total: 3726 * GB, free: 420 * GB, removable: false },
          { root: 'E:\\', label: 'USB', fs: 'exFAT', total: 128 * GB, free: 100 * GB, removable: true },
        ];
        case 'storage_start': scanStart = Date.now(); scanning = true; done = false; return null;
        case 'storage_cancel': scanning = false; return null;
        case 'storage_status': {
          if (!scanning && !done) return { phase: 'idle', root: null, files: 0, dirs: 0, bytes: 0, errors: 0, cloud_bytes: 0, elapsed_ms: 0 };
          const t = Date.now() - scanStart, f = Math.min(1, t / (window.MOCK_SCAN_MS ?? 2500));
          if (f >= 1) { scanning = false; done = true; }
          return { phase: done ? 'done' : 'scanning', root: 'C:\\', files: Math.round(f * 431200), dirs: Math.round(f * 61000), bytes: f * nodes[0].size, errors: 14, cloud_bytes: 3.2 * GB, elapsed_ms: t };
        }
        case 'storage_node': { const n = nodes[args.id]; const crumbs = []; let c = args.id; while (c != null) { crumbs.unshift({ id: c, name: c === 0 ? 'C:' : nodes[c].name }); c = nodes[c].parent; } let o = null, k = args.id; while (k != null && !o) { o = nodes[k].tag; k = nodes[k].parent; } return { node: view(args.id, 2), path: pathOf(args.id), crumbs, owner: o }; }
        case 'storage_top_files': return [
          { path: 'C:\\pagefile.sys', size: 24 * GB }, { path: 'C:\\Users\\Alex\\Videos\\ShapeFX\\Klient\\export_final_v3.mov', size: 18.4 * GB },
          { path: 'C:\\hiberfil.sys', size: 12 * GB }, { path: 'C:\\Users\\Alex\\Downloads\\Win11_24H2.iso', size: 5.8 * GB },
          { path: 'C:\\Program Files\\Steam\\steamapps\\common\\Cyberpunk 2077\\archive\\pc\\content\\basegame_4_gamedata.archive', size: 4.1 * GB },
        ];
        case 'storage_caches': return [
          { label: 'DaVinci Resolve – render cache', hint: 'Resolve si ju vytvorí znova. Zmaž ju v Resolve (Playback → Delete Render Cache) alebo keď Resolve nebeží.', path: 'C:\\Users\\Alex\\Videos\\ShapeFX\\CacheClip', size: 41 * GB, files: 3200, id: 1 },
          { label: 'Adobe – media cache (Premiere, After Effects)', hint: 'Bezpečné zmazať, keď Adobe appky nebežia. Vytvorí sa znova pri otvorení projektu.', path: 'C:\\Users\\Alex\\AppData\\Roaming\\Adobe\\Common\\Media Cache Files', size: 12.6 * GB, files: 800, id: 2 },
          { label: 'Dočasné súbory', hint: 'Bezpečné zmazať. Súbory, ktoré sa práve používajú, Windows zmazať nedovolí, a to je v poriadku.', path: 'C:\\Users\\Alex\\AppData\\Local\\Temp', size: 6.3 * GB, files: 12000, id: 3 },
          { label: 'Shader cache grafiky', hint: 'Hry si ju vytvoria znova. Prvé spustenie hry potom môže chvíľu sekať.', path: 'C:\\Users\\Alex\\AppData\\Local\\NVIDIA\\DXCache', size: 2.1 * GB, files: 400, id: 4 },
        ];
        case 'storage_reveal': return null;
        case 'storage_owners': {
          const g = nodes.filter((n) => n.tag?.kind === 'game').map((n) => ({ id: n.id, label: n.tag.label, source: n.tag.source, path: pathOf(n.id), size: n.size, age_days: n.name === 'Counter-Strike 2' ? 420 : 12 })).sort((a, b) => b.size - a.size);
          const a = nodes.filter((n) => n.tag?.kind === 'app').map((n) => ({ id: n.id, label: n.tag.label, source: null, path: pathOf(n.id), size: n.size, age_days: 20 })).sort((a, b) => b.size - a.size);
          return { games: g, apps: a, games_total: g.reduce((s, x) => s + x.size, 0) };
        }
        case 'storage_old': return [
          { id: 1, path: 'C:\\Users\\Alex\\Downloads\\Win11_24H2_Slovak_x64.iso', size: 5.8 * GB, age_days: 210, reason: 'installer' },
          { id: 2, path: 'C:\\Users\\Alex\\Downloads\\DaVinci_Resolve_19.1_Windows.zip', size: 3.2 * GB, age_days: 95, reason: 'installer' },
          { id: 3, path: 'C:\\Users\\Alex\\Downloads\\OBS-Studio-30.2-Windows-Installer.exe', size: 140 * MB, age_days: 320, reason: 'installer' },
          { id: 4, path: 'C:\\Users\\Alex\\Videos\\Klipy\\2023-09 render_final.mov', size: 9.4 * GB, age_days: 760, reason: 'old' },
          { id: 5, path: 'C:\\Users\\Alex\\Documents\\zaloha_telefon_2022.zip', size: 4.1 * GB, age_days: 1040, reason: 'old' },
        ];
        case 'storage_search': {
          const q = args.query.toLowerCase();
          return nodes.filter((n) => n.parent != null && n.kind !== 'small' && n.name.toLowerCase().includes(q)).slice(0, 50)
            .map((n) => { let o = null, k = n.id; while (k != null && !o) { o = nodes[k].tag; k = nodes[k].parent; } return { id: n.id, parent: n.parent, name: n.name, path: pathOf(n.id), size: n.size, kind: n.kind, age_days: n.age, owner: o?.label || null }; })
            .sort((a, b) => b.size - a.size);
        }
        case 'quick_check': return {
          caches: [
            { label: 'Dočasné súbory', hint: '', path: 'C:\\Users\\Alex\\AppData\\Local\\Temp', size: 6.3 * GB, files: 12000 },
            { label: 'Adobe – media cache (Premiere, After Effects)', hint: '', path: '', size: 12.6 * GB, files: 800 },
            { label: 'Stiahnuté aktualizácie Windowsu', hint: '', path: '', size: 3.4 * GB, files: 200 },
            { label: 'Shader cache grafiky', hint: '', path: '', size: 2.1 * GB, files: 400 },
          ],
          installers: [
            { id: 0, path: 'C:\\Users\\Alex\\Downloads\\Win11_24H2_Slovak_x64.iso', size: 5.8 * GB, age_days: 210, reason: 'installer' },
            { id: 0, path: 'C:\\Users\\Alex\\Downloads\\DaVinci_Resolve_19.1_Windows.zip', size: 3.2 * GB, age_days: 95, reason: 'installer' },
          ],
        };
        case 'system_facts': return { power_guid: '381b4222-f694-41f0-9685-ff5bb260df2e', power_name: 'Vyvážený', game_mode: true, hags: false, hibernation: true, hiberfil_bytes: 25.4 * GB, os: 'Windows 11 Pro 24H2', build: 26100 };
        case 'pc_specs': return {
          cpu: 'Intel(R) Core(TM) i7-14700K', threads: 28,
          gpus: [{ name: 'NVIDIA GeForce RTX 5080', vram: 16 * GB }],
          board: 'MSI MAG Z790 TOMAHAWK WIFI (MS-7D91)', ram_total: 32 * GB,
          ram_modules: [
            { slot: 'DIMMA2', size_mb: 16384, speed: 6400, configured_speed: 6000, kind: 'DDR5', manufacturer: 'G Skill Intl', part: 'F5-6400J3239G16G' },
            { slot: 'DIMMB2', size_mb: 16384, speed: 6400, configured_speed: 6000, kind: 'DDR5', manufacturer: 'G Skill Intl', part: 'F5-6400J3239G16G' },
          ],
          disks: ['Samsung SSD 990 PRO 2TB', 'WD_BLACK SN850X 4000GB'], os: 'Windows 11 Pro 24H2',
        };
        case 'save_card': return 'C:\\Users\\Alex\\Pictures\\WhatsInside\\WhatsInside-PC-1760900000.png';
        case 'procs_snapshot': return procs();
        case 'autostart_list': return [
          { source: 'Služba', name: 'MSI Central Service', command: 'C:\\Program Files (x86)\\MSI\\MSI Center\\MSI.CentralServer.exe', app: 'MSI Center / Mystic Light', verdict: 'rgb', about: 'Ovláda svetlá dosky a všetkého, čo je na nej zapojené (vetráky, AIO). Beží stále, aj keď ju práve nepoužívaš.', enabled: true, running: true },
          { source: 'Služba', name: 'GIGABYTE Control Center Service', command: '"C:\\Program Files\\GIGABYTE\\Control Center\\Lib\\GBT_VGA\\Service\\GvService.exe"', app: 'GIGABYTE Control Center', verdict: 'rgb', about: 'Ovláda svetlá a displej grafiky.', enabled: true, running: true },
          { source: 'Pri prihlásení (používateľ)', name: 'Discord', command: 'C:\\Users\\Alex\\AppData\\Local\\Discord\\Update.exe --processStart Discord.exe', app: 'Discord', verdict: 'optional', about: 'Spúšťa sa s Windowsom. Dá sa vypnúť v Discorde → Nastavenia → Nastavenia systému Windows.', enabled: true, running: null },
          { source: 'Pri prihlásení (používateľ)', name: 'Steam', command: '"C:\\Program Files\\Steam\\steam.exe" -silent', app: 'Steam', verdict: 'optional', about: 'Nemusí bežať, kým nehráš.', enabled: false, running: null },
          { source: 'Pri prihlásení (používateľ)', name: 'LGHUB', command: '"C:\\Program Files\\LGHUB\\system_tray\\lghub_system_tray.exe" --minimized', app: 'Logitech G HUB', verdict: 'peripheral', about: 'Nastavenia myši a klávesnice Logitech.', enabled: true, running: null },
          { source: 'Priečinok Po spustení (používateľ)', name: 'LookAtThis', command: 'C:\\Users\\Alex\\AppData\\Roaming\\Microsoft\\Windows\\Start Menu\\Programs\\Startup\\LookAtThis.lnk', app: null, verdict: 'unknown', about: null, enabled: true, running: null },
        ];
        case 'hw_scan': return {
          nvapi: true, errors: [], devices: [
            { kind: 'RGB radič', name: 'MSI MAG Z790 TOMAHAWK WIFI – Mystic Light', id: 'USB 1462:7D91', note: '185-bajtový protokol, per-LED direct mód. Ovláda aj ARGB headery (AIO, vetráky cez hub).', supported: true },
            { kind: 'Grafika', name: 'GIGABYTE AORUS GeForce RTX 5080 MASTER ICE', id: 'PCI 10DE:2C02 SUBSYS 418C1458 · RGB na I2C 0x71', note: 'RGB Fusion 2 cez I2C. Svetlá áno, LCD displej zatiaľ nie (protokol nie je zmapovaný).', supported: true },
          ],
        };
      }
      throw new Error('mock: ' + cmd);
    },
  };
})();
