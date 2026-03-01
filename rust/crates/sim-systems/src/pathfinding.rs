use std::collections::{HashMap, HashSet};

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
    if !grid.is_walkable(to.x, to.y) {
        return Vec::new();
    }

    let max_steps = if max_steps == 0 {
        DEFAULT_MAX_STEPS
    } else {
        max_steps
    };

    let mut open_set: Vec<GridPos> = vec![from];
    let mut in_open: HashSet<GridPos> = HashSet::from([from]);
    let mut came_from: HashMap<GridPos, GridPos> = HashMap::new();
    let mut g_score: HashMap<GridPos, f32> = HashMap::new();
    let mut f_score: HashMap<GridPos, f32> = HashMap::new();
    let mut closed_set: HashSet<GridPos> = HashSet::new();

    g_score.insert(from, 0.0);
    f_score.insert(from, chebyshev(from, to));

    let mut steps = 0usize;
    while !open_set.is_empty() && steps < max_steps {
        steps += 1;

        let mut best_idx = 0usize;
        let mut best_f = *f_score.get(&open_set[0]).unwrap_or(&INF_SCORE);
        for (idx, pos) in open_set.iter().enumerate().skip(1) {
            let f = *f_score.get(pos).unwrap_or(&INF_SCORE);
            if f < best_f {
                best_f = f;
                best_idx = idx;
            }
        }

        let current = open_set.remove(best_idx);
        in_open.remove(&current);

        if current == to {
            return reconstruct_path(&came_from, current);
        }

        closed_set.insert(current);

        for dy in -1..=1 {
            for dx in -1..=1 {
                if dx == 0 && dy == 0 {
                    continue;
                }

                let neighbor = GridPos::new(current.x + dx, current.y + dy);
                if closed_set.contains(&neighbor) {
                    continue;
                }
                if !grid.is_walkable(neighbor.x, neighbor.y) {
                    continue;
                }

                let step_cost = if dx.abs() + dy.abs() == 1 {
                    CARDINAL_COST
                } else {
                    DIAGONAL_COST
                };
                let terrain_cost = grid.get_move_cost(neighbor.x, neighbor.y);
                let tentative_g = g_score.get(&current).copied().unwrap_or(INF_SCORE)
                    + (step_cost * terrain_cost);

                if tentative_g < g_score.get(&neighbor).copied().unwrap_or(INF_SCORE) {
                    came_from.insert(neighbor, current);
                    g_score.insert(neighbor, tentative_g);
                    f_score.insert(neighbor, tentative_g + chebyshev(neighbor, to));

                    if !in_open.contains(&neighbor) {
                        open_set.push(neighbor);
                        in_open.insert(neighbor);
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

fn reconstruct_path(came_from: &HashMap<GridPos, GridPos>, mut current: GridPos) -> Vec<GridPos> {
    let mut path = vec![current];
    while let Some(prev) = came_from.get(&current).copied() {
        current = prev;
        path.push(current);
    }
    path.reverse();
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
}
