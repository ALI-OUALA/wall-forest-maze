# Wall Forest Maze

*A maze generator that grows walls instead of carving a solution path.*

Wall Forest Maze starts with an empty grid. It grows several independent wall trees through that open space, and the walkable maze is the space left between them.

The generator does not build a route from `S` to `E`. Search is a separate step that runs after generation.

## Run the application

Install Rust, then run:

```bash
cargo run --release
```

The default grid is `61×31`. The application scales the grid to the window and keeps the open floor visually continuous.

## Controls

| Input | Action |
| --- | --- |
| Mouse `−` | Generate the next smaller grid preset |
| Mouse `+` | Generate the next larger grid preset |
| `B` | Animate breadth-first search |
| `D` | Animate depth-first search |
| `A` | Animate A* search |
| `R` | Generate a new maze at the current size |
| `Esc` | Quit |

Available grid presets are `41×21`, `61×31`, `81×41`, and `101×51`.

## How Wall Forest Maze works

1. Create a fully open grid.
2. Seed the perimeter as an open-ended wall tree. `S` is fixed at `(1,1)` and `E` is the bottom-right cell, equivalent to `(-1,-1)`.
3. Keep the two-cell exit notch required to reach a corner `E` with four-direction movement.
4. Place four independent interior wall roots. A root is accepted only when it touches no existing wall.
5. Choose a random wall already in the frontier and grow to a random legal neighbor.
6. Remove a frontier wall when it has no legal growth candidates.
7. Stop when no legal candidates remain.

Each candidate wall must satisfy two structural rules:

- It touches at most one existing wall. This keeps wall trees separate and prevents wall loops.
- Temporarily placing it must leave every open cell connected to `S`. This prevents the trees from cutting the floor into disconnected regions.

The result is a wall forest: independently seeded wall components with branches, turns, and dead ends. The final walkable route emerges from the remaining open floor.

## Rendering direction

The UI uses a dark navy surface, restrained teal/cyan accents, thin connected wall strokes, boxed `S`/`E` terminals, and continuous empty floor space. Explored search cells are white, the current cell is orange, and the final search path is red.

The reference image is used only as visual direction; no copied reference artwork is included in the repository.

## Project structure

```text
src/main.rs   generator, runtime UI, rendering, animation, tests
src/bfs.rs    breadth-first search
src/dfs.rs    depth-first search
src/a_star.rs A* search
```

## Complexity note

Search and rendering are linear in the number of grid cells. Generation uses a flood-fill validation when a proposed wall could affect connectivity, so its worst-case cost is higher than a simple linear wall walk. This deliberate validation is what enforces the no-cutting rule.

## Validate the project

```bash
cargo fmt -- --check
cargo test
cargo clippy --all-targets --all-features -- -D warnings
```

Wall Forest Maze is an experiment in wall-first generation:

> Do not construct the solution. Construct enough structure for a solution to emerge.
