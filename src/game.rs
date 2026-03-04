use crate::{Location, Maze, Path};
use std::io;
use std::time::{Duration, Instant};

pub const BLOCKED_MOVE_ERROR_DURATION: Duration = Duration::from_secs(3);

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum MoveDirection {
    Up,
    Right,
    Down,
    Left,
}

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum GameInput {
    Move(MoveDirection),
    Quit,
}

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum MoveOutcome {
    Moved,
    Blocked,
    Finished,
}

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum InputOutcome {
    Move(MoveOutcome),
    Quit,
}

pub trait Clock {
    fn now(&self) -> Instant;
}

#[derive(Debug, Copy, Clone)]
pub struct SystemClock;

impl Clock for SystemClock {
    fn now(&self) -> Instant {
        Instant::now()
    }
}

pub trait InputSource {
    fn next_input(&mut self, timeout: Duration) -> io::Result<Option<GameInput>>;
}

#[derive(Debug, Clone)]
pub struct GameState {
    trail: Path,
    position: Location,
    blocked_until: Option<Instant>,
}

impl Default for GameState {
    fn default() -> Self {
        Self::new()
    }
}

impl GameState {
    pub fn new() -> Self {
        let start = Location { x: 0, y: 0 };
        Self {
            trail: vec![start],
            position: start,
            blocked_until: None,
        }
    }

    pub fn position(&self) -> Location {
        self.position
    }

    pub fn trail(&self) -> &[Location] {
        &self.trail
    }

    pub fn is_finished(&self, maze: &Maze) -> bool {
        let (width, height) = maze.size();
        self.position
            == Location {
                x: width - 1,
                y: height - 1,
            }
    }

    pub fn is_blocked_error_visible(&self, now: Instant) -> bool {
        self.blocked_until.map(|until| now < until).unwrap_or(false)
    }

    pub fn apply_move(
        &mut self,
        maze: &Maze,
        direction: MoveDirection,
        now: Instant,
    ) -> MoveOutcome {
        let Some(next_position) = self.next_position(maze, direction) else {
            self.blocked_until = Some(now + BLOCKED_MOVE_ERROR_DURATION);
            return MoveOutcome::Blocked;
        };

        self.position = next_position;
        self.blocked_until = None;

        if self.trail.len() >= 2 && next_position == self.trail[self.trail.len() - 2] {
            self.trail.pop();
        } else {
            self.trail.push(next_position);
        }

        if self.is_finished(maze) {
            MoveOutcome::Finished
        } else {
            MoveOutcome::Moved
        }
    }

    pub fn apply_input(&mut self, maze: &Maze, input: GameInput, now: Instant) -> InputOutcome {
        match input {
            GameInput::Quit => InputOutcome::Quit,
            GameInput::Move(direction) => InputOutcome::Move(self.apply_move(maze, direction, now)),
        }
    }

    pub fn render(&self, maze: &Maze, now: Instant) -> String {
        maze.render_game(
            &self.trail,
            self.position,
            self.is_blocked_error_visible(now),
        )
    }

    fn next_position(&self, maze: &Maze, direction: MoveDirection) -> Option<Location> {
        let candidate = match direction {
            MoveDirection::Up if self.position.y > 0 => Location {
                x: self.position.x,
                y: self.position.y - 1,
            },
            MoveDirection::Right => Location {
                x: self.position.x + 1,
                y: self.position.y,
            },
            MoveDirection::Down => Location {
                x: self.position.x,
                y: self.position.y + 1,
            },
            MoveDirection::Left if self.position.x > 0 => Location {
                x: self.position.x - 1,
                y: self.position.y,
            },
            _ => return None,
        };

        if maze.can_move_between(self.position, candidate) {
            Some(candidate)
        } else {
            None
        }
    }
}
