# WhatsInside

Lightweight náhrada výrobcovských appiek (MSI Center, GIGABYTE Control Center…) pre RGB a hardvér. Časť rodiny appiek **WheresTheApp**: nesmie zaťažovať systém.

> Pracovný názov. Dá sa kedykoľvek premenovať.

## Princíp: apply & exit
Appka nastaví farby, zapíše ich do hardvéru a **skončí**. Nebeží žiadna služba ani tray. Na pozadí beží len ak si zapneš živé efekty.

## Stav

| Verzia | Čo robí | Zapisuje do HW? |
|---|---|---|
| **v0.1** (teraz) | nájde podporované RGB radiče, grafiku a appky, ktoré by sa o RGB bili | **nie** |
| v0.2 | jedna statická farba na doske a grafike, apply & exit | áno |
| v0.3 | profily, per-LED, UI v štýle Čistá | áno |

## Podporovaný hardvér (zatiaľ)

| Zariadenie | Ako | Pozn. |
|---|---|---|
| MSI MAG Z790 TOMAHAWK WIFI (`1462:7D91`) | USB HID, Mystic Light 185 B | ovláda aj ARGB headery (AIO, vetráky cez hub) |
| GIGABYTE AORUS RTX 5080 MASTER / ICE | I2C cez NVAPI, adresa `0x71` | svetlá áno, LCD displej zatiaľ nie |

Bez kernel drivera. SMBus/RAM zámerne nepodporujeme (riziko poškodenia SPD).

## Bezpečnostné pravidlá
- iba známe ID zariadení a zdokumentované príkazy
- nikdy žiadne príkazy na prepis firmvéru
- pred prvým zápisom zálohovať aktuálny stav radiča
- nezapisovať, kým beží iná RGB appka

Ak by sa RGB radič dosky niekedy zasekol: vypni PC, vypni zdroj, podrž power ~30 s.

## Spustenie
Stiahni `whatsinside.exe` z [Releases](../../releases) alebo z poslednej úspešnej Actions zostavy a spusti. Report sa uloží aj do `whatsinside-report.json` vedľa exe.

```
whatsinside.exe             report + JSON súbor, čaká na Enter
whatsinside.exe --json      len JSON
whatsinside.exe --no-pause  report bez čakania
```

## Build
```
cargo build --release
```
Každý push na `main` zostaví exe cez GitHub Actions. Tag `v*` vytvorí Release.

## Licencia
GPL-2.0-only. Protokoly a ID zariadení vychádzajú z projektu [OpenRGB](https://gitlab.com/CalcProgrammer1/OpenRGB).
