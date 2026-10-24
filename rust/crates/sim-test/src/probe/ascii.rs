//! The text renderer (M1.5 goal 8.1, contract clause C2 "view independence"):
//! the colony world as a glyph grid, one character per tile, rows top to bottom.
//!
//! It reads only the pure snapshot collectors the Godot bridge's getters call
//! (`sim_bridge::ffi::collect_*`): terrain (`get_terrain_snapshot`), the order
//! overlay (`get_colony_overlay`: chop marks, zones, ground items, sites),
//! buildings (`get_building_snapshot`), trees (`get_trees`), the agent
//! snapshot with its job/carry columns (`get_agent_snapshot`), the snapshot's
//! M-ART2 appearance columns (`collect_appearance`: `--people-json`'s
//! `appearance`, [`people_looks`]) and the clock (`get_time_snapshot`). It never reads systems or engine state the bridge
//! does not expose. Stumps have no collector: like `tree_layer.gd`, the
//! [`StumpTracker`] derives them from successive `get_trees` reads (a tree
//! that disappears leaves a stump for [`STUMP_TICKS`]).
//!
//! Layers, later wins: terrain < marks/zones < items < buildings/sites <
//! trees < people. The glyph table, the carrying rule and the sleep test are
//! documented on [`legend`] and in `docs/v8/snapshot_schema.md`
//! ("Text renderer glyphs"). `worldsim_probe --scene colony --ascii` drives it
//! ([`AsciiArgs`], [`run`]).
use super::colony::{load_script, resolve, ScriptEntry};
use sim_bridge::ffi::{
    collect_agent_jobs, collect_agent_snapshot, collect_appearance, collect_building_snapshot,
    collect_colony_overlay, collect_terrain_snapshot, collect_time_snapshot, collect_trees,
    AgentSnapshotRow,
};
use sim_engine::SimEngine;
use std::collections::{BTreeMap, BTreeSet};
use std::path::PathBuf;

/// A tile `(x, y)`.
pub type Tile = (u32, u32);
/// A rectangle `(x, y, w, h)` in tiles.
pub type Rect = (u32, u32, u32, u32);

/// A felled tree's stump stays this many ticks (one game day; `tree_layer.gd` `STUMP_TICKS`).
pub const STUMP_TICKS: u64 = 1_440;
/// A ground log stack of at least this many logs draws `L`, a smaller one `l`.
pub const FULL_STACK: u16 = 9;
/// A finished hut covers this many tiles around its centre row (`agent_layer.gd` / `building_layer.gd` 3 × 3).
pub const HUT_HALF: u32 = 1;
/// Agent snapshot `states` code of a consuming agent (`AgentSnapshotRow::state_tag`).
pub const STATE_CONSUMING: u8 = 3;
/// Agent snapshot `target_kinds` code of sleep (`AgentSnapshotRow::target_kind`).
pub const TARGET_SLEEP: u8 = 3;
/// Agent snapshot `life_stages` code of a child.
pub const STAGE_CHILD: u8 = 1;

/// Terrain glyph of a terrain snapshot code (`TerrainType as u8`, declaration order).
pub fn terrain_glyph(code: u8) -> u8 {
    match code {
        0 => b'.', // Plain (grass)
        1 => b',', // Forest floor
        2 => b'#', // Mountain
        3 => b'^', // Hill
        4 => b'~', // River (fresh water)
        5 => b'_', // Coast
        6 => b':', // Desert (bare dirt, sand)
        7 => b'-', // Tundra
        8 => b'"', // Swamp
        9 => b'o', // Cave
        _ => b'?', // unknown code
    }
}

/// The legend (`--ascii-legend`): every glyph, the layer order and the rules.
pub fn legend() -> String {
    [
        "glyph legend (one character per tile; rows top to bottom; later layers win)",
        "layer 1 terrain (get_terrain_snapshot `terrain`): . plain  , forest floor  # mountain  ^ hill  ~ river",
        "                 _ coast  : desert  - tundra  \" swamp  o cave  ? unknown code",
        "layer 2 marks/zones (get_colony_overlay): = stockpile zone cell  x chop mark",
        "layer 3 items (get_colony_overlay item_*): l log stack (< 9 logs)  L log stack (>= 9)",
        "layer 4 buildings/sites (get_colony_overlay site_*, get_building_snapshot):",
        "        b blueprint site footprint (not finished)  * hearth  h shelter  H hut (3x3 around its centre)",
        "layer 5 trees (get_trees): T standing tree  x standing tree with a chop mark  t stump (felled < 1 day ago)",
        "layer 6 people (get_agent_snapshot): @ adult  a child  z asleep (states 3 and target_kinds 3)",
        "        2..9 a tile shared by that many people (9 = nine or more)",
        "        + carried load (carry_counts > 0, awake) on the cell right of its carrier, unless that cell",
        "          holds a person or lies past the map's right edge",
    ]
    .join("\n")
}

