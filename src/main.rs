mod a_star;
mod bfs;
mod dfs;

use minifb::{Key, KeyRepeat, MouseButton, MouseMode, Window, WindowOptions};
use rand::Rng;
use std::time::Instant;
use std::{thread, time::Duration};

const DEFAULT_GRID_WIDTH: usize = 61;
const DEFAULT_GRID_HEIGHT: usize = 31;
const GRID_SIZES: [(usize, usize); 4] = [
    (41, 21),
    (DEFAULT_GRID_WIDTH, DEFAULT_GRID_HEIGHT),
    (81, 41),
    (101, 51),
];
const DEFAULT_GRID_SIZE_INDEX: usize = 1;
const WINDOW_WIDTH: usize = 1280;
const WINDOW_HEIGHT: usize = 820;
const TOOLBAR_HEIGHT: usize = 78;
const MAZE_PADDING: usize = 24;
const BUTTON_WIDTH: usize = 46;
const BUTTON_HEIGHT: usize = 42;
const BUTTON_GAP: usize = 10;
const BUTTON_TOP: usize = 18;
const BUTTON_RIGHT: usize = 28;
const INITIAL_BORDER_COVERAGE_PERCENT: usize = 100;
const REQUIRED_EXIT_OPENINGS: usize = 2;
const TREE_COUNT: usize = 4;
const SEARCH_STEP: Duration = Duration::from_millis(18);

const BACKGROUND_COLOR: u32 = 0x070B14;
const PANEL_COLOR: u32 = 0x0E1626;
const PANEL_BORDER_COLOR: u32 = 0x1D2B42;
const BUTTON_COLOR: u32 = 0x17263D;
const BUTTON_HOVER_COLOR: u32 = 0x243A5B;
const BUTTON_ICON_COLOR: u32 = 0xE2E8F0;
const ACCENT_COLOR: u32 = 0x2DD4BF;
const MUTED_COLOR: u32 = 0x94A3B8;
const WALL_COLOR: u32 = 0x38BDF8;
const OPEN_COLOR: u32 = 0x0B1220;
const FLOOR_BORDER_COLOR: u32 = 0x152238;
const VISITED_COLOR: u32 = 0xF8FAFC;
const CURRENT_COLOR: u32 = 0xF59E0B;
const PATH_COLOR: u32 = 0xEF4444;
const START_COLOR: u32 = 0x22C55E;
const END_COLOR: u32 = 0xDC2626;
const LABEL_COLOR: u32 = 0xF8FAFC;

const S_GLYPH: [&str; 5] = ["11111", "10000", "11111", "00001", "11111"];
const E_GLYPH: [&str; 5] = ["11111", "10000", "11110", "10000", "11111"];

pub type Point = (usize, usize);

pub struct SearchResult {
    pub visited: Vec<Point>,
    pub path: Vec<Point>,
}

pub fn build_path(parents: &[Option<Point>], start: Point, end: Point, width: usize) -> Vec<Point> {
    if start == end {
        return vec![start];
    }

    if parents[cell_index(end, width)].is_none() {
        return Vec::new();
    }

    let mut path = vec![end];
    let mut current = end;

    while current != start {
        let Some(parent) = parents[cell_index(current, width)] else {
            return Vec::new();
        };
        current = parent;
        path.push(current);
    }

    path.reverse();
    path
}

fn cell_index((x, y): Point, width: usize) -> usize {
    y * width + x
}

pub struct Maze {
    width: usize,
    height: usize,
    cells: Vec<bool>,
}

#[derive(Clone, Copy)]
struct GridLayout {
    left: usize,
    top: usize,
    cell_size: usize,
    width: usize,
    height: usize,
}

impl GridLayout {
    fn for_grid(width: usize, height: usize) -> Self {
        let available_width = WINDOW_WIDTH - MAZE_PADDING * 2;
        let available_height = WINDOW_HEIGHT - TOOLBAR_HEIGHT - MAZE_PADDING * 2;
        let cell_size = (available_width / width)
            .min(available_height / height)
            .max(1);
        let grid_pixel_width = width * cell_size;
        let grid_pixel_height = height * cell_size;

        Self {
            left: MAZE_PADDING + (available_width - grid_pixel_width) / 2,
            top: TOOLBAR_HEIGHT + MAZE_PADDING + (available_height - grid_pixel_height) / 2,
            cell_size,
            width,
            height,
        }
    }

    fn cell_origin(self, (x, y): Point) -> (usize, usize) {
        (
            self.left + x * self.cell_size,
            self.top + y * self.cell_size,
        )
    }

