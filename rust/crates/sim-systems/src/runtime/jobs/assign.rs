//! JobAssignSystem: gives available colonists a job — by work type: Craft > Build (deliver, then build) > Quarry > Chop > Split > Haul.
//!
//! Priority: 124 (`JOB_ASSIGN_PRIORITY`). Interval: 1 tick. Gate: `WorldRules.colony`.
//! Reads: `Agent`, `Position`, `AgentState`, `Job`, `Child`, `PorterTrip`; `SimResources.colony` (trees, designations, ground items, zones, reservations, footprints), `combat_pairs`, `rules.colony`; colony sites (`ColonySite`, `MaterialNeeds`, `ConstructionSite`)
//! Writes: `Job`, `SeekTarget`, `AgentState` (`Seeking { Job }`); `SimResources.colony.reservations`, `colony.tallies`, `spatial.colony` (rebuilt when the overlay changed)
//! Events: none
//!
//! An agent is available when it is an adult or elder (no `Child`), not on a
//! porter trip, holds no `Job`, is not in a combat pair, and is `Idle` or at
//! leisure (`Seeking`/`Consuming { Activity }`): jobs replace leisure and idle
//! time, never survival work (ADR 0013). Under the economy the assignment runs
//! in rounds of (priority level, work type) — level 1 first, work types in the
//! M2 goal 3.6 order — serving in each round the people with that level for it,
//! highest skill for it first, then ascending `AgentId` (M3 goal 3.4); without
//! it (M1 rules) each person in `AgentId` order takes the first work type with a
//! target. Targets lie within
//! `job_scan_radius` (Chebyshev), nearest first, ties by `(y, x)` (goal 7.2),
//! found through the spatial index (`SimResources.spatial`, M1.5 goal 6.2).
//! Reservations are written at once, so a later agent this tick sees them.
//! A worker leaving leisure keeps its `Activity` component for one tick:
//! the routine's own cleanup at 125 records it as interrupted.
use hecs::{Entity, World};
use sim_core::colony::{
    BillMode, ItemKind, ItemSpec, Job, JobKind, JobStage, ReserveKey, SkillKind, Skills, Tool,
    WorkPriorities, WorkType,
};
use sim_core::components::{
    Agent, AgentId, AgentState, Child, Elder, PorterTrip, Position, SeekTarget, TargetKind,
};
use sim_core::rules::ColonyRule;
use sim_core::spatial::ChunkIndex;
use sim_engine::{RuntimeSystem, SimResources};
use std::collections::BTreeMap;

use super::bills::{bill_count, maintain_count};
use super::cancel::reserve;
use super::items::{chebyshev, colony_sites, room_on, SiteView};

/// Priority of [`JobAssignSystem`].
pub const JOB_ASSIGN_PRIORITY: u32 = 124;

/// See the module header.
#[derive(Debug, Default)]
pub struct JobAssignSystem;

impl JobAssignSystem {
    /// A fresh system.
    pub fn new() -> Self {
        Self
    }
}

/// Items on their way to each site, by `(site centre, kind)` (sum of `Deliver` jobs' amounts).
pub fn in_transit(world: &World) -> BTreeMap<((u32, u32), ItemKind), u16> {
    let mut out = BTreeMap::new();
    for (_, job) in world.query::<&Job>().iter() {
        if let JobKind::Deliver { site, .. } = job.kind {
            *out.entry((site, job.item)).or_insert(0) += job.amount;
        }
    }
    out
}

/// Firewood units the colony still lacks (M2 goal 3.8): the larger of the
/// hearth's need (`hearth::hearth_need`) and every firewood `Maintain` target,
/// minus the settlements' wood, the firewood a hearth can use (stockpiles and
/// hearth piles, loads in haulers' hands, crafts under way) and a carry per
/// gather job under way.
pub fn gather_shortfall(world: &World, res: &SimResources) -> u32 {
    let mut want = super::hearth::hearth_need(res);
    for shop in res.colony.workshops.values() {
        for bill in &shop.bills {
            let firewood = bill
                .recipe
                .recipe()
                .is_some_and(|r| r.output.0 == ItemKind::Firewood);
            if let (BillMode::Maintain(n), true, false) = (bill.mode, firewood, bill.suspended) {
                want = want.max(u32::from(n));
            }
        }
    }
    let carry = res
        .rules
        .colony
        .map_or(10, |r| u32::from(r.carry(ItemKind::Firewood)));
    let gathering = world
        .query::<&Job>()
        .iter()
        .filter(|(_, j)| matches!(j.kind, JobKind::Gather { .. }))
        .count() as u32;
    // Only firewood a hearth can use counts (DECISIONS D19): stacks lying
    // elsewhere (a released gatherer's load) are not fed and must not stop gathering.
    let ground = res.colony.ground_total(ItemKind::Firewood) as u32;
    let usable = maintain_count(world, res, ItemKind::Firewood)
        .saturating_sub(ground.saturating_sub(super::hearth::feedable_firewood(res)));
    let held = super::hearth::settlement_wood(res) + usable + gathering * carry;
    want.saturating_sub(held)
}

