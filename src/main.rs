use sudoku::generate;

fn main() {
    let board = generate();
    println!("{:?}", board);
}
