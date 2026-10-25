//! Pathfinding (M3, ADR 0021): the colony's passability cost map, its
//! connected regions and an integer A* with a path memo.
//!
//! Pure data and algorithms — no ECS, no rules. `sim_systems::runtime::nav`
//! builds the map from the tile grid and the colony's space sites, keeps it
//! current incrementally (`NavState::set_cost` bumps `version`) and asks for
//! paths. The baseline world never builds a map (its `NavState` stays empty).
//!
//! Moves: 8 neighbours; a diagonal step is allowed only when both orthogonal
//! neighbours are passable (no corner cutting). Entering a tile of cost `c`
//! costs `c` straight and `c · 14 / 10` diagonally (plain 10 / 14, door 12 / 16).
//! The heuristic is the octile distance at the plain cost
//! (`10 · max(dx, dy) + 4 · min(dx, dy)`), admissible and consistent because no
//! passable tile costs less than plain. Ties in the open set break by
//! `(f, h, y, x)`, so a search is a pure function of the map and its endpoints.
//! A goal on an impassable tile is reached from any passable 8-neighbour.
//!
//! Regions (connected components under the same move rule) are recomputed
//! lazily once per `version`; a request whose goal lies in another region
//! returns `None` without searching. The memo holds paths of the current
//! version by `(from, to)` (at most [`MEMO_CAPACITY`], evicted in insertion
//! order) and returns exactly what a fresh search would, so it never changes
//! behaviour. Derived parts (map, regions, memo, scratch, stats) are not saved.
use serde::{Deserialize, Serialize};
use std::cell::RefCell;
use std::cmp::Reverse;
use std::collections::{BTreeMap, BinaryHeap, VecDeque};
use std::sync::Arc;

use crate::components::AgentId;

/// A tile `(x, y)`.
pub type Tile = (u32, u32);

/// Cost byte of an impassable tile.
pub const IMPASSABLE: u8 = 0;
/// Cost byte of plain passable ground (grass, forest, floor, dug rock).
pub const COST_PLAIN: u8 = 10;
/// Cost byte of a door tile.
pub const COST_DOOR: u8 = 12;
/// Cost byte of a ford: a shallow river tile (M4, placed by the colony worldgen).
pub const COST_BANK: u8 = 30;
/// Most paths the memo keeps.
pub const MEMO_CAPACITY: usize = 4_096;

/// Chebyshev distance.
pub fn chebyshev(a: Tile, b: Tile) -> u32 {
    a.0.abs_diff(b.0).max(a.1.abs_diff(b.1))
}

/// Octile distance at the plain cost: `10 · max + 4 · min`.
pub fn octile(a: Tile, b: Tile) -> u32 {
    let dx = a.0.abs_diff(b.0);
    let dy = a.1.abs_diff(b.1);
    10 * dx.max(dy) + 4 * dx.min(dy)
}

/// One byte per tile: 0 impassable, else the cost of entering it straight.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct CostMap {
    width: u32,
    height: u32,
    cost: Vec<u8>,
}

impl CostMap {
    /// A `width × height` map of plain ground.
    pub fn new(width: u32, height: u32) -> Self {
        Self {
            width,
            height,
            cost: vec![COST_PLAIN; (width as usize) * (height as usize)],
        }
    }

    /// Width in tiles.
    pub fn width(&self) -> u32 {
        self.width
    }

    /// Height in tiles.
    pub fn height(&self) -> u32 {
        self.height
    }

    /// Whether the map has no tiles (not built).
    pub fn is_empty(&self) -> bool {
        self.cost.is_empty()
    }

    fn index(&self, t: Tile) -> Option<usize> {
        (t.0 < self.width && t.1 < self.height)
            .then(|| (t.1 as usize) * (self.width as usize) + (t.0 as usize))
    }

    /// The cost byte of `t` (0 outside the map).
    pub fn get(&self, t: Tile) -> u8 {
        self.index(t).map_or(IMPASSABLE, |i| self.cost[i])
    }

    /// Set the cost byte of `t`; returns whether it changed.
    pub fn set(&mut self, t: Tile, cost: u8) -> bool {
        match self.index(t) {
            Some(i) if self.cost[i] != cost => {
                self.cost[i] = cost;
                true
            }
            _ => false,
        }
    }

