# stockfish

A wrapper library for simple incorporation of the Stockfish chess engine into Rust. Requires an installation of the engine to be present. (May be sourced here: [stockfishchess.org](https://stockfishchess.org/download/).)

## Usage

The constructor will take the path to the Stockfish executable. 

```rust
let mut stockfish = Stockfish::new("path/to/stockfish");
```

Once created, setup the engine:

```rust
stockfish.setup_for_new_game()?;
```

Direct the engine to the desired position on the board; this may be done through a sequence of moves from the regular starting position:

```rust
stockfish.play_moves(&["e2e4", "c7c5"])?;
```

Or through setting its position via Forsyth-Edwards notation ([FEN](https://www.chessprogramming.org/Forsyth-Edwards_Notation)):

```rust
let fen = "r1bqkb1r/pppp1ppp/2n2n2/4p3/4P3/2N2N2/PPPP1PPP/\
R1BQKB1R w KQkq - 0 1";
stockfish.set_fen_position(fen)?;
```

Then, calculation may be initiated through various methods, the simplest of which
is `go`, which makes Stockfish calculate until it reaches a certain depth:

```rust
stockfish.set_depth(20); // Optional; default depth is 15
let engine_output = stockfish.go()?;
```

The returned `EngineOutput` may be worked with like so:

```rust
let best_move = engine_output.best_move();
println!("Best move according to Stockfish: {best_move}");

let eval = engine_output.eval();
match eval.eval_type() {
    EvalType::Centipawn => {
        println!("Eval: {} centipawns", eval.value());
    }
    EvalType::Mate => {
        println!("Eval: Mate in {}", eval.value());
    }
};
```

## A longer example

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

    stockfish.quit()?;

    Ok(())
}
```

## Other details

Some different ways of invoking calculation from Stockfish:

```rust
// Have stockfish calculate for five seconds, then get its output
let engine_output = stockfish.go_for(
    Duration::from_millis(5000)
)?;

// Have stockfish calculate for a variable amount of time based on
// the players' move times in the chess game
// (expressed in milliseconds)
let engine_output = stockfish.go_based_on_times(
    Some(60_000), Some(60_000)
);
```

Some configuration options:

```rust
stockfish.set_hash(64)?;
stockfish.set_threads(6)?;

// Set any UCI option for stockfish
stockfish.set_option("Move Overhead", "5")?;
```
