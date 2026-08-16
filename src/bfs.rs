use std::collections::VecDeque;

use crate::{build_path, Maze, Point, SearchResult};

pub fn search(maze: &Maze, start: Point, end: Point) -> SearchResult {
    let mut queue = VecDeque::from([start]);
    let mut visited = vec![false; maze.len()];
    let mut parents = vec![None; maze.len()];
    let mut visit_order = Vec::new();

    visited[maze.index(start)] = true;

    while let Some(current) = queue.pop_front() {
        visit_order.push(current);

        if current == end {
            break;
        }

        for neighbor in maze.open_neighbors(current) {
            let index = maze.index(neighbor);
            if visited[index] {
                continue;
            }

            visited[index] = true;
            parents[index] = Some(current);
            queue.push_back(neighbor);
        }
    }

    SearchResult {
        visited: visit_order,
        path: build_path(&parents, start, end, maze.width()),
    }
}
