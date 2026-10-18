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
    struct Reader<'a> {
        bytes: &'a [u8],
    }
    impl<'a> Reader<'a> {
        fn take(&mut self, n: usize) -> &'a [u8] {
            assert!(n <= self.bytes.len(), "truncated declump fixture");
            let (head, tail) = self.bytes.split_at(n);
            self.bytes = tail;
            head
        }
        fn u8(&mut self) -> u8 {
            self.take(1)[0]
        }
        fn u32(&mut self) -> u32 {
            u32::from_le_bytes(self.take(4).try_into().expect("four bytes"))
        }
        fn i32(&mut self) -> i32 {
            i32::from_le_bytes(self.take(4).try_into().expect("four bytes"))
        }
        fn i64(&mut self) -> i64 {
            i64::from_le_bytes(self.take(8).try_into().expect("eight bytes"))
        }
        fn u64(&mut self) -> u64 {
            u64::from_le_bytes(self.take(8).try_into().expect("eight bytes"))
        }
        fn f32(&mut self) -> f32 {
            f32::from_bits(self.u32())
        }
    }

    #[test]
    fn real_renderer_fixture_is_bit_exact_in_both_modes() {
        let root = option_env!("CARGO_MANIFEST_DIR").unwrap_or("rust/crates/sim-bridge");
        let path = std::path::Path::new(root).join("tests/fixtures/art_p12/declump_v1.bin");
        let bytes = std::fs::read(&path).unwrap_or_else(|error| {
            panic!("required real-call fixture {}: {error}", path.display())
        });
        let mut file = Reader { bytes: &bytes };
        assert_eq!(file.take(8), b"P12DCL01");
        assert_eq!(file.u32(), 1);
        let count = file.u32();
        assert!(
            count >= 4000,
            "requires both modes for at least 2000 real calls"
        );
        let mut calls = BTreeMap::new();
        for record in 0..count {
            let length = file.u32() as usize;
            let mut data = Reader {
                bytes: file.take(length),
            };
            let world = data.u8();
            let running = data.u8();
            let spatial = data.u8();
            assert!(world < 5 && running < 2 && spatial < 2);
            assert_eq!(data.u8(), 0);
            let seed = data.i64();
            let tick = data.i64();
            let frame = data.u64();
            let mask = calls
                .entry((world, running, seed, tick, frame))
                .or_insert(0_u8);
            assert_eq!(*mask & (1 << spatial), 0, "duplicate provenance/mode");
            *mask |= 1 << spatial;
            let width = data.i64();
            let height = data.i64();
            let mut row_ids = Vec::new();
            let mut visible = Vec::new();
            let mut walking = Vec::new();
            let mut place_known = Vec::new();
            for _ in 0..data.u32() {
                row_ids.push(data.i64());
                visible.push(data.u8());
                walking.push(data.u8());
                place_known.push(data.u8());
                assert_eq!(data.u8(), 0);
            }
            let mut target_ids = Vec::new();
            let mut target_xy = Vec::new();
            for _ in 0..data.u32() {
                target_ids.push(data.i64());
                target_xy.extend([data.f32(), data.f32()]);
            }
            let mut rect_ids = Vec::new();
            let mut rect_xywh = Vec::new();
            for _ in 0..data.u32() {
                rect_ids.push(data.i64());
                rect_xywh.extend([data.f32(), data.f32(), data.f32(), data.f32()]);
            }
            let terrain_length = data.u32() as usize;
            let terrain = data.take(terrain_length);
            let mut blocked_xy = Vec::new();
            for _ in 0..data.u32() {
                blocked_xy.extend([data.i32(), data.i32()]);
            }
            let mut expected_ids = Vec::new();
            let mut expected_bits = Vec::new();
            for _ in 0..data.u32() {
                expected_ids.push(data.i64());
                expected_bits.extend([data.u32(), data.u32()]);
            }
            assert!(data.bytes.is_empty(), "trailing record bytes: {record}");
            let result = declump_offsets(&DeclumpInput {
                row_ids: &row_ids,
                visible: &visible,
                walking: &walking,
                place_known: &place_known,
                target_ids: &target_ids,
                target_xy: &target_xy,
                rect_ids: &rect_ids,
                rect_xywh: &rect_xywh,
                width,
                height,
                terrain,
                blocked_xy: &blocked_xy,
                spatial: spatial != 0,
            })
            .expect("valid recorded input");
            assert_eq!(result.ids, expected_ids, "ids: record {record}");
            assert_eq!(
                result
                    .offsets
                    .iter()
                    .map(|v| v.to_bits())
                    .collect::<Vec<_>>(),
                expected_bits,
                "offset bits: record {record}"
            );
        }
        assert!(file.bytes.is_empty(), "trailing fixture bytes");
        assert!(calls.len() >= 2000);
        let mut groups = [[0_usize; 2]; 5];
        for ((world, running, _, _, _), modes) in calls {
            assert_eq!(modes, 3, "missing mode for actual renderer call");
            groups[usize::from(world)][usize::from(running)] += 1;
        }
        assert!(
            groups.iter().flatten().all(|count| *count >= 200),
            "requires 200 real calls per world/condition: {groups:?}"
        );
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