/// Items on their way to each workshop shelf, by `(workshop, kind)` (`Supply` jobs),
/// and the crafts in progress per bill (`Craft` jobs).
pub fn shelf_transit(world: &World) -> (BTreeMap<(u32, ItemKind), u16>, BTreeMap<u32, u32>) {
    let mut supply = BTreeMap::new();
    let mut crafting = BTreeMap::new();
    for (_, job) in world.query::<&Job>().iter() {
        match job.kind {
            JobKind::Supply { workshop, .. } => {
                *supply.entry((workshop, job.item)).or_insert(0) += job.amount;
            }
            JobKind::Craft { bill, .. } => *crafting.entry(bill).or_insert(0) += 1,
            _ => {}
        }
    }
    (supply, crafting)
}

/// Most crafts' inputs one supply trip brings (M3, DECISIONS D13).
pub const SUPPLY_CRAFTS: u32 = 5;

/// Which search a memo belongs to (see [`Memo`]).
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
enum Search {
    ShortSite,
    Stocked(ItemKind),
    AnyStack(ItemKind),
    ReadySite,
    MarkedTree,
    QuarryNode,
    Deadwood,
    Loose,
    OpenCell(ItemSpec),
    ClearStack,
    DigMark,
    DeconMark,
    Restock,
}

/// Searches that found no candidate anywhere on the map this tick. Within one
/// `JobAssignSystem` tick the candidates only shrink (each assignment adds
/// reservations and items in transit), so an empty search stays empty for
/// every later agent of the tick and is skipped (M1.5 DECISIONS D15). Fixed
/// fields and per-kind arrays: the memo is looked up for every agent.
#[derive(Debug, Default, Clone)]
struct Exhausted {
    short_site: Memo,
    ready_site: Memo,
    marked_tree: Memo,
    quarry_node: Memo,
    deadwood: Memo,
    loose: Memo,
    clear: Memo,
    dig: Memo,
    decon: Memo,
    restock: Memo,
    stocked: [Memo; 7],
    any_stack: [Memo; 7],
    open_cell: BTreeMap<ItemSpec, Memo>,
}

impl Exhausted {
    fn memo(&mut self, search: Search) -> &mut Memo {
        match search {
            Search::ShortSite => &mut self.short_site,
            Search::ReadySite => &mut self.ready_site,
            Search::MarkedTree => &mut self.marked_tree,
            Search::QuarryNode => &mut self.quarry_node,
            Search::Deadwood => &mut self.deadwood,
            Search::Loose => &mut self.loose,
            Search::ClearStack => &mut self.clear,
            Search::DigMark => &mut self.dig,
            Search::DeconMark => &mut self.decon,
            Search::Restock => &mut self.restock,
            Search::Stocked(kind) => &mut self.stocked[kind.code() as usize],
            Search::AnyStack(kind) => &mut self.any_stack[kind.code() as usize],
            Search::OpenCell(spec) => self.open_cell.entry(spec).or_default(),
        }
    }
}

/// What this tick has learned about one search: `empty` = nothing passes its
/// predicate anywhere on the map; `seen` = something did (a map-wide check
/// need not run again: if that candidate is taken later, the only cost is a
/// later miss that is not memoised — never a wrong answer).
#[derive(Debug, Default, Clone, Copy)]
struct Memo {
    empty: bool,
    seen: bool,
}

/// `index.nearest_where(pos, radius, pred)`, unless `memo` says nothing passes
/// `pred` anywhere; a miss runs one map-wide check per search and tick.
fn search<T: Copy + Ord>(
    index: &ChunkIndex<T>,
    pos: (u32, u32),
    radius: u32,
    memo: &mut Memo,
    mut pred: impl FnMut((u32, u32), &T) -> bool,
) -> Option<((u32, u32), T)> {
    if memo.empty {
        return None;
    }
    let found = index.nearest_where(pos, radius, &mut pred);
    if found.is_some() {
        memo.seen = true;
    } else if !memo.seen {
        if radius == u32::MAX || index.nearest_where(pos, u32::MAX, &mut pred).is_none() {
            memo.empty = true;
        } else {
            memo.seen = true;
        }
    }
    found
}

/// What one tick's assignment reads, besides the resources.
struct Ctx<'a> {
    rule: &'a ColonyRule,
    sites: &'a BTreeMap<(u32, u32), SiteView>,
    site_index: &'a ChunkIndex<()>,
    transit: &'a BTreeMap<((u32, u32), ItemKind), u16>,
    supply: &'a BTreeMap<(u32, ItemKind), u16>,
    crafting: &'a BTreeMap<u32, u32>,
    made: &'a BTreeMap<ItemKind, u32>,
    /// Deadwood spots with wood (M2), built on first use in a tick: most ticks
    /// need none (the colony is rarely short of firewood).
    deadwood: &'a std::cell::OnceCell<ChunkIndex<()>>,
    /// The world (for the lazily computed firewood shortfall).
    world: &'a World,
    /// Whether firewood goes through jobs (M2 goal 3.8).
    by_jobs: bool,
    /// Firewood units still to gather this tick: computed on first use, then
    /// lowered by a carry per gather job taken (`None` = not computed yet).
    gather_short: &'a std::cell::Cell<Option<u32>>,
    /// The hearths (the gather fallback's drop when no cell has room), on first use.
    hearth: &'a std::cell::OnceCell<Vec<(u32, u32)>>,
    /// The agent's nav region (M3, ADR 0021): only targets reachable from it are offered.
    region: u32,
}

