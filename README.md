# stockfish

[![crates-io](https://img.shields.io/crates/v/stockfish.svg)](https://crates.io/crates/stockfish)
[![api-docs](https://docs.rs/stockfish/badge.svg)](https://docs.rs/stockfish)
[![License](https://img.shields.io/crates/l/bitflags.svg)](https://img.shields.io/badge/license-MIT%20or%20Apache2.0-Green)
[![Crates.io (recent)](https://img.shields.io/crates/dr/stockfish)](https://crates.io/crates/stockfish)

A wrapper library that makes integrating Stockfish with Rust a breeze.

An [installation](https://stockfishchess.org/download/) of the Stockfish engine is needed. (Or any [UCI](https://official-stockfish.github.io/docs/stockfish-wiki/UCI-&-Commands.html)-compatible engine, although this library was mainly written with Stockfish in mind.)

## Usage

Add this to your `Cargo.toml`:

```toml
[dependencies]
stockfish = "0.2.9"
```

And this to your source code:

```rust
use stockfish::Stockfish;
```

## Example usage

Using the path to the Stockfish executable, construct an instance. (Mind the `?`; it's possible that any of these operations may fail, as they involve IO operations.)

```rust
let mut stockfish = Stockfish::new("path/to/stockfish")?;
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
is `go`, which makes Stockfish calculate until it reaches a certain [depth](https://www.chessprogramming.org/Depth):

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

Some configuration options (of which the most likely to be changed
would be [hash table size](https://www.chessprogramming.org/Hash_Table)
and [thread count](https://official-stockfish.github.io/docs/stockfish-wiki/UCI-&-Commands.html#threads)):

```rust
stockfish.set_threads(6)?;
stockfish.set_hash(64)?;

// Set any UCI option for stockfish
stockfish.set_option("Move Overhead", "5")?;
```
