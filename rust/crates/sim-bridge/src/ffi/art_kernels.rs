//! Pure rendering-only kernels. No simulation state or Godot runtime dependencies.

/// Packed inputs preserve original row and dictionary iteration order.
pub struct DeclumpInput<'a> {
    /// Agent ids in original row order.
    pub row_ids: &'a [i64],
    /// Visibility flags, one byte per row.
    pub visible: &'a [u8],
    /// Resolved moving or walking-activity flags.
    pub walking: &'a [u8],
    /// Activity-place-known flags.
    pub place_known: &'a [u8],
    /// Target dictionary ids, including non-visible rows.
    pub target_ids: &'a [i64],
    /// Interleaved target x/y f32 components.
    pub target_xy: &'a [f32],
    /// Rectangle dictionary ids, including non-visible rows.
    pub rect_ids: &'a [i64],
    /// Interleaved local rectangle position and size components.
    pub rect_xywh: &'a [f32],
    /// Terrain width in tiles.
    pub width: i64,
    /// Terrain height in tiles.
    pub height: i64,
    /// Terrain codes; code four forbids a shift.
    pub terrain: &'a [u8],
    /// Interleaved signed blocked tile coordinates.
    pub blocked_xy: &'a [i32],
    /// Whether overlap uses the reference spatial index.
    pub spatial: bool,
}

/// Result dictionary insertion order, with absent and zero offsets distinct.
#[derive(Debug, PartialEq)]
pub struct DeclumpOutput {
    /// Ids in result dictionary insertion order.
    pub ids: Vec<i64>,
    /// Interleaved horizontal and vertical f32 offsets.
    pub offsets: Vec<f32>,
}

use std::collections::{BTreeMap, BTreeSet};

type Point = [f32; 2];

#[derive(Clone, Copy)]
struct Rect {
    position: Point,
    size: Point,
}

impl Rect {
    fn translated(self, point: Point) -> Self {
        Self {
            position: add(point, self.position),
            size: self.size,
        }
    }
    fn area(self) -> f64 {
        f64::from(self.size[0] * self.size[1])
    }
    fn intersection_area(self, other: Self) -> f64 {
        let end = add(self.position, self.size);
        let other_end = add(other.position, other.size);
        if self.position[0] >= other_end[0]
            || end[0] <= other.position[0]
            || self.position[1] >= other_end[1]
            || end[1] <= other.position[1]
        {
            return 0.0;
        }
        let width = end[0].min(other_end[0]) - self.position[0].max(other.position[0]);
        let height = end[1].min(other_end[1]) - self.position[1].max(other.position[1]);
        f64::from(width * height)
    }
}

const fn add(a: Point, b: Point) -> Point {
    [a[0] + b[0], a[1] + b[1]]
}

struct Front {
    target: Point,
    rect: Rect,
}

struct Spatial {
    size: f64,
    buckets: BTreeMap<(i64, i64), Vec<usize>>,
}

impl Spatial {
    fn cells(&self, rect: Rect) -> impl Iterator<Item = (i64, i64)> {
        let position = add(
            rect.position,
            [rect.size[0].min(0.0), rect.size[1].min(0.0)],
        );
        let end = add(position, [rect.size[0].abs(), rect.size[1].abs()]);
        let first = position.map(|v| (f64::from(v) / self.size).floor() as i64);
        let last = end.map(|v| (f64::from(v) / self.size).floor() as i64);
        (first[1]..=last[1]).flat_map(move |y| (first[0]..=last[0]).map(move |x| (x, y)))
    }
    fn insert(&mut self, rect: Rect, ordinal: usize) {
        let cells: Vec<_> = self.cells(rect).collect();
        for cell in cells {
            self.buckets.entry(cell).or_default().push(ordinal);
        }
    }
    fn query(&self, rect: Rect) -> BTreeSet<usize> {
        let mut found = BTreeSet::new();
        for cell in self.cells(rect) {
            if let Some(ordinals) = self.buckets.get(&cell) {
                found.extend(ordinals);
            }
        }
        found
    }
}

fn covered(rect: Rect, fronts: &[Front], spatial: Option<&Spatial>) -> f64 {
    let area = rect.area();
    if area <= 0.0 {
        return 0.0;
    }
    let mut overlap = 0.0_f64;
    if let Some(index) = spatial {
        for ordinal in index.query(rect) {
            overlap += rect.intersection_area(fronts[ordinal].rect);
        }
    } else {
        for front in fronts {
            overlap += rect.intersection_area(front.rect);
        }
    }
    (overlap / area).min(1.0)
}

