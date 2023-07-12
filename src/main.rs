use sudoku::{generate, generate_max_empty};

fn main() {
    let board = generate_max_empty();
    println!("{:?}", board);
    println!("Number of gaps: {}", board.num_empty());
}