    fn cell_center(self, point: Point) -> (usize, usize) {
        let (x, y) = self.cell_origin(point);
        (x + self.cell_size / 2, y + self.cell_size / 2)
    }

    fn grid_pixel_width(self) -> usize {
        self.width * self.cell_size
    }

    fn grid_pixel_height(self) -> usize {
        self.height * self.cell_size
    }

    fn wall_thickness(self) -> usize {
        (self.cell_size / 5).clamp(2, 5)
    }
}

#[derive(Clone, Copy)]
struct Rect {
    x: usize,
    y: usize,
    width: usize,
    height: usize,
}

impl Rect {
    fn contains(self, x: usize, y: usize) -> bool {
        x >= self.x && x < self.x + self.width && y >= self.y && y < self.y + self.height
    }
}

#[derive(Clone, Copy)]
struct SizeControls {
    smaller: Rect,
    larger: Rect,
}

impl SizeControls {
    fn new() -> Self {
        let larger_x = WINDOW_WIDTH - BUTTON_RIGHT - BUTTON_WIDTH;
        let smaller_x = larger_x - BUTTON_GAP - BUTTON_WIDTH;

        Self {
            smaller: Rect {
                x: smaller_x,
                y: BUTTON_TOP,
                width: BUTTON_WIDTH,
                height: BUTTON_HEIGHT,
            },
            larger: Rect {
                x: larger_x,
                y: BUTTON_TOP,
                width: BUTTON_WIDTH,
                height: BUTTON_HEIGHT,
            },
        }
    }
}

impl Maze {
    fn new(width: usize, height: usize) -> Self {
        assert!(width >= 7 && height >= 7, "maze grid is too small");

        Self {
            width,
            height,
            // `true` means open. Empty maze therefore starts fully open.
            cells: vec![true; width * height],
        }
    }

    pub fn width(&self) -> usize {
        self.width
    }

    pub fn height(&self) -> usize {
        self.height
    }

    pub fn len(&self) -> usize {
        self.cells.len()
    }

    pub fn is_empty(&self) -> bool {
        self.cells.is_empty()
    }

    pub fn index(&self, point: Point) -> usize {
        cell_index(point, self.width)
    }

    pub fn is_open(&self, point: Point) -> bool {
        self.cells[self.index(point)]
    }

    fn wall(&mut self, point: Point) {
        let index = self.index(point);
        self.cells[index] = false;
    }

    fn neighbors(&self, (x, y): Point) -> Vec<Point> {
        let mut neighbors = Vec::with_capacity(4);

        if y > 0 {
            neighbors.push((x, y - 1));
        }
        if y + 1 < self.height {
            neighbors.push((x, y + 1));
        }
        if x > 0 {
            neighbors.push((x - 1, y));
        }
        if x + 1 < self.width {
            neighbors.push((x + 1, y));
        }

        neighbors
    }

    pub fn open_neighbors(&self, (x, y): Point) -> Vec<Point> {
        self.neighbors((x, y))
            .into_iter()
            .filter(|&point| self.is_open(point))
            .collect()
    }

    pub fn start(&self) -> Point {
        (1, 1)
    }

    pub fn end(&self) -> Point {
        (self.width - 1, self.height - 1)
    }

    fn wall_neighbor_count(&self, point: Point) -> usize {
        self.neighbors(point)
            .into_iter()
            .filter(|&neighbor| !self.is_open(neighbor))
            .count()
    }

    fn open_area_is_connected(&self) -> bool {
        if !self.is_open(self.start()) {
            return false;
        }

        let mut visited = vec![false; self.len()];
        let mut stack = vec![self.start()];
        visited[self.index(self.start())] = true;
        let mut reached = 0;

        while let Some(current) = stack.pop() {
            reached += 1;

            for neighbor in self.open_neighbors(current) {
                let index = self.index(neighbor);
                if visited[index] {
                    continue;
                }

                visited[index] = true;
                stack.push(neighbor);
            }
        }

        reached == self.cells.iter().filter(|&&open| open).count()
    }

    fn can_add_wall(&mut self, point: Point) -> bool {
        if !self.is_open(point) || self.wall_neighbor_count(point) > 1 {
            return false;
        }

        // Removing a cell with at most one open neighbor cannot disconnect
        // the rest of an already connected open area.
        let open_neighbor_count = self
            .neighbors(point)
            .into_iter()
            .filter(|&neighbor| self.is_open(neighbor))
            .count();
        if open_neighbor_count <= 1 {
            return true;
        }

        let index = self.index(point);
        self.cells[index] = false;
        let keeps_map_connected = self.open_area_is_connected();
        self.cells[index] = true;

        keeps_map_connected
    }

