# stockfish

(Note: Currently work-in-progress! This is my first crate.)

A small Rust library for creating and interacting with a running Stockfish process.

Requires the stockfish engine to be installed; the path to the binary file is to be specified in the constructor. 

```rust
let stockfish = Stockfish::new("path/to/stockfish");
```