/// Stumps from successive `get_trees` reads, as `tree_layer.gd` keeps them: a
/// tree in the previous set and missing from the new one leaves a stump at
/// the read's tick; a tree that stands again removes it; a stump expires
/// [`STUMP_TICKS`] after it appeared. The first read records no stump.
#[derive(Debug, Clone, Default)]
pub struct StumpTracker {
    version: Option<u64>,
    trees: BTreeSet<Tile>,
    stumps: BTreeMap<Tile, u64>,
}

impl StumpTracker {
    /// An empty tracker (no read yet).
    pub fn new() -> Self {
        Self::default()
    }

    /// Read `get_trees` (only when its version changed) and the clock; update the stumps.
    pub fn observe(&mut self, engine: &SimEngine) {
        let tick = collect_time_snapshot(engine).tick;
        let (version, tiles) = collect_trees(&engine.resources);
        if self.version != Some(version) {
            let now: BTreeSet<Tile> = tiles.into_iter().collect();
            if self.version.is_some() {
                for gone in self.trees.difference(&now) {
                    self.stumps.insert(*gone, tick);
                }
            }
            for tile in &now {
                self.stumps.remove(tile);
            }
            self.trees = now;
            self.version = Some(version);
        }
        self.stumps
            .retain(|_, at| tick >= *at && tick - *at < STUMP_TICKS);
    }

    /// The stump tiles, in `(x, y)` order.
    pub fn tiles(&self) -> Vec<Tile> {
        self.stumps.keys().copied().collect()
    }
}

/// Whether a snapshot row sleeps: consuming (`states` 3) with target sleep (`target_kinds` 3).
pub fn is_asleep(row: &AgentSnapshotRow) -> bool {
    row.state_tag == STATE_CONSUMING && row.target_kind == TARGET_SLEEP
}

/// The glyph of one person row.
pub fn person_glyph(row: &AgentSnapshotRow) -> u8 {
    if is_asleep(row) {
        b'z'
    } else if row.life_stage == STAGE_CHILD {
        b'a'
    } else {
        b'@'
    }
}

/// Every snapshot row's tile, sorted, one entry per person (a shared tile
/// repeats): the tiles where [`render`] draws a person glyph (or a head-count
/// digit for a shared tile).
pub fn people_tiles(engine: &SimEngine) -> Vec<Tile> {
    let mut tiles: Vec<Tile> = collect_agent_snapshot(&engine.world)
        .iter()
        .map(|row| (row.x, row.y))
        .collect();
    tiles.sort_unstable();
    tiles
}

/// One person as the text renderer lists it in `--people-json`'s `appearance` (M-ART2, C2 extended to
/// appearance): AgentId, tile, sex (0 male, 1 female), look index and face variant — the columns
/// `get_agent_snapshot` sends as `agent_ids`, `xs`, `ys`, `sexes`, `look_ids`, `variants`, from the same
/// collectors (`collect_agent_snapshot`, `collect_appearance`).
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct PersonLook {
    /// AgentId.
    pub agent_id: u64,
    /// Tile x.
    pub x: u32,
    /// Tile y.
    pub y: u32,
    /// 0 male, 1 female.
    pub sex: u8,
    /// Look index.
    pub look: u32,
    /// Face variant.
    pub variant: u8,
}

/// Every person's look in AgentId order; `Err` when the two collectors' rows do not line up (a bridge bug).
pub fn people_looks(engine: &SimEngine) -> Result<Vec<PersonLook>, String> {
    let rows = collect_agent_snapshot(&engine.world);
    let looks = collect_appearance(&engine.world);
    if rows.len() != looks.len() {
        return Err(format!(
            "appearance rows {} != snapshot rows {}",
            looks.len(),
            rows.len()
        ));
    }
    let mut out = Vec::with_capacity(rows.len());
    for (row, look) in rows.iter().zip(&looks) {
        if row.agent_id != look.agent_id {
            return Err(format!(
                "appearance row of agent {} beside snapshot row of agent {}",
                look.agent_id, row.agent_id
            ));
        }
        out.push(PersonLook {
            agent_id: row.agent_id,
            x: row.x,
            y: row.y,
            sex: look.sex,
            look: look.look,
            variant: look.variant,
        });
    }
    out.sort_unstable();
    Ok(out)
}

