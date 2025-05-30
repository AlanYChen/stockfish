use std::io;
use std::process::Command;
use std::sync::{mpsc, mpsc::Receiver};

use interactive_process::InteractiveProcess;

use crate::engine_eval::{EngineEval, EvalType};
use crate::engine_output::EngineOutput;

/// Wraps an InteractiveProcess in an interface for interacting with the
/// Stockfish process.
pub struct Stockfish {
    interactive_process: InteractiveProcess,
    receiver: Receiver<String>,
    depth: u32,
    version: Option<String>,
}

impl Stockfish {

    /// Given the path to the stockfish binary executable, this function
    /// initiates an InteractiveProcess for the executable, and returns an instance
    /// of the Stockfish wrapper class.
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

        let first_line = rx.recv().expect("stockfish process should have outputted a first line");
        let version = first_line.split(" ").nth(1).map(|s| s.to_string());

        Ok(Stockfish { 
            interactive_process: proc,
            receiver: rx,
            depth: 15,
            version
        })
    }

    /// Prepares the Stockfish process for a new game. Should be called
    /// to indicate to the engine that the next position it will be evaluating
    /// will be from a different game.
    pub fn setup_for_new_game(&mut self) -> io::Result<()> {
        self.ensure_ready()?;
        self.send("ucinewgame")?;
        Ok(())
    }

    /// Changes the current chess position in which Stockfish is currently playing.
    /// The argument to be passed is a string in FEN (Forsyth-Edwards Notation).
    pub fn set_fen_position(&mut self, fen: &str) -> io::Result<()> {
        let msg = String::from("position fen ") + fen;
        self.send(&msg)?;
        Ok(())
    }

    /// Reverts the current chess position to the default starting position.
    /// This is the same as calling set_fen_position() with the default
    /// fen. (rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1)
    pub fn reset_position(&mut self) -> io::Result<()> {
        self.send("position startpos")?;
        Ok(())
    }

    /// This should be called to ensure that the Stockfish process is ready
    /// to receive its inputs. Sends the UCI command "isready" to Stockfish
    /// and blocks until it sends back "readyok".
    pub fn ensure_ready(&mut self) -> io::Result<()> {
        self.send("isready")?;
        while self.read_line() != "readyok" {}
        Ok(())
    }

    /// Returns the FEN (Forsyth-Edwards Notation) of the current chess
    /// position in which Stockfish is playing.
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

    /// Plays a move on the current chess position in which Stockfish is playing.
    /// This function only updates the board, and does nothing else.
    pub fn play_move(&mut self, move_str: &str) -> io::Result<()> {
        let fen = self.get_fen()?;
        let data = format!("position fen {fen} moves {move_str}");
        self.send(&data)?;
        Ok(())
    }

    /// Makes Stockfish calculate to the depth that has been set. (The default
    /// depth is 15.) Once Stockfish has finished its calculations, this function
    /// will return an `EngineOutput` to describe the result of its calculations.
    pub fn go(&mut self) -> io::Result<EngineOutput> {
        let message = String::from("go depth ") + &self.depth.to_string();
        self.send(&message)?;
        self.get_engine_output()
    }

    /// Configures the depth to which Stockfish will calculate. When `go()` is
    /// called, this depth will be used to determine how deeply Stockfish will
    /// calculate.
    pub fn set_depth(&mut self, depth: u32) {
        self.depth = depth;
    }

    /// 
    pub fn get_engine_output(&mut self) -> io::Result<EngineOutput> {
        let fen = self.get_fen()?;

        // The output from stockfish normally displays the value of the evaluation score
        // relative to the player with the current move. Use a multiplier to flip it such that
        // the score is not relative to the player with the current move.
        let color_multiplier = if fen.contains("w") {1} else {-1};

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

            let score_type = EvalType::from_descriptor(score_type.unwrap());
            let eval = EngineEval::new(score_type, score_value.unwrap());

            let best_move = segments.next().expect("should be able to get second segment")
                .to_owned();

            let output = EngineOutput::new(eval, best_move);
            return Ok(output);
        }
    }

    pub fn get_board_display(&mut self) -> io::Result<String> {
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
        Ok(lines.join("\n"))
    }
    pub fn print_board(&mut self) -> io::Result<()> {
        let board_display = self.get_board_display()?;
        println!("{board_display}");
        Ok(())
    }

    /* Accessory Methods */

    pub fn set_hash(&mut self, hash: u32) -> io::Result<()> {
        self.send(&format!("setoption name Hash value {hash}"))
    }
    pub fn set_threads(&mut self, threads: u32) -> io::Result<()> {
        self.send(&format!("setoption name Threads value {threads}"))
    }

    /// Returns 
    pub fn get_version(&self) -> &Option<String> {
        &self.version
    }

    /* Private Methods */
    fn send(&mut self, data: &str) -> io::Result<()> {
        self.interactive_process.send(data)?;
        Ok(())
    }
    fn read_line(&mut self) -> String {
        self.receiver.recv().expect("should be able to read from receiver")
    }
}