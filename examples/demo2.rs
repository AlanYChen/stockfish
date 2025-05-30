use stockfish::Stockfish;
use std::time::Duration;

fn main() -> Result<(), std::io::Error> {
    let path = if cfg!(target_os = "windows") {
        "./stockfish.exe"
    } else {
        "stockfish"
    };

    let mut stockfish = Stockfish::new(&path)?;
    stockfish.setup_for_new_game()?;
    stockfish.print_board()?;

    let engine_output = stockfish.go_for(Duration::from_millis(5000))?;
    println!("engine_output: {engine_output:?}");

    Ok(())
}