    /// Whether `t` is on the map and passable.
    pub fn passable(&self, t: Tile) -> bool {
        self.get(t) != IMPASSABLE
    }

    /// Whether a one-tile step `a → b` (8-neighbours) is allowed: `b` passable
    /// and, for a diagonal, both orthogonal neighbours passable.
    pub fn can_step(&self, a: Tile, b: Tile) -> bool {
        if chebyshev(a, b) != 1 || !self.passable(b) {
            return false;
        }
        // Someone standing where no one can stand (walled in, or put there)
        // may step off onto any passable neighbour.
        if a.0 != b.0 && a.1 != b.1 && self.passable(a) {
            return self.passable((b.0, a.1)) && self.passable((a.0, b.1));
        }
        true
    }

    /// The cost of the step `a → b` (straight or diagonal), assuming it is allowed.
    pub fn step_cost(&self, a: Tile, b: Tile) -> u32 {
        let c = u32::from(self.get(b));
        if a.0 != b.0 && a.1 != b.1 {
            c * 14 / 10
        } else {
            c
        }
    }

    /// The 8 neighbours of `t` reachable in one allowed step, in `(dy, dx)` order.
    fn steps_from(&self, t: Tile) -> impl Iterator<Item = Tile> + '_ {
        const D: [(i64, i64); 8] = [
            (-1, -1),
            (0, -1),
            (1, -1),
            (-1, 0),
            (1, 0),
            (-1, 1),
            (0, 1),
            (1, 1),
        ];
        D.iter().filter_map(move |&(dx, dy)| {
            let nx = i64::from(t.0) + dx;
            let ny = i64::from(t.1) + dy;
            if nx < 0 || ny < 0 {
                return None;
            }
            let n = (nx as u32, ny as u32);
            self.can_step(t, n).then_some(n)
        })
    }
}

/// Connected components of the passable tiles: label 0 = impassable, labels
/// from 1 in `(y, x)` order of each component's first tile.
#[derive(Debug, Clone, Default)]
struct Regions {
    version: Option<u64>,
    labels: Vec<u32>,
}

impl Regions {
    fn compute(map: &CostMap) -> Vec<u32> {
        let mut labels = vec![0u32; map.cost.len()];
        let width = map.width as usize;
        let mut next = 1u32;
        let mut queue: VecDeque<usize> = VecDeque::new();
        for seed in 0..map.cost.len() {
            if labels[seed] != 0 || map.cost[seed] == IMPASSABLE {
                continue;
            }
            labels[seed] = next;
            queue.push_back(seed);
            while let Some(i) = queue.pop_front() {
                let x = i % width;
                let mut visit = |j: usize| {
                    if labels[j] == 0 && map.cost[j] != IMPASSABLE {
                        labels[j] = next;
                        queue.push_back(j);
                    }
                };
                // Legal diagonals already connect through passable cardinal neighbours.
                if i >= width {
                    visit(i - width);
                }
                if x > 0 {
                    visit(i - 1);
                }
                if x + 1 < width {
                    visit(i + 1);
                }
                if i + width < map.cost.len() {
                    visit(i + width);
                }
            }
            next += 1;
        }
        labels
    }
}

/// Reusable A* scratch arrays, stamped per search so nothing is cleared.
#[derive(Debug, Clone, Default)]
struct Scratch {
    stamp: Vec<u32>,
    closed: Vec<u32>,
    g: Vec<u32>,
    parent: Vec<u32>,
    generation: u32,
}

/// The path memo of one `version`.
#[derive(Debug, Clone, Default)]
struct Memo {
    version: u64,
    paths: BTreeMap<(Tile, Tile), Arc<Vec<Tile>>>,
    order: VecDeque<(Tile, Tile)>,
}

/// Measurement counters (`worldsim_perf nav-bench`); never read by a system.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct NavStats {
    /// Path requests.
    pub requests: u64,
    /// Requests answered from the memo.
    pub memo_hits: u64,
    /// A* searches run.
    pub searches: u64,
    /// Nodes expanded over every search.
    pub expanded: u64,
    /// Requests refused because the goal lies in another region.
    pub unreachable: u64,
    /// Region recomputations.
    pub region_builds: u64,
    /// Stored paths kept after a version change (re-validated).
    pub kept: u64,
    /// Stored paths re-planned.
    pub replans: u64,
}