/// The full-map grid (row-major bytes) and its width/height.
fn paint(engine: &SimEngine, stumps: &[Tile]) -> (Vec<u8>, u32, u32) {
    let terrain = collect_terrain_snapshot(&engine.resources);
    let (w, h) = (terrain.width, terrain.height);
    let mut grid: Vec<u8> = (0..(w as usize) * (h as usize))
        .map(|i| terrain.terrain.get(i).map_or(b'?', |c| terrain_glyph(*c)))
        .collect();
    let put = |grid: &mut Vec<u8>, x: u32, y: u32, glyph: u8| {
        if x < w && y < h {
            grid[(y as usize) * (w as usize) + x as usize] = glyph;
        }
    };
    let overlay = collect_colony_overlay(&engine.world, &engine.resources);
    // Layer 2: zones, then chop marks.
    for (_, (x, y)) in &overlay.zones {
        put(&mut grid, *x, *y, b'=');
    }
    for (x, y) in &overlay.designations {
        put(&mut grid, *x, *y, b'x');
    }
    // Layer 3: ground items (every item kind today is a log, code 0).
    for ((x, y), _kind, count) in &overlay.items {
        let glyph = if *count >= FULL_STACK { b'L' } else { b'l' };
        put(&mut grid, *x, *y, glyph);
    }
    // Layer 4: blueprint sites, then registered buildings.
    for site in &overlay.sites {
        for y in site.y..site.y.saturating_add(site.h) {
            for x in site.x..site.x.saturating_add(site.w) {
                put(&mut grid, x, y, b'b');
            }
        }
    }
    for row in collect_building_snapshot(&engine.resources) {
        match row.kind {
            0 => put(&mut grid, row.x, row.y, b'*'),
            1 => put(&mut grid, row.x, row.y, b'h'),
            3 => {
                for y in row.y.saturating_sub(HUT_HALF)..=row.y.saturating_add(HUT_HALF) {
                    for x in row.x.saturating_sub(HUT_HALF)..=row.x.saturating_add(HUT_HALF) {
                        put(&mut grid, x, y, b'H');
                    }
                }
            }
            // Kind 2 (the V7 stockpile marker) is not drawn, as in building_layer.gd.
            _ => {}
        }
    }
    // Layer 5: stumps, then standing trees (a marked tree draws `x`).
    for (x, y) in stumps {
        put(&mut grid, *x, *y, b't');
    }
    let marked: BTreeSet<Tile> = overlay.designations.iter().copied().collect();
    for tile in collect_trees(&engine.resources).1 {
        let glyph = if marked.contains(&tile) { b'x' } else { b'T' };
        put(&mut grid, tile.0, tile.1, glyph);
    }
    // Layer 6: people. A tile with one person shows its glyph; a tile shared by
    // k >= 2 people shows the digit k ('9' = nine or more). Then each awake
    // carrier's `+` goes on the cell to its right unless a person is there.
    let rows = collect_agent_snapshot(&engine.world);
    let jobs = collect_agent_jobs(&engine.world);
    let mut by_tile: BTreeMap<Tile, Vec<usize>> = BTreeMap::new();
    for (i, row) in rows.iter().enumerate() {
        by_tile.entry((row.x, row.y)).or_default().push(i);
    }
    for (tile, group) in &by_tile {
        let glyph = match group.as_slice() {
            [one] => person_glyph(&rows[*one]),
            many => b'0' + many.len().min(9) as u8,
        };
        put(&mut grid, tile.0, tile.1, glyph);
    }
    for (i, row) in rows.iter().enumerate() {
        let carrying = jobs.get(i).is_some_and(|j| j.2 > 0) && !is_asleep(row);
        let next = (row.x.saturating_add(1), row.y);
        if carrying && next.0 < w && !by_tile.contains_key(&next) {
            put(&mut grid, next.0, next.1, b'+');
        }
    }
    (grid, w, h)
}

