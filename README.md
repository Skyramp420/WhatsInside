# WhatsInside

Ľahký prehľad o tom, čo je v tvojom PC. Časť rodiny appiek **WheresTheApp** (téma Čistá).

Nič nemení ani nemaže. Keď ju zavrieš, nebeží nič: žiadna služba, žiadny tray.

## Čo vie (v0.3)

- **Pomocník**: skontroluje disky, cache, inštalačky v Stiahnutých, hibernáciu, pamäť, RGB appky, automatické spúšťanie, plán napájania, Herný režim a HAGS. Ku každej veci povie prečo a ako to spraviť, sám nič nemení.
- **Úložisko**: paralelný scan, treemap s plynulým priblížením pri vstupe do priečinka, vyhľadávanie, označenie hier (Steam, Epic, Xbox, GOG, Riot, Ubisoft, EA) a programov, najväčšie súbory, staré a zabudnuté súbory, cache na uvoľnenie.
- **Procesy**: živý prehľad zoskupený podľa appky s vysvetlením.
- **Po spustení**: Run kľúče, priečinok Po spustení a služby tretích strán.
- **Hardvér**: karta PC na zdieľanie (Discord): zistí procesor, grafiku aj s VRAM, dosku, RAM (typ, rýchlosť, výrobca zo SMBIOS), disky a systém; ostatné doplníš sám. Kopírovanie obrázka do schránky, uloženie PNG alebo text.

## Plán

| Verzia | Čo |
|---|---|
| ďalej | vypínanie z autostartu, mazanie cache do Koša, scan cez MFT, diagnostika (teploty, throttling), ovládanie RGB |

## Podporovaný hardvér (RGB, zatiaľ len detekcia)

| Zariadenie | Ako |
|---|---|
| MSI MAG Z790 TOMAHAWK WIFI (`1462:7D91`) | USB HID, Mystic Light 185 B |
| GIGABYTE AORUS RTX 5080 MASTER / ICE | I2C cez NVAPI, `0x71` (LCD displej zatiaľ nie) |

## Vývoj

```
npx @tauri-apps/cli@2 dev      # spustí appku
cd src-tauri && cargo test     # testy
```
UI je čisté HTML/CSS/JS v `ui/` bez build kroku. V obyčajnom prehliadači sa otvorí s testovacími dátami (`ui/mock.js`).

Každý push na `main` zostaví cez GitHub Actions `whatsinside.exe` (prenosná verzia) aj inštalátor. Tag `v*` vytvorí Release.

## Licencia
GPL-2.0-only. ID zariadení vychádzajú z projektu [OpenRGB](https://gitlab.com/CalcProgrammer1/OpenRGB). Písmo Inter (SIL OFL).