    fn growth_candidates(&mut self, anchor: Point) -> Vec<Point> {
        let start = self.start();
        let end = self.end();
        self.neighbors(anchor)
            .into_iter()
            .filter(|&candidate| candidate != start && candidate != end)
            .filter(|&candidate| self.can_add_wall(candidate))
            .collect()
    }

    fn add_checked_wall(&mut self, point: Point, frontier: &mut Vec<Point>) -> bool {
        let can_add = point != self.start() && point != self.end() && self.can_add_wall(point);
        if !can_add {
            return false;
        }

        self.wall(point);
        frontier.push(point);
        true
    }

    fn seed_border_coverage(&mut self, rng: &mut impl Rng, frontier: &mut Vec<Point>) -> bool {
        // Border becomes one open-ended wall path. E and one adjacent gate are
        // reserved so the corner exit stays connected to the interior; path
        // order ensures each new wall touches at most one old wall.
        if rng.gen_bool(0.5) {
            if !self.add_checked_wall((self.width - 1, self.height - 3), frontier) {
                return false;
            }
            for y in (0..self.height - 3).rev() {
                if !self.add_checked_wall((self.width - 1, y), frontier) {
                    return false;
                }
            }
            for x in (0..self.width - 1).rev() {
                if !self.add_checked_wall((x, 0), frontier) {
                    return false;
                }
            }
            for y in 1..self.height {
                if !self.add_checked_wall((0, y), frontier) {
                    return false;
                }
            }
            for x in 1..self.width - 1 {
                if !self.add_checked_wall((x, self.height - 1), frontier) {
                    return false;
                }
            }
        } else {
            if !self.add_checked_wall((self.width - 3, self.height - 1), frontier) {
                return false;
            }
            for x in (0..self.width - 3).rev() {
                if !self.add_checked_wall((x, self.height - 1), frontier) {
                    return false;
                }
            }
            for y in (0..self.height - 1).rev() {
                if !self.add_checked_wall((0, y), frontier) {
                    return false;
                }
            }
            for x in 1..self.width {
                if !self.add_checked_wall((x, 0), frontier) {
                    return false;
                }
            }
            for y in 1..self.height - 1 {
                if !self.add_checked_wall((self.width - 1, y), frontier) {
                    return false;
                }
            }
        }

        true
    }

    fn generate(width: usize, height: usize, rng: &mut impl Rng) -> Self {
        let mut maze = Self::new(width, height);

        let mut frontier = Vec::new();
        assert!(
            maze.seed_border_coverage(rng, &mut frontier),
            "could not seed border wall coverage"
        );
        assert_eq!(
            maze.wall_count(),
            maze.border_cell_count() * INITIAL_BORDER_COVERAGE_PERCENT / 100
                - REQUIRED_EXIT_OPENINGS
        );

        // Add independent interior roots. A root is accepted only when it
        // touches no existing wall, so these trees can never start merged.
        let mut roots = 0;
        let mut attempts = 0;
        let max_attempts = maze.len() * TREE_COUNT;
        while roots < TREE_COUNT && attempts < max_attempts {
            attempts += 1;
            let root = (
                rng.gen_range(1..maze.width - 1),
                rng.gen_range(1..maze.height - 1),
            );
            if root == maze.start()
                || root == maze.end()
                || !maze.is_open(root)
                || maze.wall_neighbor_count(root) != 0
            {
                continue;
            }

            if maze.add_checked_wall(root, &mut frontier) {
                roots += 1;
            }
        }
        assert_eq!(roots, TREE_COUNT, "could not place independent wall trees");

        // Every frontier cell can grow later. Keeping old cells in frontier
        // allows branches instead of forcing one single random walk.
        while !frontier.is_empty() {
            let frontier_index = rng.gen_range(0..frontier.len());
            let anchor = frontier[frontier_index];
            let candidates = maze.growth_candidates(anchor);

            if candidates.is_empty() {
                frontier.swap_remove(frontier_index);
                continue;
            }

            let next = candidates[rng.gen_range(0..candidates.len())];
            maze.wall(next);
            frontier.push(next);
        }

        maze
    }

    fn border_cell_count(&self) -> usize {
        2 * self.width + 2 * self.height - 4
    }

    fn wall_count(&self) -> usize {
        self.cells.iter().filter(|&&open| !open).count()
    }

