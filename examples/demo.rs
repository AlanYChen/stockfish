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