/// One agent's walk: the goal it planned for, the map version it checked
/// against, the steps (first step first) and how many are done.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AgentPath {
    /// The goal the path was planned for.
    pub goal: Tile,
    /// The `NavState::version` the remaining steps were last checked against.
    pub version: u64,
    /// Every step, the first step first.
    pub steps: Vec<Tile>,
    /// Steps already taken.
    pub next: u32,
    /// Tick it was (re)planned (the job layer's stuck rule restarts from it).
    pub planned: u64,
    /// The tile it was planned from.
    pub from: Tile,
}

impl AgentPath {
    /// A fresh path from `from` for `goal`, planned at `tick` against `version`.
    pub fn new(goal: Tile, version: u64, from: Tile, steps: Vec<Tile>, tick: u64) -> Self {
        Self {
            goal,
            version,
            steps,
            next: 0,
            planned: tick,
            from,
        }
    }

    /// The tile the walk started from (where the agent stands before its first step).
    pub fn planned_from(&self) -> Option<Tile> {
        Some(self.from)
    }

    /// Steps still to take.
    pub fn remaining(&self) -> u32 {
        (self.steps.len() as u32).saturating_sub(self.next)
    }

    /// The next step, if any.
    pub fn peek(&self) -> Option<Tile> {
        self.steps.get(self.next as usize).copied()
    }
}

/// The colony's navigation state (`SimResources::nav`).
#[derive(Debug, Default, Serialize, Deserialize)]
pub struct NavState {
    /// Bumped by every cost change; paths and the memo are valid for one version.
    pub version: u64,
    /// Each walking agent's path (saved: progress along a path is state).
    pub paths: BTreeMap<AgentId, AgentPath>,
    /// Agents whose goal turned out unreachable at this tick's move (written by
    /// `AgentMovementSystem`, read by the job layer the same tick).
    pub blocked: BTreeMap<AgentId, Tile>,
    #[serde(skip)]
    map: CostMap,
    #[serde(skip)]
    regions: RefCell<Regions>,
    #[serde(skip)]
    memo: RefCell<Memo>,
    #[serde(skip)]
    scratch: RefCell<Scratch>,
    /// Measurement only.
    #[serde(skip)]
    pub stats: RefCell<NavStats>,
}

impl NavState {
    /// The cost map (empty until built).
    pub fn map(&self) -> &CostMap {
        &self.map
    }

    /// Whether the cost map has been built.
    pub fn is_built(&self) -> bool {
        !self.map.is_empty()
    }

    /// Replace the whole cost map (a build or a rebuild after load); the
    /// version is unchanged, derived parts are dropped.
    pub fn install(&mut self, map: CostMap) {
        self.map = map;
        *self.regions.borrow_mut() = Regions::default();
        *self.memo.borrow_mut() = Memo::default();
    }

    /// Set `t`'s cost; a change bumps `version` (paths re-validate, the memo
    /// and the regions are dropped on next use). Returns whether it changed.
    pub fn set_cost(&mut self, t: Tile, cost: u8) -> bool {
        let changed = self.map.set(t, cost);
        if changed {
            self.version += 1;
        }
        changed
    }

    /// Whether `t` is passable (false before the map is built).
    pub fn passable(&self, t: Tile) -> bool {
        self.map.passable(t)
    }

    /// Whether the step `a → b` is allowed.
    pub fn can_step(&self, a: Tile, b: Tile) -> bool {
        self.map.can_step(a, b)
    }

    fn with_regions<R>(&self, f: impl FnOnce(&[u32]) -> R) -> R {
        let mut regions = self.regions.borrow_mut();
        if regions.version != Some(self.version) || regions.labels.len() != self.map.cost.len() {
            regions.labels = Regions::compute(&self.map);
            regions.version = Some(self.version);
            self.stats.borrow_mut().region_builds += 1;
        }
        f(&regions.labels)
    }