/// Render the grid with stumps from `stumps` (`None`: no stumps; see
/// [`StumpTracker`]), cropped to `rect` (`(x, y, w, h)`, clipped to the map;
/// `None` = the whole map). One line per row, each ending in `\n`.
pub fn render_with(
    engine: &SimEngine,
    rect: Option<Rect>,
    stumps: Option<&StumpTracker>,
) -> String {
    let stump_tiles = stumps.map(StumpTracker::tiles).unwrap_or_default();
    let (grid, w, h) = paint(engine, &stump_tiles);
    let (x0, y0, rw, rh) = rect.unwrap_or((0, 0, w, h));
    let (x0, y0) = (x0.min(w), y0.min(h));
    let (x1, y1) = (x0.saturating_add(rw).min(w), y0.saturating_add(rh).min(h));
    let mut out = String::with_capacity(((x1 - x0 + 1) * (y1 - y0)) as usize);
    for y in y0..y1 {
        let start = (y as usize) * (w as usize);
        out.extend(
            grid[start + x0 as usize..start + x1 as usize]
                .iter()
                .map(|b| char::from(*b)),
        );
        out.push('\n');
    }
    out
}

/// Render the grid without stumps (a single snapshot cannot know them),
/// cropped to `rect` as [`render_with`] does.
pub fn render(engine: &SimEngine, rect: Option<Rect>) -> String {
    render_with(engine, rect, None)
}

/// The people a rendered grid shows: 1 per `@`, `a` or `z`, k per digit k
/// (exact while no tile holds more than nine people).
pub fn people_shown(grid: &str) -> usize {
    grid.bytes()
        .map(|b| match b {
            b'@' | b'a' | b'z' => 1,
            b'2'..=b'9' => usize::from(b - b'0'),
            _ => 0,
        })
        .sum()
}

/// `sim_test::colony_engine(seed)` ticked to `tick` completed ticks, applying
/// `script` exactly as the colony probe's `run_seed` does (entries resolved
/// with `colony::resolve` and submitted right before their tick's systems;
/// entries at `tick` or later are not applied), and a [`StumpTracker`]
/// observed after every tick.
pub fn engine_at(
    seed: u64,
    script: &[ScriptEntry],
    tick: u64,
) -> Result<(SimEngine, StumpTracker), String> {
    let mut engine = crate::scenes::colony_scene(seed);
    let mut stumps = StumpTracker::new();
    stumps.observe(&engine);
    let mut next = 0usize;
    for t in 0..tick {
        let mut avoid = Vec::new();
        while next < script.len() && script[next].tick == t {
            for cmd in resolve(&engine, &script[next].spec, &mut avoid)? {
                sim_systems::commands::submit(&mut engine, cmd);
            }
            next += 1;
        }
        engine.tick();
        stumps.observe(&engine);
    }
    Ok((engine, stumps))
}

/// Parsed `worldsim_probe --scene colony --ascii` arguments.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AsciiArgs {
    /// The one seed (`--seeds S..S+1`).
    pub seed: u64,
    /// Completed ticks before rendering (`--tick T`).
    pub tick: u64,
    /// Crop (`--rect x,y,w,h`).
    pub rect: Option<Rect>,
    /// Command script (`--commands FILE`).
    pub commands: Option<PathBuf>,
    /// Print the legend first (`--ascii-legend`).
    pub legend: bool,
    /// Write the grid here instead of stdout (`--out FILE`).
    pub out: Option<PathBuf>,
    /// Write the people tiles as JSON here (`--people-json FILE`).
    pub people_json: Option<PathBuf>,
}

/// The longest run `--tick` accepts (480 days, the colony probe's limit).
pub const MAX_TICK: u64 = super::colony::MAX_DAYS * 1_440;

fn parse_rect(value: &str) -> Result<Rect, String> {
    let parts: Vec<u32> = value
        .split(',')
        .map(|p| p.trim().parse::<u32>())
        .collect::<Result<_, _>>()
        .map_err(|_| format!("--rect {value}: expected x,y,w,h (non-negative integers)"))?;
    match parts.as_slice() {
        [x, y, w, h] if *w > 0 && *h > 0 => Ok((*x, *y, *w, *h)),
        _ => Err(format!("--rect {value}: expected x,y,w,h with w, h > 0")),
    }
}