    fn open_count(&self) -> usize {
        self.cells.iter().filter(|&&open| open).count()
    }

    fn draw(&self, buffer: &mut [u32], animation: &Animation, layout: GridLayout) {
        let grid_x = layout.left;
        let grid_y = layout.top;
        let grid_right = grid_x + layout.grid_pixel_width();
        let grid_bottom = grid_y + layout.grid_pixel_height();

        fill_rect(buffer, grid_x, grid_y, grid_right, grid_bottom, OPEN_COLOR);
        fill_rect(
            buffer,
            grid_x,
            grid_y,
            grid_right,
            grid_y + 1,
            FLOOR_BORDER_COLOR,
        );
        fill_rect(
            buffer,
            grid_x,
            grid_bottom.saturating_sub(1),
            grid_right,
            grid_bottom,
            FLOOR_BORDER_COLOR,
        );
        fill_rect(
            buffer,
            grid_x,
            grid_y,
            grid_x + 1,
            grid_bottom,
            FLOOR_BORDER_COLOR,
        );
        fill_rect(
            buffer,
            grid_right.saturating_sub(1),
            grid_y,
            grid_right,
            grid_bottom,
            FLOOR_BORDER_COLOR,
        );

        for &point in animation.explored() {
            Self::draw_marker(buffer, layout, point, VISITED_COLOR, 5);
        }
        for &point in animation.path() {
            Self::draw_marker(buffer, layout, point, PATH_COLOR, 3);
        }
        if let Some(point) = animation.current() {
            Self::draw_marker(buffer, layout, point, CURRENT_COLOR, 2);
        }

        // A wall cell is rendered as a thin connected stroke between cell
        // centers. The surrounding cell remains empty floor space.
        for y in 0..self.height {
            for x in 0..self.width {
                let point = (x, y);
                if self.is_open(point) {
                    continue;
                }

                Self::draw_wall_node(buffer, layout, point);
                if x + 1 < self.width && !self.is_open((x + 1, y)) {
                    Self::draw_wall_segment(buffer, layout, point, (x + 1, y));
                }
                if y + 1 < self.height && !self.is_open((x, y + 1)) {
                    Self::draw_wall_segment(buffer, layout, point, (x, y + 1));
                }
            }
        }

        Self::draw_terminal_box(buffer, layout, self.start(), START_COLOR);
        Self::draw_terminal_box(buffer, layout, self.end(), END_COLOR);
        Self::draw_glyph(buffer, layout, self.start(), &S_GLYPH, LABEL_COLOR);
        Self::draw_glyph(buffer, layout, self.end(), &E_GLYPH, LABEL_COLOR);
    }

    fn draw_wall_node(buffer: &mut [u32], layout: GridLayout, point: Point) {
        let (center_x, center_y) = layout.cell_center(point);
        let radius = layout.wall_thickness() / 2 + 1;
        fill_rect(
            buffer,
            center_x.saturating_sub(radius),
            center_y.saturating_sub(radius),
            center_x + radius + 1,
            center_y + radius + 1,
            WALL_COLOR,
        );
    }

    fn draw_wall_segment(buffer: &mut [u32], layout: GridLayout, first: Point, second: Point) {
        let (first_x, first_y) = layout.cell_center(first);
        let (second_x, second_y) = layout.cell_center(second);
        let half_thickness = layout.wall_thickness() / 2;

        if first_x == second_x {
            fill_rect(
                buffer,
                first_x.saturating_sub(half_thickness),
                first_y.min(second_y),
                first_x + half_thickness + 1,
                first_y.max(second_y) + 1,
                WALL_COLOR,
            );
        } else {
            fill_rect(
                buffer,
                first_x.min(second_x),
                first_y.saturating_sub(half_thickness),
                first_x.max(second_x) + 1,
                first_y + half_thickness + 1,
                WALL_COLOR,
            );
        }
    }

    fn draw_marker(
        buffer: &mut [u32],
        layout: GridLayout,
        point: Point,
        color: u32,
        inset_divisor: usize,
    ) {
        let (center_x, center_y) = layout.cell_center(point);
        let radius = (layout.cell_size / inset_divisor).max(2);
        fill_rect(
            buffer,
            center_x.saturating_sub(radius),
            center_y.saturating_sub(radius),
            center_x + radius + 1,
            center_y + radius + 1,
            color,
        );
    }

