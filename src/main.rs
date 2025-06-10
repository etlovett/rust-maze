use rand::random_bool;
use rand::random_range;
use std::fmt;
use std::io;

type Size = usize;

enum Direction {
    North,
    East,
    South,
    West,
}

const ALL_DIRECTIONS: [Direction; 4] = [
    Direction::North,
    Direction::East,
    Direction::South,
    Direction::West,
];

#[derive(Debug, Copy, Clone)]
struct Location {
    x: Size,
    y: Size,
}

#[derive(Debug, Copy, Clone)]
struct MazeSize {
    width: Size,
    height: Size,
}

#[derive(Debug, Copy, Clone)]
struct Cell {
    maze_size: MazeSize,
    location: Location,
    can_exit_south: bool,
    can_exit_east: bool,
}

#[derive(Debug)]
struct Maze {
    size: MazeSize,
    cells: Vec<Vec<Cell>>,
}

type Path = Vec<Location>;

impl PartialEq for Location {
    fn eq(&self, other: &Self) -> bool {
        self.x == other.x && self.y == other.y
    }
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

        let mut maze = Maze {
            cells,
            size: MazeSize { width, height },
        };
        maze.make_valid();
        maze
    }

    fn make_valid(&mut self) {
        for y in 0..self.size.height {
            for x in 0..self.size.width {
                // While the cell at (x, y) does not have any valid moves, try to open a random direction.
                while self.get_valid_moves(&self.cells[y][x]).len() == 0 {
                    let direction_to_open = &ALL_DIRECTIONS[random_range(0..ALL_DIRECTIONS.len())];
                    let (nx, ny) = match direction_to_open {
                        Direction::North if y > 0 => (x, y - 1),
                        Direction::East if x + 1 < self.size.width => (x + 1, y),
                        Direction::South if y + 1 < self.size.height => (x, y + 1),
                        Direction::West if x > 0 => (x - 1, y),
                        _ => continue,
                    };

                    match direction_to_open {
                        Direction::North => {
                            self.cells[ny][nx].can_exit_south = true;
                        }
                        Direction::East => {
                            self.cells[y][x].can_exit_east = true;
                        }
                        Direction::South => {
                            self.cells[y][x].can_exit_south = true;
                        }
                        Direction::West => {
                            self.cells[ny][nx].can_exit_east = true;
                        }
                    }
                }
            }
        }

        // TODO(eric): edit the maze to actually have a solution
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

    fn solve(&self) -> Option<Path> {
        let start_cell = self.get_cell_at(&Location { x: 0, y: 0 });
        let mut path = Vec::new();
        return if self.solve_helper(start_cell, &mut path) {
            Some(path)
        } else {
            None
        };
    }

    fn solve_helper(&self, cur_cell: &Cell, path_to_here: &mut Path) -> bool {
        // Now that we're visiting this cell, add it to the path.
        path_to_here.push(cur_cell.location);

        if cur_cell.is_finish() {
            return true;
        }

        // Collect all valid moves, avoiding cycles by filtering out cells already in the path.
        let valid_moves = self
            .get_valid_moves(cur_cell)
            .into_iter()
            .filter(|cell| !path_to_here.contains(&cell.location))
            .collect::<Vec<_>>();
        for next_cell in valid_moves {
            if self.solve_helper(next_cell, path_to_here) {
                // If we find a solution, return. We've already added this cell to the path.
                return true;
            }
        }

        // If we didn't find a solution from this cell, backtrack.
        path_to_here.pop();
        false
    }

    fn get_valid_moves(&self, cur_cell: &Cell) -> Vec<&Cell> {
        ALL_DIRECTIONS
            .iter()
            .filter(|dir| self.can_move(cur_cell, dir))
            .filter_map(|dir| self.get_cell_in(cur_cell, dir))
            .collect::<Vec<_>>()
    }

    fn can_move(&self, cur_cell: &Cell, direction: &Direction) -> bool {
        match direction {
            Direction::North => self
                .get_cell_in(cur_cell, &Direction::North)
                .map(|north_cell| north_cell.can_exit_south)
                .unwrap_or(false),
            Direction::East => cur_cell.can_exit_east,
            Direction::South => cur_cell.can_exit_south,
            Direction::West => self
                .get_cell_in(cur_cell, &Direction::West)
                .map(|west_cell| west_cell.can_exit_east)
                .unwrap_or(false),
        }
    }

    fn get_location(&self, cur_location: &Location, direction: &Direction) -> Option<Location> {
        match direction {
            Direction::North => {
                if cur_location.y >= 1 {
                    Some(Location {
                        x: cur_location.x,
                        y: cur_location.y - 1,
                    })
                } else {
                    None
                }
            }
            Direction::East => {
                if cur_location.x < self.size.width - 1 {
                    Some(Location {
                        x: cur_location.x + 1,
                        y: cur_location.y,
                    })
                } else {
                    None
                }
            }
            Direction::South => {
                if cur_location.y < self.size.height - 1 {
                    Some(Location {
                        x: cur_location.x,
                        y: cur_location.y + 1,
                    })
                } else {
                    None
                }
            }
            Direction::West => {
                if cur_location.x >= 1 {
                    Some(Location {
                        x: cur_location.x - 1,
                        y: cur_location.y,
                    })
                } else {
                    None
                }
            }
        }
    }

    fn get_cell_at(&self, location: &Location) -> &Cell {
        &self.cells[usize::from(location.y)][usize::from(location.x)]
    }

    fn get_cell_in(&self, cur_cell: &Cell, direction: &Direction) -> Option<&Cell> {
        self.get_location(&cur_cell.location, direction)
            .map(|new_location| self.get_cell_at(&new_location))
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

    println!("{}", maze.to_string());

    let solution = maze.solve();
    println!("{solution:#?}");
}