impl AsciiArgs {
    /// Parse argv (without the program name); `--scene colony` and `--ascii` are accepted and ignored.
    pub fn parse(args: impl IntoIterator<Item = String>) -> Result<Self, String> {
        let mut seeds: Option<std::ops::Range<u64>> = None;
        let mut tick: Option<u64> = None;
        let mut parsed = Self {
            seed: 0,
            tick: 0,
            rect: None,
            commands: None,
            legend: false,
            out: None,
            people_json: None,
        };
        let mut args = args.into_iter();
        while let Some(flag) = args.next() {
            match flag.as_str() {
                "--ascii" => continue,
                "--ascii-legend" => {
                    parsed.legend = true;
                    continue;
                }
                _ => {}
            }
            let value = args
                .next()
                .ok_or_else(|| format!("missing value for {flag}"))?;
            match flag.as_str() {
                "--scene" if value == "colony" => {}
                "--seeds" => {
                    let (a, b) = value
                        .split_once("..")
                        .ok_or("seeds must be a..b (half-open)")?;
                    seeds = Some(
                        a.parse().map_err(|_| "invalid seed start")?
                            ..b.parse().map_err(|_| "invalid seed end")?,
                    );
                }
                "--tick" => tick = Some(value.parse().map_err(|_| "invalid tick")?),
                "--rect" => parsed.rect = Some(parse_rect(&value)?),
                "--commands" => parsed.commands = Some(value.into()),
                "--out" => parsed.out = Some(value.into()),
                "--people-json" => parsed.people_json = Some(value.into()),
                _ => return Err(format!("unknown --ascii argument {flag} {value}")),
            }
        }
        let seeds = seeds.ok_or("--ascii needs --seeds S..S+1")?;
        if seeds.end != seeds.start.saturating_add(1) {
            return Err("--ascii renders exactly one seed: --seeds S..S+1".into());
        }
        parsed.seed = seeds.start;
        parsed.tick = tick.ok_or("--ascii needs --tick T")?;
        if parsed.tick > MAX_TICK {
            return Err(format!("--tick must be at most {MAX_TICK}"));
        }
        Ok(parsed)
    }
}

/// Run the colony to `--tick`, print (or write) the grid, optionally the legend and the people JSON.
pub fn run(args: &AsciiArgs) -> Result<(), String> {
    let script = match &args.commands {
        Some(path) => load_script(path)?,
        None => Vec::new(),
    };
    let (engine, stumps) = engine_at(args.seed, &script, args.tick)?;
    let mut text = String::new();
    if args.legend {
        text.push_str(&legend());
        text.push('\n');
    }
    text.push_str(&render_with(&engine, args.rect, Some(&stumps)));
    match &args.out {
        Some(path) => {
            std::fs::write(path, &text).map_err(|e| format!("{}: {e}", path.display()))?
        }
        None => print!("{text}"),
    }
    if let Some(path) = &args.people_json {
        let tiles = people_tiles(&engine);
        let looks = people_looks(&engine)?;
        let json = serde_json::json!({
            "seed": args.seed,
            "tick": collect_time_snapshot(&engine).tick,
            "count": tiles.len(),
            "people": tiles.iter().map(|t| [t.0, t.1]).collect::<Vec<_>>(),
            "appearance": looks
                .iter()
                .map(|p| serde_json::json!({
                    "agent_id": p.agent_id,
                    "x": p.x,
                    "y": p.y,
                    "sex": p.sex,
                    "look": p.look,
                    "variant": p.variant,
                }))
                .collect::<Vec<_>>(),
        });
        std::fs::write(
            path,
            serde_json::to_string_pretty(&json).map_err(|e| e.to_string())?,
        )
        .map_err(|e| format!("{}: {e}", path.display()))?;
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ascii_args_parse_and_reject() {
        let a = AsciiArgs::parse(
            [
                "--scene",
                "colony",
                "--seeds",
                "100..101",
                "--ascii",
                "--tick",
                "900",
                "--rect",
                "80,80,40,30",
                "--ascii-legend",
            ]
            .map(String::from),
        )
        .expect("valid");
        assert_eq!(
            (a.seed, a.tick, a.rect, a.legend),
            (100, 900, Some((80, 80, 40, 30)), true)
        );
        for bad in [
            vec!["--seeds", "1..3", "--tick", "5"],
            vec!["--seeds", "1..2"],
            vec!["--seeds", "1..2", "--tick", "5", "--rect", "1,2,3"],
            vec!["--seeds", "1..2", "--tick", "5", "--rect", "1,2,0,4"],
            vec!["--seeds", "1..2", "--tick", "5", "--bogus", "1"],
        ] {
            assert!(AsciiArgs::parse(bad.iter().map(|s| s.to_string())).is_err());
        }
    }

    #[test]
    fn every_terrain_code_has_a_distinct_glyph() {
        let glyphs: BTreeSet<u8> = (0..10).map(terrain_glyph).collect();
        assert_eq!(glyphs.len(), 10);
        assert_eq!(terrain_glyph(200), b'?');
    }
}