/// A picked job: kind, amount, item.
type Pick = (JobKind, u16, ItemKind);

/// The first job of work type `work` for an agent at `pos`, or `None`.
///
/// M1.5 (goal 6.2): every candidate search goes through the spatial index —
/// `ChunkIndex::nearest_where` returns the same tile the M1 linear
/// `min_by_key((Chebyshev distance, y, x))` over the filtered candidates
/// returned (ties `(y, x)`).
fn pick_work(
    res: &SimResources,
    ctx: &Ctx,
    work: WorkType,
    pos: (u32, u32),
    ex: &mut Exhausted,
) -> Option<Pick> {
    let colony = &res.colony;
    let index = &res.spatial.colony;
    let rule = ctx.rule;
    let radius = rule.job_scan_radius;
    let free_stack =
        |tile: &(u32, u32)| !colony.reservations.contains_key(&ReserveKey::Stack(*tile));
    // A stack of `kind` (any kind with `None`) someone may take (`zoned` = in a stockpile zone).
    let stack_of = |tile: (u32, u32), kind: Option<ItemKind>, zoned: Option<bool>| {
        colony.ground_items.get(&tile).is_some_and(|stack| {
            kind.is_none_or(|k| stack.kind == k)
                && stack.count > 0
                && free_stack(&tile)
                && zoned.is_none_or(|z| colony.zone_of_tile.contains_key(&tile) == z)
        })
    };
    let count = |tile: (u32, u32)| colony.ground_items.get(&tile).map_or(0, |s| s.count);
    // M3: a target the agent cannot walk to is not a candidate.
    let reach = |tile: (u32, u32)| res.nav.reaches(ctx.region, tile);
    let stack_of = |tile: (u32, u32), kind: Option<ItemKind>, zoned: Option<bool>| {
        stack_of(tile, kind, zoned) && reach(tile)
    };
    match work {
        WorkType::Craft | WorkType::Split => {
            // Workshops whose active bill is of this work type, nearest work tile
            // first (ties `(y, x)`, then id); workshops are few, so a sorted list
            // stands in for an index.
            let mut shops: Vec<(u32, u32, u32, u32)> = colony
                .workshops
                .values()
                .filter_map(|shop| {
                    let bill = shop.bill(shop.active?)?;
                    let recipe = bill.recipe.recipe()?;
                    let tile = shop.work_tile();
                    let d = chebyshev(pos, tile);
                    (recipe.work == work && d <= radius && reach(tile))
                        .then_some((d, tile.1, tile.0, shop.id))
                })
                .collect();
            shops.sort_unstable();
            for (_, _, _, id) in shops {
                let Some(shop) = colony.workshops.get(&id) else {
                    continue;
                };
                let Some(bill) = shop.active.and_then(|b| shop.bill(b)) else {
                    continue;
                };
                let Some(recipe) = bill.recipe.recipe() else {
                    continue;
                };
                let in_progress = ctx.crafting.get(&bill.id).copied().unwrap_or(0);
                if bill.remaining().is_some_and(|left| left <= in_progress) {
                    continue; // every remaining craft is already being made
                }
                if let BillMode::Maintain(n) = bill.mode {
                    let made = ctx.made.get(&recipe.output.0).copied().unwrap_or(0);
                    if made >= u32::from(n) {
                        continue; // the stock (with crafts under way) is full
                    }
                }
                let to = shop.work_tile();
                // Supply: the first input short of one craft, from the nearest stack.
                // M3 (DECISIONS D13): a trip brings inputs for up to `SUPPLY_CRAFTS`
                // of the bill's remaining crafts (one craft's under M2 rules).
                let crafts = if rule.space.is_some() {
                    let left = match bill.mode {
                        BillMode::Make(_) => bill.remaining().unwrap_or(1),
                        BillMode::Maintain(n) => {
                            let made = ctx.made.get(&recipe.output.0).copied().unwrap_or(0);
                            u32::from(n)
                                .saturating_sub(made)
                                .div_ceil(u32::from(recipe.output.1.max(1)))
                        }
                    };
                    left.saturating_sub(in_progress).clamp(1, SUPPLY_CRAFTS) as u16
                } else {
                    1
                };
                for &(kind, n) in &recipe.inputs {
                    let coming = ctx.supply.get(&(id, kind)).copied().unwrap_or(0);
                    let have = shop.shelved(kind) + coming;
                    if have >= n {
                        continue;
                    }
                    let n = n.saturating_mul(crafts);
                    let stocked = search(
                        &index.ground,
                        pos,
                        radius,
                        ex.memo(Search::Stocked(kind)),
                        |t, _| stack_of(t, Some(kind), Some(true)),
                    );
                    let source = stocked.or_else(|| {
                        search(
                            &index.ground,
                            pos,
                            radius,
                            ex.memo(Search::AnyStack(kind)),
                            |t, _| stack_of(t, Some(kind), None),
                        )
                    });
                    if let Some((from, ())) = source {
                        let amount = rule.carry(kind).min(count(from)).min(n - have);
                        return Some((
                            JobKind::Supply {
                                from,
                                workshop: id,
                                to,
                            },
                            amount,
                            kind,
                        ));
                    }
                }
                // Craft: every input on the shelf and the bench free.
                let ready = recipe
                    .inputs
                    .iter()
                    .all(|&(kind, n)| shop.shelved(kind) >= n);
                if ready && !colony.reservations.contains_key(&ReserveKey::Bench(to)) {
                    return Some((
                        JobKind::Craft {
                            workshop: id,
                            bill: bill.id,
                            at: to,
                        },
                        0,
                        recipe.output.0,
                    ));
                }
            }
            if work == WorkType::Split {
                return gather_pick(res, ctx, pos, ex);
            }
            None
        }
        WorkType::Build => {
            // M3: first carry the items off a space site's tile (it cannot be
            // built while they lie there; ADR 0022's haul-away).
            let on_space_site = |t: (u32, u32)| colony.space_sites.contains_key(&t);
            if let Some((from, ())) = search(
                &index.ground,
                pos,
                radius,
                ex.memo(Search::ClearStack),
                |t, _| on_space_site(t) && stack_of(t, None, None),
            ) {
                if let Some(stack) = colony.ground_items.get(&from) {
                    let spec = stack.spec();
                    if let Some(to) = clear_destination(res, ctx, from, spec) {
                        let amount = rule
                            .carry(spec.kind)
                            .min(stack.count)
                            .min(room_on(res, to, spec));
                        if amount > 0 {
                            return Some((JobKind::Clear { from, to }, amount, spec.kind));
                        }
                    }
                }
            }
            // First half: deliver a material to the nearest site still short of one.
            let missing = |centre: &(u32, u32), kind: ItemKind| {
                ctx.sites.get(centre).and_then(|view| {
                    let coming = ctx.transit.get(&(*centre, kind)).copied().unwrap_or(0);
                    let short = view.needs.missing(kind).saturating_sub(coming);
                    (short > 0).then_some(short)
                })
            };
            let short_kinds = |centre: &(u32, u32)| -> Vec<ItemKind> {
                ctx.sites.get(centre).map_or(Vec::new(), |view| {
                    view.needs
                        .used()
                        .map(|line| line.kind)
                        .filter(|kind| missing(centre, *kind).is_some())
                        .collect()
                })
            };
            let short = search(
                ctx.site_index,
                pos,
                radius,
                ex.memo(Search::ShortSite),
                |c, _| reach(c) && !short_kinds(&c).is_empty(),
            );
            if let Some((site, ())) = short {
                for kind in short_kinds(&site) {
                    let need = missing(&site, kind).unwrap_or(0);
                    let stocked = search(
                        &index.ground,
                        pos,
                        radius,
                        ex.memo(Search::Stocked(kind)),
                        |t, _| stack_of(t, Some(kind), Some(true)),
                    );
                    let source = stocked.or_else(|| {
                        search(
                            &index.ground,
                            pos,
                            radius,
                            ex.memo(Search::AnyStack(kind)),
                            |t, _| stack_of(t, Some(kind), None),
                        )
                    });
                    if let Some((from, ())) = source {
                        let amount = rule.carry(kind).min(count(from)).min(need);
                        return Some((JobKind::Deliver { from, site }, amount, kind));
                    }
                }
            }
            // Second half: build a site whose materials are all there (M3: a
            // space site also needs its tile clear of items).
            let ready = |centre: (u32, u32)| {
                ctx.sites.get(&centre).is_some_and(|view| {
                    view.needs.complete()
                        && view.progress < view.required
                        && !colony.reservations.contains_key(&ReserveKey::Site(centre))
                        && !(on_space_site(centre) && colony.ground_items.contains_key(&centre))
                })
            };
            if let Some((site, ())) = search(
                ctx.site_index,
                pos,
                radius,
                ex.memo(Search::ReadySite),
                |c, _| reach(c) && ready(c),
            ) {
                return Some((JobKind::Build { site }, 0, ItemKind::Log));
            }
            // M3: take apart what is marked for deconstruction.
            let open = |t: (u32, u32)| {
                colony.designations.get(&t) == Some(&sim_core::colony::Designation::Deconstruct)
                    && !colony.reservations.contains_key(&ReserveKey::Work(t))
                    && super::space::built_at(res, t).is_some()
                    && reach(t)
            };
            search(
                &index.deconstruct,
                pos,
                radius,
                ex.memo(Search::DeconMark),
                |t, _| open(t),
            )
            .map(|(tile, ())| (JobKind::Deconstruct { tile }, 0, ItemKind::Log))
        }
        WorkType::Mine => {
            // M3: the nearest marked hill tile someone can stand beside.
            let open = |t: (u32, u32)| {
                colony.designations.get(&t) == Some(&sim_core::colony::Designation::Dig)
                    && !colony.reservations.contains_key(&ReserveKey::Work(t))
                    && super::space::diggable(res, t)
                    && reach(t)
            };
            search(&index.dig, pos, radius, ex.memo(Search::DigMark), |t, _| {
                open(t)
            })
            .map(|(tile, ())| (JobKind::Dig { tile }, 0, ItemKind::Stone))
        }
        WorkType::Quarry => {
            let quarriable = |tile: (u32, u32)| {
                colony.nodes.contains_key(&tile)
                    && !colony.reservations.contains_key(&ReserveKey::Node(tile))
                    && reach(tile)
            };
            search(
                &index.quarry,
                pos,
                radius,
                ex.memo(Search::QuarryNode),
                |t, _| quarriable(t),
            )
            .map(|(node, ())| (JobKind::Quarry { node }, 0, ItemKind::Log))
        }
        WorkType::Chop => {
            // The nearest marked tree no one has taken.
            let choppable = |tile: (u32, u32)| {
                colony.trees.contains_key(&tile)
                    && !colony.reservations.contains_key(&ReserveKey::Tree(tile))
                    && reach(tile)
            };
            search(
                &index.marked,
                pos,
                radius,
                ex.memo(Search::MarkedTree),
                |t, _| choppable(t),
            )
            .map(|(tree, ())| (JobKind::Chop { tree }, 0, ItemKind::Log))
        }
        WorkType::Haul => {
            // The nearest loose stack, to the stockpile cell nearest that stack
            // with room for it. Stacks whose spec no cell takes (a memo found
            // none this tick) are skipped; the predicate only grows stricter
            // within the tick, so the `Loose` memo stays valid.
            loop {
                let blocked: Vec<ItemSpec> = ex
                    .open_cell
                    .iter()
                    .filter(|(_, memo)| memo.empty)
                    .map(|(spec, _)| *spec)
                    .collect();
                let loose = search(
                    &index.ground,
                    pos,
                    radius,
                    ex.memo(Search::Loose),
                    |t, _| {
                        stack_of(t, None, Some(false))
                            && colony
                                .ground_items
                                .get(&t)
                                .is_some_and(|stack| !blocked.contains(&stack.spec()))
                    },
                );
                let Some((from, ())) = loose else {
                    return restock(res, ctx, pos, ex);
                };
                let spec = colony.ground_items.get(&from)?.spec();
                let open = |cell: (u32, u32)| {
                    room_on(res, cell, spec) > 0
                        && !colony.reservations.contains_key(&ReserveKey::Cell(cell))
                        && reach(cell)
                };
                let cell = search(
                    &index.cells,
                    from,
                    u32::MAX,
                    ex.memo(Search::OpenCell(spec)),
                    |c, _| open(c),
                )
                .map(|(to, ())| prefer_covered(res, ctx, from, to, &open));
                if let Some(to) = cell {
                    let amount = rule
                        .carry(spec.kind)
                        .min(count(from))
                        .min(room_on(res, to, spec));
                    return Some((JobKind::Haul { from, to }, amount, spec.kind));
                }
                // The whole map has no cell for `spec`: mark it (the memo may
                // have `seen` a cell earlier this tick), so the next round
                // skips stacks of it and the loop ends.
                ex.memo(Search::OpenCell(spec)).empty = true;
            }
        }
    }
}

