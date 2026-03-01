const DEFAULT_MAX_STEPS: usize = 200;
const CARDINAL_COST: f32 = 1.0;
const DIAGONAL_COST: f32 = 1.414;
const INF_SCORE: f32 = 999_999.0;

/// Integer grid position used by pathfinding.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct GridPos {
    pub x: i32,
    pub y: i32,
}

impl GridPos {
    pub const fn new(x: i32, y: i32) -> Self {
        Self { x, y }
    }
}

/// Lightweight grid view for A* pathfinding.
///
/// - `walkable`: whether each tile is traversable.
/// - `move_cost`: multiplicative terrain movement cost per tile.
#[derive(Debug, Clone)]
pub struct GridCostMap {
    width: i32,
    height: i32,
    walkable: Vec<bool>,
    move_cost: Vec<f32>,
}

impl GridCostMap {
    /// Creates a new grid. By default every tile is walkable with cost 1.0.
    pub fn new(width: i32, height: i32) -> Self {
        let size = (width * height) as usize;
        Self {
            width,
            height,
            walkable: vec![true; size],
            move_cost: vec![1.0; size],
        }
    }

    /// Builds a grid directly from flat bool/cost slices without per-cell setters.
    ///
    /// Caller must ensure lengths match `width * height`.
    pub fn from_flat_unchecked(width: i32, height: i32, walkable: &[bool], move_cost: &[f32]) -> Self {
        debug_assert_eq!(walkable.len(), (width * height) as usize);
        debug_assert_eq!(move_cost.len(), (width * height) as usize);
        let mut clamped_move_cost: Vec<f32> = Vec::with_capacity(move_cost.len());
        for &cost in move_cost {
            clamped_move_cost.push(cost.max(0.0));
        }
        Self {
            width,
            height,
            walkable: walkable.to_vec(),
            move_cost: clamped_move_cost,
        }
    }

    /// Builds a grid directly from flat byte flags (0=blocked, non-zero=walkable).
    ///
    /// Caller must ensure lengths match `width * height`.
    pub fn from_flat_bytes_unchecked(
        width: i32,
        height: i32,
        walkable: &[u8],
        move_cost: &[f32],
    ) -> Self {
        debug_assert_eq!(walkable.len(), (width * height) as usize);
        debug_assert_eq!(move_cost.len(), (width * height) as usize);
        let mut bool_walkable: Vec<bool> = Vec::with_capacity(walkable.len());
        for &flag in walkable {
            bool_walkable.push(flag != 0);
        }
        let mut clamped_move_cost: Vec<f32> = Vec::with_capacity(move_cost.len());
        for &cost in move_cost {
            clamped_move_cost.push(cost.max(0.0));
        }
        Self {
            width,
            height,
            walkable: bool_walkable,
            move_cost: clamped_move_cost,
        }
    }

    #[inline]
    fn index(&self, x: i32, y: i32) -> Option<usize> {
        if x < 0 || y < 0 || x >= self.width || y >= self.height {
            return None;
        }
        Some((y * self.width + x) as usize)
    }

    /// Sets whether a tile is walkable.
    pub fn set_walkable(&mut self, x: i32, y: i32, is_walkable: bool) {
        if let Some(idx) = self.index(x, y) {
            self.walkable[idx] = is_walkable;
        }
    }

    /// Sets the terrain move-cost multiplier for a tile.
    pub fn set_move_cost(&mut self, x: i32, y: i32, cost: f32) {
        if let Some(idx) = self.index(x, y) {
            self.move_cost[idx] = cost.max(0.0);
        }
    }

    /// Returns whether a tile is walkable. Out-of-bounds is not walkable.
    pub fn is_walkable(&self, x: i32, y: i32) -> bool {
        match self.index(x, y) {
            Some(idx) => self.walkable[idx],
            None => false,
        }
    }

    /// Returns the terrain move-cost multiplier for a tile.
    /// Out-of-bounds falls back to 1.0.
    pub fn get_move_cost(&self, x: i32, y: i32) -> f32 {
        match self.index(x, y) {
            Some(idx) => self.move_cost[idx],
            None => 1.0,
        }
    }
}

