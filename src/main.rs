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
struct MazeSize {
    width: Size,
    height: Size,
}

#[derive(Debug)]
struct Cell {
    maze_size: MazeSize,
    location: Location,
    can_exit_south: bool,
    can_exit_east: bool,
}

impl Cell {
    fn new(x: Size, y: Size, width: Size, height: Size) -> Cell {
        Cell {
            maze_size: MazeSize { width, height },
            location: Location { x, y },
            can_exit_south: Cell::get_can_exit(y, height),
            can_exit_east: Cell::get_can_exit(x, width),
        }
    }

    fn get_can_exit(x_or_y: Size, width_or_height: Size) -> bool {
        if x_or_y < width_or_height - 1 {
            random_bool(0.5)
        } else {
            false
        }
    }

    fn is_first_in_row(&self) -> bool {
        self.location.x == 0
    }

    fn is_last_in_row(&self) -> bool {
        self.location.x == self.maze_size.width - 1
    }

    fn is_first_in_col(&self) -> bool {
        self.location.y == 0
    }

    fn is_last_in_col(&self) -> bool {
        self.location.y == self.maze_size.height - 1
    }

    fn is_start(&self) -> bool {
        self.is_first_in_row() && self.is_first_in_col()
    }

    fn is_finish(&self) -> bool {
        self.is_last_in_row() && self.is_last_in_col()
    }

    fn get_center_char(&self) -> char {
        if self.is_start() {
            'S'
        } else if self.is_finish() {
            'F'
        } else {
            ' '
        }
    }
}

#[derive(Debug)]
struct Maze {
    size: MazeSize,
    cells: Vec<Vec<Cell>>,
}

impl Maze {
    fn new(width: Size, height: Size) -> Maze {
        let cells = (0..height)
            .map(|y| {
                let row = (0..width)
                    .map(|x| Cell::new(x, y, width, height))
                    .collect::<Vec<_>>();
                row
            })
            .collect::<Vec<_>>();

        // TODO(eric): edit the maze to actually have a solution
        // TODO(eric): make sure every cell in the maze has at least one entrance/exit
        Maze {
            cells,
            size: MazeSize { width, height },
        }
    }

    fn as_str(&self) -> String {
        let mut maze_str = String::new();

        // Generate first line border.
        maze_str.push('╔');
        for i in 0..self.size.width {
            let is_last_cell = i == self.size.width - 1;

            maze_str.push_str("═════");
            maze_str.push(if is_last_cell { '╗' } else { '╦' });
        }
        maze_str.push('\n');

        for (i, row) in self.cells.iter().enumerate() {
            let is_last_row = i == self.cells.len() - 1;

            // Generate the body of each cell for the row, including left and right borders.
            let mut row_str = String::new();
            row_str.push('║');
            for cell in row {
                row_str.push_str(&format!("  {}  ", cell.get_center_char()));
                row_str.push(if cell.can_exit_east { ' ' } else { '║' });
            }
            row_str.push('\n');
            maze_str.push_str(&row_str);
            maze_str.push_str(&row_str);

            // Generate the border below the cell.
            maze_str.push(if is_last_row { '╚' } else { '╠' });
            for cell in row {
                maze_str.push_str(if cell.can_exit_south {
                    "     "
                } else {
                    "═════"
                });
                maze_str.push(if cell.is_last_in_row() && cell.is_last_in_col() {
                    '╝'
                } else if cell.is_last_in_row() {
                    '╣'
                } else if cell.is_last_in_col() {
                    '╩'
                } else {
                    '╬'
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
