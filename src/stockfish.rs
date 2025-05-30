use std::io;
use std::process::Command;
use std::sync::{mpsc, mpsc::Receiver};
use std::time::Duration;

use interactive_process::InteractiveProcess;

use crate::engine_eval::{EngineEval, EvalType};
use crate::engine_output::EngineOutput;

/// Wraps an `InteractiveProcess` in an interface for interacting with the
/// stockfish process.
pub struct Stockfish {
    interactive_process: InteractiveProcess,
    receiver: Receiver<String>,
    depth: u32,
    version: Option<String>,
}

impl Stockfish {

    /// Given the path to the stockfish binary executable, this function
    /// initiates an `InteractiveProcess` for the executable, and returns an instance
    /// of the `Stockfish` wrapper class.
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

    /// Prepares the stockfish process for a new game. Should be called
    /// to indicate to the engine that the next position it will be evaluating
    /// will be from a different game.
    pub fn setup_for_new_game(&mut self) -> io::Result<()> {
        self.ensure_ready()?;
        self.send("ucinewgame")?;
        Ok(())
    }

    /// Changes the current chess position in which stockfish is currently playing.
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

    /// This should be called to ensure that the stockfish process is ready
    /// to receive its inputs. Sends the UCI command "isready" to stockfish
    /// and blocks until it sends back "readyok".
    pub fn ensure_ready(&mut self) -> io::Result<()> {
        self.send("isready")?;
        while self.read_line() != "readyok" {}
        Ok(())
    }

    /// Returns the FEN (Forsyth-Edwards Notation) of the current chess
    /// position in which stockfish is playing.
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

    /// Plays a move on the current chess position in which stockfish is playing.
    /// This function only updates the board, and does nothing else.
    pub fn play_move(&mut self, move_str: &str) -> io::Result<()> {
        let fen = self.get_fen()?;
        let data = format!("position fen {fen} moves {move_str}");
        self.send(&data)?;
        Ok(())
    }

    /// Makes tockfish calculate to the depth that has been set. (The default
    /// depth is 15.) Once tockfish has finished its calculations, this function
    /// will return an `EngineOutput` to describe the result of its calculations.
    pub fn go(&mut self) -> io::Result<EngineOutput> {
        let message = String::from("go depth ") + &self.depth.to_string();
        self.send(&message)?;
        self.get_engine_output()
    }

    /// Makes stockfish calculate for a specified amount of time. Blocks the calling thread
    /// for the duration of the specified calculation time.
    pub fn go_for(&mut self, calculation_time: Duration) -> io::Result<EngineOutput> {
        self.send("go")?;
        std::thread::sleep(calculation_time);
        self.send("stop")?;
        self.get_engine_output()
    }

    /// Makes stockfish calculate for a variable time based on the times given as parameters.
    /// If for example stockfish was analyzing from the white side and `white_time` was low
    /// (say, only 10 seconds), then stockfish will use less calculation time.
    /// The parameters are given in milliseconds.
    pub fn go_based_on_times(&mut self, white_time: Option<u32>, black_time: Option<u32>) -> io::Result<EngineOutput> {
        let mut message = String::from("go");
        if let Some(time) = white_time {
            message += &format!(" wtime {time}");
        }
        if let Some(time) = black_time {
            message += &format!(" btime {time}");
        }

        self.send(&message)?;
        self.get_engine_output()
    }

    /// Configures the depth to which stockfish will calculate. When `go()` is
    /// called, this depth will be used to determine how deeply stockfish will
    /// calculate.
    pub fn set_depth(&mut self, depth: u32) {
        self.depth = depth;
    }

    /// This function is meaant to only be called after stockfish has received
    /// a command for calculating a position.
    /// Reads the lines outputted by the stockfish process and returns an `EngineOutput`
    /// value describing stockfish's evaluation and its chosen best move.
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

    /// Returns a string showing a visual display of the current chess position
    /// in which stockfish is playing.
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

    /// This function simply prints the result from `get_board_display()`
    pub fn print_board(&mut self) -> io::Result<()> {
        let board_display = self.get_board_display()?;
        println!("{board_display}");
        Ok(())
    }

    /// Sets a UCI option for the stockfish engine.
    /// A listing of some of the possible options and their default values:
    /// - "Threads": 1,
    /// - "Hash": 16,
    /// - "Debug Log File": "",
    /// - "Contempt": 0,
    /// - "Min Split Depth": 0,
    /// - "Ponder": "false",
    /// - "MultiPV": 1,
    /// - "Move Overhead": 10,
    /// - "Minimum Thinking Time": 20,
    /// - "Slow Mover": 100,
    /// - "UCI_Chess960": "false",
    /// - "UCI_LimitStrength": "false",
    /// - "UCI_Elo": 1350,
    /// - "Skill Level": 20,
    pub fn set_option(&mut self, option_name: &str, option_value: &str) -> io::Result<()> {
        self.send(&format!("setoption name {option_name} value {option_value}"))
    }

    /// Sets the size of stockfish's hashtable/transposition table.
    /// Value is given in megabytes. Generally, the larger the table, the
    /// faster the engine will run.
    pub fn set_hash(&mut self, hash: u32) -> io::Result<()> {
        self.send(&format!("setoption name Hash value {hash}"))
    }

    /// Sets the number of threads that stockfish will use.
    pub fn set_threads(&mut self, threads: u32) -> io::Result<()> {
        self.send(&format!("setoption name Threads value {threads}"))
    }

    /// Returns a string representing the version of stockfish being run.
    /// Returns none if the version wasn't able to be parsed.
    pub fn get_version(&self) -> &Option<String> {
        &self.version
    }

    /// Sends the "quit" UCI command to the stockfish process.
    pub fn quit(&mut self) -> io::Result<()> {
        self.send("quit")
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