/// The Split fallback (M2 goal 3.8): while the colony is short of firewood
/// and no firewood bill can run, gather the nearest deadwood spot no one has
/// taken, for the stockpile cell nearest it that takes firewood, or the hearth
/// pile when none has room.
fn gather_pick(res: &SimResources, ctx: &Ctx, pos: (u32, u32), ex: &mut Exhausted) -> Option<Pick> {
    if !ctx.by_jobs {
        return None;
    }
    let short = ctx.gather_short.get().unwrap_or_else(|| {
        let v = gather_shortfall(ctx.world, res);
        ctx.gather_short.set(Some(v));
        v
    });
    if short == 0 {
        return None;
    }
    let colony = &res.colony;
    let free = |tile: (u32, u32)| {
        res.wood_tiles.get(&tile).is_some_and(|units| *units > 0)
            && !colony.reservations.contains_key(&ReserveKey::Wood(tile))
            && res.nav.reaches(ctx.region, tile)
    };
    let deadwood = ctx.deadwood.get_or_init(|| {
        let mut index = ChunkIndex::new();
        for (tile, units) in &res.wood_tiles {
            if *units > 0 {
                index.insert(*tile, ());
            }
        }
        index
    });
    let (spot, ()) = search(
        deadwood,
        pos,
        ctx.rule.job_scan_radius,
        ex.memo(Search::Deadwood),
        |t, _| free(t),
    )?;
    let spec = ItemSpec::raw(ItemKind::Firewood, Some(sim_core::colony::materials::OAK));
    let open = |cell: (u32, u32)| {
        room_on(res, cell, spec) > 0
            && !colony.reservations.contains_key(&ReserveKey::Cell(cell))
            && res.nav.reaches(ctx.region, cell)
    };
    let cell = res
        .spatial
        .colony
        .cells
        .nearest_where(spot, u32::MAX, |c, _| open(c))
        .map(|(c, ())| c);
    // The settlement's hearth (the feed reads its pile), nearest the spot when several.
    let near = ctx
        .hearth
        .get_or_init(|| super::hearth::hearth_tiles(res))
        .iter()
        .copied()
        .min_by_key(|t| (chebyshev(spot, *t), t.1, t.0));
    let to = cell.or(near)?;
    Some((JobKind::Gather { spot, to }, 0, ItemKind::Firewood))
}

