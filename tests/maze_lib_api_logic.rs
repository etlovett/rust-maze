use maze::{Location, Maze};
use std::collections::VecDeque;

#[derive(Debug)]
struct MazeRender {
    east_open: Vec<Vec<bool>>,
    south_open: Vec<Vec<bool>>,
}

fn char_at(line: &str, index: usize) -> char {
    line.chars()
        .nth(index)
        .expect("line shorter than expected while parsing maze")
}

fn parse_render(width: usize, height: usize, maze_text: &str) -> MazeRender {
    let lines: Vec<String> = maze_text.lines().map(str::to_string).collect();
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
fn top_level_maze_generation_preserves_boundaries_and_connectivity() {
    for (width, height) in [(2, 2), (4, 4), (7, 5)] {
        let maze = Maze::new(width, height);
        let maze_text = maze.to_string();
        let render = parse_render(width, height, &maze_text);

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
fn top_level_solver_returns_a_valid_traversable_path() {
    for (width, height) in [(2, 2), (5, 4), (8, 6)] {
        let maze = Maze::new(width, height);
        let maze_text = maze.to_string();
        let render = parse_render(width, height, &maze_text);
        let path = maze.solve().expect("solver should return a path");

        assert_eq!(path.first().copied(), Some(Location { x: 0, y: 0 }));
        assert_eq!(
            path.last().copied(),
            Some(Location {
                x: width - 1,
                y: height - 1,
            })
        );

        for p in &path {
            assert!(p.x < width, "path x out of bounds: {:?}", p);
            assert!(p.y < height, "path y out of bounds: {:?}", p);
        }

        for window in path.windows(2) {
            let a = window[0];
            let b = window[1];
            let manhattan = a.x.abs_diff(b.x) + a.y.abs_diff(b.y);
            assert_eq!(
                manhattan, 1,
                "non-adjacent step in path: {:?} -> {:?}",
                a, b
            );
            assert!(
                can_move(&render, a, b),
                "path step crosses a wall: {:?} -> {:?}",
                a,
                b
            );
        }
    }
}
