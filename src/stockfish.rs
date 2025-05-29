use std::io;
use std::process::Command;
use std::sync::{mpsc, mpsc::Receiver};

use interactive_process::InteractiveProcess;

use crate::engine_eval::{EngineEval, EvalType};
use crate::engine_output::EngineOutput;

pub struct Stockfish {
    interactive_process: InteractiveProcess,
    receiver: Receiver<String>,
}

impl Stockfish {
    pub fn new(stockfish_path: &str) -> io::Result<Stockfish> {
        let mut command = Command::new(stockfish_path);

        let (tx, rx) = mpsc::channel();

        let proc = InteractiveProcess::new(&mut command, move |line| {
            let line = line.unwrap();
            let send_result = tx.send(line);
            if send_result.is_err() {
                println!("receiving end of mpsc channel disconnected")
            }
        })?;

        rx.recv().expect("should be able to read first line from rx");

        Ok(Stockfish { 
            interactive_process: proc,
            receiver: rx,
        })
    }

    pub fn setup_for_new_game(&mut self, start_pos: &str) -> io::Result<()> {
        self.ensure_ready()?;
        self.send("ucinewgame")?;

        if start_pos == "s" {
            self.send("position startpos")?;
        } else {
            let msg = String::from("position fen ") + &start_pos;
            self.send(&msg)?;
        }

        Ok(())
    }

    pub fn ensure_ready(&mut self) -> io::Result<()> {
        self.send("isready")?;
        while self.read_line() != "readyok" {}
        Ok(())
    }

    pub fn send(&mut self, data: &str) -> io::Result<()> {
        self.interactive_process.send(data)?;
        Ok(())
    }

    pub fn get_fen(&mut self) -> io::Result<String> {
        self.send("d")?;
        loop {
            let line = self.read_line();
            let mut segments= line.split(" ");
            if segments.next().unwrap() == "Fen:" {
                let fen = segments.collect::<Vec<&str>>().join(" ");

                // Keep reading lines until reached "Checkers", which is in the last line
                while !self.read_line().contains("Checkers") {}

                return Ok(fen);
            }
        }
    }

    pub fn play_move(&mut self, move_str: &str) -> io::Result<()> {
        let fen = self.get_fen()?;
        let data = format!("position fen {fen} moves {move_str}");
        self.send(&data)?;
        Ok(())
    }

    pub fn go_to_depth(&mut self, depth: u8) -> io::Result<EngineOutput> {
        let message = String::from("go depth ") + &depth.to_string();
        self.send(&message)?;
        self.get_engine_output()
    }

    pub fn get_engine_output(&mut self) -> io::Result<EngineOutput> {
        let fen = self.get_fen()?;
        let color_multiplier = if fen.contains("w") {1} else {-1};
        // Stockfish shows advantage relative to current player. This function will instead
        // use positive to represent advantage white, and negative for advantage black.

        let mut previous_line: Option<String> = None;

        loop {
            let line = self.read_line();
            let mut segments = line.split(" ");
            let first_segment = segments.next().expect("should be able to get first segment");
            if first_segment != "bestmove" {
                previous_line = Some(line);
                continue;
            }
            
            let previous_line = previous_line.unwrap();
            let previous_segments: Vec<&str> = previous_line.split(" ").collect();

            let mut score_type = None;
            let mut score_value: Option<i32> = None;

            for (i, segment) in previous_segments.iter().enumerate() {
                if *segment == "score" {
                    score_type = Some(previous_segments[i + 1]);
                    score_value = Some(
                        previous_segments[i + 2].parse::<i32>().expect("should be able to parse score_value from stockfish info line in output")
                            * color_multiplier);
                    break;
                }
            }

            let score_type = EvalType::from_str(score_type.unwrap());
            let eval = EngineEval::new(score_type, score_value.unwrap());

            let best_move = segments.next().expect("should be able to get second segment")
                .to_owned();

            let output = EngineOutput::new(eval, best_move);
            return Ok(output);
        }
    }

    pub fn print_board(&mut self) -> io::Result<()> {
        self.send("d")?;

        let mut lines: Vec<String> = Vec::with_capacity(20);

        loop {
            let line = self.read_line();
            if line.is_empty() {
                continue;
            }

            let first_segment = line.split(" ").next().expect("non-empty line should have segment");
            if first_segment == "Fen:" {
                break
            } else {
                lines.push(line);
            }
        }
        println!("{}", lines.join("\n"));
        Ok(())
    }

    /* Accessory Methods */
    pub fn set_hash(&mut self, hash: u32) -> io::Result<()> {
        self.send(&format!("setoption name Hash value {hash}"))
    }

    /* Private Methods */
    fn read_line(&mut self) -> String {
        let line = self.receiver.recv().expect("should be able to read from receiver");
        line
    }
}