    fn draw_terminal_box(buffer: &mut [u32], layout: GridLayout, point: Point, color: u32) {
        let (center_x, center_y) = layout.cell_center(point);
        let half_size = (layout.cell_size * 3 / 8).max(4);
        let thickness = (layout.cell_size / 10).clamp(1, 2);
        let left = center_x.saturating_sub(half_size);
        let top = center_y.saturating_sub(half_size);
        let right = center_x + half_size + 1;
        let bottom = center_y + half_size + 1;

        fill_rect(buffer, left, top, right, top + thickness, color);
        fill_rect(buffer, left, bottom - thickness, right, bottom, color);
        fill_rect(buffer, left, top, left + thickness, bottom, color);
        fill_rect(buffer, right - thickness, top, right, bottom, color);
    }

    fn draw_glyph(
        buffer: &mut [u32],
        layout: GridLayout,
        point: Point,
        glyph: &[&str; 5],
        color: u32,
    ) {
        let scale = (layout.cell_size / 6).max(1);
        let glyph_width = 5 * scale;
        let glyph_height = 5 * scale;
        let (cell_x, cell_y) = layout.cell_origin(point);
        let origin_x = cell_x + layout.cell_size.saturating_sub(glyph_width) / 2;
        let origin_y = cell_y + layout.cell_size.saturating_sub(glyph_height) / 2;

        for (glyph_y, row) in glyph.iter().enumerate() {
            for (glyph_x, bit) in row.bytes().enumerate() {
                if bit != b'1' {
                    continue;
                }

                fill_rect(
                    buffer,
                    origin_x + glyph_x * scale,
                    origin_y + glyph_y * scale,
                    origin_x + (glyph_x + 1) * scale,
                    origin_y + (glyph_y + 1) * scale,
                    color,
                );
            }
        }
    }
}

fn fill_rect(buffer: &mut [u32], left: usize, top: usize, right: usize, bottom: usize, color: u32) {
    let left = left.min(WINDOW_WIDTH);
    let right = right.min(WINDOW_WIDTH);
    let top = top.min(WINDOW_HEIGHT);
    let bottom = bottom.min(WINDOW_HEIGHT);

    if left >= right || top >= bottom {
        return;
    }

    for y in top..bottom {
        let row = &mut buffer[y * WINDOW_WIDTH + left..y * WINDOW_WIDTH + right];
        row.fill(color);
    }
}

fn draw_toolbar(
    buffer: &mut [u32],
    controls: SizeControls,
    mouse: Option<(f32, f32)>,
    grid_width: usize,
    grid_height: usize,
) {
    fill_rect(buffer, 0, 0, WINDOW_WIDTH, TOOLBAR_HEIGHT, PANEL_COLOR);
    fill_rect(
        buffer,
        0,
        TOOLBAR_HEIGHT - 1,
        WINDOW_WIDTH,
        TOOLBAR_HEIGHT,
        PANEL_BORDER_COLOR,
    );

    draw_text(buffer, "WALL", 32, 17, 2, LABEL_COLOR);
    draw_text(buffer, "FOREST", 74, 17, 2, ACCENT_COLOR);
    draw_text(buffer, "MAZE", 32, 43, 1, MUTED_COLOR);
    fill_rect(buffer, 32, 65, 220, 67, ACCENT_COLOR);
    draw_text(buffer, "GRID", 930, 24, 1, MUTED_COLOR);
    draw_text(
        buffer,
        &format!("{}X{}", grid_width, grid_height),
        970,
        24,
        1,
        LABEL_COLOR,
    );

    let mouse = mouse.map(|(x, y)| (x as usize, y as usize));
    draw_button(
        buffer,
        controls.smaller,
        mouse.is_some_and(|(x, y)| controls.smaller.contains(x, y)),
        false,
    );
    draw_button(
        buffer,
        controls.larger,
        mouse.is_some_and(|(x, y)| controls.larger.contains(x, y)),
        true,
    );
}