fn allowed(point: Point, input: &DeclumpInput<'_>, blocked: &BTreeSet<(i64, i64)>) -> bool {
    let x = ((point[0] - 456.0_f32) / 16.0_f32).round() as i64;
    let y = ((point[1] - 42.0_f32) / 16.0_f32).round() as i64;
    if blocked.contains(&(x, y)) || x < 0 || x >= input.width || y < 0 || y >= input.height {
        return false;
    }
    y.checked_mul(input.width)
        .and_then(|v| v.checked_add(x))
        .and_then(|v| usize::try_from(v).ok())
        .and_then(|v| input.terrain.get(v))
        .is_some_and(|cell| *cell != 4)
}

fn validate(input: &DeclumpInput<'_>) -> Result<(), &'static str> {
    let n = input.row_ids.len();
    if [
        input.visible.len(),
        input.walking.len(),
        input.place_known.len(),
    ]
    .iter()
    .any(|len| *len != n)
        || input.target_ids.len().checked_mul(2) != Some(input.target_xy.len())
        || input.rect_ids.len().checked_mul(4) != Some(input.rect_xywh.len())
        || !input.blocked_xy.len().is_multiple_of(2)
    {
        return Err("declump packed column length mismatch");
    }
    if input
        .target_xy
        .iter()
        .chain(input.rect_xywh)
        .any(|v| !v.is_finite())
    {
        return Err("declump coordinates must be finite");
    }
    if input
        .visible
        .iter()
        .chain(input.walking)
        .chain(input.place_known)
        .any(|v| *v > 1)
    {
        return Err("declump flags must be zero or one");
    }
    Ok(())
}

/// Compute the reference's horizontal relief without modifying simulation positions.
pub fn declump_offsets(input: &DeclumpInput<'_>) -> Result<DeclumpOutput, &'static str> {
    validate(input)?;
    let targets: BTreeMap<i64, Point> = input
        .target_ids
        .iter()
        .copied()
        .zip(input.target_xy.chunks_exact(2).map(|v| [v[0], v[1]]))
        .collect();
    let rects: BTreeMap<i64, Rect> = input
        .rect_ids
        .iter()
        .copied()
        .zip(input.rect_xywh.chunks_exact(4).map(|v| Rect {
            position: [v[0], v[1]],
            size: [v[2], v[3]],
        }))
        .collect();
    let blocked: BTreeSet<_> = input
        .blocked_xy
        .chunks_exact(2)
        .map(|v| (i64::from(v[0]), i64::from(v[1])))
        .collect();
    let mut ordered: Vec<_> = input
        .row_ids
        .iter()
        .copied()
        .enumerate()
        .filter_map(|(row, id)| {
            if input.visible[row] == 0 {
                return None;
            }
            Some((row, id, *targets.get(&id)?, *rects.get(&id)?))
        })
        .collect();
    ordered.sort_by(|a, b| {
        if a.2[1] != b.2[1] {
            b.2[1]
                .partial_cmp(&a.2[1])
                .unwrap_or(std::cmp::Ordering::Equal)
        } else {
            b.1.cmp(&a.1)
        }
    });
    let mut extent = 0.0_f64;
    for values in input.rect_xywh.chunks_exact(4) {
        extent = extent.max(f64::from(values[2]).abs().max(f64::from(values[3]).abs()));
    }
    let mut spatial = input.spatial.then(|| Spatial {
        size: 8.0_f64.max((extent + 8.0).ceil()),
        buckets: BTreeMap::new(),
    });
    let mut fronts: Vec<Front> = Vec::with_capacity(ordered.len());
    let mut result = DeclumpOutput {
        ids: Vec::with_capacity(ordered.len()),
        offsets: Vec::with_capacity(ordered.len() * 2),
    };
    let mut output_slots = BTreeMap::new();
    for (row, id, target, local) in ordered {
        let mut offset = [0.0_f32; 2];
        if input.place_known[row] == 0 && input.walking[row] == 0 {
            if covered(local.translated(target), &fronts, spatial.as_ref()) >= 0.30 {
                let mut nearest = target;
                let mut distance = f64::INFINITY;
                for front in &fronts {
                    let dx = target[0] - front.target[0];
                    let dy = target[1] - front.target[1];
                    let candidate = f64::from(dx * dx + dy * dy);
                    if candidate < distance {
                        distance = candidate;
                        nearest = front.target;
                    }
                }
                let side = if target[0] == nearest[0] {
                    if id.rem_euclid(2) == 0 {
                        1.0_f64
                    } else {
                        -1.0_f64
                    }
                } else if target[0] > nearest[0] {
                    1.0_f64
                } else {
                    -1.0_f64
                };
                let mut best = covered(local.translated(target), &fronts, spatial.as_ref());
                'search: for magnitude in 1..9 {
                    for direction in [side, -side] {
                        let shift = [(direction * f64::from(magnitude)) as f32, 0.0_f32];
                        let shifted = add(target, shift);
                        if !allowed(shifted, input, &blocked) {
                            continue;
                        }
                        let coverage =
                            covered(local.translated(shifted), &fronts, spatial.as_ref());
                        if coverage < best {
                            best = coverage;
                            offset = shift;
                        }
                        if coverage < 0.30 {
                            break 'search;
                        }
                    }
                }
            }
            let slot = *output_slots.entry(id).or_insert_with(|| {
                let slot = result.ids.len();
                result.ids.push(id);
                result.offsets.extend([0.0_f32; 2]);
                slot
            });
            result.offsets[slot * 2..slot * 2 + 2].copy_from_slice(&offset);
        }
        let shifted = add(target, offset);
        let rect = local.translated(shifted);
        if let Some(index) = spatial.as_mut() {
            index.insert(rect, fronts.len());
        }
        fronts.push(Front {
            target: shifted,
            rect,
        });
    }
    Ok(result)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn equal_x_even_id_moves_right_after_front_is_inserted() {
        let terrain = vec![0; 64 * 64];
        for spatial in [false, true] {
            let input = DeclumpInput {
                row_ids: &[2, 3],
                visible: &[1, 1],
                walking: &[0, 0],
                place_known: &[0, 0],
                target_ids: &[2, 3],
                target_xy: &[936.0, 522.0, 936.0, 538.0],
                rect_ids: &[2, 3],
                rect_xywh: &[-8.0, -29.0, 16.0, 30.0, -8.0, -29.0, 16.0, 30.0],
                width: 64,
                height: 64,
                terrain: &terrain,
                blocked_xy: &[],
                spatial,
            };
            let result = declump_offsets(&input).expect("valid input");
            assert_eq!(result.ids, [3, 2]);
            assert_eq!(result.offsets, [0.0, 0.0, 6.0, 0.0]);
        }
    }
    #[test]
    fn moving_front_occludes_but_has_no_result_key_and_water_changes_side() {
        let mut terrain = vec![0; 64 * 64];
        terrain[30 * 64 + 31] = 4;
        for spatial in [false, true] {
            let result = declump_offsets(&DeclumpInput {
                row_ids: &[2, 3],
                visible: &[1, 1],
                walking: &[0, 1],
                place_known: &[0, 0],
                target_ids: &[2, 3],
                target_xy: &[943.0, 522.0, 943.0, 538.0],
                rect_ids: &[2, 3],
                rect_xywh: &[-8.0, -29.0, 16.0, 30.0, -8.0, -29.0, 16.0, 30.0],
                width: 64,
                height: 64,
                terrain: &terrain,
                blocked_xy: &[],
                spatial,
            })
            .expect("valid input");
            assert_eq!(result.ids, [2]);
            assert_eq!(result.offsets, [-6.0, 0.0]);
        }
    }
    #[test]
    fn observed_godot_46_numeric_bits_match() {
        // Godot official 4.6, double=false; godot-numeric-clean.json.
        let a = Rect {
            position: [f32::from_bits(1203982352), f32::from_bits(1067320914)],
            size: [f32::from_bits(1099496663), f32::from_bits(1103095479)],
        };
        let b = Rect {
            position: [f32::from_bits(1203982464), f32::from_bits(1073741824)],
            size: [f32::from_bits(1101004800), f32::from_bits(1106247680)],
        };
        assert_eq!(a.area().to_bits(), 0x4079_ac06_6000_0000);
        assert_eq!(a.intersection_area(b).to_bits(), 0x4077_95c7_0000_0000);
        let touching = Rect {
            position: [0.0, 0.0],
            size: [2.0, 2.0],
        };
        assert_eq!(
            touching
                .intersection_area(touching.translated([2.0, 0.0]))
                .to_bits(),
            0
        );
        let negative_zero = Rect {
            position: [-0.0, -0.0],
            size: [-0.0, 2.0],
        };
        assert_eq!(negative_zero.area().to_bits(), 0x8000_0000_0000_0000);
        for (input, output) in [
            (3223322624, 3225419776),
            (3217031168, 3221225472),
            (3204448256, 3212836864),
            (2147483648, 2147483648),
            (0, 0),
            (1056964608, 1065353216),
            (1069547520, 1073741824),
            (1075838976, 1077936128),
        ] {
            assert_eq!(f32::from_bits(input).round().to_bits(), output);
        }
        let negative_size = Rect {
            position: [4.0, 5.0],
            size: [-3.0, -2.0],
        };
        let index = Spatial {
            size: 2.0,
            buckets: BTreeMap::new(),
        };
        assert_eq!(
            index.cells(negative_size).collect::<Vec<_>>(),
            [(0, 1), (1, 1), (2, 1), (0, 2), (1, 2), (2, 2)]
        );
    }
}

/// Real walker-call inputs; geometry targets are the already rounded positions.
pub struct WalkerInput<'a> {
    /// Existing packed geometry/flags layout, with target columns holding positions.
    pub geometry: DeclumpInput<'a>,
    /// Interleaved untransformed walking direction components, one pair per row.
    pub direction_xy: &'a [f32],
    /// Canvas columns in x.x, x.y, y.x, y.y, origin.x, origin.y order.
    pub canvas: &'a [f32],
}

