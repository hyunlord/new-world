use std::cmp::Ordering;
use std::collections::BinaryHeap;

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

#[derive(Debug, Clone, Copy, PartialEq)]
struct OpenEntry {
    idx: usize,
    f: f32,
}

impl Eq for OpenEntry {}

impl Ord for OpenEntry {
    fn cmp(&self, other: &Self) -> Ordering {
        other
            .f
            .partial_cmp(&self.f)
            .unwrap_or(Ordering::Equal)
            .then_with(|| other.idx.cmp(&self.idx))
    }
}

impl PartialOrd for OpenEntry {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

/// Reusable scratch buffers for repeated pathfinding on the same grid size.
#[derive(Debug, Clone)]
pub struct PathfindWorkspace {
    node_count: usize,
    open_set: BinaryHeap<OpenEntry>,
    query_id: u32,
    seen_gen: Vec<u32>,
    closed_gen: Vec<u32>,
    came_from: Vec<usize>,
    g_score: Vec<f32>,
    f_score: Vec<f32>,
}

impl PathfindWorkspace {
    /// Creates a workspace sized for `node_count` grid nodes.
    pub fn new(node_count: usize) -> Self {
        Self {
            node_count,
            open_set: BinaryHeap::new(),
            query_id: 0,
            seen_gen: vec![0; node_count],
            closed_gen: vec![0; node_count],
            came_from: vec![usize::MAX; node_count],
            g_score: vec![INF_SCORE; node_count],
            f_score: vec![INF_SCORE; node_count],
        }
    }

    fn ensure_node_count(&mut self, node_count: usize) {
        if self.node_count == node_count {
            return;
        }
        self.node_count = node_count;
        self.open_set.clear();
        self.query_id = 0;
        self.seen_gen = vec![0; node_count];
        self.closed_gen = vec![0; node_count];
        self.came_from = vec![usize::MAX; node_count];
        self.g_score = vec![INF_SCORE; node_count];
        self.f_score = vec![INF_SCORE; node_count];
    }

