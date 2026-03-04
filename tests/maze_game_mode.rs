use maze::game::{BLOCKED_MOVE_ERROR_DURATION, GameState, MoveDirection, MoveOutcome};
use maze::{Location, Maze};
use std::time::{Duration, Instant};

fn fixture_maze() -> Maze {
    let east_open = vec![vec![true, false], vec![false, false]];
    let south_open = vec![vec![true, true], vec![false, false]];
    Maze::from_topology(2, 2, east_open, south_open)
        .expect("fixture topology should be valid and connected")
}

#[test]
fn game_starts_at_start_cell() {
    let state = GameState::new();
    assert_eq!(state.position(), Location { x: 0, y: 0 });
    assert_eq!(state.trail(), [Location { x: 0, y: 0 }]);
}

#[test]
fn legal_move_advances_player_and_trail() {
    let maze = fixture_maze();
    let mut state = GameState::new();

    let now = Instant::now();
    let outcome = state.apply_move(&maze, MoveDirection::Right, now);
    assert_eq!(outcome, MoveOutcome::Moved);
    assert_eq!(state.position(), Location { x: 1, y: 0 });
    assert_eq!(
        state.trail(),
        [Location { x: 0, y: 0 }, Location { x: 1, y: 0 }]
    );
}

#[test]
fn moving_backwards_pops_the_trail() {
    let maze = fixture_maze();
    let mut state = GameState::new();
    let now = Instant::now();

    assert_eq!(
        state.apply_move(&maze, MoveDirection::Right, now),
        MoveOutcome::Moved
    );
    assert_eq!(
        state.apply_move(&maze, MoveDirection::Left, now + Duration::from_millis(10)),
        MoveOutcome::Moved
    );
    assert_eq!(state.position(), Location { x: 0, y: 0 });
    assert_eq!(state.trail(), [Location { x: 0, y: 0 }]);
}

#[test]
fn blocked_move_shows_temporary_error() {
    let maze = fixture_maze();
    let mut state = GameState::new();
    let now = Instant::now();

    let outcome = state.apply_move(&maze, MoveDirection::Up, now);
    assert_eq!(outcome, MoveOutcome::Blocked);
    assert_eq!(state.position(), Location { x: 0, y: 0 });
    assert!(state.is_blocked_error_visible(now + Duration::from_secs(1)));
    assert!(!state.is_blocked_error_visible(now + BLOCKED_MOVE_ERROR_DURATION));
}

#[test]
fn reaching_finish_returns_finished() {
    let maze = fixture_maze();
    let mut state = GameState::new();
    let now = Instant::now();

    assert_eq!(
        state.apply_move(&maze, MoveDirection::Right, now),
        MoveOutcome::Moved
    );
    assert_eq!(
        state.apply_move(&maze, MoveDirection::Down, now + Duration::from_secs(1)),
        MoveOutcome::Finished
    );
    assert!(state.is_finished(&maze));
    assert_eq!(state.position(), Location { x: 1, y: 1 });
}

#[test]
fn game_render_keeps_single_start_and_finish_markers() {
    let maze = fixture_maze();
    let mut state = GameState::new();
    let now = Instant::now();
    let _ = state.apply_move(&maze, MoveDirection::Right, now);
    let rendered = state.render(&maze, now + Duration::from_millis(50));

    assert_eq!(rendered.matches('S').count(), 1);
    assert_eq!(rendered.matches('F').count(), 1);
    assert_eq!(rendered.matches('@').count(), 1);
}

#[test]
fn game_render_includes_and_hides_error_indicator_by_time() {
    let maze = fixture_maze();
    let mut state = GameState::new();
    let now = Instant::now();
    let _ = state.apply_move(&maze, MoveDirection::Up, now);

    let with_error = state.render(&maze, now + Duration::from_millis(1));
    assert!(with_error.contains("Blocked by wall. Try a different direction."));

    let without_error = state.render(&maze, now + BLOCKED_MOVE_ERROR_DURATION);
    assert!(!without_error.contains("Blocked by wall. Try a different direction."));
}