#[derive(Clone, Copy)]
struct WalkerCanvas {
    x: Point,
    y: Point,
    origin: Point,
}

impl WalkerCanvas {
    const fn basis(self, point: Point) -> Point {
        [
            self.x[0] * point[0] + self.y[0] * point[1],
            self.x[1] * point[0] + self.y[1] * point[1],
        ]
    }

    fn affine_inverse(self) -> Result<Self, &'static str> {
        let determinant = self.x[0] * self.y[1] - self.x[1] * self.y[0];
        if determinant == 0.0 || !determinant.is_finite() {
            return Err("walker canvas determinant must be finite and nonzero");
        }
        let reciprocal = 1.0_f32 / determinant;
        // Godot swaps the diagonal, then component-multiplies the two columns.
        let mut inverse = Self {
            x: [self.y[1] * reciprocal, self.x[1] * -reciprocal],
            y: [self.y[0] * -reciprocal, self.x[0] * reciprocal],
            origin: [0.0; 2],
        };
        inverse.origin = inverse.basis([-self.origin[0], -self.origin[1]]);
        Ok(inverse)
    }
}

const fn walker_dot(a: Point, b: Point) -> f32 {
    a[0] * b[0] + a[1] * b[1]
}

fn walker_normalized(mut point: Point) -> Point {
    let squared = walker_dot(point, point);
    if squared != 0.0 {
        let length = squared.sqrt();
        point[0] /= length;
        point[1] /= length;
    }
    point
}

