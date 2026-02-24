# rust-maze

A small Rust project for generating, rendering, and solving grid mazes.

## Run

Run the CLI app:

```bash
cargo run
```

You will be prompted for `width` and `height` (both must be integers `>= 2`).

## Lint And Test

Format check:

```bash
cargo fmt -- --check
```

Clippy (warnings as errors):

```bash
cargo clippy --all-targets --all-features -- -D warnings
```

Run all tests:

```bash
cargo test --all-targets
```

## Library Usage

The crate exposes a `Maze` type from `src/lib.rs`.

### Construct From Deterministic Topology

Use `Maze::from_topology` when you want deterministic mazes (for example, tests):

```rust
use maze::{Maze, FromTopologyError};

let east_open = vec![
    vec![true, false],
    vec![false, false],
];
let south_open = vec![
    vec![true, true],
    vec![false, false],
];

let maze = Maze::from_topology(2, 2, east_open, south_open)
    .map_err(|e: FromTopologyError| format!("invalid topology: {e}"))?;
# Ok::<(), String>(())
```

### Query Size

```rust
use maze::Maze;

let maze = Maze::new(4, 3);
let (width, height) = maze.size();
assert_eq!((width, height), (4, 3));
```

### Query Traversability

`can_move_between` returns `true` only for in-bounds, cardinally adjacent cells with an open connection.

```rust
use maze::{Location, Maze};

let maze = Maze::new(4, 4);
let a = Location { x: 0, y: 0 };
let b = Location { x: 1, y: 0 };

let can_move = maze.can_move_between(a, b);
println!("Can move from {:?} to {:?}: {}", a, b, can_move);
```

## Notes

- `Maze::new(width, height)` builds a random maze and ensures all cells are reachable from the start.
- `Maze::solve()` returns a path from start `(0,0)` to finish `(width-1,height-1)` when one exists.
- Rendering uses Unicode box-drawing characters.
