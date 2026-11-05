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
  const view = (id, depth, skip) => {
    const n = nodes[id];
    let src = id; const via = [];
    while (skip && depth && via.length < 4) {
      const c = nodes[src], b = c.children[0] != null && nodes[c.children[0]];
      if (!b || b.kind !== 'dir' || b.size * 5 < c.size * 4) break;
      via.push(b.name); src = c.children[0];
    }
    const sn = nodes[src];
    return { id, name: n.name, size: n.size, files: n.files, kind: n.kind, age_days: n.age, tag: n.tag, via, children: depth && sn.kind === 'dir' ? sn.children.map((c) => view(c, depth - 1, true)) : null };
  };
  const pathOf = (id) => { const p = []; let c = id; while (nodes[c].parent != null) { p.unshift(nodes[c].name); c = nodes[c].parent; } return 'C:' + (p.length ? '\\' + p.join('\\') : '\\'); };


  const A = (o) => ({ exe: null, app: null, about: null, advice: null, running: null, hidden: false, needs_admin: false, enabled: true, toggle: { t: 'approved', hive: 'HKCU', key: 'x', name: o.name }, ...o });
  const mockChanges = [];
  const mockBench = { phase: 'idle' };
  let mockRestore = 0, mockMethod = 'walk';
  const mockCaches = [
          { label: 'DaVinci Resolve – render cache', hint: 'Resolve si ju vytvorí znova. Zmaž ju v Resolve (Playback → Delete Render Cache) alebo keď Resolve nebeží.', path: 'C:\\Users\\Alex\\Videos\\ShapeFX\\CacheClip', size: 41 * GB, files: 3200, id: 1, how: 'contents' },
          { label: 'Adobe – media cache (Premiere, After Effects)', hint: 'Bezpečné zmazať, keď Adobe appky nebežia. Vytvorí sa znova pri otvorení projektu.', path: 'C:\\Users\\Alex\\AppData\\Roaming\\Adobe\\Common\\Media Cache Files', size: 12.6 * GB, files: 800, id: 2, how: 'contents' },
          { label: 'Dočasné súbory', hint: 'Bezpečné zmazať. Súbory, ktoré sa práve používajú, Windows zmazať nedovolí, a to je v poriadku.', path: 'C:\\Users\\Alex\\AppData\\Local\\Temp', size: 6.3 * GB, files: 12000, id: 3, how: 'contents' },
          { label: 'Shader cache grafiky', hint: 'Hry si ju vytvoria znova. Prvé spustenie hry potom môže chvíľu sekať.', path: 'C:\\Users\\Alex\\AppData\\Local\\NVIDIA\\DXCache', size: 2.1 * GB, files: 400, id: 4, how: 'contents' },
          { label: 'Kôš', hint: 'Súbory, ktoré si už zmazal. Vysypaj Kôš.', path: 'C:\\$Recycle.Bin', size: 4.2 * GB, files: 50, id: 5, how: 'bin' },
          { label: 'Stará verzia Windowsu', hint: 'Maž cez Nastavenia → Systém → Úložisko → Dočasné súbory.', path: 'C:\\Windows.old', size: 22 * GB, files: 90000, id: 6, how: 'settings' },
        ];
  const mockApps = [
    { name: 'Clipchamp.Clipchamp', full: 'Clipchamp.Clipchamp_3.1_neutral__yxz', title: 'Clipchamp', about: 'Jednoduchý strihač videa od Microsoftu.', junk: true },
    { name: 'Microsoft.BingNews', full: 'Microsoft.BingNews_4_x64__8we', title: 'Správy', about: 'Microsoft Start, správy s reklamami.', junk: true },
    { name: 'Microsoft.MicrosoftSolitaireCollection', full: 'Microsoft.MicrosoftSolitaireCollection_4_x64__8we', title: 'Solitaire', about: 'Kartové hry s reklamami a predplatným.', junk: true },
    { name: 'Microsoft.MicrosoftOfficeHub', full: 'Microsoft.MicrosoftOfficeHub_18_x64__8we', title: 'Microsoft 365 (Office)', about: 'Len rozcestník a ponuka predplatného. Nainštalovaný Word ani Excel neodinštaluje.', junk: true },
    { name: 'Microsoft.BingWeather', full: 'Microsoft.BingWeather_4_x64__8we', title: 'Počasie', about: 'Ak počasie pozeráš v mobile, netreba.', junk: false },
    { name: 'Microsoft.YourPhone', full: 'Microsoft.YourPhone_1_x64__8we', title: 'Prepojenie s telefónom', about: 'Správy a fotky z mobilu na PC.', junk: false },
  ];
  const mockRuns = [
    { id: Date.now() - 864e5, exe: 'Cyberpunk2077.exe', label: 'Pred zmenou', seconds: 60, frames: 5400, avg_fps: 88.1, low1: 61.4, low01: 48.2, frametime: 11.35, cpu_busy: 4.2, gpu_busy: 11.1, bound: 'gpu', gpu: { name: 'NVIDIA GeForce RTX 4070 SUPER', temp_avg: 80, temp_max: 88, clock_avg: 2380, clock_max: 3105, load_avg: 99, power_avg: 205, power_limit: 220, thermal_share: 0.18, power_cap_share: 0.3, samples: 40 } },
  ];
  const T = (key, title, about, effect, category, admin, restart, on, icon = 'sparkle') => ({ key, title, about, effect, category, admin, restart, on, icon, available: true });
  const mockTweaks = [
    T('classic_menu', 'Klasické kontextové menu', 'Pravý klik hneď ukáže všetky možnosti, bez „Zobraziť ďalšie možnosti“.', 'big', 'w_explorer', false, 'explorer', false, 'menu'),
    T('file_ext', 'Zobrazovať prípony súborov', 'Uvidíš, či je súbor .jpg, .png alebo .exe.', 'big', 'w_explorer', false, 'explorer', false, 'file'),
    T('explorer_thispc', 'Prieskumník otvárať na Tento počítač', 'Namiesto stránky Domov uvidíš hneď disky.', 'small', 'w_explorer', false, '', false, 'pc'),
    T('taskbar_left', 'Panel úloh zarovnaný doľava', 'Štart a appky vľavo ako vo Windowse 10.', 'small', 'w_explorer', false, '', false, 'taskbar'),
    T('end_task', '„Ukončiť úlohu“ v paneli úloh', 'Pravý klik na appku v paneli úloh ju ukončí.', 'big', 'w_explorer', false, '', false, 'x'),
    T('start_web', 'Štart hľadá len v PC, nie na Bingu', 'Vyhľadávanie v Štarte je rýchlejšie a ukáže tvoje súbory a appky.', 'big', 'w_clean', false, 'explorer', false, 'search'),
    T('start_ads', 'Bez odporúčaní a reklám v Štarte', 'Štart prestane ponúkať „odporúčané“ appky.', 'small', 'w_clean', false, '', false, 'ad'),
    T('lock_ads', 'Bez tipov na zamknutej obrazovke', 'Ostane len tvoj obrázok a čas.', 'small', 'w_clean', false, '', false, 'lock'),
    T('setup_nags', 'Bez „Dokončite nastavenie zariadenia“', 'Žiadne ponuky OneDrive a Microsoft 365 po aktualizácii.', 'small', 'w_clean', false, '', false, 'bell'),
    T('widgets_off', 'Vypnúť Miniaplikácie (Widgets)', 'Zmizne panel so správami a počasím.', 'small', 'w_clean', true, 'explorer', false, 'layers'),
    T('ad_id', 'Vypnúť reklamné ID', 'Microsoft podľa neho cieli reklamy.', 'small', 'w_privacy', false, '', false, 'eye'),
    T('no_driver_updates', 'Windows Update nesmie meniť ovládače', 'Windows rád nainštaluje starší ovládač grafiky cez ten tvoj.', 'big', 'w_update', true, '', false, 'gpu'),
    T('no_auto_restart', 'Nereštartovať sám, keď som prihlásený', 'Žiadna stratená práca ani prerušený render cez noc.', 'big', 'w_update', true, '', false, 'restart'),
    T('fast_startup_off', 'Vypnúť Rýchle spustenie', 'Vypnutie PC potom znamená naozajstné vypnutie.', 'small', 'w_update', true, '', false, 'power'),
    T('clipboard_history', 'História schránky (Win + V)', 'Win + V ukáže posledné veci, ktoré si skopíroval.', 'big', 'w_features', false, '', false, 'clipboard'),
    T('storage_sense', 'Senzor úložiska', 'Windows sám maže dočasné súbory a starý Kôš.', 'small', 'w_features', false, '', true, 'broom'),

    T('startup_delay', 'Spúšťať appky po prihlásení hneď', 'Windows po prihlásení čaká niekoľko sekúnd, kým spustí appky zo spúšťania. Na SSD je to čakanie zbytočné, plocha je skôr použiteľná.', 'big', 'responsive', false, '', false, 'rocket'),
    T('menu_delay', 'Rýchlejšie podponuky', 'Windows čaká 0,4 s, kým otvorí podponuku. Takto stačí 0,1 s, menu pôsobia svižnejšie.', 'small', 'responsive', false, 'logoff', false, 'menu'),
    T('power_plan', 'Plán napájania Vysoký výkon', 'Procesor menej zaspáva a rýchlejšie reaguje. Na stolnom PC je rozdiel malý, ale merateľný v odozve. Na notebooku viac vybíja batériu.', 'small', 'responsive', false, '', false, 'bolt'),
    T('game_dvr', 'Vypnúť nahrávanie hier na pozadí', 'Xbox Game Bar vie stále nahrávať posledné minúty hry, čo zaberá grafiku. Ak klipy robíš inou appkou alebo vôbec, netreba to.', 'big', 'games', false, '', false, 'video'),
    T('game_mode', 'Herný režim', 'Počas hry Windows odkladá aktualizácie a dáva hre prednosť. Na dnešných PC neškodí, odporúčam mať zapnutý.', 'small', 'games', false, '', true, 'game'),
    T('hags', 'Hardvérové plánovanie GPU (HAGS)', 'Grafika si sama plánuje prácu, čo trochu znižuje oneskorenie. Na NVIDIA RTX 40 a novších ho potrebuje Frame Generation (DLSS 3 a 4).', 'small', 'games', true, 'reboot', false, 'gpu'),
    T('hvci_off', 'Vypnúť integritu pamäte (HVCI)', 'Bezpečnostná funkcia Windowsu, ktorá v hrách berie zhruba 5 % výkonu, na starších procesoroch aj viac. Vypnutím sa PC stáva o niečo zraniteľnejším.', 'big', 'games', true, 'reboot', false, 'shield'),
    T('mouse_accel', 'Vypnúť zrýchlenie myši', 'Kurzor sa posunie o rovnaký kus podľa toho, kam myšou pohneš, nie podľa toho, ako rýchlo.', 'small', 'games', false, 'logoff', false, 'mouse'),
    T('background_apps', 'Zakázať appkám zo Storu beh na pozadí', 'Appky z Microsoft Store nebudú bežať na pozadí a ich upozornenia môžu chodiť neskôr.', 'small', 'background', false, '', false, 'app'),
    T('edge_boost', 'Edge sa neprednačítava a po zavretí nebeží', 'Edge sa spúšťa už so štartom Windowsu a beží aj po zavretí okna.', 'small', 'background', true, '', false, 'globe'),
    T('tips', 'Vypnúť tipy a návrhy Windowsu', 'Žiadne „Tipy na používanie Windowsu“, návrhy v Štarte ani v Nastaveniach.', 'small', 'background', false, '', false, 'bell'),
    T('transparency', 'Vypnúť priehľadnosť', 'Pomôže na slabších PC a notebookoch s integrovanou grafikou.', 'small', 'looks', false, '', false, 'layers'),
    T('min_animate', 'Okná bez animácie pri minimalizovaní', 'Okná sa minimalizujú a obnovujú hneď.', 'small', 'looks', false, 'logoff', false, 'window'),
  ];

  const mockAuto = [
    A({ id: 'service:msi_central_service', kind: 'service', when: 'Po štarte Windowsu (oneskorene)', source: 'Služba MSI_Central_Service', name: 'MSI Central Service', command: 'C:\\Program Files (x86)\\MSI\\MSI Center\\MSI.CentralServer.exe', exe: 'msi.centralserver.exe', app: 'MSI Center / Mystic Light', verdict: 'rgb', about: 'Ovláda svetlá dosky a všetkého, čo je na nej zapojené (vetráky, AIO). Beží stále, aj keď ju práve nepoužívaš.', running: true, hidden: true, needs_admin: true, toggle: { t: 'service', name: 'MSI_Central_Service', on: 'delayed-auto' } }),
    A({ id: 'task:\\msi\\ledkeeper', kind: 'task', when: 'Pri prihlásení', source: 'Naplánovaná úloha \\MSI\\MSI Task Host - LEDKeeper', name: 'MSI Task Host - LEDKeeper', command: '"C:\\Program Files (x86)\\MSI\\MSI Center\\Mystic Light\\LEDKeeper2.exe"', exe: 'ledkeeper2.exe', app: 'MSI Center / Mystic Light', verdict: 'rgb', about: 'Ovláda svetlá dosky a všetkého, čo je na nej zapojené (vetráky, AIO).', hidden: true, needs_admin: true, toggle: { t: 'task', path: '\\MSI\\LEDKeeper' } }),
    A({ id: 'service:gvservice', kind: 'service', when: 'Pri štarte Windowsu', source: 'Služba GvService', name: 'GIGABYTE Control Center Service', command: '"C:\\Program Files\\GIGABYTE\\Control Center\\Lib\\GBT_VGA\\Service\\GvService.exe"', app: 'GIGABYTE Control Center', verdict: 'rgb', about: 'Ovláda svetlá a displej grafiky.', running: true, hidden: true, needs_admin: true, toggle: { t: 'service', name: 'GvService', on: 'auto' } }),
    A({ id: 'run:discord', kind: 'run', when: 'Pri prihlásení', source: 'Run (používateľ)', name: 'Discord', command: 'C:\\Users\\Alex\\AppData\\Local\\Discord\\Update.exe --processStart Discord.exe', exe: 'update.exe', app: 'Discord', verdict: 'optional', about: 'Spúšťa sa s Windowsom. Dá sa vypnúť v Discorde → Nastavenia → Nastavenia systému Windows.' }),
    A({ id: 'run:steam', kind: 'run', when: 'Pri prihlásení', source: 'Run (používateľ)', name: 'Steam', command: '"C:\\Program Files\\Steam\\steam.exe" -silent', exe: 'steam.exe', app: 'Steam', verdict: 'optional', about: 'Nemusí bežať, kým nehráš.', enabled: false }),
    A({ id: 'task:\\googleupdatertaskuser', kind: 'task', when: 'Pri prihlásení', source: 'Naplánovaná úloha \\GoogleUpdaterTaskUser{4F1C}', name: 'GoogleUpdaterTaskUser{4F1C}', command: 'C:\\Users\\Alex\\AppData\\Local\\Google\\GoogleUpdater\\updater.exe --wake', exe: 'updater.exe', app: 'Google aktualizácie', verdict: 'optional', about: 'Aktualizuje Chrome a iné appky od Googlu. Ak ho vypneš z automatického spúšťania, Chrome sa aktualizuje sám pri spustení.', hidden: true, toggle: { t: 'task', path: '\\GoogleUpdaterTaskUser{4F1C}' } }),
    A({ id: 'service:edgeupdate', kind: 'service', when: 'Po štarte Windowsu (oneskorene)', source: 'Služba edgeupdate', name: 'Microsoft Edge Update Service (edgeupdate)', command: '"C:\\Program Files (x86)\\Microsoft\\EdgeUpdate\\MicrosoftEdgeUpdate.exe" /svc', app: 'Microsoft Edge aktualizácie', verdict: 'optional', about: 'Aktualizuje Edge a WebView2. Môže štartovať ručne, keď ho Edge potrebuje.', running: false, hidden: true, needs_admin: true, toggle: { t: 'service', name: 'edgeupdate', on: 'delayed-auto' } }),
    A({ id: 'service:diagtrack', kind: 'service', when: 'Pri štarte Windowsu', source: 'Služba DiagTrack', name: 'Pripojené používateľské prostredie a telemetria', command: 'C:\\WINDOWS\\System32\\svchost.exe -k utcsvc -p', app: 'Telemetria Windowsu', verdict: 'optional', advice: 'off', about: 'Posiela Microsoftu diagnostické údaje. Windows funguje rovnako aj bez nej.', running: true, hidden: true, needs_admin: true, toggle: { t: 'service', name: 'DiagTrack', on: 'auto' } }),
    A({ id: 'service:spooler', kind: 'service', when: 'Pri štarte Windowsu', source: 'Služba Spooler', name: 'Zaraďovač tlače', command: 'C:\\WINDOWS\\System32\\spoolsv.exe', app: 'Tlač (Print Spooler)', verdict: 'optional', advice: 'maybe', about: 'Potrebná na tlač aj „Tlačiť do PDF“. Ak netlačíš, môže štartovať ručne.', running: true, hidden: true, needs_admin: true, toggle: { t: 'service', name: 'Spooler', on: 'auto' } }),
    A({ id: 'run:lghub', kind: 'run', when: 'Pri prihlásení', source: 'Run (používateľ)', name: 'LGHUB', command: '"C:\\Program Files\\LGHUB\\system_tray\\lghub_system_tray.exe" --minimized', exe: 'lghub_system_tray.exe', app: 'Logitech G HUB', verdict: 'peripheral', about: 'Nastavenia myši a klávesnice Logitech.' }),
    A({ id: 'folder:lookatthis', kind: 'folder', when: 'Pri prihlásení', source: 'Priečinok Po spustení (používateľ)', name: 'LookAtThis', command: 'C:\\Users\\Alex\\AppData\\Roaming\\Microsoft\\Windows\\Start Menu\\Programs\\Startup\\LookAtThis.lnk', app: 'LookAtThis', verdict: 'needed', about: 'Z rodiny WheresTheApp. Klipy z hier, beží nenápadne na pozadí.' }),
    A({ id: 'service:vgc', kind: 'service', when: 'Pri štarte Windowsu', source: 'Služba vgc', name: 'Vanguard', command: '"C:\\Program Files\\Riot Vanguard\\vgc.exe"', app: 'Riot Vanguard', verdict: 'needed', about: 'Anti-cheat pre Valorant a League of Legends.', running: true, hidden: true, needs_admin: true, toggle: { t: 'service', name: 'vgc', on: 'auto' } }),
    A({ id: 'task:\\vendor\\x', kind: 'task', when: 'Pravidelne na pozadí', source: 'Naplánovaná úloha \\Razer\\Cortex Check', name: 'Cortex Check', command: 'C:\\Program Files\\Razer\\Cortex\\check.exe', verdict: 'unknown', hidden: true, toggle: { t: 'task', path: '\\Razer\\Cortex Check' } }),
    A({ id: 'service:audiosrv', kind: 'service', when: 'Pri štarte Windowsu', source: 'Služba Audiosrv', name: 'Zvuk systému Windows', command: 'C:\\WINDOWS\\System32\\svchost.exe -k LocalServiceNetworkRestricted -p', verdict: 'system', about: 'Služba Windowsu. Nechaj tak.', running: true, hidden: true, needs_admin: true, toggle: { t: 'service', name: 'Audiosrv', on: 'auto' } }),
  ];
  const R = (n) => n * MB;
  function procs() {
    snapN++;
    const j = (x) => x * (0.9 + 0.2 * Math.random());
    const g = (key, name, verdict, about, list) => ({ key, name, verdict, about, procs: list, cpu: list.reduce((a, p) => a + p.cpu, 0), ram: list.reduce((a, p) => a + (p.ram || 0), 0), io_bps: list.reduce((a, p) => a + p.io_bps, 0) });
    const p = (pid, exe, ram, cpu = 0, io = 0, path = null) => ({ pid, exe, ram: R(ram), cpu: snapN > 1 ? j(cpu) : 0, io_bps: snapN > 1 ? j(io) : 0, path });
    return {
      cpu_count: 24, cpu_total: snapN > 1 ? j(14) : 0, ram_total: 64 * GB, ram_used: 23.4 * GB,
      groups: [
        g('exe:cyberpunk2077.exe', 'Cyberpunk2077', 'unknown', null, [p(30, 'Cyberpunk2077.exe', 7800, 22, 0, 'D:\\SteamLibrary\\steamapps\\common\\Cyberpunk 2077\\bin\\x64\\Cyberpunk2077.exe')]),
        g('exe:chrome.exe', 'chrome', 'unknown', null, [p(1, 'chrome.exe', 610, 3.1, 120000, 'C:\\Program Files\\Google\\Chrome\\Application\\chrome.exe'), p(2, 'chrome.exe', 380, 0.4), p(3, 'chrome.exe', 210, 0.2)]),
        g('windows', 'Windows', 'system', 'Súčasti Windowsu. Nechaj ich tak.', [p(4, 'dwm.exe', 240, 1.2), p(5, 'explorer.exe', 180, 0.3), { ...p(6, 'svchost.exe', 48, 0.5), services: ['Windows Update', 'Background Intelligent Transfer Service'] }]),
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
      if (window.MOCK_DELAY) await new Promise((r) => setTimeout(r, Math.random() * window.MOCK_DELAY));
      await new Promise((r) => setTimeout(r, 40));
      switch (cmd) {
        case 'storage_drives': return [
          { root: 'C:\\', label: 'Systém', fs: 'NTFS', total: 1863 * GB, free: 1150 * GB, removable: false },
          { root: 'D:\\', label: 'Projekty', fs: 'NTFS', total: 3726 * GB, free: 420 * GB, removable: false },
          { root: 'E:\\', label: 'USB', fs: 'exFAT', total: 128 * GB, free: 100 * GB, removable: true },
        ];
        case 'storage_start': scanStart = Date.now(); scanning = true; done = false; mockMethod = args.fast ? 'mft' : 'walk'; return null;
        case 'storage_cancel': scanning = false; return null;
        case 'storage_status': {
          if (!scanning && !done) return { phase: 'idle', root: null, files: 0, dirs: 0, bytes: 0, errors: 0, cloud_bytes: 0, elapsed_ms: 0 };
          const t = Date.now() - scanStart, f = Math.min(1, t / (window.MOCK_SCAN_MS ?? 2500));
          if (f >= 1) { scanning = false; done = true; }
          return { method: mockMethod, note: null, phase: done ? 'done' : 'scanning', root: 'C:\\', files: f < 0.3 && mockMethod === 'mft' ? 0 : Math.round(f * 431200), dirs: Math.round(f * 61000), bytes: f * nodes[0].size, errors: 14, cloud_bytes: 3.2 * GB, elapsed_ms: t };
        }
        case 'storage_node': { const n = nodes[args.id]; const crumbs = []; let c = args.id; while (c != null) { crumbs.unshift({ id: c, name: c === 0 ? 'C:' : nodes[c].name }); c = nodes[c].parent; } let o = null, k = args.id; while (k != null && !o) { o = nodes[k].tag; k = nodes[k].parent; } return { node: view(args.id, 2), path: pathOf(args.id), crumbs, owner: o }; }
        case 'storage_top_files': return [
          { path: 'C:\\pagefile.sys', size: 24 * GB }, { path: 'C:\\Users\\Alex\\Videos\\ShapeFX\\Klient\\export_final_v3.mov', size: 18.4 * GB },
          { path: 'C:\\hiberfil.sys', size: 12 * GB }, { path: 'C:\\Users\\Alex\\Downloads\\Win11_24H2.iso', size: 5.8 * GB },
          { path: 'C:\\Program Files\\Steam\\steamapps\\common\\Cyberpunk 2077\\archive\\pc\\content\\basegame_4_gamedata.archive', size: 4.1 * GB },
        ];
        case 'storage_caches': return mockCaches;
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
            { label: 'Dočasné súbory', hint: '', path: 'C:\\Users\\Alex\\AppData\\Local\\Temp', size: 6.3 * GB, files: 12000, how: 'contents' },
            { label: 'Adobe – media cache (Premiere, After Effects)', hint: '', path: '', size: 12.6 * GB, files: 800 },
            { label: 'Stiahnuté aktualizácie Windowsu', hint: '', path: '', size: 3.4 * GB, files: 200 },
            { label: 'Shader cache grafiky', hint: '', path: 'C:\\Users\\Alex\\AppData\\Local\\NVIDIA\\DXCache', size: 2.1 * GB, files: 400, how: 'contents' },
          ],
          installers: [
            { id: 0, path: 'C:\\Users\\Alex\\Downloads\\Win11_24H2_Slovak_x64.iso', size: 5.8 * GB, age_days: 210, reason: 'installer' },
            { id: 0, path: 'C:\\Users\\Alex\\Downloads\\DaVinci_Resolve_19.1_Windows.zip', size: 3.2 * GB, age_days: 95, reason: 'installer' },
          ],
        };
        case 'system_facts': return { onedrive_folders: ['Plocha', 'Dokumenty'], hvci: true, power_guid: '381b4222-f694-41f0-9685-ff5bb260df2e', power_name: 'Vyvážený', game_mode: true, hags: false, hibernation: true, hiberfil_bytes: 25.4 * GB, os: 'Windows 11 Home 24H2', build: 26100, has_battery: false };
        case 'pc_specs': return {
          cpu: 'AMD Ryzen 7 2700 Eight-Core Processor', threads: 16,
          gpus: [{ name: 'NVIDIA GeForce RTX 4070 SUPER', vram: 12 * GB, driver_date: '1-15-2026', driver_version: '32.0.15.9144' }],
          board: 'ASUS TUF GAMING B650-PLUS WIFI', ram_total: 32 * GB,
          ram_modules: [
            { slot: 'DIMM_A2', size_mb: 16384, speed: 4800, configured_speed: 4800, kind: 'DDR5', manufacturer: 'Kingston', part: 'KF560C36-16', rated_speed: 6000, channel: 'A' },
            { slot: 'DIMM_B2', size_mb: 16384, speed: 4800, configured_speed: 4800, kind: 'DDR5', manufacturer: 'Kingston', part: 'KF560C36-16', rated_speed: 6000, channel: 'B' },
          ],
          disks: ['Samsung SSD 990 PRO 2TB', 'Crucial P3 1TB'], os: 'Windows 11 Home 24H2',
        };
        case 'displays_list': return [
          { adapter: 'NVIDIA GeForce RTX 4070 SUPER', primary: true, width: 2560, height: 1440, hz: 60, max_hz: 165 },
          { adapter: 'NVIDIA GeForce RTX 4070 SUPER', primary: false, width: 1920, height: 1080, hz: 60, max_hz: 60 },
        ];
        case 'games_list': return [
          { name: 'Counter-Strike 2', source: 'Steam', dir: 'C:\\Program Files (x86)\\Steam\\steamapps\\common\\Counter-Strike Global Offensive', size: 35 * GB, appid: '730' },
          { name: 'Cyberpunk 2077', source: 'Steam', dir: 'D:\\SteamLibrary\\steamapps\\common\\Cyberpunk 2077', size: 92 * GB, appid: '1091500' },
          { name: 'Fortnite', source: 'Epic Games', dir: 'C:\\Program Files\\Epic Games\\Fortnite', size: 46 * GB, appid: 'Fortnite' },
          { name: 'VALORANT', source: 'Riot', dir: 'C:\\Riot Games\\VALORANT', size: 0, appid: null },
          { name: 'Minecraft: Java Edition', source: 'Minecraft', dir: 'C:\\Users\\Alex\\AppData\\Roaming\\.minecraft', size: 0, appid: null },
          { name: 'Stardew Valley', source: 'Steam', dir: 'C:\\Program Files (x86)\\Steam\\steamapps\\common\\Stardew Valley', size: 0.6 * GB, appid: '413150' },
        ];
        case 'bench_list': return mockRuns;
        case 'bench_delete': { const i = mockRuns.findIndex((r) => r.id === args.id); if (i >= 0) mockRuns.splice(i, 1); return null; }
        case 'bench_status': {
          if (mockBench.phase === 'running' && Date.now() > mockBench.started_ms + (mockBench.delay + mockBench.seconds) * 1000) {
            const r = { id: Date.now(), exe: mockBench.exe, label: mockBench.label, seconds: mockBench.seconds, frames: 9000, avg_fps: 148.3, low1: 97.2, low01: 71.5, frametime: 6.74, cpu_busy: 3.1, gpu_busy: 6.5, bound: 'gpu', gpu: { name: 'NVIDIA GeForce RTX 4070 SUPER', temp_avg: 71, temp_max: 74, clock_avg: 2610, clock_max: 3105, load_avg: 98, power_avg: 214, power_limit: 220, thermal_share: 0, power_cap_share: 0.4, samples: 40 } };
            mockRuns.push(r); mockBench.phase = 'done'; mockBench.result = r;
          }
          return { ...mockBench, live: mockBench.phase === 'running' ? { name: 'NVIDIA GeForce RTX 4070 SUPER', temp: 72, clock: 2600, clock_max: 3105, load: 97, power: 210, power_limit: 220, fan: 55, thermal: false, power_cap: true } : null };
        }
        case 'bench_start': Object.assign(mockBench, { phase: 'running', exe: args.exe, label: args.label, seconds: window.MOCK_BENCH_S || args.seconds, delay: window.MOCK_BENCH_D ?? args.delay, started_ms: Date.now(), result: null, error: null }); return null;
        case 'bench_cancel': mockBench.phase = 'error'; mockBench.error = 'Meranie zastavené.'; return null;
        case 'gpu_sample': return { name: 'NVIDIA GeForce RTX 4070 SUPER', temp: 41, clock: 210, clock_max: 3105, load: 3, power: 14, power_limit: 220, fan: 0, thermal: false, power_cap: false };
        case 'shader_caches': return [
          { label: 'NVIDIA DirectX', path: 'C:\\Users\\Alex\\AppData\\Local\\NVIDIA\\DXCache', size: 3.9 * GB },
          { label: 'Windows (DirectX)', path: 'C:\\Users\\Alex\\AppData\\Local\\D3DSCache', size: 0.4 * GB },
        ];
        case 'appx_list': return mockApps;
        case 'stability_report': return {
          days: 30, read_ok: true, whea_fatal: 2, whea_corrected: 7, disk_errors: 0, disk_resets: 0, gpu_driver: 'nvlddmkm',
          blue_screens: [{ time: Date.now() / 1000 - 3 * 86400, code: '0x00000124', name: 'WHEA_UNCORRECTABLE_ERROR', meaning: 'Hardvérová chyba, ktorú nebolo možné opraviť. Najčastejšie procesor: pretaktovanie, napätie alebo starý BIOS.' },
            { time: Date.now() / 1000 - 9 * 86400, code: '0x00000124', name: 'WHEA_UNCORRECTABLE_ERROR', meaning: '' }],
          sudden_offs: [Date.now() / 1000 - 12 * 86400], gpu_resets: [Date.now() / 1000 - 2 * 86400],
          apps: [{ app: 'Cyberpunk2077.exe', count: 4, hangs: 0, last: Date.now() / 1000 - 86400, module: 'nvwgf2umx.dll', hint: 'Ovládač grafiky NVIDIA' },
            { app: 'explorer.exe', count: 0, hangs: 2, last: Date.now() / 1000 - 5 * 86400, module: null, hint: null }],
        };
        case 'disks_list': return [
          { id: '0', name: 'Samsung SSD 990 PRO 2TB', media: 'SSD', bus: 'NVMe', health: 'Healthy', size: 2000 * GB, system: true },
          { id: '1', name: 'WD_BLACK SN850X 4000GB', media: 'SSD', bus: 'NVMe', health: 'Healthy', size: 4000 * GB, system: false },
        ];
        case 'deep_check': return {
          disks: [{ id: '0', wear: 3, temp: 41, temp_max: 68, hours: 5120, read_err: 0, write_err: 0, read_unc: 0, write_unc: 0 }, { id: '1', wear: 1, temp: 38, temp_max: 55, hours: 2100 }],
          boots: Array.from({ length: 14 }, (_, i) => ({ at: new Date(Date.now() - (14 - i) * 2 * 864e5).toISOString(), boot: 18000 + i * 2500, main: 11000 + i * 200, post: 7000 + i * 2300 })),
          restore: [{ at: new Date(Date.now() - 6 * 864e5).toISOString(), desc: 'Windows Update' }],
        };
        case 'restore_status': return { protection: true, last: mockRestore, error: null };
        case 'storage_growth': return { since: Date.now() / 1000 - 21 * 86400, total_before: 701 * GB, total_after: 743 * GB, items: [
          { path: 'C:\\Users\\Alex\\Videos\\ShapeFX\\CacheClip', before: 12 * GB, after: 41 * GB, delta: 29 * GB },
          { path: 'C:\\Program Files\\Steam\\steamapps\\common\\Baldurs Gate 3', before: 0, after: 128 * GB, delta: 128 * GB },
          { path: 'C:\\Users\\Alex\\Downloads', before: 52 * GB, after: 38 * GB, delta: -14 * GB },
        ] };
        case 'storage_clean': return args.items.map((it) => { const i = mockCaches.findIndex((c) => c.path === it.path); const f = i >= 0 ? mockCaches[i].size : 3 * GB; if (i >= 0) mockCaches.splice(i, 1); mockChanges.push({ at: Date.now() / 1000, id: (it.kind === 'cache' ? 'clean:' : it.kind === 'bin' ? 'bin' : 'trash:') + it.path, name: it.label, toggle: null, enabled: false, undone: false, detail: (it.kind === 'trash' ? 'Do Koša · ' : 'Uvoľnené ') + (f / GB).toFixed(1).replace('.', ',') + ' GB' }); return { ok: true, error: null, elevated: false, freed: f }; });
        case 'tweaks_list': return mockTweaks.map((t) => ({ ...t }));
        case 'save_card': return 'C:\\Users\\Alex\\Pictures\\WhatsInside\\WhatsInside-PC-1760900000.png';
        case 'procs_snapshot': return procs();
        case 'autostart_list': return mockAuto.map((x) => ({ ...x }));
        case 'changes_list': return mockChanges;
        case 'changes_undo': { const c = mockChanges[args.index]; c.undone = true; const it = mockAuto.find((x) => x.id === c.id); if (it) it.enabled = !c.enabled; const tw = c.toggle && mockTweaks.find((x) => 'tweak:' + x.key === c.id); if (tw) tw.on = !c.enabled; return null; }
        case 'actions_apply': return args.actions.map((a) => {
          if (a.op === 'restart_explorer' || a.op === 'launch' || a.op === 'enable_protection') return { ok: true, error: null, elevated: false };
          if (a.op === 'restore_point') { mockRestore = Date.now() / 1000; mockChanges.push({ at: Date.now() / 1000, id: 'restore', name: 'Bod obnovenia systému', toggle: null, enabled: true, undone: false, detail: 'Vytvorený pred zmenami' }); return { ok: true, error: null, elevated: true }; }
          if (a.op === 'uninstall') { const i = mockApps.findIndex((x) => x.full === a.full); if (i >= 0) mockApps.splice(i, 1); mockChanges.push({ at: Date.now() / 1000, id: 'appx:' + a.full.split('_')[0], name: a.name, toggle: null, enabled: false, undone: false }); return { ok: true, error: null, elevated: false }; }
          if (a.op === 'kill') { mockChanges.push({ at: Date.now() / 1000, id: 'kill:' + a.pid, name: a.exe, toggle: null, enabled: false, undone: false }); return { ok: true, error: null, elevated: false }; }
          if (a.toggle.t === 'tweak') { const t = mockTweaks.find((x) => x.key === a.toggle.key); if (t) t.on = a.enable; }
          const it = mockAuto.find((x) => x.id === a.id); if (it) it.enabled = a.enable;
          mockChanges.push({ at: Date.now() / 1000, id: a.id, name: a.name, toggle: a.toggle, enabled: a.enable, undone: false });
          return { ok: true, error: null, elevated: a.toggle.t === 'service' };
        });
        case 'update_check': return { current: '0.4.0', newer: true, installed: true, error: null, latest: { version: '0.4.1', url: '', sha256: '', notes: '### Nové\n- Rýchlejší scan\n- Opravy' } };
        case 'update_install': throw new Error('V náhľade sa aktualizovať nedá.');
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
