use std::{
    io,
    process::Command,
    sync::{mpsc, mpsc::Receiver},
    time::Duration,
    string::ToString,
};

use interactive_process::InteractiveProcess;

use crate::engine_eval::{EngineEval, EvalType};
use crate::engine_output::EngineOutput;

/// The interface for interacting with a Stockfish process.
pub struct Stockfish {
    interactive_process: InteractiveProcess,
    receiver: Receiver<String>,
    depth: u32,
    version: Option<String>,
}

impl Stockfish {

    /// Given the path to the Stockfish binary executable, this function
    /// initiates an [`InteractiveProcess`] for the executable, and returns an instance
    /// of the [Stockfish] wrapper class.
    /// 
    /// # Example
    /// 
    /// ```rust
    /// use stockfish::Stockfish;
    /// let stockfish = Stockfish::new("stockfish.exe").unwrap();
    /// ```
    /// 
    /// # Error
    /// 
    /// Returns an [`io::Error`] if an error occurred while trying to
    /// create/communicate with the engine. 
    pub fn new(path: &str) -> io::Result<Stockfish> {
        let mut command = Command::new(path);

        let (tx, rx) = mpsc::channel();

        let proc = InteractiveProcess::new(&mut command, move |line| {
            let line = line.unwrap();
            let send_result = tx.send(line);
            if send_result.is_err() {
                println!("receiving end of mpsc channel disconnected");
            }
        })?;

        let first_line = rx.recv().expect("stockfish process should have outputted a first line");
        let version = first_line.split(' ').nth(1).map(ToString::to_string);

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
    /// 
    /// # Example
    /// 
    /// ```rust
    /// let mut stockfish = Stockfish::new("stockfish.exe")?;
    /// stockfish.setup_for_new_game()?;
    /// ```
    /// 
    /// # Error
    /// 
    /// Returns an [`io::Error`] if an error occurred while trying to
    /// communicate with the engine. 
    pub fn setup_for_new_game(&mut self) -> io::Result<()> {
        self.ensure_ready()?;
        self.uci_send("ucinewgame")?;
        Ok(())
    }

    /// Changes the current chess position in which Stockfish is currently playing.
    /// The argument to be passed is a string in FEN (Forsyth-Edwards Notation).
    /// 
    /// # Example
    /// 
    /// ```rust
    /// let mut stockfish = Stockfish::new("stockfish.exe")?;
    /// stockfish.set_fen_position("r1bqk2r/ppppppbp/2n2np1/8/8/2N2NP1/PPPPPPBP/R1BQK2R w KQkq - 0 1")?;
    /// stockfish.print_board()?;
    /// ```
    /// 
    /// # Error
    /// 
    /// Returns an [`io::Error`] if an error occurred while trying to
    /// communicate with the engine. 
    pub fn set_fen_position(&mut self, fen: &str) -> io::Result<()> {
        let msg = String::from("position fen ") + fen;
        self.uci_send(&msg)?;
        Ok(())
    }

    /// Reverts the current chess position to the default starting position.
    /// This is the same as calling `set_fen_position` with the default
    /// fen. (`rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1`)
    /// 
    /// # Example
    /// 
    /// ```rust
    /// let mut stockfish = Stockfish::new("stockfish.exe")?;
    /// stockfish.set_fen_position("r1bqk2r/ppppppbp/2n2np1/8/8/2N2NP1/PPPPPPBP/R1BQK2R w KQkq - 0 1")?;
    /// stockfish.reset_position()?;
    /// 
    /// // See that the board has been reverted to the default position
    /// stockfish.print_board()?;
    /// ```
    /// 
    /// # Error
    /// 
    /// Returns an [`io::Error`] if an error occurred while trying to
    /// communicate with the engine. 
    pub fn reset_position(&mut self) -> io::Result<()> {
        self.uci_send("position startpos")?;
        Ok(())
    }

    /// This should be called to ensure that the Stockfish process is ready
    /// to receive its inputs. Sends the UCI command `"isready"` to Stockfish
    /// and blocks until it sends back `"readyok"`.
    /// 
    /// # Example
    /// 
    /// ```rust
    /// let mut stockfish = Stockfish::new("stockfish.exe")?;
    /// stockfish.ensure_ready()?;
    /// stockfish.setup_for_new_game()?;
    /// ```
    /// 
    /// # Error
    /// 
    /// Returns an [`io::Error`] if an error occurred while trying to
    /// communicate with the engine. 
    pub fn ensure_ready(&mut self) -> io::Result<()> {
        self.uci_send("isready")?;
        while self.read_line() != "readyok" {}
        Ok(())
    }

    /// Returns a string Forsyth-Edwards notation (FEN) describing the current chess position
    /// in which Stockfish is playing.
    /// 
    /// # Example
    /// 
    /// ```rust
    /// let mut stockfish = Stockfish::new("stockfish.exe")?;
    /// stockfish.play_move("e2e4")?;
    /// 
    /// let fen = stockfish.get_fen()?;
    /// println!("fen after move was played: {fen}");
    /// ```
    /// 
    /// # Error
    /// 
    /// Returns an [`io::Error`] if an error occurred while trying to
    /// communicate with the engine. 
    pub fn get_fen(&mut self) -> io::Result<String> {
        self.uci_send("d")?;
        loop {
            let line = self.read_line();
            let mut segments= line.split(' ');
            if segments.next().unwrap() == "Fen:" {
                let fen = segments.collect::<Vec<&str>>().join(" ");

                // Keep reading lines until reached "Checkers", which is in the last line
                while !self.read_line().contains("Checkers") {}

                return Ok(fen);
            }
        }
    }

    /// Plays a move on the current chess position in which Stockfish is playing.
    /// This function only updates the board; it does not prompt Stockfish to begin calculating.
    /// 
    /// # Example
    /// 
    /// ```rust
    /// let mut stockfish = Stockfish::new("stockfish.exe")?;
    /// 
    /// stockfish.print_board()?;
    /// 
    /// stockfish.play_move("e2e4")?;
    /// 
    /// // See that the move has been played on the board
    /// stockfish.print_board()?;
    /// ```
    /// 
    /// # Error
    /// 
    /// Returns an [`io::Error`] if an error occurred while trying to
    /// communicate with the engine. 
    pub fn play_move(&mut self, move_str: &str) -> io::Result<()> {
        let fen = self.get_fen()?;
        let data = format!("position fen {fen} moves {move_str}");
        self.uci_send(&data)?;
        Ok(())
    }

    /// Plays a sequence of moves on the current chess position in which Stockfish is playing.
    /// This function only updates the board; it does not prompt Stockfish to begin calculating.
    /// 
    /// # Example
    /// 
    /// ```rust
    /// let mut stockfish = Stockfish::new("stockfish.exe")?;
    /// 
    /// stockfish.print_board()?;
    /// 
    /// let moves = ["e2e4", "e7e5", "f1c4"];
    /// stockfish.play_moves(&moves)?;
    /// 
    /// // See that the moves have been played on the board
    /// stockfish.print_board()?;
    /// ```
    /// 
    /// # Error
    /// 
    /// Returns an [`io::Error`] if an error occurred while trying to
    /// communicate with the engine. 
    pub fn play_moves(&mut self, moves: &[&str]) -> io::Result<()> {
        let fen = self.get_fen()?;
        let moves = moves.join(" ");

        let data = format!("position fen {fen} moves {moves}");
        self.uci_send(&data)?;
        Ok(())
    }

    /// Makes Stockfish calculate to the depth that has been set. (The default
    /// depth is 15.)
    /// 
    /// Once Stockfish has finished its calculations, this function should return
    /// an [`EngineOutput`] describing the result of its calculations.
    /// 
    /// # Example
    /// 
    /// ```rust
    /// let mut stockfish = Stockfish::new("stockfish.exe")?;
    /// 
    /// let engine_output = stockfish.go()?;
    /// println!("output from stockfish: {engine_output:?}");
    /// ```
    /// 
    /// # Error
    /// 
    /// Returns an [`io::Error`] if an error occurred while trying to
    /// communicate with the engine. 
    pub fn go(&mut self) -> io::Result<EngineOutput> {
        let message = String::from("go depth ") + &self.depth.to_string();
        self.uci_send(&message)?;
        self.get_engine_output()
    }

    /// Makes Stockfish calculate for a specified amount of time. Blocks the calling thread
    /// for the duration of the specified calculation time.
    ///
    /// Once Stockfish has finished its calculations, this function should return
    /// an [`EngineOutput`] describing the result of its calculations.
    /// 
    /// # Example
    /// 
    /// ```rust
    /// use std::time::Duration;
    /// let mut stockfish = Stockfish::new("stockfish.exe")?;
    /// 
    /// let engine_output = stockfish.go_for(Duration::from_millis(500))?;
    /// println!("output from stockfish: {engine_output:?}");
    /// ```
    /// 
    /// # Error
    /// 
    /// Returns an [`io::Error`] if an error occurred while trying to
    /// communicate with the engine. 
    pub fn go_for(&mut self, calculation_time: Duration) -> io::Result<EngineOutput> {
        self.uci_send("go")?;
        std::thread::sleep(calculation_time);
        self.uci_send("stop")?;
        self.get_engine_output()
    }

    /// Makes Stockfish calculate for a variable time based on the times given as parameters.
    /// If for example Stockfish were analyzing from the white side and `white_time` is low
    /// (say, only 10 seconds), then Stockfish will use less time for its calculation.
    /// The parameters are to be given in milliseconds.
    ///
    /// Once Stockfish has finished its calculations, this function should return
    /// an [`EngineOutput`] describing the result of its calculations.
    /// 
    /// # Example
    /// 
    /// ```rust
    /// let mut stockfish = Stockfish::new("stockfish.exe")?;
    /// 
    /// let engine_output = stockfish.go_based_on_times(
    ///     Some(50_000), // White has 50 seconds
    ///     Some(55_000), // Black has 55 seconds
    /// )?;
    /// println!("output from stockfish: {engine_output:?}");
    /// ```
    /// 
    /// # Error
    /// 
    /// Returns an [`io::Error`] if an error occurred while trying to
    /// communicate with the engine. 
    pub fn go_based_on_times(&mut self, white_time: Option<u32>, black_time: Option<u32>) -> io::Result<EngineOutput> {
        let mut message = String::from("go");
        if let Some(time) = white_time {
            message += &format!(" wtime {time}");
        }
        if let Some(time) = black_time {
            message += &format!(" btime {time}");
        }

        self.uci_send(&message)?;
        self.get_engine_output()
    }

    /// Configures the depth to which Stockfish will calculate. When methods like `go`
    /// and `go_for` are called, this field is used to determine how deeply Stockfish will
    /// calculate.
    /// 
    /// # Example
    /// 
    /// ```rust
    /// let mut stockfish = Stockfish::new("stockfish.exe")?;
    /// stockfish.set_depth(20)?;
    /// let engine_output = stockfish.go()?; // Stockfish will calculate to the newly set depth
    /// println!("output from stockfish: {engine_output:?}");
    /// ```
    /// 
    /// # Error
    /// 
    /// Returns an [`io::Error`] if an error occurred while trying to
    /// communicate with the engine. 
    pub fn set_depth(&mut self, depth: u32) {
        self.depth = depth;
    }

    /// This method is meant to only be called after Stockfish has received
    /// a command for calculating a position.
    /// Reads the lines outputted by the Stockfish process and returns an [`EngineOutput`]
    /// value describing Stockfish's evaluation and its chosen best move.
    fn get_engine_output(&mut self) -> io::Result<EngineOutput> {
        let fen = self.get_fen()?;

        // The output from stockfish normally displays the value of the evaluation score
        // relative to the player with the current move. Use a multiplier to flip it such that
        // the score is not relative to the player with the current move.
        let color_multiplier = if fen.contains('w') {1} else {-1};

        let mut previous_line: Option<String> = None;

        loop {
            let line = self.read_line();
            let mut segments = line.split(' ');
            let first_segment = segments.next().expect("should be able to get first segment");
            if first_segment != "bestmove" {
                previous_line = Some(line);
                continue;
            }
            
            let previous_line = previous_line.unwrap();
            let previous_segments: Vec<&str> = previous_line.split(' ').collect();

            let mut score_type = None;
            let mut score_value: Option<i32> = None;

            for (i, segment) in previous_segments.iter().enumerate() {
                if *segment == "score" {
                    score_type = Some(previous_segments[i + 1]);
                    score_value = Some(
                        previous_segments[i + 2].parse::<i32>()
                            .expect("should be able to parse score_value")
                            * color_multiplier);
                    break;
                }
            }

            let score_type = EvalType::from_descriptor(score_type.unwrap());
            let eval = EngineEval::new(score_type, score_value.unwrap());

            let best_move = segments.next()
                .expect("should be able to get second segment")
                .to_owned();

            let output = EngineOutput::new(eval, best_move);
            return Ok(output);
        }
    }

    /// Returns a string showing a visual display of the current chess position
    /// in which Stockfish is playing.
    /// 
    /// # Example
    /// 
    /// ```rust
    /// let mut stockfish = Stockfish::new("stockfish.exe")?;
    /// stockfish.play_move("d2d4")?;
    /// 
    /// let board = stockfish.get_board_display()?;
    /// println!("board: {board}");
    /// ```
    /// 
    /// # Illustration
    /// Example of the output from Stockfish:
    /// 
    /// ```
    /// +---+---+---+---+---+---+---+---+
    /// | r | n | b | q | k | b | n | r | 8
    /// +---+---+---+---+---+---+---+---+
    /// | p | p | p | p | p | p | p | p | 7
    /// +---+---+---+---+---+---+---+---+
    /// |   |   |   |   |   |   |   |   | 6
    /// +---+---+---+---+---+---+---+---+
    /// |   |   |   |   |   |   |   |   | 5
    /// +---+---+---+---+---+---+---+---+
    /// |   |   |   |   |   |   |   |   | 4
    /// +---+---+---+---+---+---+---+---+
    /// |   |   |   |   |   |   |   |   | 3
    /// +---+---+---+---+---+---+---+---+
    /// | P | P | P | P | P | P | P | P | 2
    /// +---+---+---+---+---+---+---+---+
    /// | R | N | B | Q | K | B | N | R | 1
    /// +---+---+---+---+---+---+---+---+
    ///   a   b   c   d   e   f   g   h
    /// ```
    /// 
    /// # Error
    /// 
    /// Returns an [`io::Error`] if an error occurred while trying to
    /// communicate with the engine. 
    pub fn get_board_display(&mut self) -> io::Result<String> {
        self.uci_send("d")?;

        let mut lines: Vec<String> = Vec::with_capacity(20);

        loop {
            let line = self.read_line();
            if line.is_empty() {
                continue;
            }

            let first_segment = line.split(' ').next()
                .expect("non-empty line should have segment");
            if first_segment == "Fen:" {
                break;
            }
            lines.push(line);
        }
        Ok(lines.join("\n"))
    }

    /// This function simply prints the result from `get_board_display`.
    /// 
    /// # Example
    /// 
    /// ```rust
    /// let mut stockfish = Stockfish::new("stockfish.exe")?;
    /// 
    /// stockfish.play_move("d2d4")?;
    /// stockfish.print_board()?;
    /// 
    /// stockfish.play_move("d7d5")?;
    /// stockfish.print_board()?;
    /// ```
    /// 
    /// # Error
    /// 
    /// Returns an [`io::Error`] if an error occurred while trying to
    /// communicate with the engine. 
    pub fn print_board(&mut self) -> io::Result<()> {
        let board_display = self.get_board_display()?;
        println!("{board_display}");
        Ok(())
    }

    /// Sets a UCI option for the Stockfish engine. This is used for changing the engine's
    /// internal parameters.
    /// 
    /// # Example
    /// 
    /// ```rust
    /// let mut stockfish = Stockfish::new("stockfish.exe")?;
    /// stockfish.set_option("Move Overhead", "20")?;
    /// ```
    /// The following is a listing of some of the possible options and their default values.
    /// (Note: these may be subject to change, and may not be universal.)
    /// - `"Threads"`: 1,
    /// - `"Hash"`: 16,
    /// - `"Debug Log File"`: "",
    /// - `"Contempt"`: 0,
    /// - `"Min Split Depth"`: 0,
    /// - `"Ponder"`: "false",
    /// - `"MultiPV"`: 1,
    /// - `"Move Overhead"`: 10,
    /// - `"Minimum Thinking Time"`: 20,
    /// - `"Slow Mover"`: 100,
    /// - `"UCI_Chess960"`: "false",
    /// - `"UCI_LimitStrength"`: "false",
    /// - `"UCI_Elo"`: 1350,
    /// - `"Skill Level"`: 20,
    /// 
    /// # Error
    /// 
    /// Returns an [`io::Error`] if an error occurred while trying to
    /// communicate with the engine. 
    pub fn set_option(&mut self, option_name: &str, option_value: &str) -> io::Result<()> {
        self.uci_send(&format!("setoption name {option_name} value {option_value}"))
    }

    /// Sets the size of Stockfish's hashtable/transposition table.
    /// Value is given in megabytes. Generally, the larger the table, the
    /// faster the engine will run.
    /// 
    /// # Example
    /// 
    /// ```rust
    /// let mut stockfish = Stockfish::new("stockfish.exe")?;
    /// stockfish.set_hash(64)?;
    /// ```
    /// 
    /// # Error
    /// 
    /// Returns an [`io::Error`] if an error occurred while trying to
    /// communicate with the engine. 
    pub fn set_hash(&mut self, hash: u32) -> io::Result<()> {
        self.set_option("Hash", &hash.to_string())
    }

    /// Sets the number of CPU threads that Stockfish will use in its calculations.
    /// For Stockfish, it's recommended to set this equal to the number of available
    /// CPU cores on your machine.
    /// 
    /// # Example
    /// 
    /// ```rust
    /// 
    /// let mut stockfish = Stockfish::new("stockfish.exe")?;
    /// stockfish.set_threads(16)?;
    /// ```
    /// 
    /// # Error
    /// 
    /// Returns an [`io::Error`] if an error occurred while trying to
    /// communicate with the engine. 
    pub fn set_threads(&mut self, threads: u32) -> io::Result<()> {
        self.set_option("Threads", &threads.to_string())
    }

    /// Sets the elo at which Stockfish will aim to play.
    /// 
    /// Similar to `set_skill_level` in functionality, however, calling
    /// either one of these two functions will **override** the effect of the other.
    /// 
    /// # Example
    /// 
    /// ```rust
    /// let mut stockfish = Stockfish::new("stockfish.exe")?;
    /// stockfish.set_fen_position("r1bqkbnr/ppp1pppp/2np4/8/8/3P4/PPPBPPPP/RN1QKBNR w KQkq - 0 1")?;
    /// 
    /// stockfish.set_skill_level(10)?; // The effect of this call is overridden by set_elo
    /// 
    /// stockfish.set_elo(1450)?;
    /// 
    /// let engine_output = stockfish.go()?;
    /// ```
    /// 
    /// # Error
    /// 
    /// Returns an [`io::Error`] if an error occurred while trying to
    /// communicate with the engine. 
    pub fn set_elo(&mut self, elo: u32) -> io::Result<()> {
        self.set_option("UCI_LimitStrength", "true")?;
        self.set_option("Elo", &elo.to_string())
    }

    /// Sets the skill level at which Stockfish will aim to play. Skill level
    /// is given in the range of 0 to 20 (from weakest to strongest.)
    /// 
    /// Similar to `set_elo` in functionality, however, calling either one of
    /// these two functions will **override** the effect of the other.
    /// 
    /// # Example
    /// 
    /// ```rust
    /// let mut stockfish = Stockfish::new("stockfish.exe")?;
    /// stockfish.set_fen_position("r1bqkbnr/ppp1pppp/2np4/8/8/3P4/PPPBPPPP/RN1QKBNR w KQkq - 0 1")?;
    /// 
    /// stockfish.set_elo(2500)?; // The effect of this call is overridden by set_skill_level
    /// 
    /// stockfish.set_skill_level(16)?;
    /// 
    /// let engine_output = stockfish.go()?;
    /// ```
    /// 
    /// # Error
    /// 
    /// Returns an [`io::Error`] if an error occurred while trying to
    /// communicate with the engine. 
    pub fn set_skill_level(&mut self, skill_level: u32) -> io::Result<()> {
        self.set_option("UCI_LimitStrength", "false")?;
        self.set_option("Skill Level", &skill_level.to_string())
    }

    /// Returns a string representing the version of Stockfish being run.
    /// Returns [`None`] if the version wasn't able to be parsed from Stockfish's
    /// output.
    pub fn get_version(&self) -> &Option<String> {
        &self.version
    }

    /// Sends the `"quit"` UCI command to the Stockfish process, whereupon it
    /// will attempt to quit the program as soon as possible.
    /// 
    /// # Example
    /// 
    /// ```rust
    /// let mut stockfish = Stockfish::new("stockfish.exe")?;
    /// stockfish.set_fen_position("rn1qkbnr/pbpppppp/1p6/8/8/1P2P3/PBPP1PPP/RN1QKBNR w KQkq - 0 1")?;
    /// let engine_output = stockfish.go()?;
    /// 
    /// stockfish.quit()?;
    /// ```
    /// 
    /// # Error
    /// 
    /// Returns an [`io::Error`] if an error occurred while trying to
    /// communicate with the engine. 
    pub fn quit(&mut self) -> io::Result<()> {
        self.uci_send("quit")
    }

    /// Sends a [UCI](https://gist.github.com/DOBRO/2592c6dad754ba67e6dcaec8c90165bf)
    /// command to the engine.
    /// 
    /// Use carefully; this function itself does not handle or return output
    /// that may be incurred by the sent command.
    pub fn uci_send(&mut self, command: &str) -> io::Result<()> {
        self.interactive_process.send(command)?;
        Ok(())
    }

    /* Private Methods */
    fn read_line(&mut self) -> String {
        self.receiver.recv().expect("should be able to read from receiver")
    }
}