/// The best unreserved axe stack on a stockpile cell within `reach` of `pos`
/// that the agent can walk to: the highest quality, then the nearest
/// `(distance, y, x)` (M2 goal 3.7).
fn best_axe(res: &SimResources, pos: (u32, u32), reach: u32) -> Option<(u32, u32)> {
    let region = res.nav.region_of(pos);
    let colony = &res.colony;
    let index = &res.spatial.colony.ground;
    let chunks = reach / sim_core::spatial::CHUNK + 1;
    let mut best: Option<(std::cmp::Reverse<u8>, u32, u32, u32)> = None;
    for r in 0..=chunks {
        for &(tile, ()) in index.ring(pos, r) {
            let d = chebyshev(pos, tile);
            let Some(stack) = colony.ground_items.get(&tile) else {
                continue;
            };
            if d > reach
                || stack.kind != ItemKind::StoneAxe
                || !colony.zone_of_tile.contains_key(&tile)
                || colony.reservations.contains_key(&ReserveKey::Stack(tile))
                || !res.nav.reaches(region, tile)
            {
                continue;
            }
            let key = (std::cmp::Reverse(stack.quality), d, tile.1, tile.0);
            if best.is_none_or(|b| key < b) {
                best = Some(key);
            }
        }
    }
    best.map(|(_, _, y, x)| (x, y))
}