    /// The region of `t` (0 = impassable or off the map).
    pub fn region_of(&self, t: Tile) -> u32 {
        let Some(i) = self.map.index(t) else {
            return 0;
        };
        self.with_regions(|labels| labels[i])
    }

    /// The regions from which `to` can be reached: its own, or for an
    /// impassable `to` those of its passable 8-neighbours (sorted, unique).
    pub fn goal_regions(&self, to: Tile) -> Vec<u32> {
        let map = &self.map;
        self.with_regions(|labels| {
            let label = |t: Tile| map.index(t).map_or(0, |i| labels[i]);
            if map.passable(to) {
                return vec![label(to)];
            }
            let mut out: Vec<u32> = Vec::new();
            for dy in -1i64..=1 {
                for dx in -1i64..=1 {
                    let (nx, ny) = (i64::from(to.0) + dx, i64::from(to.1) + dy);
                    if (dx, dy) == (0, 0) || nx < 0 || ny < 0 {
                        continue;
                    }
                    let l = label((nx as u32, ny as u32));
                    if l != 0 && !out.contains(&l) {
                        out.push(l);
                    }
                }
            }
            out.sort_unstable();
            out
        })
    }

    /// Whether `to` can be reached from region `region` (see [`Self::goal_regions`]),
    /// without allocating.
    pub fn reaches(&self, region: u32, to: Tile) -> bool {
        if region == 0 {
            return false;
        }
        let map = &self.map;
        self.with_regions(|labels| {
            let label = |t: Tile| map.index(t).map_or(0, |i| labels[i]);
            if map.passable(to) {
                return label(to) == region;
            }
            for dy in -1i64..=1 {
                for dx in -1i64..=1 {
                    let (nx, ny) = (i64::from(to.0) + dx, i64::from(to.1) + dy);
                    if (dx, dy) != (0, 0)
                        && nx >= 0
                        && ny >= 0
                        && label((nx as u32, ny as u32)) == region
                    {
                        return true;
                    }
                }
            }
            false
        })
    }

    /// Whether `to` can be reached from `from` (`from` itself must be passable).
    pub fn reachable(&self, from: Tile, to: Tile) -> bool {
        let r = self.region_of(from);
        r != 0 && self.goal_regions(to).contains(&r)
    }

    /// Whether `pos` counts as at `goal`: on it, or 8-adjacent to an impassable goal.
    pub fn arrived(&self, pos: Tile, goal: Tile) -> bool {
        pos == goal || (self.is_built() && chebyshev(pos, goal) == 1 && !self.map.passable(goal))
    }

    /// The path from `from` to `to` (first step first; empty when already
    /// there), or `None` when `to` cannot be reached. Memoised per version.
    pub fn path(&self, from: Tile, to: Tile) -> Option<Arc<Vec<Tile>>> {
        if self.is_built() && !self.map.passable(from) {
            // Step off an impassable tile first: the passable neighbour that
            // reaches `to`, nearest to it by octile, ties `(y, x)`.
            let off = self
                .map
                .steps_from(from)
                .filter(|n| self.reachable(*n, to))
                .min_by_key(|n| (octile(*n, to), n.1, n.0))?;
            let rest = self.path(off, to)?;
            let mut steps = Vec::with_capacity(rest.len() + 1);
            steps.push(off);
            steps.extend(rest.iter().copied());
            return Some(Arc::new(steps));
        }
        self.stats.borrow_mut().requests += 1;
        if !self.reachable(from, to) {
            self.stats.borrow_mut().unreachable += 1;
            return None;
        }
        {
            let mut memo = self.memo.borrow_mut();
            if memo.version != self.version {
                *memo = Memo {
                    version: self.version,
                    ..Memo::default()
                };
            }
            if let Some(found) = memo.paths.get(&(from, to)) {
                self.stats.borrow_mut().memo_hits += 1;
                return Some(Arc::clone(found));
            }
        }
        let found = Arc::new(self.search(from, to)?);
        let mut memo = self.memo.borrow_mut();
        if memo.paths.len() >= MEMO_CAPACITY {
            if let Some(old) = memo.order.pop_front() {
                memo.paths.remove(&old);
            }
        }
        memo.paths.insert((from, to), Arc::clone(&found));
        memo.order.push_back((from, to));
        Some(found)
    }

