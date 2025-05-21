use rand::random_bool;
use std::fmt;
use std::io;

type Size = u8;

#[derive(Debug)]
struct Location {
    x: Size,
    y: Size,
}

#[derive(Debug)]
struct Cell {
    location: Location,
    can_exit_south: bool,
    can_exit_east: bool,
}

#[derive(Debug)]
struct Maze {
    cells: Vec<Vec<Cell>>,
}

impl Maze {
    fn get_can_exit(x_or_y: Size, width_or_height: Size) -> bool {
        if x_or_y < width_or_height - 1 {
            random_bool(0.5)
        } else {
            false
        }
    }

    fn new(width: Size, height: Size) -> Maze {
        let mut cells = Vec::new();
        for y in 0..height {
            let mut row = Vec::new();
            for x in 0..width {
                row.push(Cell {
                    location: Location { x, y },
                    can_exit_south: Maze::get_can_exit(y, height),
                    can_exit_east: Maze::get_can_exit(x, width),
                })
            }
            cells.push(row);
        }

        // TODO(eric): edit the maze to actually have a solution
        // TODO(eric): make sure every cell in the maze has at least one entrance/exit
        Maze { cells }
    }

    fn as_str(&self) -> String {
        let mut maze_str = String::new();

        maze_str.push('┌');
        for (i, _) in self.cells[0].iter().enumerate() {
            let is_last_cell = i == self.cells[0].len() - 1;

            maze_str.push('─');
            maze_str.push(if is_last_cell { '┐' } else { '┬' });
        }
        maze_str.push('\n');

        for (i, row) in self.cells.iter().enumerate() {
            let is_last_row = i == self.cells.len() - 1;

            maze_str.push('│');
            for cell in row {
                maze_str.push(' ');
                maze_str.push(if cell.can_exit_east { ' ' } else { '│' });
            }
            maze_str.push('\n');

            maze_str.push(if is_last_row { '└' } else { '├' });
            for (j, cell) in row.iter().enumerate() {
                let is_last_cell = j == row.len() - 1;

                maze_str.push(if cell.can_exit_south { ' ' } else { '─' });
                maze_str.push(if is_last_row {
                    if is_last_cell {
                        '┘'
                    } else {
                        '┴'
                    }
                } else {
                    if is_last_cell {
                        '┤'
                    } else {
                        '┼'
                    }
                });
            }
            maze_str.push('\n');
        }

        maze_str
    }
}

impl fmt::Display for Maze {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

fn get_size(dimension: &str) -> Size {
    loop {
        let mut size = String::new();

        println!("Please enter the {}.", dimension);
        io::stdin()
            .read_line(&mut size)
            .expect("Failed to read line");
        match size.trim().parse() {
            Ok(num) => return num,
            Err(_) => {
                println!("Please enter a valid number.");
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

    println!("{}", maze.to_string())
}