fn draw_button(buffer: &mut [u32], rect: Rect, hovered: bool, plus: bool) {
    let color = if hovered {
        BUTTON_HOVER_COLOR
    } else {
        BUTTON_COLOR
    };
    fill_rect(
        buffer,
        rect.x,
        rect.y,
        rect.x + rect.width,
        rect.y + rect.height,
        color,
    );
    fill_rect(
        buffer,
        rect.x,
        rect.y,
        rect.x + rect.width,
        rect.y + 1,
        PANEL_BORDER_COLOR,
    );
    fill_rect(
        buffer,
        rect.x,
        rect.y + rect.height - 1,
        rect.x + rect.width,
        rect.y + rect.height,
        PANEL_BORDER_COLOR,
    );
    fill_rect(
        buffer,
        rect.x,
        rect.y,
        rect.x + 1,
        rect.y + rect.height,
        PANEL_BORDER_COLOR,
    );
    fill_rect(
        buffer,
        rect.x + rect.width - 1,
        rect.y,
        rect.x + rect.width,
        rect.y + rect.height,
        PANEL_BORDER_COLOR,
    );

    let center_x = rect.x + rect.width / 2;
    let center_y = rect.y + rect.height / 2;
    fill_rect(
        buffer,
        center_x.saturating_sub(10),
        center_y.saturating_sub(2),
        center_x + 11,
        center_y + 3,
        BUTTON_ICON_COLOR,
    );
    if plus {
        fill_rect(
            buffer,
            center_x.saturating_sub(2),
            center_y.saturating_sub(10),
            center_x + 3,
            center_y + 11,
            BUTTON_ICON_COLOR,
        );
    }
}

fn draw_text(buffer: &mut [u32], text: &str, left: usize, top: usize, scale: usize, color: u32) {
    let mut cursor_x = left;
    for character in text.chars() {
        let glyph = glyph_3x5(character);
        for (glyph_y, row) in glyph.iter().enumerate() {
            for (glyph_x, bit) in row.bytes().enumerate() {
                if bit != b'1' {
                    continue;
                }

                fill_rect(
                    buffer,
                    cursor_x + glyph_x * scale,
                    top + glyph_y * scale,
                    cursor_x + (glyph_x + 1) * scale,
                    top + (glyph_y + 1) * scale,
                    color,
                );
            }
        }
        cursor_x += 4 * scale;
    }
}

fn glyph_3x5(character: char) -> [&'static str; 5] {
    match character {
        'A' => ["010", "101", "111", "101", "101"],
        'B' => ["110", "101", "110", "101", "110"],
        'D' => ["110", "101", "101", "101", "110"],
        'E' => ["111", "100", "110", "100", "111"],
        'F' => ["111", "100", "110", "100", "100"],
        'G' => ["111", "100", "101", "101", "111"],
        'I' => ["111", "010", "010", "010", "111"],
        'L' => ["100", "100", "100", "100", "111"],
        'M' => ["101", "111", "111", "101", "101"],
        'O' => ["111", "101", "101", "101", "111"],
        'R' => ["110", "101", "110", "101", "101"],
        'S' => ["111", "100", "111", "001", "111"],
        'T' => ["111", "010", "010", "010", "010"],
        'W' => ["101", "101", "111", "111", "101"],
        'X' => ["101", "101", "010", "101", "101"],
        'Z' => ["111", "001", "010", "100", "111"],
        '0' => ["111", "101", "101", "101", "111"],
        '1' => ["010", "110", "010", "010", "111"],
        '2' => ["110", "001", "010", "100", "111"],
        '3' => ["110", "001", "010", "001", "110"],
        '4' => ["101", "101", "111", "001", "001"],
        '5' => ["111", "100", "110", "001", "110"],
        '6' => ["011", "100", "111", "101", "111"],
        '7' => ["111", "001", "010", "010", "010"],
        '8' => ["111", "101", "111", "101", "111"],
        '9' => ["111", "101", "111", "001", "110"],
        _ => ["000", "000", "000", "000", "000"],
    }
}

#[derive(Clone, Copy)]
enum Algorithm {
    Bfs,
    Dfs,
    AStar,
}

impl Algorithm {
    fn label(self) -> &'static str {
        match self {
            Self::Bfs => "BFS",
            Self::Dfs => "DFS",
            Self::AStar => "A*",
        }
    }

    fn search(self, maze: &Maze) -> SearchResult {
        let start = maze.start();
        let end = maze.end();
        match self {
            Self::Bfs => bfs::search(maze, start, end),
            Self::Dfs => dfs::search(maze, start, end),
            Self::AStar => a_star::search(maze, start, end),
        }
    }
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum AnimationPhase {
    Exploring,
    DrawingPath,
    Done,
}

struct Animation {
    algorithm: Algorithm,
    result: SearchResult,
    phase: AnimationPhase,
    visited_count: usize,
    path_count: usize,
    last_step: Instant,
}

impl Animation {
    fn new(maze: &Maze, algorithm: Algorithm) -> Self {
        Self {
            algorithm,
            result: algorithm.search(maze),
            phase: AnimationPhase::Exploring,
            visited_count: 0,
            path_count: 0,
            last_step: Instant::now() - SEARCH_STEP,
        }
    }

