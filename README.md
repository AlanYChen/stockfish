# stockfish

A small Rust library for creating and interacting with a running Stockfish process.

Requires the stockfish engine to be installed. ([stockfishchess.org](https://stockfishchess.org/download/)) The path to the binary file is to be specified in the constructor. 

```rust
let stockfish = Stockfish::new("path/to/stockfish");
```

A longer example:

```rust
use stockfish::Stockfish;

fn main() -> Result<(), std::io::Error> {
    let path = if cfg!(target_os = "windows") {
        "./stockfish.exe"
    } else {
        "stockfish"
    };

    let mut stockfish = Stockfish::new(&path)?;
    stockfish.setup_for_new_game()?;
    stockfish.print_board()?;

    println!("Stockfish version: {:?}", stockfish.get_version());

    stockfish.set_depth(20); // Optional; default depth is 15

    let engine_output = stockfish.go()?;
    println!("engine_output: {engine_output:?}");

    // Play some moves!
    let moves = ["e2e4", "e7e5", "g1f3"];
    for move_str in moves {
        stockfish.play_move(move_str)?;
        stockfish.print_board()?;

        let engine_output = stockfish.go()?;
        println!("engine_output: {engine_output:?}");
    }

    stockfish.quit();

    Ok(())
}
```

Some different ways of invoking calculation from stockfish:

```rust
// Have stockfish calculate for five seconds, then get its output
let engine_output = stockfish.go_for(Duration::from_millis(5000))?;

// Have stockfish calculate for a variable amount of time based on
// the players' move times in the chess game (expressed in milliseconds)
let engine_output = stockfish.go_based_on_times(Some(60_000), Some(60_000));
```

Some configuration options:

```rust
stockfish.set_hash(64)?;
stockfish.set_threads(6)?;

// Set any UCI option for stockfish
stockfish.set_option("Move Overhead", "5")?;
```

FEN-related methods:

```rust
let fen = "r1bqkb1r/pppp1ppp/2n2n2/4p3/4P3/2N2N2/PPPP1PPP/R1BQKB1R w KQkq - 0 1";

stockfish.set_fen_position(fen)?;

assert_eq!(fen, stockfish.get_fen()?);
```

(Note: Very much work-in-progress! This is my first crate.)