/// A person's skill level for work type `work` (M3 goal 3.4 tie-break):
/// crafting counts the best of `crafts` (the skills of the crafts on offer;
/// knapping and woodworking when none is), splitting woodworking, hauling none.
pub fn skill_for(skills: &Skills, work: WorkType, crafts: &[SkillKind]) -> u8 {
    match work {
        WorkType::Craft if !crafts.is_empty() => {
            crafts.iter().map(|k| skills.level(*k)).max().unwrap_or(0)
        }
        WorkType::Craft => skills
            .level(SkillKind::Knap)
            .max(skills.level(SkillKind::Woodwork)),
        WorkType::Build => skills.level(SkillKind::Build),
        WorkType::Quarry => skills.level(SkillKind::Quarry),
        WorkType::Chop => skills.level(SkillKind::Chop),
        WorkType::Split => skills.level(SkillKind::Woodwork),
        WorkType::Haul => 0,
        WorkType::Mine => skills.level(SkillKind::Mining),
    }
}

/// M3 (ADR 0023): with no loose stack left, carry stock lying on an open-air
/// stockpile cell into a roofed storehouse cell that takes it (the nearest
/// such stack, then the covered cell nearest it).
fn restock(res: &SimResources, ctx: &Ctx, pos: (u32, u32), ex: &mut Exhausted) -> Option<Pick> {
    if ctx.rule.space.is_none() || !crate::runtime::rooms::storehouse_exists(res) {
        return None;
    }
    let colony = &res.colony;
    let outdoor = |t: (u32, u32)| {
        colony.zone_of_tile.contains_key(&t)
            && !crate::runtime::rooms::covered(res, t)
            && colony.ground_items.get(&t).is_some_and(|s| s.count > 0)
            && !colony.reservations.contains_key(&ReserveKey::Stack(t))
            && res.nav.reaches(ctx.region, t)
    };
    let (from, ()) = search(
        &res.spatial.colony.ground,
        pos,
        ctx.rule.job_scan_radius,
        ex.memo(Search::Restock),
        |t, _| outdoor(t),
    )?;
    let stack = colony.ground_items.get(&from)?;
    let spec = stack.spec();
    let (to, ()) = res
        .spatial
        .colony
        .cells
        .nearest_where(from, u32::MAX, |c, _| {
            crate::runtime::rooms::covered(res, c)
                && room_on(res, c, spec) > 0
                && !colony.reservations.contains_key(&ReserveKey::Cell(c))
                && res.nav.reaches(ctx.region, c)
        })?;
    let amount = ctx
        .rule
        .carry(spec.kind)
        .min(stack.count)
        .min(room_on(res, to, spec));
    (amount > 0).then_some((JobKind::Haul { from, to }, amount, spec.kind))
}

/// M3 (ADR 0023): with a roofed storehouse, a hauled stack goes to the nearest
/// open cell under its roof unless that is more than
/// `SpaceRule::storehouse_bonus_tiles` farther than the nearest open cell
/// `nearest` (the bonus is worth that many tiles of distance).
fn prefer_covered(
    res: &SimResources,
    ctx: &Ctx,
    from: (u32, u32),
    nearest: (u32, u32),
    open: &dyn Fn((u32, u32)) -> bool,
) -> (u32, u32) {
    let Some(bonus) = ctx.rule.space.map(|s| s.storehouse_bonus_tiles) else {
        return nearest;
    };
    if crate::runtime::rooms::covered(res, nearest)
        || !crate::runtime::rooms::storehouse_exists(res)
    {
        return nearest;
    }
    let reach = chebyshev(from, nearest).saturating_add(bonus);
    res.spatial
        .colony
        .cells
        .nearest_where(from, reach, |c, _| {
            crate::runtime::rooms::covered(res, c) && open(c)
        })
        .map_or(nearest, |(c, ())| c)
}

