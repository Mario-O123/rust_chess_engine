use crate::uci::enginestate::EngineState;
use std::io::{self, Read, Write};

pub struct UciHandler {
    out: io::Stdout,
    state: EngineState,
}

impl UciHandler {
    pub fn new(state: EngineState) -> Self {
        Self {
            out: io::stdout(),
            state,
        }
    }

    fn send(&mut self, s: &str) {
        // WICHTIG: stdout nur für UCI-Antworten verwenden
        // Debug -> eprintln!()
        writeln!(self.out, "{s}").unwrap();
        self.out.flush().unwrap();
    }

    pub fn run(&mut self) {
        let stdin = io::stdin();
        for line in stdin.lock().lines() {
            match line {
                Ok(l) => self.handle_line(&l),
                Err(_) => break,
            }
        }
    }

    fn handle_line(&mut self, line: &str) {
        let line = line.trim();
        if line.is_empty() {
            return;
        }

        // eprintln!("[uci] <- {line}"); // Debug nur auf stderr

        match line {
            "uci" => {
                self.send("id name RustEngine 1.0");
                self.send("id author Mario Orsolic, Emil Sitka, Julien Kriebel, Noah Schuller");
                self.send("uciok");
            }
            "isready" => self.send("readyok"),
            "ucinewgame" => {
                // Optional, aber nett:
                // TODO: self.state.new_game();
            }
            "quit" => {
                // Optional, aber empfehlenswert
                // TODO: cleanup falls nötig
                std::process::exit(0);
            }
            _ => {
                if line.starts_with("position ") {
                    self.handle_position(line);
                } else if line.starts_with("go ") || line == "go" {
                    self.handle_go(line);
                } else if line.starts_with("setoption ") {
                    // Kannst du komplett ignorieren für Minimal-Setup
                    // (dein Lichess-Bot sendet es aktuell nicht)
                } else if line == "stop" {
                    // Kannst du für Minimal-Setup ignorieren
                } else {
                    // Unbekanntes Kommando ignorieren
                }
            }
        }
    }

    fn handle_position(&mut self, line: &str) {
        // Erwartet vom Bot: "position startpos moves e2e4 e7e5 ..."
        // Minimal: nur startpos + moves unterstützen

        let mut parts = line.split_whitespace();
        let _position_kw = parts.next(); // "position"

        match parts.next() {
            Some("startpos") => {
                // TODO: Startstellung setzen
                // Beispiel:
                // self.state.set_startpos();
            }
            Some("fen") => {
                // Optional, kannst du erstmal NICHT implementieren
                // Wenn du willst: FEN aus den nächsten 6 Feldern zusammenbauen bis "moves"
                // Für Minimal-Setup reicht startpos.
                eprintln!("[uci] fen not supported yet");
                return;
            }
            _ => return,
        }

        // Wenn "moves" kommt, alles danach sind UCI-Züge
        let mut saw_moves = false;
        for tok in parts {
            if tok == "moves" {
                saw_moves = true;
                continue;
            }
            if !saw_moves {
                continue;
            }

            // TODO: UCI-Zug anwenden (tok ist z.B. "e2e4" oder "e7e8q")
            // Beispiel:
            // if let Err(e) = self.state.apply_uci(tok) { eprintln!("bad move {tok}: {e}"); }
            // Für Minimal: wenn apply fehlschlägt -> ignorieren oder abbrechen.
        }
    }

    fn handle_go(&mut self, line: &str) {
        // Erwartet vom Bot meist: "go movetime 1000"
        // Minimal: movetime parsen (oder default), dann bestmove ausgeben.

        let movetime_ms = parse_movetime_ms(line).unwrap_or(1000);

        // TODO: hier musst du deinen Movefinder aufrufen
        // Er soll einen UCI-Move zurückgeben (z.B. "g1f3" oder "e7e8q")
        //
        // Beispiel-API:
        // let best = self.state.search_bestmove(movetime_ms);
        //
        // Als Notnagel: irgendeinen legalen Zug wählen (Random/first legal).
        let bestmove = {
            // TODO: ersetzen!
            // self.state.bestmove(movetime_ms)
            // .unwrap_or_else(|| "0000".to_string())
            "0000".to_string()
        };

        self.send(&format!("bestmove {bestmove}"));
    }
}

fn parse_movetime_ms(line: &str) -> Option<u64> {
    // sehr simpel: sucht "movetime <n>"
    let mut it = line.split_whitespace().peekable();
    while let Some(tok) = it.next() {
        if tok == "movetime" {
            return it.next()?.parse::<u64>().ok();
        }
    }
    None
}
