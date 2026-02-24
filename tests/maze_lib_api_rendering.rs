use maze::Maze;

#[derive(Debug)]
struct MazeRender {
    lines: Vec<String>,
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

    MazeRender { lines }
}

#[test]
fn top_level_rendering_has_expected_shape_and_markers() {
    for (width, height) in [(2, 2), (3, 4), (5, 3), (6, 6)] {
        let maze = Maze::new(width, height);
        let maze_text = maze.to_string();
        let render = parse_render(width, height, &maze_text);

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
fn top_level_solution_rendering_uses_single_line_overlay_chars() {
    let maze = Maze::new(4, 4);
    let solution = maze.solve().expect("solver should return a path");
    let rendered = maze.render_with_solution(&solution);

    let has_overlay = rendered.chars().any(|c| {
        matches!(
            c,
            '─' | '│' | '┌' | '┐' | '└' | '┘' | '├' | '┤' | '┬' | '┴' | '┼'
        )
    });
    assert!(
        has_overlay,
        "expected rendered maze to include solution overlay"
    );
    assert_eq!(rendered.matches('S').count(), 1);
    assert_eq!(rendered.matches('F').count(), 1);
}
