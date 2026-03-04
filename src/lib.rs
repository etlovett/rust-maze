use rand::random_bool;
use rand::seq::SliceRandom;
use std::fmt;

pub mod game;

pub type Size = usize;

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

const CELL_WALL: &str = "═════";
const CELL_OPENING: &str = "     ";
const TOP_LEFT_CORNER: char = '╔';
const TOP_RIGHT_CORNER: char = '╗';
const TOP_T_JUNCTION: char = '╦';
const LEFT_T_JUNCTION: char = '╠';
const RIGHT_T_JUNCTION: char = '╣';
const BOTTOM_T_JUNCTION: char = '╩';
const CENTER_JUNCTION: char = '╬';
const LEFT_WALL: char = '║';
const RIGHT_WALL: char = '║';
const BOTTOM_LEFT_CORNER: char = '╚';
const BOTTOM_RIGHT_CORNER: char = '╝';

#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub struct Location {
    pub x: Size,
    pub y: Size,
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
pub struct Maze {
    size: MazeSize,
    cells: Vec<Vec<Cell>>,
}

pub type Path = Vec<Location>;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum FromTopologyError {
    InvalidDimensions,
    EastOpenHeightMismatch,
    SouthOpenHeightMismatch,
    EastOpenRowWidthMismatch { row: usize, expected: usize },
    SouthOpenRowWidthMismatch { row: usize, expected: usize },
    EastOpenRightBoundaryOpen { row: usize },
    SouthOpenBottomBoundaryOpen { column: usize },
    NotFullyConnected,
}

impl fmt::Display for FromTopologyError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            FromTopologyError::InvalidDimensions => {
                write!(f, "width and height must be >= 2")
            }
            FromTopologyError::EastOpenHeightMismatch => {
                write!(f, "east_open must have one row per maze row")
            }
            FromTopologyError::SouthOpenHeightMismatch => {
                write!(f, "south_open must have one row per maze row")
            }
            FromTopologyError::EastOpenRowWidthMismatch { row, expected } => {
                write!(f, "east_open row {row} must have length {expected}")
            }
            FromTopologyError::SouthOpenRowWidthMismatch { row, expected } => {
                write!(f, "south_open row {row} must have length {expected}")
            }
            FromTopologyError::EastOpenRightBoundaryOpen { row } => {
                write!(f, "east_open right boundary must be closed at row {row}")
            }
            FromTopologyError::SouthOpenBottomBoundaryOpen { column } => write!(
                f,
                "south_open bottom boundary must be closed at column {column}"
            ),
            FromTopologyError::NotFullyConnected => {
                write!(f, "maze must be fully connected from start")
            }
        }
    }
}