    fn update(&mut self) {
        if self.last_step.elapsed() < SEARCH_STEP {
            return;
        }

        self.last_step = Instant::now();

        match self.phase {
            AnimationPhase::Exploring if self.visited_count < self.result.visited.len() => {
                self.visited_count += 1;
            }
            AnimationPhase::Exploring if self.result.path.is_empty() => {
                self.phase = AnimationPhase::Done;
            }
            AnimationPhase::Exploring => {
                self.phase = AnimationPhase::DrawingPath;
            }
            AnimationPhase::DrawingPath if self.path_count < self.result.path.len() => {
                self.path_count += 1;
            }
            AnimationPhase::DrawingPath => {
                self.phase = AnimationPhase::Done;
            }
            AnimationPhase::Done => {}
        }
    }

    fn explored(&self) -> &[Point] {
        &self.result.visited[..self.visited_count]
    }

    fn path(&self) -> &[Point] {
        &self.result.path[..self.path_count]
    }

    fn current(&self) -> Option<Point> {
        match self.phase {
            AnimationPhase::Exploring => self
                .result
                .visited
                .get(self.visited_count.saturating_sub(1))
                .copied(),
            AnimationPhase::DrawingPath => self
                .result
                .path
                .get(self.path_count.saturating_sub(1))
                .copied(),
            AnimationPhase::Done => None,
        }
    }

    fn title(&self, maze: &Maze) -> String {
        let (phase, current, total) = match self.phase {
            AnimationPhase::Exploring => {
                ("exploring", self.visited_count, self.result.visited.len())
            }
            AnimationPhase::DrawingPath => ("path", self.path_count, self.result.path.len()),
            AnimationPhase::Done => ("complete", self.result.path.len(), self.result.path.len()),
        };

        format!(
            "MAZE // {}x{} // {} // {} {}/{} // walls={}/{} open={}   |   B BFS   D DFS   A A*   R new maze   mouse -/+ size   ESC quit",
            maze.width(),
            maze.height(),
            self.algorithm.label(),
            phase,
            current,
            total,
            maze.wall_count(),
            maze.len(),
            maze.open_count()
        )
    }
}

