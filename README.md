# stockfish

A small Rust library for creating and interacting with a running Stockfish process.

Requires the stockfish engine to be installed; the path to the binary file is to be specified in the constructor. 

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

    Ok(())
}
```

(Note: Very much work-in-progress! This is my first crate.)