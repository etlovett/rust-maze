use std::collections::VecDeque;
use std::io::Write;
use std::process::{Command, Stdio};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct Location {
    x: usize,
    y: usize,
}

#[derive(Debug)]
struct MazeRender {
    lines: Vec<String>,
    east_open: Vec<Vec<bool>>,
    south_open: Vec<Vec<bool>>,
}

fn run_maze(width: usize, height: usize) -> String {
    run_maze_with_input(&format!("{width}\n{height}\nno\n"))
}

fn run_maze_with_solution_answer(width: usize, height: usize, answer: &str) -> String {
    run_maze_with_input(&format!("{width}\n{height}\n{answer}\n"))
}

fn run_maze_with_input(input: &str) -> String {
    let bin = env!("CARGO_BIN_EXE_maze");
    let mut child = Command::new(bin)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .expect("failed to spawn maze binary");

    {
        let stdin = child.stdin.as_mut().expect("failed to get stdin");
        write!(stdin, "{input}").expect("failed to write stdin");
    }

    let output = child
        .wait_with_output()
        .expect("failed to wait for maze process");

    assert!(
        output.status.success(),
        "maze process failed\nstdout:\n{}\nstderr:\n{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );

    String::from_utf8(output.stdout).expect("stdout was not valid UTF-8")
}

fn parse_maze_lines(stdout: &str) -> Vec<String> {
    let mut lines = Vec::new();
    let mut in_maze = false;

    for line in stdout.lines() {
        if line.starts_with('╔') {
            in_maze = true;
        }
        if in_maze {
            lines.push(line.to_string());
            if line.starts_with('╚') {
                break;
            }
        }
    }

    assert!(
        !lines.is_empty(),
        "failed to find maze in stdout:\n{stdout}"
    );
    lines
}

fn has_solution_overlay(stdout: &str) -> bool {
    stdout.chars().any(|c| {
        matches!(
            c,
            '─' | '│' | '┌' | '┐' | '└' | '┘' | '├' | '┤' | '┬' | '┴' | '┼'
        )
    })
}

fn char_at(line: &str, index: usize) -> char {
    line.chars()
        .nth(index)
        .expect("line shorter than expected while parsing maze")
}

fn parse_render(width: usize, height: usize, stdout: &str) -> MazeRender {
    let lines = parse_maze_lines(stdout);
    let expected_lines = 1 + height * 3;
    assert_eq!(
        lines.len(),
        expected_lines,
        "unexpected rendered maze height\n{}",
        lines.join("\n")
    );

    let expected_line_len = 1 + width * 6;
    for line in &lines {
        assert_eq!(
            line.chars().count(),
            expected_line_len,
            "unexpected rendered maze line width for line: {line}"
        );
    }

    let mut east_open = vec![vec![false; width]; height];
    let mut south_open = vec![vec![false; width]; height];

    for y in 0..height {
        let body_line = &lines[1 + y * 3];
        let border_line = &lines[3 + y * 3];

        for x in 0..width {
            let east_sep_idx = 6 + x * 6;
            east_open[y][x] = char_at(body_line, east_sep_idx) == ' ';

            let segment_start = 1 + x * 6;
            let segment: String = border_line.chars().skip(segment_start).take(5).collect();
            south_open[y][x] = segment == "     ";
        }
    }

    MazeRender {
        lines,
        east_open,
        south_open,
    }
}

fn can_move(render: &MazeRender, from: Location, to: Location) -> bool {
    if from.x == to.x && from.y + 1 == to.y {
        return render.south_open[from.y][from.x];
    }
    if from.x == to.x && from.y == to.y + 1 {
        return render.south_open[to.y][to.x];
    }
    if from.y == to.y && from.x + 1 == to.x {
        return render.east_open[from.y][from.x];
    }
    if from.y == to.y && from.x == to.x + 1 {
        return render.east_open[to.y][to.x];
    }

    false
}

fn reachable_count(render: &MazeRender, width: usize, height: usize) -> usize {
    let mut visited = vec![vec![false; width]; height];
    let mut queue = VecDeque::new();

    visited[0][0] = true;
    queue.push_back(Location { x: 0, y: 0 });

    while let Some(cur) = queue.pop_front() {
        let neighbors = [
            Location {
                x: cur.x.saturating_sub(1),
                y: cur.y,
            },
            Location {
                x: cur.x + 1,
                y: cur.y,
            },
            Location {
                x: cur.x,
                y: cur.y.saturating_sub(1),
            },
            Location {
                x: cur.x,
                y: cur.y + 1,
            },
        ];

        for next in neighbors {
            if next.x >= width || next.y >= height {
                continue;
            }
            if visited[next.y][next.x] {
                continue;
            }
            if can_move(render, cur, next) {
                visited[next.y][next.x] = true;
                queue.push_back(next);
            }
        }
    }

    visited
        .iter()
        .map(|row| row.iter().filter(|&&v| v).count())
        .sum()
}

#[test]
fn rendering_invariants_hold_for_multiple_sizes() {
    for (width, height) in [(2, 2), (3, 4), (5, 3), (6, 6)] {
        let stdout = run_maze(width, height);
        let render = parse_render(width, height, &stdout);
        let maze_text = render.lines.join("\n");

        assert_eq!(maze_text.matches('S').count(), 2);
        assert_eq!(maze_text.matches('F').count(), 2);
        assert_eq!(render.lines[0].chars().next(), Some('╔'));
        assert_eq!(render.lines[0].chars().last(), Some('╗'));
        assert_eq!(
            render.lines.last().and_then(|l| l.chars().next()),
            Some('╚')
        );
        assert_eq!(
            render.lines.last().and_then(|l| l.chars().last()),
            Some('╝')
        );
    }
}

#[test]
fn movement_boundaries_and_connectivity_are_valid() {
    for (width, height) in [(2, 2), (4, 4), (7, 5)] {
        let stdout = run_maze(width, height);
        let render = parse_render(width, height, &stdout);

        for y in 0..height {
            assert!(
                !render.east_open[y][width - 1],
                "right boundary must be closed at row {y}"
            );
        }
        for x in 0..width {
            assert!(
                !render.south_open[height - 1][x],
                "bottom boundary must be closed at column {x}"
            );
        }

        assert_eq!(
            reachable_count(&render, width, height),
            width * height,
            "maze should be fully connected from start"
        );
    }
}

#[test]
fn solver_path_is_bounded_adjacent_and_traversable() {
    for (width, height) in [(2, 2), (5, 4), (8, 6)] {
        let stdout = run_maze_with_solution_answer(width, height, "yes");
        assert!(has_solution_overlay(&stdout));
        assert!(!stdout.contains("Some("));
    }
}

#[test]
fn cli_reprompts_after_non_numeric_width_input() {
    let stdout = run_maze_with_input("abc\n2\n2\ny\n");

    assert!(stdout.contains("Please enter a valid integer >=2."));
    assert_eq!(stdout.matches("Please enter the width.").count(), 2);
    assert_eq!(stdout.matches("Please enter the height.").count(), 1);
    assert_eq!(stdout.matches("Print solution? (y/yes/n/no)").count(), 1);
    assert!(stdout.contains("Creating a maze of size 2x2."));
    assert!(has_solution_overlay(&stdout));
}

#[test]
fn cli_reprompts_after_too_small_height_input() {
    let stdout = run_maze_with_input("2\n1\n2\ny\n");

    assert!(stdout.contains("Please enter a valid integer >=2."));
    assert_eq!(stdout.matches("Please enter the width.").count(), 1);
    assert_eq!(stdout.matches("Please enter the height.").count(), 2);
    assert_eq!(stdout.matches("Print solution? (y/yes/n/no)").count(), 1);
    assert!(stdout.contains("Creating a maze of size 2x2."));
    assert!(has_solution_overlay(&stdout));
}

#[test]
fn cli_reprompts_after_too_small_width_input() {
    let stdout = run_maze_with_input("1\n2\n2\ny\n");

    assert!(stdout.contains("Please enter a valid integer >=2."));
    assert_eq!(stdout.matches("Please enter the width.").count(), 2);
    assert_eq!(stdout.matches("Please enter the height.").count(), 1);
    assert_eq!(stdout.matches("Print solution? (y/yes/n/no)").count(), 1);
    assert!(stdout.contains("Creating a maze of size 2x2."));
    assert!(has_solution_overlay(&stdout));
}

#[test]
fn cli_prints_solution_for_affirmative_answer() {
    let stdout = run_maze_with_input("2\n2\nyes\n");

    assert_eq!(stdout.matches("Print solution? (y/yes/n/no)").count(), 1);
    assert!(has_solution_overlay(&stdout));
    assert!(!stdout.contains("Some("));
}

#[test]
fn cli_does_not_print_solution_for_negative_answer() {
    let stdout = run_maze_with_input("2\n2\nn\n");

    assert_eq!(stdout.matches("Print solution? (y/yes/n/no)").count(), 1);
    assert!(!has_solution_overlay(&stdout));
    assert!(!stdout.contains("Some("));
}

#[test]
fn cli_reprompts_after_invalid_solution_answer() {
    let stdout = run_maze_with_input("2\n2\nmaybe\nno\n");

    assert_eq!(stdout.matches("Print solution? (y/yes/n/no)").count(), 2);
    assert!(stdout.contains("Please enter y, yes, n, or no."));
    assert!(!has_solution_overlay(&stdout));
    assert!(!stdout.contains("Some("));
}

#[test]
fn cli_prints_solution_for_short_affirmative_answer() {
    let stdout = run_maze_with_input("2\n2\ny\n");

    assert_eq!(stdout.matches("Print solution? (y/yes/n/no)").count(), 1);
    assert!(has_solution_overlay(&stdout));
    assert!(!stdout.contains("Some("));
}

#[test]
fn cli_does_not_print_solution_for_full_negative_answer() {
    let stdout = run_maze_with_input("2\n2\nno\n");

    assert_eq!(stdout.matches("Print solution? (y/yes/n/no)").count(), 1);
    assert!(!has_solution_overlay(&stdout));
    assert!(!stdout.contains("Some("));
}

#[test]
fn cli_accepts_mixed_case_affirmative_answer() {
    let stdout = run_maze_with_input("2\n2\nyEs\n");

    assert_eq!(stdout.matches("Print solution? (y/yes/n/no)").count(), 1);
    assert!(has_solution_overlay(&stdout));
    assert!(!stdout.contains("Some("));
}

#[test]
fn cli_accepts_mixed_case_negative_answer() {
    let stdout = run_maze_with_input("2\n2\nnO\n");

    assert_eq!(stdout.matches("Print solution? (y/yes/n/no)").count(), 1);
    assert!(!has_solution_overlay(&stdout));
    assert!(!stdout.contains("Some("));
}
