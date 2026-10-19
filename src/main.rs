//! WhatsInside v0.1 – read-only scan.
//!
//! Finds the RGB controllers we plan to support and the vendor apps that
//! would fight us over them. It never writes to hardware.
//!
//! Usage:
//!   whatsinside.exe            print report, save whatsinside-report.json, wait for Enter
//!   whatsinside.exe --json     print JSON only (no pause)
//!   whatsinside.exe --no-pause print report without waiting

#![cfg_attr(not(windows), allow(dead_code))]

mod known;
mod report;
mod scan;

use std::io::{self, BufRead, Write};

fn main() {
    let args: Vec<String> = std::env::args().skip(1).collect();
    let json_only = args.iter().any(|a| a == "--json");
    let no_pause = json_only || args.iter().any(|a| a == "--no-pause");

    let report = scan::run();
    let json = serde_json::to_string_pretty(&report).unwrap_or_default();

    if json_only {
        println!("{json}");
        return;
    }

    print!("{}", report.render());

    // Save next to the exe so it's easy to send over for debugging.
    let path = std::env::current_exe()
        .ok()
        .and_then(|p| p.parent().map(|d| d.join("whatsinside-report.json")))
        .unwrap_or_else(|| "whatsinside-report.json".into());
    match std::fs::write(&path, &json) {
        Ok(_) => println!("\nReport uložený: {}", path.display()),
        Err(e) => println!("\nReport sa nepodarilo uložiť: {e}"),
    }

    if !no_pause {
        print!("\nStlač Enter pre ukončenie…");
        let _ = io::stdout().flush();
        let _ = io::stdin().lock().lines().next();
    }
}