    fn begin_query(&mut self) -> u32 {
        self.open_set.clear();
        if self.query_id == u32::MAX {
            self.seen_gen.fill(0);
            self.closed_gen.fill(0);
            self.query_id = 1;
        } else {
            self.query_id += 1;
        }
        self.query_id
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
    pub fn from_flat_unchecked(
        width: i32,
        height: i32,
        walkable: &[bool],
        move_cost: &[f32],
    ) -> Self {
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

    /// Builds a grid from owned flat vectors, clamping move-cost in place.
    ///
    /// Caller must ensure lengths match `width * height`.
    pub fn from_flat_owned_unchecked(
        width: i32,
        height: i32,
        walkable: Vec<bool>,
        mut move_cost: Vec<f32>,
    ) -> Self {
        debug_assert_eq!(walkable.len(), (width * height) as usize);
        debug_assert_eq!(move_cost.len(), (width * height) as usize);
        for cost in &mut move_cost {
            *cost = (*cost).max(0.0);
        }
        Self {
            width,
            height,
            walkable,
            move_cost,
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
    let node_count = (grid.width * grid.height) as usize;
    let mut workspace = PathfindWorkspace::new(node_count);
    find_path_with_workspace(grid, from, to, max_steps, &mut workspace)
}

/// A* pathfinding using caller-provided reusable scratch buffers.
pub fn find_path_with_workspace(
    grid: &GridCostMap,
    from: GridPos,
    to: GridPos,
    max_steps: usize,
    workspace: &mut PathfindWorkspace,
) -> Vec<GridPos> {
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
    let to_x = to.x;
    let to_y = to.y;
    let node_count = (grid.width * grid.height) as usize;
    workspace.ensure_node_count(node_count);
    let query_id = workspace.begin_query();

    workspace.seen_gen[from_idx] = query_id;
    workspace.came_from[from_idx] = from_idx;
    workspace.g_score[from_idx] = 0.0;
    workspace.f_score[from_idx] = chebyshev_xy(from.x, from.y, to_x, to_y);
    workspace.open_set.push(OpenEntry {
        idx: from_idx,
        f: workspace.f_score[from_idx],
    });

    let mut steps = 0usize;
    while let Some(current_entry) = workspace.open_set.pop() {
        let current_idx = current_entry.idx;
        if workspace.seen_gen[current_idx] != query_id {
            continue;
        }
        if workspace.closed_gen[current_idx] == query_id {
            continue;
        }
        if current_entry.f > workspace.f_score[current_idx] {
            continue;
        }
        if steps >= max_steps {
            break;
        }
        steps += 1;

        if current_idx == to_idx {
            return reconstruct_path(&workspace.came_from, current_idx, from_idx, grid.width);
        }

        workspace.closed_gen[current_idx] = query_id;
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
                if workspace.closed_gen[neighbor_idx] == query_id || !grid.walkable[neighbor_idx] {
                    continue;
                }

                let step_cost = if dx.abs() + dy.abs() == 1 {
                    CARDINAL_COST
                } else {
                    DIAGONAL_COST
                };
                let tentative_g =
                    workspace.g_score[current_idx] + (step_cost * grid.move_cost[neighbor_idx]);

                if workspace.seen_gen[neighbor_idx] != query_id
                    || tentative_g < workspace.g_score[neighbor_idx]
                {
                    workspace.seen_gen[neighbor_idx] = query_id;
                    workspace.came_from[neighbor_idx] = current_idx;
                    workspace.g_score[neighbor_idx] = tentative_g;
                    let neighbor_f = tentative_g + chebyshev_xy(neighbor_x, neighbor_y, to_x, to_y);
                    workspace.f_score[neighbor_idx] = neighbor_f;
                    workspace.open_set.push(OpenEntry {
                        idx: neighbor_idx,
                        f: neighbor_f,
                    });
                }
            }
        }
    }

    Vec::new()
}

#[inline]
fn chebyshev_xy(ax: i32, ay: i32, bx: i32, by: i32) -> f32 {
    (ax.abs_diff(bx).max(ay.abs_diff(by))) as f32
}

fn reconstruct_path(
    came_from: &[usize],
    mut current_idx: usize,
    start_idx: usize,
    width: i32,
) -> Vec<GridPos> {
    let mut path_indices: Vec<usize> = vec![current_idx];
    while current_idx != start_idx {
        let prev_idx = came_from[current_idx];
        if prev_idx == usize::MAX {
            return Vec::new();
        }
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
    use super::{find_path, find_path_with_workspace, GridCostMap, GridPos, PathfindWorkspace};

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
    fn builds_grid_from_owned_flat_vectors_with_clamped_costs() {
        let walkable = vec![true, false, true, true];
        let move_cost = vec![1.0_f32, -2.0_f32, 3.5_f32, 0.2_f32];
        let grid = GridCostMap::from_flat_owned_unchecked(2, 2, walkable, move_cost);

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

    #[test]
    fn reuses_workspace_without_state_leak() {
        let mut grid = GridCostMap::new(8, 8);
        grid.set_walkable(3, 3, false);

        let mut ws = PathfindWorkspace::new(64);
        let path_a =
            find_path_with_workspace(&grid, GridPos::new(0, 0), GridPos::new(7, 7), 300, &mut ws);
        let path_b =
            find_path_with_workspace(&grid, GridPos::new(7, 0), GridPos::new(0, 7), 300, &mut ws);

        let expected_a = find_path(&grid, GridPos::new(0, 0), GridPos::new(7, 7), 300);
        let expected_b = find_path(&grid, GridPos::new(7, 0), GridPos::new(0, 7), 300);
        assert_eq!(path_a, expected_a);
        assert_eq!(path_b, expected_b);
    }

    #[test]
    fn wraps_workspace_generation_counter_without_state_leak() {
        let grid = GridCostMap::new(8, 8);
        let mut ws = PathfindWorkspace::new(64);
        ws.query_id = u32::MAX;
        ws.seen_gen.fill(u32::MAX);
        ws.closed_gen.fill(u32::MAX);

        let path_a =
            find_path_with_workspace(&grid, GridPos::new(0, 0), GridPos::new(7, 7), 300, &mut ws);
        let path_b =
            find_path_with_workspace(&grid, GridPos::new(7, 0), GridPos::new(0, 7), 300, &mut ws);

        let expected_a = find_path(&grid, GridPos::new(0, 0), GridPos::new(7, 7), 300);
        let expected_b = find_path(&grid, GridPos::new(7, 0), GridPos::new(0, 7), 300);

        assert_eq!(ws.query_id, 2);
        assert_eq!(path_a, expected_a);
        assert_eq!(path_b, expected_b);
    }
}
