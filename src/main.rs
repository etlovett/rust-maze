use crossterm::cursor::{Hide, MoveTo, Show};
use crossterm::event::{self, Event, KeyCode, KeyEventKind};
use crossterm::execute;
use crossterm::terminal::{
    Clear, ClearType, EnterAlternateScreen, LeaveAlternateScreen, disable_raw_mode, enable_raw_mode,
};
use maze::game::{Clock, GameState, InputSource, MoveDirection, MoveOutcome, SystemClock};
use maze::{Maze, Size};
use std::io::{self, IsTerminal, Write};
use std::time::Duration;

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

struct CrosstermInput;

impl InputSource for CrosstermInput {
    fn next_direction(&mut self, timeout: Duration) -> io::Result<Option<MoveDirection>> {
        if !event::poll(timeout)? {
            return Ok(None);
        }

        let event = event::read()?;
        let Event::Key(key_event) = event else {
            return Ok(None);
        };

        if key_event.kind != KeyEventKind::Press {
            return Ok(None);
        }

        let direction = match key_event.code {
            KeyCode::Up => Some(MoveDirection::Up),
            KeyCode::Right => Some(MoveDirection::Right),
            KeyCode::Down => Some(MoveDirection::Down),
            KeyCode::Left => Some(MoveDirection::Left),
            _ => None,
        };

        Ok(direction)
    }
}

struct TerminalGuard;

impl TerminalGuard {
    fn enter() -> io::Result<Self> {
        enable_raw_mode()?;
        execute!(io::stdout(), EnterAlternateScreen, Hide)?;
        Ok(Self)
    }
}

impl Drop for TerminalGuard {
    fn drop(&mut self) {
        let _ = execute!(io::stdout(), Show, LeaveAlternateScreen);
        let _ = disable_raw_mode();
    }
}

fn draw_frame<W: Write>(writer: &mut W, frame: &str) -> io::Result<()> {
    execute!(writer, MoveTo(0, 0), Clear(ClearType::All))?;
    write!(writer, "{frame}")?;
    writer.flush()?;
    Ok(())
}

fn run_game_loop<W: Write, I: InputSource, C: Clock>(
    writer: &mut W,
    maze: &Maze,
    input: &mut I,
    clock: &C,
) -> io::Result<()> {
    let mut state = GameState::new();

    loop {
        let now = clock.now();
        draw_frame(writer, &state.render(maze, now))?;

        if state.is_finished(maze) {
            break;
        }

        if let Some(direction) = input.next_direction(Duration::from_millis(50))? {
            let now = clock.now();
            let move_result = state.apply_move(maze, direction, now);
            if move_result == MoveOutcome::Finished {
                draw_frame(writer, &state.render(maze, now))?;
                break;
            }
        }
    }

    Ok(())
}

fn run_interactive_game(maze: &Maze) -> io::Result<()> {
    let _guard = TerminalGuard::enter()?;
    let mut stdout = io::stdout();
    let mut input = CrosstermInput;
    let clock = SystemClock;
    run_game_loop(&mut stdout, maze, &mut input, &clock)
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
    } else if io::stdin().is_terminal() && io::stdout().is_terminal() {
        if let Err(err) = run_interactive_game(&maze) {
            eprintln!("Failed to run interactive game: {err}");
            std::process::exit(1);
        }
    } else {
        println!("{}", maze);
    }

    // TODO(eric): Print the maze with the solution path embedded
}