    /// Whether every step of `steps` from `from` is still allowed.
    pub fn still_walkable(&self, from: Tile, steps: &[Tile]) -> bool {
        let mut at = from;
        for &s in steps {
            if !self.map.can_step(at, s) {
                return false;
            }
            at = s;
        }
        true
    }

    /// Plain A* (no memo, no region test). `None` if the search exhausts.
    fn search(&self, from: Tile, to: Tile) -> Option<Vec<Tile>> {
        let map = &self.map;
        let w = map.width as usize;
        let n = map.cost.len();
        let start = map.index(from)?;
        let goal_open = map.passable(to);
        let is_goal = |t: Tile| {
            if goal_open {
                t == to
            } else {
                chebyshev(t, to) == 1
            }
        };
        let h = |t: Tile| {
            let d = octile(t, to);
            if goal_open {
                d
            } else {
                d.saturating_sub(14)
            }
        };
        self.stats.borrow_mut().searches += 1;
        if is_goal(from) {
            return Some(Vec::new());
        }
        let mut s = self.scratch.borrow_mut();
        if s.stamp.len() != n {
            *s = Scratch {
                stamp: vec![0; n],
                closed: vec![0; n],
                g: vec![0; n],
                parent: vec![0; n],
                generation: 0,
            };
        }
        s.generation = s.generation.wrapping_add(1);
        if s.generation == 0 {
            s.stamp.iter_mut().for_each(|v| *v = 0);
            s.closed.iter_mut().for_each(|v| *v = 0);
            s.generation = 1;
        }
        let gen = s.generation;
        let mut open: BinaryHeap<Reverse<(u32, u32, u32, u32)>> = BinaryHeap::new();
        s.stamp[start] = gen;
        s.g[start] = 0;
        s.parent[start] = start as u32;
        open.push(Reverse((h(from), h(from), from.1, from.0)));
        let mut expanded = 0u64;
        let mut reached: Option<usize> = None;
        while let Some(Reverse((_, _, y, x))) = open.pop() {
            let t = (x, y);
            let i = (y as usize) * w + (x as usize);
            if s.closed[i] == gen {
                continue;
            }
            s.closed[i] = gen;
            expanded += 1;
            if is_goal(t) {
                reached = Some(i);
                break;
            }
            let gi = s.g[i];
            for nb in map.steps_from(t) {
                let j = (nb.1 as usize) * w + (nb.0 as usize);
                if s.closed[j] == gen {
                    continue;
                }
                let g = gi + map.step_cost(t, nb);
                if s.stamp[j] != gen || g < s.g[j] {
                    s.stamp[j] = gen;
                    s.g[j] = g;
                    s.parent[j] = i as u32;
                    let hn = h(nb);
                    open.push(Reverse((g + hn, hn, nb.1, nb.0)));
                }
            }
        }
        self.stats.borrow_mut().expanded += expanded;
        let mut at = reached?;
        let mut out = Vec::new();
        while at != start {
            out.push(((at % w) as u32, (at / w) as u32));
            at = s.parent[at] as usize;
        }
        out.reverse();
        Some(out)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// A map from rows of `.` (plain), `#` (impassable), `D` (door).
    fn map_of(rows: &[&str]) -> CostMap {
        let mut m = CostMap::new(rows[0].len() as u32, rows.len() as u32);
        for (y, row) in rows.iter().enumerate() {
            for (x, c) in row.chars().enumerate() {
                let cost = match c {
                    '#' => IMPASSABLE,
                    'D' => COST_DOOR,
                    _ => COST_PLAIN,
                };
                m.set((x as u32, y as u32), cost);
            }
        }
        m
    }

    fn nav(rows: &[&str]) -> NavState {
        let mut n = NavState::default();
        n.install(map_of(rows));
        n
    }

    fn cost_of(n: &NavState, from: Tile, path: &[Tile]) -> u32 {
        let mut at = from;
        let mut total = 0;
        for &s in path {
            assert!(n.can_step(at, s), "illegal step {at:?} -> {s:?}");
            total += n.map().step_cost(at, s);
            at = s;
        }
        total
    }

    #[test]
    fn nav_astar_basic() {
        // Open ground: (0, 0) → (4, 2) costs 2 diagonals + 2 straight = 28 + 20.
        let n = nav(&[".....", ".....", "....."]);
        let p = n.path((0, 0), (4, 2)).expect("path");
        assert_eq!(p.len(), 4);
        assert_eq!(p.last(), Some(&(4, 2)));
        assert_eq!(cost_of(&n, (0, 0), &p), 48);
        // The tie-break is (f, h, y, x): with equal f the lowest-h node goes
        // first, then lower y — the exact path below (computed by hand).
        assert_eq!(*p, vec![(1, 1), (2, 2), (3, 2), (4, 2)]);
        // A wall with one gap: the path goes through the gap.
        let n = nav(&[".#...", ".#...", "...#.", ".#..."]);
        let p = n.path((0, 0), (4, 0)).expect("around");
        assert!(p.contains(&(1, 2)), "{p:?}");
        // By hand: down to (0, 2), through the gap to (2, 2) (the corners at (1, 1)
        // and (1, 3) forbid the diagonals), up to (2, 1), diagonal to (3, 0), on to (4, 0).
        assert_eq!(cost_of(&n, (0, 0), &p), 10 + 10 + 10 + 10 + 10 + 14 + 10);
        // Already there: an empty path.
        assert_eq!(n.path((2, 2), (2, 2)).map(|p| p.len()), Some(0));
    }

    #[test]
    fn nav_corner_cutting() {
        // (0,0) → (1,1) diagonally would squeeze between two walls.
        let n = nav(&[".#", "#."]);
        assert!(!n.can_step((0, 0), (1, 1)));
        assert!(n.path((0, 0), (1, 1)).is_none());
        // One wall: the diagonal past its corner is refused, the detour costs 20.
        let n = nav(&[".#", ".."]);
        assert!(!n.can_step((0, 0), (1, 1)));
        let p = n.path((0, 0), (1, 1)).expect("detour");
        assert_eq!(*p, vec![(0, 1), (1, 1)]);
    }

    #[test]
    fn nav_unreachable() {
        let n = nav(&["..#..", "..#..", "..#.."]);
        assert!(!n.reachable((0, 0), (4, 0)));
        assert!(n.path((0, 0), (4, 0)).is_none());
        assert_eq!(n.stats.borrow().searches, 0, "no search for another region");
        assert_eq!(n.stats.borrow().unreachable, 1);
        // An impassable goal is reached from a neighbour: the wall tile (2, 1)
        // from the left side.
        let p = n.path((0, 1), (2, 1)).expect("adjacent");
        assert_eq!(p.last(), Some(&(1, 1)));
        assert!(n.arrived((1, 1), (2, 1)));
        assert!(!n.arrived((1, 1), (3, 1)));
    }

    #[test]
    fn nav_incremental_update() {
        let mut n = nav(&[".....", ".....", "....."]);
        let v0 = n.version;
        let before = n.path((0, 1), (4, 1)).expect("straight");
        assert_eq!(before.len(), 4);
        // Build a wall across rows 0-1 at x = 2: the version bumps and the path bends.
        assert!(n.set_cost((2, 0), IMPASSABLE));
        assert!(n.set_cost((2, 1), IMPASSABLE));
        assert!(!n.set_cost((2, 1), IMPASSABLE), "no change, no bump");
        assert_eq!(n.version, v0 + 2);
        assert!(!n.still_walkable((0, 1), &before));
        let after = n.path((0, 1), (4, 1)).expect("around");
        assert!(after.contains(&(2, 2)), "{after:?}");
        // Close the gap: unreachable. Open a door: reachable through it at cost 12.
        n.set_cost((2, 2), IMPASSABLE);
        assert!(n.path((0, 1), (4, 1)).is_none());
        n.set_cost((2, 1), COST_DOOR);
        let p = n.path((0, 1), (4, 1)).expect("door");
        assert_eq!(*p, vec![(1, 1), (2, 1), (3, 1), (4, 1)]);
        assert_eq!(cost_of(&n, (0, 1), &p), 10 + 12 + 10 + 10);
    }

    #[test]
    fn nav_memo_hit() {
        let mut n = nav(&["........", "........", "........"]);
        let a = n.path((0, 0), (7, 2)).expect("a");
        let b = n.path((0, 0), (7, 2)).expect("b");
        assert!(Arc::ptr_eq(&a, &b));
        let s = *n.stats.borrow();
        assert_eq!((s.requests, s.memo_hits, s.searches), (2, 1, 1));
        // A version change drops the memo; the fresh search gives the same path.
        n.set_cost((5, 0), IMPASSABLE);
        n.set_cost((5, 0), COST_PLAIN);
        let c = n.path((0, 0), (7, 2)).expect("c");
        assert!(!Arc::ptr_eq(&a, &c));
        assert_eq!(*a, *c);
        assert_eq!(n.stats.borrow().searches, 2);
    }

    #[test]
    fn someone_on_an_impassable_tile_steps_off() {
        // (1, 1) is impassable and its straight neighbours too; only the corners are open.
        let n = nav(&[".#.", "###", "..."]);
        assert!(n.can_step((1, 1), (0, 0)), "off a wall onto a corner");
        assert!(!n.can_step((0, 0), (1, 1)), "not onto it");
        let p = n.path((1, 1), (0, 2)).expect("step off, then walk");
        assert_eq!(*p, vec![(0, 2)]);
        let p = n.path((1, 1), (2, 0)).expect("the other corner");
        assert_eq!(*p, vec![(2, 0)]);
    }

    #[test]
    fn regions_follow_the_move_rule() {
        // Two passable tiles touching only at a corner between walls are separate regions.
        let n = nav(&[".#", "#."]);
        assert_ne!(n.region_of((0, 0)), n.region_of((1, 1)));
        assert_eq!(n.region_of((1, 0)), 0);
        let n = nav(&["...", "...", "..."]);
        assert_eq!(n.region_of((0, 0)), n.region_of((2, 2)));
        assert_eq!(n.region_of((9, 9)), 0, "off the map");
    }

    #[test]
    fn a_search_matches_a_dijkstra_cost_on_random_maps() {
        // 200 random 24×24 maps with 30 % walls (an LCG, no float): the A*
        // path cost equals an independent Dijkstra's, and every step is legal.
        let mut seed = 0x1234_5678_u64;
        let mut next = || {
            seed = seed
                .wrapping_mul(6364136223846793005)
                .wrapping_add(1442695040888963407);
            (seed >> 33) as u32
        };
        for _ in 0..200 {
            let mut m = CostMap::new(24, 24);
            for y in 0..24 {
                for x in 0..24 {
                    let r = next() % 100;
                    let c = if r < 30 {
                        IMPASSABLE
                    } else if r < 35 {
                        COST_DOOR
                    } else {
                        COST_PLAIN
                    };
                    m.set((x, y), c);
                }
            }
            let from = (next() % 24, next() % 24);
            let to = (next() % 24, next() % 24);
            m.set(from, COST_PLAIN);
            let mut n = NavState::default();
            n.install(m.clone());
            // Dijkstra over the same moves (goal passable or not).
            let mut dist = vec![u32::MAX; 24 * 24];
            let mut heap = BinaryHeap::new();
            dist[(from.1 * 24 + from.0) as usize] = 0;
            heap.push(Reverse((0u32, from.1, from.0)));
            let mut best = u32::MAX;
            while let Some(Reverse((d, y, x))) = heap.pop() {
                if d > dist[(y * 24 + x) as usize] {
                    continue;
                }
                let t = (x, y);
                if n.arrived(t, to) {
                    best = best.min(d);
                }
                for nb in m.steps_from(t) {
                    let nd = d + m.step_cost(t, nb);
                    let j = (nb.1 * 24 + nb.0) as usize;
                    if nd < dist[j] {
                        dist[j] = nd;
                        heap.push(Reverse((nd, nb.1, nb.0)));
                    }
                }
            }
            match n.path(from, to) {
                Some(p) => assert_eq!(cost_of(&n, from, &p), best, "{from:?} -> {to:?}"),
                None => assert_eq!(best, u32::MAX, "{from:?} -> {to:?} reachable"),
            }
        }
    }
}

#[cfg(test)]
#[path = "nav_regions_tests.rs"]
mod region_optimization_tests;
