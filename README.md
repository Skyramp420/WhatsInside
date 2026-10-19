# WhatsInside

Ľahký prehľad o tom, čo je v tvojom PC. Časť rodiny appiek **WheresTheApp** (téma Čistá).

Nič nemení ani nemaže. Keď ju zavrieš, nebeží nič: žiadna služba, žiadny tray.

## Čo vie (v0.2)

- **Úložisko**: paralelný scan disku, treemap s preklikom do priečinkov, najväčšie súbory, cache, ktorú sa dá bezpečne uvoľniť (DaVinci Resolve, Adobe, shader cache, Temp, aktualizácie Windowsu…). Súbory z OneDrive, ktoré sú len online, nerátam.
- **Procesy**: živý prehľad zoskupený podľa appky (napr. 4 procesy GIGABYTE Control Center ako jedna položka), CPU / RAM / disk a vysvetlenie pri známych appkách.
- **Po spustení**: čo sa spúšťa s Windowsom, vrátane služieb tretích strán (tak sa spúšťajú MSI Center aj GCC).
- **Hardvér**: podporované RGB radiče a grafika, plus koľko RAM ťa stoja výrobcovské RGB appky.

## Plán

| Verzia | Čo |
|---|---|
| v0.3 | vypínanie z autostartu (cez rovnaký prepínač ako Správca úloh), mazanie cache do Koša |
| ďalej | scan cez MFT (rýchlosť ako WizTree), diagnostika (teploty, throttling), ovládanie RGB |

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
