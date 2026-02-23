use maze::{Location, Maze};

#[test]
fn from_topology_renders_exact_output_for_2x2_fixture() {
    let east_open = vec![vec![true, false], vec![false, false]];
    let south_open = vec![vec![true, true], vec![false, false]];

    let maze = Maze::from_topology(2, 2, east_open, south_open)
        .expect("fixture topology should be valid and connected");

    let expected = concat!(
        "╔═════╦═════╗\n",
        "║  S        ║\n",
        "║  S        ║\n",
        "╠     ╬     ╣\n",
        "║     ║  F  ║\n",
        "║     ║  F  ║\n",
        "╚═════╩═════╝\n"
    );

    assert_eq!(maze.to_string(), expected);
}

#[test]
fn from_topology_places_start_and_finish_markers() {
    let east_open = vec![vec![true, true, false], vec![true, true, false]];
    let south_open = vec![vec![true, false, false], vec![false, false, false]];

    let maze = Maze::from_topology(3, 2, east_open, south_open)
        .expect("fixture topology should be valid and connected");

    let rendered = maze.to_string();

    assert_eq!(rendered.matches('S').count(), 2);
    assert_eq!(rendered.matches('F').count(), 2);
}

#[test]
fn from_topology_rejects_bad_dimensions() {
    let err = Maze::from_topology(
        1,
        2,
        vec![vec![false], vec![false]],
        vec![vec![false], vec![false]],
    )
    .expect_err("width < 2 should be rejected");
    assert!(err.contains("width and height must be >= 2"));
}

#[test]
fn from_topology_rejects_height_below_minimum() {
    let err = Maze::from_topology(2, 1, vec![vec![false, false]], vec![vec![false, false]])
        .expect_err("height < 2 should be rejected");
    assert!(err.contains("width and height must be >= 2"));
}

#[test]
fn from_topology_rejects_matrix_shape_mismatch() {
    let err = Maze::from_topology(
        3,
        2,
        vec![vec![false, false, false]],
        vec![vec![false, false, false], vec![false, false, false]],
    )
    .expect_err("east_open with wrong height should be rejected");
    assert!(err.contains("east_open must have one row per maze row"));

    let err = Maze::from_topology(
        3,
        2,
        vec![vec![false, false], vec![false, false, false]],
        vec![vec![false, false, false], vec![false, false, false]],
    )
    .expect_err("east_open row with wrong width should be rejected");
    assert!(err.contains("east_open row 0 must have length 3"));

    let err = Maze::from_topology(
        3,
        2,
        vec![vec![false, false, false], vec![false, false, false]],
        vec![vec![false, false, false]],
    )
    .expect_err("south_open with wrong height should be rejected");
    assert!(err.contains("south_open must have one row per maze row"));

    let err = Maze::from_topology(
        3,
        2,
        vec![vec![false, false, false], vec![false, false, false]],
        vec![vec![false, false], vec![false, false, false]],
    )
    .expect_err("south_open row with wrong width should be rejected");
    assert!(err.contains("south_open row 0 must have length 3"));
}

#[test]
fn from_topology_rejects_open_boundaries() {
    let err = Maze::from_topology(
        2,
        2,
        vec![vec![true, true], vec![false, false]],
        vec![vec![false, false], vec![false, false]],
    )
    .expect_err("open right boundary should be rejected");
    assert!(err.contains("east_open right boundary must be closed at row 0"));

    let err = Maze::from_topology(
        2,
        2,
        vec![vec![true, false], vec![false, false]],
        vec![vec![false, false], vec![true, false]],
    )
    .expect_err("open bottom boundary should be rejected");
    assert!(err.contains("south_open bottom boundary must be closed at column 0"));

    let err = Maze::from_topology(
        3,
        3,
        vec![
            vec![false, false, false],
            vec![false, false, true],
            vec![false, false, false],
        ],
        vec![
            vec![false, false, false],
            vec![false, false, false],
            vec![false, false, false],
        ],
    )
    .expect_err("open right boundary on non-zero row should be rejected");
    assert!(err.contains("east_open right boundary must be closed at row 1"));

    let err = Maze::from_topology(
        3,
        3,
        vec![
            vec![false, false, false],
            vec![false, false, false],
            vec![false, false, false],
        ],
        vec![
            vec![false, false, false],
            vec![false, false, false],
            vec![false, true, false],
        ],
    )
    .expect_err("open bottom boundary on non-zero column should be rejected");
    assert!(err.contains("south_open bottom boundary must be closed at column 1"));
}

#[test]
fn from_topology_rejects_disconnected_cells() {
    let east_open = vec![vec![false, false], vec![false, false]];
    let south_open = vec![vec![false, false], vec![false, false]];

    let err = Maze::from_topology(2, 2, east_open, south_open)
        .expect_err("disconnected topology should be rejected");

    assert!(err.contains("maze must be fully connected from start"));
}

#[test]
fn from_topology_solver_matches_known_fixture_path_shape() {
    let east_open = vec![vec![true, false], vec![false, false]];
    let south_open = vec![vec![true, true], vec![false, false]];
    let maze = Maze::from_topology(2, 2, east_open, south_open)
        .expect("fixture topology should be valid and connected");

    let path = maze.solve().expect("solver should return a path");
    let expected = vec![
        Location { x: 0, y: 0 },
        Location { x: 1, y: 0 },
        Location { x: 1, y: 1 },
    ];

    assert_eq!(path, expected);
    assert_eq!(path.first().copied(), Some(Location { x: 0, y: 0 }));
    assert_eq!(path.last().copied(), Some(Location { x: 1, y: 1 }));

    for pair in path.windows(2) {
        assert!(
            maze.can_move_between(pair[0], pair[1]),
            "expected traversable step in known fixture: {:?} -> {:?}",
            pair[0],
            pair[1]
        );
    }
}

#[test]
fn from_topology_solver_validates_connected_2x3_fixture() {
    let east_open = vec![vec![true, false], vec![true, false], vec![true, false]];
    let south_open = vec![vec![false, true], vec![false, true], vec![false, false]];
    let maze = Maze::from_topology(2, 3, east_open, south_open)
        .expect("fixture topology should be valid and connected");

    let path = maze.solve().expect("solver should return a path");
    assert_eq!(path.first().copied(), Some(Location { x: 0, y: 0 }));
    assert_eq!(path.last().copied(), Some(Location { x: 1, y: 2 }));

    for pair in path.windows(2) {
        let a = pair[0];
        let b = pair[1];
        let manhattan = a.x.abs_diff(b.x) + a.y.abs_diff(b.y);
        assert_eq!(
            manhattan, 1,
            "path step must be adjacent: {:?} -> {:?}",
            a, b
        );
        assert!(
            maze.can_move_between(a, b),
            "expected traversable step in 2x3 fixture: {:?} -> {:?}",
            a,
            b
        );
    }
}