// Godot's ID comparator has no secondary key. Its introsort is deliberately
// reproduced for duplicate IDs rather than substituting a stable Rust sort.
struct WalkerOrder<'a> {
    ids: &'a [i64],
}

impl WalkerOrder<'_> {
    fn before(&self, a: usize, b: usize) -> bool {
        self.ids[a] < self.ids[b]
    }

    fn median(&self, rows: &[usize]) -> usize {
        let (a, b, c) = (0, rows.len() / 2, rows.len() - 1);
        if self.before(rows[a], rows[b]) {
            if self.before(rows[b], rows[c]) {
                b
            } else if self.before(rows[a], rows[c]) {
                c
            } else {
                a
            }
        } else if self.before(rows[a], rows[c]) {
            a
        } else if self.before(rows[b], rows[c]) {
            c
        } else {
            b
        }
    }

    fn partition(&self, rows: &mut [usize]) -> usize {
        let original_pivot = self.median(rows);
        let mut pivot = original_pivot;
        let (mut first, mut last) = (0, rows.len());
        loop {
            while first != original_pivot && self.before(rows[first], rows[pivot]) {
                first += 1;
            }
            last -= 1;
            while last != original_pivot && self.before(rows[pivot], rows[last]) {
                last -= 1;
            }
            if first >= last {
                return first;
            }
            if pivot == first {
                pivot = last;
            } else if pivot == last {
                pivot = first;
            }
            rows.swap(first, last);
            first += 1;
        }
    }

    fn adjust_heap(&self, rows: &mut [usize], mut hole: usize, value: usize) {
        let top = hole;
        let mut child = 2 * hole + 2;
        while child < rows.len() {
            if self.before(rows[child], rows[child - 1]) {
                child -= 1;
            }
            rows[hole] = rows[child];
            hole = child;
            child = 2 * (child + 1);
        }
        if child == rows.len() {
            rows[hole] = rows[child - 1];
            hole = child - 1;
        }
        while hole > top {
            let parent = (hole - 1) / 2;
            if !self.before(rows[parent], value) {
                break;
            }
            rows[hole] = rows[parent];
            hole = parent;
        }
        rows[hole] = value;
    }

    fn heap_sort(&self, rows: &mut [usize]) {
        if rows.len() < 2 {
            return;
        }
        for parent in (0..=(rows.len() - 2) / 2).rev() {
            let value = rows[parent];
            self.adjust_heap(rows, parent, value);
        }
        for last in (1..rows.len()).rev() {
            let value = rows[last];
            rows[last] = rows[0];
            self.adjust_heap(&mut rows[..last], 0, value);
        }
    }

    fn introsort(&self, mut rows: &mut [usize], mut depth: u32) {
        while rows.len() > 16 {
            if depth == 0 {
                self.heap_sort(rows);
                return;
            }
            depth -= 1;
            let cut = self.partition(rows);
            let (left, right) = rows.split_at_mut(cut);
            self.introsort(right, depth);
            rows = left;
        }
    }

    fn sort(&self, rows: &mut [usize]) {
        if rows.is_empty() {
            return;
        }
        let depth = rows.len().ilog2() * 2;
        self.introsort(rows, depth);
        // Equivalent to the final guarded prefix plus unguarded insertion pass:
        // the sorted prefix is the sentinel, so the guard cannot change swaps.
        for index in 1..rows.len() {
            let value = rows[index];
            let mut hole = index;
            while hole > 0 && self.before(value, rows[hole - 1]) {
                rows[hole] = rows[hole - 1];
                hole -= 1;
            }
            rows[hole] = value;
        }
    }
}

