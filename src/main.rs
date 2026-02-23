use maze::{Maze, Size};
use std::io;

fn get_size(dimension: &str) -> Size {
    loop {
        let mut size = String::new();

        println!("Please enter the {}.", dimension);
        io::stdin()
            .read_line(&mut size)
            .expect("Failed to read line");
        match size.trim().parse() {
            Ok(num) if num >= 2 => return num,
            _ => {
                println!("Please enter a valid u8 number >=2.");
                continue;
            }
        };
    }
}

fn main() {
    let width = get_size("width");
    let height = get_size("height");
    println!("Creating a maze of size {}x{}.", width, height);

    let maze = Maze::new(width, height);

    println!("{}", maze);

    let solution = maze.solve();
    println!("{solution:#?}");

    // TODO(eric): Print the maze with the solution path embedded
}
