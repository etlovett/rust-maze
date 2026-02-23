use maze::{Location, Maze};
use std::collections::VecDeque;

fn neighbors(loc: Location) -> [Location; 4] {
    [
        Location {
            x: loc.x.saturating_sub(1),
            y: loc.y,
        },
        Location {
            x: loc.x + 1,
            y: loc.y,
        },
        Location {
            x: loc.x,
            y: loc.y.saturating_sub(1),
        },
        Location {
            x: loc.x,
            y: loc.y + 1,
        },
    ]
}

fn reachable_count(maze: &Maze) -> usize {
    let (width, height) = maze.size();
    let mut visited = vec![vec![false; width]; height];
    let mut queue = VecDeque::new();

    visited[0][0] = true;
    queue.push_back(Location { x: 0, y: 0 });

    while let Some(cur) = queue.pop_front() {
        for next in neighbors(cur) {
            if next.x >= width || next.y >= height {
                continue;
            }
            if visited[next.y][next.x] {
                continue;
            }
            if maze.can_move_between(cur, next) {
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
fn topology_api_size_matches_constructor_dimensions() {
    for (width, height) in [(2, 2), (3, 4), (7, 5)] {
        let maze = Maze::new(width, height);
        assert_eq!(maze.size(), (width, height));
    }
}

#[test]
fn can_move_between_rejects_out_of_bounds_and_invalid_moves() {
    let maze = Maze::new(4, 4);

    assert!(!maze.can_move_between(Location { x: 0, y: 0 }, Location { x: 0, y: 0 }));
    assert!(!maze.can_move_between(Location { x: 1, y: 1 }, Location { x: 2, y: 2 }));
    assert!(!maze.can_move_between(Location { x: 1, y: 1 }, Location { x: 3, y: 1 }));

    assert!(!maze.can_move_between(Location { x: 99, y: 0 }, Location { x: 0, y: 0 }));
    assert!(!maze.can_move_between(Location { x: 0, y: 0 }, Location { x: 99, y: 0 }));
    assert!(!maze.can_move_between(
        Location {
            x: usize::MAX,
            y: 0
        },
        Location { x: 0, y: 0 },
    ));
}

#[test]
fn can_move_between_rejects_non_adjacent_vertical_jump() {
    let maze = Maze::new(4, 4);
    assert!(!maze.can_move_between(Location { x: 1, y: 0 }, Location { x: 1, y: 2 }));
}

#[test]
fn can_move_between_is_symmetric_for_adjacent_cells() {
    let maze = Maze::new(8, 6);
    let (width, height) = maze.size();

    for y in 0..height {
        for x in 0..width {
            let from = Location { x, y };

            let adjacent = [
                Location {
                    x: x.saturating_sub(1),
                    y,
                },
                Location { x: x + 1, y },
                Location {
                    x,
                    y: y.saturating_sub(1),
                },
                Location { x, y: y + 1 },
            ];

            for to in adjacent {
                if to.x >= width || to.y >= height {
                    continue;
                }
                assert_eq!(
                    maze.can_move_between(from, to),
                    maze.can_move_between(to, from),
                    "can_move_between should be symmetric for adjacent cells: {:?} <-> {:?}",
                    from,
                    to
                );
            }
        }
    }
}

#[test]
fn topology_api_validates_boundaries_and_connectivity() {
    for (width, height) in [(2, 2), (4, 4), (8, 6)] {
        let maze = Maze::new(width, height);

        for y in 0..height {
            assert!(
                !maze.can_move_between(Location { x: width - 1, y }, Location { x: width, y },),
                "right boundary must reject out-of-bounds movement"
            );
        }

        for x in 0..width {
            assert!(
                !maze.can_move_between(Location { x, y: height - 1 }, Location { x, y: height },),
                "bottom boundary must reject out-of-bounds movement"
            );
        }

        assert_eq!(
            reachable_count(&maze),
            width * height,
            "maze should be fully connected from start"
        );
    }
}

#[test]
fn solver_path_is_valid_against_topology_api() {
    for (width, height) in [(2, 2), (5, 4), (8, 6)] {
        let maze = Maze::new(width, height);
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
                maze.can_move_between(a, b),
                "path step crosses blocked edge: {:?} -> {:?}",
                a,
                b
            );
        }

        for (idx, point) in path.iter().enumerate() {
            assert!(
                !path[..idx].contains(point),
                "path should not contain duplicate locations: {:?}",
                point
            );
        }
    }
}