struct WalkerSpatial {
    index: Spatial,
    rectangles: Vec<Rect>,
    ids: Vec<i64>,
    ordinals: BTreeMap<i64, Vec<usize>>,
}

impl WalkerSpatial {
    fn move_agent(&mut self, id: i64, rect: Rect) {
        if let Some(ordinals) = self.ordinals.get(&id) {
            for &ordinal in ordinals {
                let cells: Vec<_> = self.index.cells(self.rectangles[ordinal]).collect();
                for cell in cells {
                    if let Some(bucket) = self.index.buckets.get_mut(&cell) {
                        // Each ordinal is present once per cell, as in Array.erase.
                        if let Some(slot) = bucket.iter().position(|&value| value == ordinal) {
                            bucket.remove(slot);
                        }
                        if bucket.is_empty() {
                            self.index.buckets.remove(&cell);
                        }
                    }
                }
                self.rectangles[ordinal] = rect;
                self.index.insert(rect, ordinal);
            }
        }
    }

    fn overlap(&self, rect: Rect, id: i64) -> f64 {
        let mut area = 0.0_f64;
        for ordinal in self.index.query(rect) {
            if self.ids[ordinal] != id {
                area += rect.intersection_area(self.rectangles[ordinal]);
            }
        }
        area
    }
}

struct WalkerOverlap<'a> {
    spatial: Option<&'a WalkerSpatial>,
    others: &'a [Rect],
    id: i64,
}

impl WalkerOverlap<'_> {
    fn area(&self, rect: Rect) -> f64 {
        if let Some(spatial) = self.spatial {
            return spatial.overlap(rect, self.id);
        }
        let mut area = 0.0_f64;
        for &other in self.others {
            area += rect.intersection_area(other);
        }
        area
    }
}