/// Where the items cleared off a space site go (M3): the nearest open
/// stockpile cell that takes them, else the nearest tile within 12 that can
/// hold them, is not a site and is not reserved; both reachable.
fn clear_destination(
    res: &SimResources,
    ctx: &Ctx,
    from: (u32, u32),
    spec: ItemSpec,
) -> Option<(u32, u32)> {
    let colony = &res.colony;
    let free = |c: (u32, u32)| {
        room_on(res, c, spec) > 0
            && !colony.reservations.contains_key(&ReserveKey::Cell(c))
            && res.nav.reaches(ctx.region, c)
    };
    if let Some((cell, ())) = res
        .spatial
        .colony
        .cells
        .nearest_where(from, u32::MAX, |c, _| free(c))
    {
        return Some(cell);
    }
    let (w, h) = (res.tile_grid.width, res.tile_grid.height);
    for r in 1..=12u32 {
        let (cx, cy) = (i64::from(from.0), i64::from(from.1));
        let r = i64::from(r);
        for y in (cy - r)..=(cy + r) {
            for x in (cx - r)..=(cx + r) {
                if (x - cx).abs().max((y - cy).abs()) != r || x < 0 || y < 0 {
                    continue;
                }
                let t = (x as u32, y as u32);
                if t.0 < w && t.1 < h && free(t) {
                    return Some(t);
                }
            }
        }
    }
    None
}

/// The first job for an agent at `pos` in work-type order, or `None`.
fn pick(
    res: &SimResources,
    ctx: &Ctx,
    priorities: &WorkPriorities,
    pass: u8,
    pos: (u32, u32),
    ex: &mut Exhausted,
) -> Option<Pick> {
    WorkType::ALL
        .into_iter()
        .filter(|w| pass == 0 || priorities.level(*w) == pass)
        .find_map(|work| pick_work(res, ctx, work, pos, ex))
}

impl RuntimeSystem for JobAssignSystem {
    fn name(&self) -> &str {
        "JobAssignSystem"
    }

    fn priority(&self) -> u32 {
        JOB_ASSIGN_PRIORITY
    }

    fn tick_interval(&self) -> u64 {
        1
    }