impl Cell {
    /// Generate a new cell.
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
    /// Generate a new valid maze.
    pub fn new(width: Size, height: Size) -> Maze {
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

    pub fn from_topology(
        width: usize,
        height: usize,
        east_open: Vec<Vec<bool>>,
        south_open: Vec<Vec<bool>>,
    ) -> Result<Maze, FromTopologyError> {
        if width < 2 || height < 2 {
            return Err(FromTopologyError::InvalidDimensions);
        }

        if east_open.len() != height {
            return Err(FromTopologyError::EastOpenHeightMismatch);
        }
        if south_open.len() != height {
            return Err(FromTopologyError::SouthOpenHeightMismatch);
        }

        for (y, row) in east_open.iter().enumerate() {
            if row.len() != width {
                return Err(FromTopologyError::EastOpenRowWidthMismatch {
                    row: y,
                    expected: width,
                });
            }
        }
        for (y, row) in south_open.iter().enumerate() {
            if row.len() != width {
                return Err(FromTopologyError::SouthOpenRowWidthMismatch {
                    row: y,
                    expected: width,
                });
            }
        }

        for (y, row) in east_open.iter().enumerate() {
            if row[width - 1] {
                return Err(FromTopologyError::EastOpenRightBoundaryOpen { row: y });
            }
        }
        for x in 0..width {
            if south_open[height - 1][x] {
                return Err(FromTopologyError::SouthOpenBottomBoundaryOpen { column: x });
            }
        }

        let maze_size = MazeSize { width, height };
        let cells = (0..height)
            .map(|y| {
                (0..width)
                    .map(|x| Cell {
                        maze_size,
                        location: Location { x, y },
                        can_exit_south: south_open[y][x],
                        can_exit_east: east_open[y][x],
                    })
                    .collect::<Vec<_>>()
            })
            .collect::<Vec<_>>();

        let maze = Maze {
            size: maze_size,
            cells,
        };

        if !maze.is_fully_connected() {
            return Err(FromTopologyError::NotFullyConnected);
        }

        Ok(maze)
    }

    pub fn size(&self) -> (usize, usize) {
        (self.size.width, self.size.height)
    }

    pub fn can_move_between(&self, from: Location, to: Location) -> bool {
        if !self.is_in_bounds(from) || !self.is_in_bounds(to) {
            return false;
        }

        if from.x == to.x && from.y + 1 == to.y {
            return self.can_move(&self.cells[from.y][from.x], &Direction::South);
        }
        if from.x == to.x && from.y == to.y + 1 {
            return self.can_move(&self.cells[from.y][from.x], &Direction::North);
        }
        if from.y == to.y && from.x + 1 == to.x {
            return self.can_move(&self.cells[from.y][from.x], &Direction::East);
        }
        if from.y == to.y && from.x == to.x + 1 {
            return self.can_move(&self.cells[from.y][from.x], &Direction::West);
        }

        false
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
            let location = disconnected_location.expect("checked is_some above");
            let mut opened = false;
            // Shuffle the directions to randomize the direction we open.
            let mut shuffled_directions = ALL_DIRECTIONS.to_vec();
            shuffled_directions.shuffle(&mut rand::rng());
            for dir in &shuffled_directions {
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
            for dir in &ALL_DIRECTIONS {
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

    fn is_fully_connected(&self) -> bool {
        self.generate_visitable_map()
            .iter()
            .flatten()
            .all(|&is_visited| is_visited)
    }

    fn as_str(&self) -> String {
        self.as_str_with_solution(None)
    }

    pub fn render_with_solution(&self, path: &Path) -> String {
        self.as_str_with_solution(Some(path))
    }

    pub fn render_game(&self, path: &Path, player: Location, show_error: bool) -> String {
        let with_path = self.render_with_solution(path);
        let mut rows = with_path
            .lines()
            .map(|line| line.chars().collect::<Vec<_>>())
            .collect::<Vec<_>>();

        let start = Location { x: 0, y: 0 };
        let finish = Location {
            x: self.size.width - 1,
            y: self.size.height - 1,
        };
        self.clear_lower_marker_copy(&mut rows, start);
        self.clear_lower_marker_copy(&mut rows, finish);

        if self.is_in_bounds(player) {
            let (center_row, center_col) = self.get_render_center(player);
            let player_row = center_row + 1;
            if player_row < rows.len() && center_col < rows[player_row].len() {
                rows[player_row][center_col] = '@';
            }
        }

        let mut output = String::new();
        for row in rows {
            for c in row {
                output.push(c);
            }
            output.push('\n');
        }

        if show_error {
            output.push_str("Blocked by wall. Try a different direction.\n");
        }

        output
    }

    fn clear_lower_marker_copy(&self, rows: &mut [Vec<char>], location: Location) {
        if !self.is_in_bounds(location) {
            return;
        }

        let (center_row, center_col) = self.get_render_center(location);
        let lower_row = center_row + 1;
        if lower_row < rows.len()
            && center_col < rows[lower_row].len()
            && matches!(rows[lower_row][center_col], 'S' | 'F')
        {
            rows[lower_row][center_col] = ' ';
        }
    }

    fn as_str_with_solution(&self, path: Option<&Path>) -> String {
        let base_str = self.base_as_str();
        let mut rows = base_str
            .lines()
            .map(|line| line.chars().collect::<Vec<_>>())
            .collect::<Vec<_>>();

        if let Some(path) = path {
            self.overlay_solution_path(&mut rows, path);
        }

        let mut output = String::new();
        for row in rows {
            for c in row {
                output.push(c);
            }
            output.push('\n');
        }

        output
    }

    fn base_as_str(&self) -> String {
        let mut maze_str = String::new();

        // Generate first line border.
        maze_str.push(TOP_LEFT_CORNER);
        for i in 0..self.size.width {
            let is_last_cell = i == self.size.width - 1;

            maze_str.push_str(CELL_WALL);
            maze_str.push(if is_last_cell {
                TOP_RIGHT_CORNER
            } else {
                TOP_T_JUNCTION
            });
        }
        maze_str.push('\n');

        for (i, row) in self.cells.iter().enumerate() {
            let is_last_row = i == self.cells.len() - 1;

            // Generate the body of each cell for the row, including left and right borders.
            let mut row_str = String::new();
            row_str.push(LEFT_WALL);
            for cell in row {
                row_str.push_str(&format!("  {}  ", cell.get_center_char()));
                row_str.push(if cell.can_exit_east { ' ' } else { RIGHT_WALL });
            }
            row_str.push('\n');
            maze_str.push_str(&row_str);
            maze_str.push_str(&row_str);

            // Generate the border below the cell.
            maze_str.push(if is_last_row {
                BOTTOM_LEFT_CORNER
            } else {
                LEFT_T_JUNCTION
            });
            for cell in row {
                maze_str.push_str(if cell.can_exit_south {
                    CELL_OPENING
                } else {
                    CELL_WALL
                });
                maze_str.push(if cell.is_last_in_row() && cell.is_last_in_col() {
                    BOTTOM_RIGHT_CORNER
                } else if cell.is_last_in_row() {
                    RIGHT_T_JUNCTION
                } else if cell.is_last_in_col() {
                    BOTTOM_T_JUNCTION
                } else {
                    CENTER_JUNCTION
                });
            }
            maze_str.push('\n');
        }

        maze_str
    }

    fn overlay_solution_path(&self, rows: &mut [Vec<char>], path: &Path) {
        if path.is_empty() {
            return;
        }

        for window in path.windows(2) {
            let from = window[0];
            let to = window[1];
            self.draw_solution_segment(rows, from, to);
        }

        for (idx, location) in path.iter().enumerate() {
            if !self.is_in_bounds(*location) {
                continue;
            }

            let (center_row, center_col) = self.get_render_center(*location);
            if center_row + 1 < rows.len() && center_col < rows[center_row + 1].len() {
                // Remove the duplicated S/F marker from the lower body row for a cleaner overlay.
                if matches!(rows[center_row + 1][center_col], 'S' | 'F') {
                    rows[center_row + 1][center_col] = ' ';
                }
            }

            let mut connections = 0u8;
            if idx > 0 {
                if let Some(direction) = Self::direction_between(*location, path[idx - 1]) {
                    connections |= Self::direction_mask(direction);
                }
            }
            if idx + 1 < path.len() {
                if let Some(direction) = Self::direction_between(*location, path[idx + 1]) {
                    connections |= Self::direction_mask(direction);
                }
            }

            rows[center_row][center_col] = if self.is_start_location(*location) {
                'S'
            } else if self.is_finish_location(*location) {
                'F'
            } else {
                Self::path_char_for_connections(connections)
            };
        }
    }

    fn draw_solution_segment(&self, rows: &mut [Vec<char>], from: Location, to: Location) {
        let (from_row, from_col) = self.get_render_center(from);
        let (to_row, to_col) = self.get_render_center(to);

        if from_row == to_row {
            let start = from_col.min(to_col) + 1;
            let end = from_col.max(to_col);
            for col in start..end {
                rows[from_row][col] = '─';
            }
        } else if from_col == to_col {
            let start = from_row.min(to_row) + 1;
            let end = from_row.max(to_row);
            for row in rows.iter_mut().take(end).skip(start) {
                row[from_col] = '│';
            }
        }
    }

    fn get_render_center(&self, location: Location) -> (usize, usize) {
        let row = 1 + location.y * 3;
        let col = 3 + location.x * 6;
        (row, col)
    }

    fn direction_between(from: Location, to: Location) -> Option<Direction> {
        if from.x == to.x && from.y + 1 == to.y {
            Some(Direction::South)
        } else if from.x == to.x && from.y == to.y + 1 {
            Some(Direction::North)
        } else if from.y == to.y && from.x + 1 == to.x {
            Some(Direction::East)
        } else if from.y == to.y && from.x == to.x + 1 {
            Some(Direction::West)
        } else {
            None
        }
    }

    fn direction_mask(direction: Direction) -> u8 {
        match direction {
            Direction::North => 0b0001,
            Direction::East => 0b0010,
            Direction::South => 0b0100,
            Direction::West => 0b1000,
        }
    }

    fn path_char_for_connections(mask: u8) -> char {
        match mask {
            0b0001 | 0b0100 | 0b0101 => '│',
            0b0010 | 0b1000 | 0b1010 => '─',
            0b0011 => '└',
            0b1001 => '┘',
            0b0110 => '┌',
            0b1100 => '┐',
            0b0111 => '├',
            0b1101 => '┤',
            0b1011 => '┴',
            0b1110 => '┬',
            0b1111 => '┼',
            _ => '•',
        }
    }

    /// Produce a solution path for the maze, if one exists.
    pub fn solve(&self) -> Option<Path> {
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

    fn is_in_bounds(&self, location: Location) -> bool {
        location.x < self.size.width && location.y < self.size.height
    }

    fn is_start_location(&self, location: Location) -> bool {
        location == Location { x: 0, y: 0 }
    }

    fn is_finish_location(&self, location: Location) -> bool {
        location
            == Location {
                x: self.size.width - 1,
                y: self.size.height - 1,
            }
    }
}

impl fmt::Display for Maze {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.as_str())
    }
}
