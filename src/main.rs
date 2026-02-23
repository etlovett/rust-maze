use rand::random_bool;
use rand::seq::SliceRandom;
use std::fmt;
use std::io;

type Size = usize;

#[derive(Debug, Copy, Clone)]
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
    /**
     * Generate a new cell.
     */
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
            random_bool(0.35)
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
    /**
     * Generate a new valid maze.
     */
    fn new(width: Size, height: Size) -> Maze {
        let cells = (0..height)
            .map(|y| {
                (0..width)
                    .map(|x| Cell::new(x, y, width, height))
                    .collect::<Vec<_>>()
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
        loop {
            let visited_map = self.generate_visitable_map();
            let has_disconnected_cell = visited_map.iter().flatten().any(|&is_visited| !is_visited);
            if !has_disconnected_cell {
                break;
            }

            // Find a disconnected location adjacent to a connected one.
            let disconnected_location = visited_map.iter().enumerate().find_map(|(y, row)| {
                row.iter().enumerate().find_map(|(x, &is_visited)| {
                    if is_visited {
                        return None;
                    }

                    // Check if this cell has any adjacent connected cells.
                    let location = Location { x, y };
                    let has_adjacent_connected_location = ALL_DIRECTIONS.iter().any(|dir| {
                        self.get_location(&location, dir)
                            .map(|adjacent_location| {
                                visited_map[adjacent_location.y][adjacent_location.x]
                            })
                            .unwrap_or(false)
                    });

                    if has_adjacent_connected_location {
                        Some(location)
                    } else {
                        None
                    }
                })
            });

            if disconnected_location.is_none() {
                println!("Should have found a disconnected cell, but didn't!");
                break;
            }

            // Open the found cell to an adjacent connected cell.
            let location = disconnected_location.unwrap();
            let mut opened = false;
            // Shuffle the directions to randomize the direction we open.
            let mut shuffled_directions = ALL_DIRECTIONS.to_vec();
            shuffled_directions.shuffle(&mut rand::rng());
            for dir in shuffled_directions.iter() {
                if let Some(adjacent_location) = self.get_location(&location, dir) {
                    if visited_map[adjacent_location.y][adjacent_location.x] {
                        // Open the direction from the disconnected location to the adjacent connected location.
                        match dir {
                            Direction::North => {
                                self.cells[adjacent_location.y][adjacent_location.x]
                                    .can_exit_south = true;
                            }
                            Direction::East => {
                                self.cells[location.y][location.x].can_exit_east = true;
                            }
                            Direction::South => {
                                self.cells[location.y][location.x].can_exit_south = true;
                            }
                            Direction::West => {
                                self.cells[adjacent_location.y][adjacent_location.x]
                                    .can_exit_east = true;
                            }
                        }
                        opened = true;
                        break; // Exit the loop after opening one direction.
                    }
                }
            }
            if !opened {
                println!(
                    "Failed to open a direction for a disconnected cell at {} {}.",
                    location.x, location.y
                );
                break;
            }
        }
    }

    fn generate_visitable_map(&self) -> Vec<Vec<bool>> {
        let mut visited_map = vec![vec![false; self.size.width]; self.size.height];

        // Flood fill via DFS from the start cell.
        fn dfs(maze: &Maze, location: &Location, visited: &mut Vec<Vec<bool>>) {
            if visited[location.y][location.x] {
                return;
            }
            visited[location.y][location.x] = true;
            let cell = &maze.cells[location.y][location.x];
            for dir in ALL_DIRECTIONS.iter() {
                if maze.can_move(cell, dir) {
                    if let Some(next_location) = maze.get_location(&cell.location, dir) {
                        dfs(maze, &next_location, visited);
                    }
                }
            }
        }
        dfs(self, &Location { x: 0, y: 0 }, &mut visited_map);

        visited_map
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

    /**
     * Produce a solution path for the maze, if one exists.
     */
    fn solve(&self) -> Option<Path> {
        let start_cell = &self.cells[0][0];
        let mut path = Vec::new();
        if self.solve_helper(start_cell, &mut path) {
            Some(path)
        } else {
            None
        }
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
            .filter_map(|dir| {
                if self.can_move(cur_cell, dir) {
                    self.get_cell_in(cur_cell, dir)
                } else {
                    None
                }
            })
            .collect()
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
            Direction::North if cur_location.y >= 1 => Some(Location {
                x: cur_location.x,
                y: cur_location.y - 1,
            }),
            Direction::East if cur_location.x < self.size.width - 1 => Some(Location {
                x: cur_location.x + 1,
                y: cur_location.y,
            }),
            Direction::South if cur_location.y < self.size.height - 1 => Some(Location {
                x: cur_location.x,
                y: cur_location.y + 1,
            }),
            Direction::West if cur_location.x >= 1 => Some(Location {
                x: cur_location.x - 1,
                y: cur_location.y,
            }),
            _ => None,
        }
    }

    fn get_cell_in(&self, cur_cell: &Cell, direction: &Direction) -> Option<&Cell> {
        self.get_location(&cur_cell.location, direction)
            .map(|new_location| &self.cells[new_location.y][new_location.x])
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

    println!("{}", maze);

    let solution = maze.solve();
    println!("{solution:#?}");

    // TODO(eric): Print the maze with the solution path embedded
}
