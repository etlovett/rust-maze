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
                println!("Please enter a valid integer >=2.");
                continue;
            }
        };
    }
}

fn should_print_solution() -> bool {
    loop {
        let mut answer = String::new();

        println!("Print solution? (y/yes/n/no)");
        io::stdin()
            .read_line(&mut answer)
            .expect("Failed to read line");

        match answer.trim().to_ascii_lowercase().as_str() {
            "y" | "yes" => return true,
            "n" | "no" => return false,
            _ => {
                println!("Please enter y, yes, n, or no.");
                continue;
            }
        };
    }
}

fn main() {
    let width = get_size("width");
    let height = get_size("height");
    let print_solution = should_print_solution();
    println!("Creating a maze of size {}x{}.", width, height);

    let maze = Maze::new(width, height);

    if print_solution {
        let solution = maze
            .solve()
            .expect("solver should always return a path for a valid maze");
        println!("{}", maze.render_with_solution(&solution));
    } else {
        println!("{}", maze);
    }

    // TODO(eric): Print the maze with the solution path embedded
}
