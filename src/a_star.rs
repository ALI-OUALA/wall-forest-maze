use std::cmp::Reverse;
use std::collections::BinaryHeap;

use crate::{build_path, Maze, Point, SearchResult};

pub fn search(maze: &Maze, start: Point, end: Point) -> SearchResult {
    let mut open_set = BinaryHeap::new();
    let mut g_scores = vec![usize::MAX; maze.len()];
    let mut parents = vec![None; maze.len()];
    let mut visit_order = Vec::new();

    g_scores[maze.index(start)] = 0;
    open_set.push(Reverse((heuristic(start, end), 0, start)));

    while let Some(Reverse((_f_score, g_score, current))) = open_set.pop() {
        if g_score != g_scores[maze.index(current)] {
            continue;
        }

        visit_order.push(current);

        if current == end {
            break;
        }

        for neighbor in maze.open_neighbors(current) {
            let tentative_g_score = g_score + 1;
            let index = maze.index(neighbor);

            if tentative_g_score >= g_scores[index] {
                continue;
            }

            g_scores[index] = tentative_g_score;
            parents[index] = Some(current);
            let f_score = tentative_g_score + heuristic(neighbor, end);
            open_set.push(Reverse((f_score, tentative_g_score, neighbor)));
        }
    }

    SearchResult {
        visited: visit_order,
        path: build_path(&parents, start, end, maze.width()),
    }
}

fn heuristic((x, y): Point, (end_x, end_y): Point) -> usize {
    x.abs_diff(end_x) + y.abs_diff(end_y)
}
