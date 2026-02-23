use maze::Maze;

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
}

#[test]
fn from_topology_rejects_disconnected_cells() {
    let east_open = vec![vec![false, false], vec![false, false]];
    let south_open = vec![vec![false, false], vec![false, false]];

    let err = Maze::from_topology(2, 2, east_open, south_open)
        .expect_err("disconnected topology should be rejected");

    assert!(err.contains("maze must be fully connected from start"));
}
