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

    let engine_output = stockfish.go_based_on_times(Some(50000), Some(10))?;
    println!("engine_output: {engine_output:?}");

    let fen = "r1bqkb1r/pppp1ppp/2n2n2/4p3/4P3/2N2N2/PPPP1PPP/R1BQKB1R w KQkq - 0 1";

    stockfish.set_fen_position(fen)?;

    assert_eq!(fen, stockfish.get_fen()?);

    Ok(())
}