fn main() {
    let mut rng = rand::thread_rng();
    let mut size_index = DEFAULT_GRID_SIZE_INDEX;
    let (width, height) = GRID_SIZES[size_index];
    let mut maze = Maze::generate(width, height, &mut rng);
    let mut animation = Animation::new(&maze, Algorithm::Bfs);
    let mut buffer = vec![BACKGROUND_COLOR; WINDOW_WIDTH * WINDOW_HEIGHT];
    let controls = SizeControls::new();
    let mut left_mouse_was_down = false;

    let mut window = Window::new(
        "Wall Forest Maze // click -/+ to resize",
        WINDOW_WIDTH,
        WINDOW_HEIGHT,
        WindowOptions::default(),
    )
    .expect("could not create maze window");

    while window.is_open() && !window.is_key_down(Key::Escape) {
        let requested_algorithm = if window.is_key_pressed(Key::B, KeyRepeat::No) {
            Some(Algorithm::Bfs)
        } else if window.is_key_pressed(Key::D, KeyRepeat::No) {
            Some(Algorithm::Dfs)
        } else if window.is_key_pressed(Key::A, KeyRepeat::No) {
            Some(Algorithm::AStar)
        } else {
            None
        };

        let mouse = window.get_mouse_pos(MouseMode::Discard);
        let left_mouse_down = window.get_mouse_down(MouseButton::Left);
        let clicked = left_mouse_down && !left_mouse_was_down;
        left_mouse_was_down = left_mouse_down;

        let mut regenerate = window.is_key_pressed(Key::R, KeyRepeat::No);
        if clicked {
            if mouse.is_some_and(|(x, y)| controls.smaller.contains(x as usize, y as usize))
                && size_index > 0
            {
                size_index -= 1;
                regenerate = true;
            } else if mouse.is_some_and(|(x, y)| controls.larger.contains(x as usize, y as usize))
                && size_index + 1 < GRID_SIZES.len()
            {
                size_index += 1;
                regenerate = true;
            }
        }

        if let Some(algorithm) = requested_algorithm {
            animation = Animation::new(&maze, algorithm);
        }
        if regenerate {
            let (width, height) = GRID_SIZES[size_index];
            maze = Maze::generate(width, height, &mut rng);
            animation = Animation::new(&maze, requested_algorithm.unwrap_or(Algorithm::Bfs));
        }

        animation.update();
        buffer.fill(BACKGROUND_COLOR);
        let layout = GridLayout::for_grid(maze.width(), maze.height());
        maze.draw(&mut buffer, &animation, layout);
        draw_toolbar(&mut buffer, controls, mouse, maze.width(), maze.height());
        window.set_title(&animation.title(&maze));
        window
            .update_with_buffer(&buffer, WINDOW_WIDTH, WINDOW_HEIGHT)
            .expect("could not draw maze window");
        thread::sleep(Duration::from_millis(16));
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rand::SeedableRng;

    fn has_wall_cycle(maze: &Maze) -> bool {
        let mut visited = vec![false; maze.len()];

        for y in 0..maze.height() {
            for x in 0..maze.width() {
                let root = (x, y);
                let root_index = maze.index(root);
                if maze.is_open(root) || visited[root_index] {
                    continue;
                }

                let mut stack = vec![(root, None)];
                visited[root_index] = true;

                while let Some((current, parent)) = stack.pop() {
                    for neighbor in maze.neighbors(current) {
                        if maze.is_open(neighbor) {
                            continue;
                        }

                        let index = maze.index(neighbor);
                        if !visited[index] {
                            visited[index] = true;
                            stack.push((neighbor, Some(current)));
                        } else if parent != Some(neighbor) {
                            return true;
                        }
                    }
                }
            }
        }

        false
    }

    fn wall_component_count(maze: &Maze) -> usize {
        let mut visited = vec![false; maze.len()];
        let mut components = 0;

        for y in 0..maze.height() {
            for x in 0..maze.width() {
                let root = (x, y);
                let root_index = maze.index(root);
                if maze.is_open(root) || visited[root_index] {
                    continue;
                }

                components += 1;
                let mut stack = vec![root];
                visited[root_index] = true;

                while let Some(current) = stack.pop() {
                    for neighbor in maze.neighbors(current) {
                        if maze.is_open(neighbor) {
                            continue;
                        }

                        let index = maze.index(neighbor);
                        if !visited[index] {
                            visited[index] = true;
                            stack.push(neighbor);
                        }
                    }
                }
            }
        }

        components
    }

    #[test]
    fn generation_reaches_end() {
        let mut rng = rand::thread_rng();
        let maze = Maze::generate(DEFAULT_GRID_WIDTH, DEFAULT_GRID_HEIGHT, &mut rng);

        assert!(maze.is_open(maze.start()));
        assert!(maze.is_open(maze.end()));
    }

    #[test]
    fn maze_constructor_starts_empty() {
        let maze = Maze::new(41, 21);

        assert!(maze.cells.iter().all(|&cell| cell));
    }

    #[test]
    fn initial_scaffold_covers_border_before_growth() {
        let mut rng = rand::rngs::StdRng::seed_from_u64(11);
        let mut maze = Maze::new(41, 21);
        let mut frontier = Vec::new();

        assert!(maze.seed_border_coverage(&mut rng, &mut frontier));
        assert_eq!(
            maze.wall_count(),
            maze.border_cell_count() * INITIAL_BORDER_COVERAGE_PERCENT / 100
                - REQUIRED_EXIT_OPENINGS
        );
        assert_eq!(frontier.len(), maze.wall_count());
        assert!(maze.is_open(maze.start()));
        assert!(maze.is_open(maze.end()));
        assert!(maze.open_area_is_connected());
        assert!(!has_wall_cycle(&maze));
    }

    #[test]
    fn wall_growth_keeps_open_area_connected_and_loop_free() {
        let mut rng = rand::rngs::StdRng::seed_from_u64(7);
        for (width, height) in [(41, 21), (61, 31), (81, 41)] {
            let maze = Maze::generate(width, height, &mut rng);

            assert!(maze.open_area_is_connected());
            assert!(!has_wall_cycle(&maze));
            assert!(wall_component_count(&maze) > 1);
            assert!(maze.wall_count() > 0);
        }
    }

    #[test]
    fn every_search_finds_a_path() {
        let mut rng = rand::rngs::StdRng::seed_from_u64(20260814);
        for (width, height) in [(41, 21), (61, 31)] {
            let maze = Maze::generate(width, height, &mut rng);

            for algorithm in [Algorithm::Bfs, Algorithm::Dfs, Algorithm::AStar] {
                let result = algorithm.search(&maze);
                assert!(!result.visited.is_empty());
                assert_eq!(result.path.first(), Some(&maze.start()));
                assert_eq!(result.path.last(), Some(&maze.end()));
            }
        }
    }
}
