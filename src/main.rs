use sudoku::generate;

fn main() {
    let board = generate();
    println!("{:?}", board);
    println!("Number of gaps: {}", board.num_empty());
}