    fn tick(&mut self, world: &mut World, resources: &mut SimResources) {
        let Some(rule) = resources.rules.colony else {
            return;
        };
        let tick = resources.current_tick;
        let in_combat = |id: AgentId| {
            resources
                .combat_pairs
                .iter()
                .any(|&(a, b)| a == id || b == id)
        };
        // (id, entity, tile, work priorities, holds a tool, skills)
        type Row = (AgentId, Entity, (u32, u32), WorkPriorities, bool, Skills);
        // M3 (DECISIONS D16): a person whose hunger or thirst is at its seek
        // threshold is about to leave for food or water; no new job then.
        let needy = |e: Entity| {
            use crate::runtime::decision::needs as n;
            rule.space.is_some()
                && (world
                    .get::<&sim_core::components::Hunger>(e)
                    .is_ok_and(|h| h.value >= n::HUNGER_THRESHOLD)
                    || world
                        .get::<&sim_core::components::Thirst>(e)
                        .is_ok_and(|t| t.value >= n::THIRST_THRESHOLD))
        };
        let mut available: Vec<Row> = world
            .query::<(
                &Agent,
                &Position,
                &AgentState,
                Option<&WorkPriorities>,
                Option<&Elder>,
                Option<&Tool>,
                Option<&Skills>,
            )>()
            .without::<&Job>()
            .without::<&Child>()
            .without::<&PorterTrip>()
            .iter()
            .filter(|(entity, _)| !needy(*entity))
            .filter(|(_, (agent, _, state, _, _, _, _))| {
                matches!(
                    state,
                    AgentState::Idle
                        | AgentState::Seeking {
                            target: TargetKind::Activity
                        }
                        | AgentState::Consuming {
                            target: TargetKind::Activity
                        }
                ) && !in_combat(agent.id)
            })
            .map(
                |(entity, (agent, pos, _, priorities, elder, tool, skills))| {
                    let p = priorities
                        .copied()
                        .unwrap_or(WorkPriorities::defaults(elder.is_some()));
                    let skills = skills.copied().unwrap_or_default();
                    (agent.id, entity, (pos.x, pos.y), p, tool.is_some(), skills)
                },
            )
            .collect();
        if available.is_empty() {
            return;
        }
        available.sort_by_key(|row| row.0);
        resources.spatial.colony.refresh(&resources.colony);
        crate::runtime::nav::ensure(resources);
        resources
            .colony
            .unreachable
            .retain(|_, until| *until > tick);
        let sites = colony_sites(world);
        let mut site_index = ChunkIndex::new();
        for centre in sites.keys() {
            site_index.insert(*centre, ());
        }
        let mut transit = in_transit(world);
        let (mut supply, mut crafting) = shelf_transit(world);
        // `Maintain` counts of every output kind (BillSystem's count), updated
        // as crafts are taken this tick.
        let mut made: BTreeMap<ItemKind, u32> = BTreeMap::new();
        for shop in resources.colony.workshops.values() {
            for bill in &shop.bills {
                if let (BillMode::Maintain(_), Some(r)) = (bill.mode, bill.recipe.recipe()) {
                    made.entry(r.output.0)
                        .or_insert_with(|| bill_count(world, resources, r.output.0));
                }
            }
        }
        // M2 goal 3.8: the gather fallback's inputs.
        let by_jobs = super::hearth::firewood_by_jobs(resources);
        let deadwood = std::cell::OnceCell::new();
        let gather_short = std::cell::Cell::new(None);
        let hearth = std::cell::OnceCell::new();
        let mut taken: Vec<(Entity, Job)> = Vec::new();
        // M3: the "found nothing" memos hold per nav region (the searches filter by it).
        let mut exhausted: BTreeMap<u32, Exhausted> = BTreeMap::new();
        // M2 (goal 3.6, DECISIONS D14): passes by priority level — level 1
        // first, then 2, … 4, so a person with priority 1 gets first pick of
        // that work. M3 (goal 3.4, M3 DECISIONS D3): within a level, one round
        // per work type (tie-break order), and in it the people with that level
        // for it ordered by their skill for it, highest first, then `AgentId`.
        // Without the economy one pass over every work type per person (M1).
        let rounds: Vec<(u8, Option<WorkType>)> = if rule.economy.is_some() {
            (1..=sim_core::colony::skills::LOWEST_PRIORITY)
                .flat_map(|pass| WorkType::ALL.into_iter().map(move |w| (pass, Some(w))))
                .collect()
        } else {
            vec![(0, None)]
        };
        let mut assigned: std::collections::BTreeSet<AgentId> = std::collections::BTreeSet::new();
        // The skills of the crafts on offer (active bills of work type Craft), for the craft round's order.
        let mut craft_skills: Vec<SkillKind> = resources
            .colony
            .workshops
            .values()
            .filter_map(|shop| shop.bill(shop.active?)?.recipe.recipe())
            .filter(|r| r.work == WorkType::Craft)
            .map(|r| r.skill)
            .collect();
        craft_skills.sort_unstable();
        craft_skills.dedup();
        for (pass, work) in rounds {
            let mut order: Vec<&Row> = available
                .iter()
                .filter(|row| {
                    !assigned.contains(&row.0) && work.is_none_or(|w| row.3.level(w) == pass)
                })
                .collect();
            if let Some(w) = work {
                order.sort_by_key(|row| {
                    (
                        std::cmp::Reverse(skill_for(&row.5, w, &craft_skills)),
                        row.0,
                    )
                });
            }
            for &&(id, entity, pos, priorities, has_tool, _) in &order {
                let ctx = Ctx {
                    rule: &rule,
                    sites: &sites,
                    site_index: &site_index,
                    transit: &transit,
                    supply: &supply,
                    crafting: &crafting,
                    made: &made,
                    deadwood: &deadwood,
                    world,
                    by_jobs,
                    gather_short: &gather_short,
                    hearth: &hearth,
                    region: resources.nav.region_of(pos),
                };
                let ex = exhausted.entry(ctx.region).or_default();
                let found = match work {
                    Some(w) => pick_work(resources, &ctx, w, pos, ex),
                    None => pick(resources, &ctx, &priorities, pass, pos, ex),
                };
                let Some((kind, amount, item)) = found else {
                    continue;
                };
                // M3: a target this person found unreachable lately is skipped.
                if resources
                    .colony
                    .unreachable
                    .get(&(id, kind.source()))
                    .is_some_and(|until| *until > tick)
                {
                    continue;
                }
                // A chopper without a tool stops for the best axe in a stockpile on the way.
                let fetch = match (kind, rule.economy) {
                    (JobKind::Chop { .. }, Some(e)) if !has_tool => {
                        best_axe(resources, pos, e.axe_reach)
                    }
                    _ => None,
                };
                let job = Job {
                    kind,
                    stage: JobStage::ToSource,
                    taken_at: tick,
                    amount,
                    item,
                    material: None,
                    fetch,
                    best_distance: chebyshev(pos, fetch.unwrap_or(kind.source())),
                    progress_tick: tick,
                };
                for key in job.keys() {
                    reserve(resources, key, id);
                }
                match kind {
                    JobKind::Deliver { site, .. } => {
                        *transit.entry((site, item)).or_insert(0) += amount;
                    }
                    JobKind::Supply { workshop, .. } => {
                        *supply.entry((workshop, item)).or_insert(0) += amount;
                    }
                    JobKind::Gather { .. } => {
                        let carry = u32::from(rule.carry(ItemKind::Firewood));
                        gather_short.set(gather_short.get().map(|v| v.saturating_sub(carry)));
                    }
                    JobKind::Craft { bill, workshop, .. } => {
                        *crafting.entry(bill).or_insert(0) += 1;
                        let output = resources
                            .colony
                            .workshops
                            .get(&workshop)
                            .and_then(|shop| shop.bill(bill))
                            .and_then(|b| b.recipe.recipe())
                            .map(|r| r.output);
                        if let Some((kind, n)) = output {
                            if let Some(count) = made.get_mut(&kind) {
                                *count += u32::from(n);
                            }
                        }
                    }
                    _ => {}
                }
                resources.colony.tallies.jobs_taken += 1;
                *resources
                    .colony
                    .tallies
                    .taken_by
                    .entry((id, kind.code()))
                    .or_insert(0) += 1;
                taken.push((entity, job));
                assigned.insert(id);
            }
        }
        for (entity, job) in taken {
            let goal = job.fetch.unwrap_or(job.kind.source());
            let _ = world.insert(entity, (job, SeekTarget::new(goal)));
            if let Ok(mut state) = world.get::<&mut AgentState>(entity) {
                *state = AgentState::Seeking {
                    target: TargetKind::Job,
                };
            }
        }
    }
}
