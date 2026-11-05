# WhatsInside

Ľahký prehľad o tom, čo je v tvojom PC. Časť rodiny appiek **WheresTheApp** (téma Čistá).

Nič nemení bez opýtania, každá zmena sa dá vrátiť. Keď ju zavrieš, nebeží nič: žiadna služba, žiadny tray.

**Stiahnuť:** [najnovší inštalátor](https://github.com/Skyramp420/WhatsInside/releases/latest). Ďalšie verzie si appka stiahne sama.

## Čo vie (v0.8)

- **Pomocník**: jedna obrazovka so všetkým, čo stojí za pozornosť: disky, cache, staré inštalačky, pamäť, automatické spúšťanie a nastavenie hardvéru.
- **Výkon**: kontrola hardvéru, ktorá na mnohých PC odhalí najväčšie straty: RAM bez zapnutého XMP/EXPO, jeden kanál pamäte, monitor nastavený na 60 Hz, kábel monitora v základnej doske namiesto grafiky, starý ovládač grafiky. K tomu úpravy Windowsu, ktoré naozaj niečo robia (oneskorenie po prihlásení, nahrávanie hier na pozadí, HAGS, integrita pamäte, plán napájania…), každá s prepínačom späť. A zoznam obľúbených „optimalizácií“, ktoré zámerne nerobíme, s vysvetlením prečo.
- **Hry**: meranie FPS v akejkoľvek hre (priemer, 1 % a 0,1 % lows) cez Intel PresentMon, porovnanie pred a po zmene a odhad, či hru brzdí procesor alebo grafika. Počas merania sleduje teplotu a takty grafiky (NVIDIA) a odhalí prehrievanie. Režim „Idem hrať“ zavrie vybrané appky a po hre ich spustí späť. Nájde nainštalované hry (Steam, Epic, Riot, Battle.net, Minecraft) a ku známym poradí DLSS/FSR, Reflex a nastavenia podľa tvojej grafiky. Ukáže aj veľkosť shader cache.
- **Ladenie**: rady k XMP/EXPO, undervoltu a pretaktovaniu podľa tvojho procesora a grafiky (bez zásahu do taktov) a kontrola stability po zmene: modré obrazovky, WHEA chyby, resety grafiky a FPS pred a po.
- **Windows**: úpravy Windowsu 11 s prepínačom späť (klasické kontextové menu, Štart bez Bingu a reklám, prípony súborov, „Ukončiť úlohu“, Windows Update bez prepisovania ovládačov a bez reštartu bez opýtania, história schránky…), kontrola OneDrive priečinkov a odinštalovanie predinštalovaných appiek.
- **Úložisko**: rýchly scan cez MFT (ako WizTree) alebo paralelný scan, mazanie cache a súborov do Koša, porovnanie s predchádzajúcim skenom, prehľadná mapa s ikonami a náhľadom obsahu, vyhľadávanie, označenie hier (Steam, Epic, Xbox, GOG, Riot, Ubisoft, EA, Battle.net) a programov, najväčšie súbory, staré súbory, cache na uvoľnenie.
- **Procesy**: živý prehľad zoskupený podľa appky s vysvetlením, služby bežiace v svchost, značka „So systémom“, ukončenie procesu a 30-sekundové meranie, čo pracuje, keď nič nerobíš.
- **Po spustení**: Run kľúče, priečinok Po spustení, naplánované úlohy, služby, RunOnce a Winlogon, aj to, čo Správca úloh neukáže. Vypínanie jedným klikom, pri službách s jediným oknom UAC.
- **Hardvér**: komponenty PC a zberateľská karta na zdieľanie (Discord).
- **Stabilita**: modré obrazovky, hardvérové chyby, pády appiek a resety grafiky z denníka Windowsu v ľudskej reči, zdravie SSD, čas štartu a appky, služby a ovládače, ktoré ho zdržiavajú.
- **História zmien**: všetko, čo WhatsInside kedy zmenil, po dňoch, s tlačidlom Vrátiť.
- **Aktualizácie**: z GitHub Releases, overené cez SHA-256.

Čo nerobí: nič nemaže bez potvrdenia, nemení nič na pozadí, nemá službu ani ikonu v tray.

## Plán

| Kedy | Čo |
|---|---|
| ďalej | angličtina, podpísané exe, teploty procesora, samostatná záložka na ovládanie svetiel |

## Svetlá (RGB)

Zatiaľ len rozpoznanie zariadení a výrobcovských appiek (MSI Center, GIGABYTE Control Center, Armoury Crate, iCUE, SignalRGB…). Ovládanie bude mať vlastnú záložku, ID zariadení vychádzajú z [OpenRGB](https://gitlab.com/CalcProgrammer1/OpenRGB).

## Vývoj

```
npx @tauri-apps/cli@2 dev      # spustí appku
cd src-tauri && cargo test     # testy
```
UI je čisté HTML/CSS/JS v `ui/` bez build kroku. V obyčajnom prehliadači sa otvorí s testovacími dátami (`ui/mock.js`).

Každý push na `main` zostaví cez GitHub Actions `whatsinside.exe` (prenosná verzia) aj inštalátor.

Vydanie: zvýš verziu v `src-tauri/Cargo.toml` a `src-tauri/tauri.conf.json`, napíš `notes/vX.Y.Z.md` a pushni tag `vX.Y.Z`. CI vytvorí Release s inštalátorom a `latest.json`, z ktorého si appky aktualizáciu stiahnu.

## Licencia
GPL-2.0-only. Písmo Inter (SIL OFL). Na meranie FPS je pribalený [Intel PresentMon](https://github.com/GameTechDev/PresentMon) (MIT), licencie sú v `ui/third-party.txt`.