/// A* pathfinding with 8-direction movement and Chebyshev heuristic.
///
/// This mirrors the existing GDScript movement pathfinding behavior:
/// - returns `[from]` when start and target are the same
/// - returns empty path when target is not walkable
/// - bounded by `max_steps` to avoid pathological spikes
pub fn find_path(grid: &GridCostMap, from: GridPos, to: GridPos, max_steps: usize) -> Vec<GridPos> {
    if from == to {
        return vec![from];
    }
    let Some(to_idx) = grid.index(to.x, to.y) else {
        return Vec::new();
    };
    if !grid.walkable[to_idx] {
        return Vec::new();
    }
    let Some(from_idx) = grid.index(from.x, from.y) else {
        return Vec::new();
    };

    let max_steps = if max_steps == 0 {
        DEFAULT_MAX_STEPS
    } else {
        max_steps
    };
    let node_count = (grid.width * grid.height) as usize;

    let mut open_set: Vec<usize> = vec![from_idx];
    let mut in_open: Vec<bool> = vec![false; node_count];
    let mut came_from: Vec<Option<usize>> = vec![None; node_count];
    let mut g_score: Vec<f32> = vec![INF_SCORE; node_count];
    let mut f_score: Vec<f32> = vec![INF_SCORE; node_count];
    let mut closed_set: Vec<bool> = vec![false; node_count];

    in_open[from_idx] = true;
    g_score[from_idx] = 0.0;
    f_score[from_idx] = chebyshev(from, to);

    let mut steps = 0usize;
    while !open_set.is_empty() && steps < max_steps {
        steps += 1;

        let mut best_idx = 0usize;
        let mut best_f = f_score[open_set[0]];
        for (idx, node_idx) in open_set.iter().enumerate().skip(1) {
            let f = f_score[*node_idx];
            if f < best_f {
                best_f = f;
                best_idx = idx;
            }
        }

        let current_idx = open_set.remove(best_idx);
        in_open[current_idx] = false;

        if current_idx == to_idx {
            return reconstruct_path(&came_from, current_idx, grid.width);
        }

        closed_set[current_idx] = true;
        let current_x = (current_idx as i32) % grid.width;
        let current_y = (current_idx as i32) / grid.width;

        for dy in -1..=1 {
            for dx in -1..=1 {
                if dx == 0 && dy == 0 {
                    continue;
                }

                let neighbor_x = current_x + dx;
                let neighbor_y = current_y + dy;
                if neighbor_x < 0
                    || neighbor_y < 0
                    || neighbor_x >= grid.width
                    || neighbor_y >= grid.height
                {
                    continue;
                }
                let neighbor_idx = (neighbor_y * grid.width + neighbor_x) as usize;
                if closed_set[neighbor_idx] || !grid.walkable[neighbor_idx] {
                    continue;
                }

                let step_cost = if dx.abs() + dy.abs() == 1 {
                    CARDINAL_COST
                } else {
                    DIAGONAL_COST
                };
                let tentative_g = g_score[current_idx] + (step_cost * grid.move_cost[neighbor_idx]);

                if tentative_g < g_score[neighbor_idx] {
                    came_from[neighbor_idx] = Some(current_idx);
                    g_score[neighbor_idx] = tentative_g;
                    f_score[neighbor_idx] =
                        tentative_g + chebyshev(GridPos::new(neighbor_x, neighbor_y), to);

                    if !in_open[neighbor_idx] {
                        open_set.push(neighbor_idx);
                        in_open[neighbor_idx] = true;
                    }
                }
            }
        }
    }

    Vec::new()
}

#[inline]
fn chebyshev(a: GridPos, b: GridPos) -> f32 {
    (a.x.abs_diff(b.x).max(a.y.abs_diff(b.y))) as f32
}

fn reconstruct_path(came_from: &[Option<usize>], mut current_idx: usize, width: i32) -> Vec<GridPos> {
    let mut path_indices: Vec<usize> = vec![current_idx];
    while let Some(prev_idx) = came_from[current_idx] {
        current_idx = prev_idx;
        path_indices.push(current_idx);
    }
    path_indices.reverse();
    let mut path: Vec<GridPos> = Vec::with_capacity(path_indices.len());
    for idx in path_indices {
        let x = (idx as i32) % width;
        let y = (idx as i32) / width;
        path.push(GridPos::new(x, y));
    }
    path
}

#[cfg(test)]
mod tests {
    use super::{find_path, GridCostMap, GridPos};

    #[test]
    fn returns_singleton_when_from_equals_to() {
        let grid = GridCostMap::new(8, 8);
        let from = GridPos::new(2, 3);
        let path = find_path(&grid, from, from, 200);
        assert_eq!(path, vec![from]);
    }

    #[test]
    fn returns_empty_for_blocked_target() {
        let mut grid = GridCostMap::new(8, 8);
        grid.set_walkable(6, 6, false);
        let path = find_path(&grid, GridPos::new(0, 0), GridPos::new(6, 6), 200);
        assert!(path.is_empty());
    }

    #[test]
    fn finds_basic_straight_path() {
        let grid = GridCostMap::new(8, 8);
        let path = find_path(&grid, GridPos::new(0, 0), GridPos::new(3, 0), 200);
        assert_eq!(path.first().copied(), Some(GridPos::new(0, 0)));
        assert_eq!(path.last().copied(), Some(GridPos::new(3, 0)));
        assert_eq!(path.len(), 4);
    }

    #[test]
    fn routes_around_wall_gap() {
        let mut grid = GridCostMap::new(8, 8);
        for y in 0..8 {
            if y != 4 {
                grid.set_walkable(3, y, false);
            }
        }

        let path = find_path(&grid, GridPos::new(1, 1), GridPos::new(6, 6), 300);
        assert!(!path.is_empty());
        assert_eq!(path.first().copied(), Some(GridPos::new(1, 1)));
        assert_eq!(path.last().copied(), Some(GridPos::new(6, 6)));
        assert!(path.contains(&GridPos::new(3, 4)));
    }

    #[test]
    fn builds_grid_from_flat_bytes_with_clamped_costs() {
        let walkable = vec![1_u8, 0_u8, 1_u8, 1_u8];
        let move_cost = vec![1.0_f32, -2.0_f32, 3.5_f32, 0.2_f32];
        let grid = GridCostMap::from_flat_bytes_unchecked(2, 2, &walkable, &move_cost);

        assert!(grid.is_walkable(0, 0));
        assert!(!grid.is_walkable(1, 0));
        assert_eq!(grid.get_move_cost(0, 0), 1.0);
        assert_eq!(grid.get_move_cost(1, 0), 0.0);
        assert_eq!(grid.get_move_cost(0, 1), 3.5);
    }

    #[test]
    fn returns_empty_when_start_is_out_of_bounds() {
        let grid = GridCostMap::new(8, 8);
        let path = find_path(&grid, GridPos::new(-1, 0), GridPos::new(6, 6), 200);
        assert!(path.is_empty());
    }
}