/// Reproduce screen-pixel walker relief without changing simulation positions.
///
/// Errors reject malformed packed columns, nonfinite geometry, or a singular canvas.
pub fn walker_offsets(input: &WalkerInput<'_>) -> Result<DeclumpOutput, &'static str> {
    let geometry = &input.geometry;
    validate(geometry)?;
    if geometry.row_ids.len().checked_mul(2) != Some(input.direction_xy.len())
        || input.canvas.len() != 6
    {
        return Err("walker packed column length mismatch");
    }
    if input
        .direction_xy
        .iter()
        .chain(input.canvas)
        .any(|v| !v.is_finite())
    {
        return Err("walker direction and canvas must be finite");
    }
    let canvas = WalkerCanvas {
        x: [input.canvas[0], input.canvas[1]],
        y: [input.canvas[2], input.canvas[3]],
        origin: [input.canvas[4], input.canvas[5]],
    };
    let inverse = canvas.affine_inverse()?;
    let positions: BTreeMap<_, _> = geometry
        .target_ids
        .iter()
        .copied()
        .zip(geometry.target_xy.chunks_exact(2).map(|v| [v[0], v[1]]))
        .collect();
    let rects: BTreeMap<_, _> = geometry
        .rect_ids
        .iter()
        .copied()
        .zip(geometry.rect_xywh.chunks_exact(4).map(|v| Rect {
            position: [v[0], v[1]],
            size: [v[2], v[3]],
        }))
        .collect();
    let blocked: BTreeSet<_> = geometry
        .blocked_xy
        .chunks_exact(2)
        .map(|v| (i64::from(v[0]), i64::from(v[1])))
        .collect();
    let mut ordered: Vec<_> = geometry
        .row_ids
        .iter()
        .enumerate()
        .filter_map(|(row, id)| {
            (geometry.visible[row] != 0 && positions.contains_key(id) && rects.contains_key(id))
                .then_some(row)
        })
        .collect();
    WalkerOrder {
        ids: geometry.row_ids,
    }
    .sort(&mut ordered);
    let mut extent = 0.0_f64;
    for rect in rects.values() {
        extent = extent.max(
            f64::from(rect.size[0])
                .abs()
                .max(f64::from(rect.size[1]).abs()),
        );
    }
    let mut spatial = geometry.spatial.then(|| {
        let mut spatial = WalkerSpatial {
            index: Spatial {
                size: 8.0_f64.max((extent + 8.0).ceil()),
                buckets: BTreeMap::new(),
            },
            rectangles: Vec::with_capacity(ordered.len()),
            ids: Vec::with_capacity(ordered.len()),
            ordinals: BTreeMap::new(),
        };
        for &row in &ordered {
            let id = geometry.row_ids[row];
            let rect = rects[&id].translated(positions[&id]);
            let ordinal = spatial.rectangles.len();
            spatial.rectangles.push(rect);
            spatial.ids.push(id);
            spatial.ordinals.entry(id).or_default().push(ordinal);
            spatial.index.insert(rect, ordinal);
        }
        spatial
    });
    let mut result = DeclumpOutput {
        ids: Vec::with_capacity(ordered.len()),
        offsets: Vec::with_capacity(ordered.len() * 2),
    };
    let mut slots = BTreeMap::new();
    let mut others = Vec::with_capacity(ordered.len());
    for &row in &ordered {
        if geometry.place_known[row] != 0 || geometry.walking[row] == 0 {
            continue;
        }
        let id = geometry.row_ids[row];
        let position = positions[&id];
        let direction = walker_normalized(
            canvas.basis([input.direction_xy[row * 2], input.direction_xy[row * 2 + 1]]),
        );
        if direction[0].abs() < 0.00001_f32 && direction[1].abs() < 0.00001_f32 {
            continue;
        }
        let local = rects[&id];
        others.clear();
        if spatial.is_none() {
            for &other in &ordered {
                let other_id = geometry.row_ids[other];
                if other_id != id {
                    let prior = slots.get(&other_id).map_or([0.0; 2], |&slot: &usize| {
                        [result.offsets[slot * 2], result.offsets[slot * 2 + 1]]
                    });
                    others.push(rects[&other_id].translated(add(positions[&other_id], prior)));
                }
            }
        }
        let overlap = WalkerOverlap {
            spatial: spatial.as_ref(),
            others: &others,
            id,
        };
        let mut best = overlap.area(local.translated(position));
        if best <= 0.0 {
            continue;
        }
        let side = if id.rem_euclid(2) == 0 {
            1.0_f32
        } else {
            -1.0_f32
        };
        let perpendicular = [-direction[1], direction[0]];
        let mut offset = [0.0_f32; 2];
        for magnitude in [1.0_f32, 2.0, 3.0] {
            for sign in [side, -side] {
                let screen = perpendicular.map(|value| ((value * magnitude) * sign).round());
                if f64::from(walker_dot(screen, screen).sqrt()) > 3.0
                    || f64::from(walker_dot(screen, direction)).abs() > 0.001
                {
                    continue;
                }
                let shift = inverse.basis(screen);
                if !allowed(add(position, shift), geometry, &blocked) {
                    continue;
                }
                let candidate = overlap.area(local.translated(add(position, shift)));
                if candidate < best - 0.00001 {
                    best = candidate;
                    offset = shift;
                }
            }
        }
        if let Some(&slot) = slots.get(&id) {
            result.offsets[slot * 2] = offset[0];
            result.offsets[slot * 2 + 1] = offset[1];
        } else {
            slots.insert(id, result.ids.len());
            result.ids.push(id);
            result.offsets.extend(offset);
        }
        if let Some(index) = spatial.as_mut() {
            index.move_agent(id, local.translated(add(position, offset)));
        }
    }
    Ok(result)
}

#[cfg(test)]
mod walker_tests {
    use super::*;

    #[test]
    fn walker_moves_only_unplaced_walker_and_preserves_empty_vs_zero() {
        let terrain = vec![0; 64 * 64];
        for spatial in [false, true] {
            let mut input = WalkerInput {
                geometry: DeclumpInput {
                    row_ids: &[3, 2],
                    visible: &[1, 1],
                    walking: &[0, 1],
                    place_known: &[0, 0],
                    target_ids: &[2, 3],
                    target_xy: &[936.0, 522.0, 936.0, 522.0],
                    rect_ids: &[2, 3],
                    rect_xywh: &[-8.0, -29.0, 16.0, 30.0, -8.0, -29.0, 16.0, 30.0],
                    width: 64,
                    height: 64,
                    terrain: &terrain,
                    blocked_xy: &[],
                    spatial,
                },
                direction_xy: &[0.0, 0.0, 1.0, 0.0],
                canvas: &[1.0, 0.0, 0.0, 1.0, 0.0, 0.0],
            };
            let output = walker_offsets(&input).expect("valid walker input");
            assert_eq!(output.ids, [2]);
            assert_eq!(output.offsets, [0.0, 3.0]);
            input.geometry.blocked_xy = &[30, 30];
            let blocked = walker_offsets(&input).expect("blocked but valid");
            assert_eq!(blocked.ids, [2]);
            assert_eq!(blocked.offsets, [0.0, 0.0]);
            input.geometry.place_known = &[0, 1];
            assert!(walker_offsets(&input)
                .expect("placed walker")
                .ids
                .is_empty());
        }
    }

    #[test]
    fn inverse_preserves_component_multiplication_order_and_zero_signs() {
        let inverse = WalkerCanvas {
            x: [2.0, 0.0],
            y: [0.0, 4.0],
            origin: [10.0, -20.0],
        }
        .affine_inverse()
        .expect("invertible");
        assert_eq!(
            inverse.x.map(f32::to_bits),
            [0.5_f32.to_bits(), (-0.0_f32).to_bits()]
        );
        assert_eq!(
            inverse.y.map(f32::to_bits),
            [(-0.0_f32).to_bits(), 0.25_f32.to_bits()]
        );
        assert_eq!(inverse.origin, [-5.0, 5.0]);
        assert_eq!(
            walker_normalized([-0.0, 0.0]).map(f32::to_bits),
            [(-0.0_f32).to_bits(), 0]
        );
    }

    #[test]
    fn duplicate_ids_move_and_exclude_every_spatial_ordinal() {
        let rect = Rect {
            position: [0.0, 0.0],
            size: [10.0, 10.0],
        };
        let mut spatial = WalkerSpatial {
            index: Spatial {
                size: 18.0,
                buckets: BTreeMap::new(),
            },
            rectangles: vec![rect, rect],
            ids: vec![2, 2],
            ordinals: BTreeMap::from([(2, vec![0, 1])]),
        };
        spatial.index.insert(rect, 0);
        spatial.index.insert(rect, 1);
        assert_eq!(spatial.overlap(rect, 2), 0.0);
        assert_eq!(spatial.overlap(rect, 3), 200.0);
        let shifted = rect.translated([50.0, 0.0]);
        spatial.move_agent(2, shifted);
        assert_eq!(spatial.overlap(rect, 3), 0.0);
        assert_eq!(spatial.overlap(shifted, 3), 200.0);
    }
}
