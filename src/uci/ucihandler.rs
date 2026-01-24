use crate::uci::enginestate::EngineState;
use std::io::{self, Read, Write};

pub struct UciHandler {
    out: std::io::StdoutLock<'static>,
    state: EngineState,
}

impl UciHandler {
    pub fn send(&mut self, s: &str) {
        writeln!(self.out, "{s}").unwrap();
        self.out.flush().unwrap();
    }

    pub fn handle_line(&mut self, line: &str) {
        let line = line.trim();

        match line {
            "uci" => {
                self.send("id RustEngine 1.0");
                self.send("id Mario Orsolic, Emil Sitka, Julien Kriebel, Noah Schuller");
                self.send("uciok");
            }
            "isready" => self.send("readyok"),
            // "ucinewgame" => engine.newgame(),  not implemented yet
            // "go..." => engine.startsearch(line),
            // "stop" => engine.stopsearch(),
            // "quit" => engine.quit(),
            // "setoption..." => engine.setoption(line),
            // "position..." => engine.setposition(line),
            _ => {}
        }
    }
}
