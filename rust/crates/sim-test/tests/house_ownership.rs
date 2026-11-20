//! HOUSE1 harnesses (ADR 0072): house identity over upgrades, residents vs reachable beds,
//! legacy dates and replay, and exclusive/inherited household ownership on real colony worlds.
//! Each harness fails when its scenario never reaches the behaviour it measures.
mod spine_support;

use sim_core::chronicle::ChronicleKind;
use sim_core::colony::building_site::BuildingStatus;
use sim_core::colony::household::{unpack_ownership_change, HouseId, HouseView, HouseholdId};
use sim_core::colony::{Command, ItemKind, ItemSpec};
use sim_core::components::{AgentId, BuildingKind};
use sim_engine::SimEngine;
use sim_systems::runtime::food::items::{create_ground, FoodSource};
use sim_systems::runtime::housing::house_views;
use sim_systems::runtime::survival::despawn_agent;
use spine_support::{anchor, camp, command, complete, scene, until};
use std::collections::{BTreeMap, BTreeSet};

fn raw_stock(engine: &mut SimEngine, name: &str, mut count: u16) {
    let kind = ItemKind::from_name(name).expect("raw fixture item");
    let centre = camp(engine);
    let mut tiles: Vec<_> = (centre.1.saturating_sub(12)..=centre.1 + 12)
        .flat_map(|y| (centre.0.saturating_sub(12)..=centre.0 + 12).map(move |x| (x, y)))
        .collect();
    tiles.sort_by_key(|&(x, y)| (x.abs_diff(centre.0) + y.abs_diff(centre.1), y, x));
    for tile in tiles {
        if count == 0 {
            break;
        }
        if !sim_systems::runtime::nav::passable(&engine.resources, tile)
            || engine.resources.colony.ground_items.contains_key(&tile)
            || !engine
                .resources
                .nav
                .reaches(engine.resources.nav.region_of(centre), tile)
        {
            continue;
        }
        let take = count.min(kind.definition().stack_max);
        let day = sim_systems::runtime::food::day(&engine.resources);
        if create_ground(
            &mut engine.resources,
            tile,
            ItemSpec::raw(kind, None),
            take,
            day,
            FoodSource::Debug,
        )
        .is_ok()
        {
            count -= take;
        }
    }
    assert_eq!(count, 0, "enough real ground owners for fixture {name}");
}

/// A connected Dwelling template chosen by data: the first with an upgrade target (`upgradable`)
/// or the first whose single piece is a multi-slot hut-like dwelling (not upgradable).
fn dwelling(upgradable: bool, single_piece: bool) -> String {
    let catalog = sim_core::colony::content::installed_or_bundled();
    catalog
        .building_templates()
        .buildings
        .iter()
        .filter(|t| {
            t.category == sim_core::colony::building_template::BuildingCategory::Dwelling
                && t.status == sim_core::colony::knowledge::ContentStatus::Connected
                && t.upgrade_to.is_some() == upgradable
                && (t.pieces.len() == 1) == single_piece
                && (!upgradable || t.upgrade_from.is_some())
        })
        .map(|t| t.id.clone())
        .next()
        .expect("a data-selected dwelling template")
}

fn build(engine: &mut SimEngine, name: &str) -> u32 {
    let at = anchor(engine, name);
    let id = engine.resources.colony.buildings.next_site_id;
    command(
        engine,
        Command::BuildTemplate {
            id: name.into(),
            anchor: at,
            rotation: 0,
            station: None,
        },
    );
    until(engine, name, |e| complete(e, id));
    id
}

fn minute(engine: &SimEngine) -> u64 {
    engine
        .current_tick()
        .saturating_add(engine.resources.rules.clock_offset_ticks)
        % engine.resources.ticks_per_day.max(1)
}

/// Tick to the minute after the next 20:00 bed reassignment (the housing boundary).
fn after_housing_boundary(engine: &mut SimEngine) {
    engine.tick();
    while minute(engine) != 1201 {
        engine.tick();
    }
}

fn view(engine: &SimEngine, id: u32) -> HouseView {
    house_views(&engine.world, &engine.resources)
        .into_iter()
        .find(|v| v.id == HouseId(id))
        .expect("house row")
}

/// Residents of every row equal the agents assigned to a bed inside it.
fn residents_match_assignment(engine: &SimEngine) -> usize {
    let mut mismatches = 0;
    for row in house_views(&engine.world, &engine.resources) {
        let tiles: BTreeSet<_> = row.bed_refs.iter().map(|bed| bed.tile).collect();
        let expected: Vec<AgentId> = engine
            .resources
            .bed_of
            .iter()
            .filter(|(_, bed)| tiles.contains(*bed))
            .map(|(id, _)| *id)
            .collect();
        mismatches += usize::from(expected != row.resident_ids);
    }
    mismatches
}

/// Room ids shared by two rows plus room ids that name no detected room.
fn room_reference_errors(engine: &SimEngine) -> usize {
    let mut seen = BTreeSet::new();
    let mut errors = 0;
    for row in house_views(&engine.world, &engine.resources) {
        for room in row.room_ids {
            errors += usize::from(!seen.insert(room));
            errors += usize::from(!engine.resources.colony.rooms.contains_key(&room));
        }
    }
    errors
}

fn household_members(engine: &SimEngine, id: HouseholdId) -> BTreeSet<AgentId> {
    let state = &engine.resources.colony.buildings.housing;
    let mut out: BTreeSet<AgentId> = state
        .households
        .get(&id)
        .map(|h| h.members.clone())
        .unwrap_or_default();
    for sub in state
        .households
        .values()
        .filter(|h| h.parent == Some(id) && !state.ownership.values().any(|owner| *owner == h.id))
    {
        out.extend(sub.members.iter().copied());
    }
    out
}

/// Residents of owned houses who are neither owner members nor its houseless sub-households.
fn exclusivity_violations(engine: &SimEngine) -> usize {
    house_views(&engine.world, &engine.resources)
        .into_iter()
        .filter_map(|row| row.owner_household.map(|owner| (row, owner)))
        .map(|(row, owner)| {
            let allowed = household_members(engine, owner);
            row.resident_ids
                .iter()
                .filter(|id| !allowed.contains(id))
                .count()
        })
        .sum()
}

/// Owners that no longer exist or have no living member.
fn dangling_owners(engine: &SimEngine) -> usize {
    let living: BTreeSet<AgentId> = engine
        .world
        .query::<&sim_core::components::Agent>()
        .iter()
        .map(|(_, agent)| agent.id)
        .collect();
    let state = &engine.resources.colony.buildings.housing;
    state
        .ownership
        .values()
        .filter(|owner| {
            state
                .households
                .get(owner)
                .is_none_or(|h| !h.members.iter().any(|m| living.contains(m)))
        })
        .count()
}

/// Living adults (no `Child`).
fn adults(engine: &SimEngine) -> BTreeSet<AgentId> {
    engine
        .world
        .query::<(
            &sim_core::components::Agent,
            Option<&sim_core::components::Child>,
        )>()
        .iter()
        .filter(|(_, (_, child))| child.is_none())
        .map(|(_, (agent, _))| agent.id)
        .collect()
}

/// Kill every adult of the lowest owned house's household (preferring one with children) through
/// the ordinary death path, so rule 5 runs even when no natural death orphans a house in 48 days.
fn kill_owning_adults(engine: &mut SimEngine) -> Option<(HouseId, HouseholdId)> {
    let grown = adults(engine);
    let state = &engine.resources.colony.buildings.housing;
    let has_minor = |owner: &HouseholdId| {
        state
            .households
            .get(owner)
            .is_some_and(|h| h.members.iter().any(|m| !grown.contains(m)))
    };
    let (house, owner) = state
        .ownership
        .iter()
        .find(|(_, owner)| has_minor(owner))
        .or_else(|| state.ownership.iter().next())
        .map(|(house, owner)| (*house, *owner))?;
    let members = state.households.get(&owner)?.members.clone();
    let victims: Vec<_> = engine
        .world
        .query::<(&sim_core::components::Agent, &sim_core::components::Position)>()
        .iter()
        .filter(|(_, (agent, _))| members.contains(&agent.id) && grown.contains(&agent.id))
        .map(|(entity, (agent, pos))| (entity, agent.id, (pos.x, pos.y)))
        .collect();
    let tick = engine.current_tick();
    for (entity, id, pos) in victims {
        despawn_agent(
            &mut engine.world,
            &mut engine.resources,
            entity,
            id,
            pos,
            sim_core::causal::event::DeathReason::Combat,
            tick,
        );
    }
    Some((house, owner))
}

/// Adult-led households without a house while a vacant usable house of their settlement stands.
fn homeless_beside_vacancy(engine: &SimEngine) -> usize {
    let res = &engine.resources;
    let state = &res.colony.buildings.housing;
    let grown = adults(engine);
    let vacant: BTreeSet<_> = house_views(&engine.world, res)
        .into_iter()
        .filter(|row| {
            row.owner_household.is_none()
                && row.status == sim_core::colony::household::HouseStatus::Complete
        })
        .filter_map(|row| row.settlement)
        .collect();
    state
        .households
        .values()
        .filter(|h| !state.ownership.values().any(|owner| *owner == h.id))
        .filter(|h| h.members.iter().any(|m| grown.contains(m)))
        .filter(|h| {
            res.settlements
                .iter()
                .any(|(sid, s)| s.member_agents.contains(&h.head) && vacant.contains(sid))
        })
        .count()
}

/// Rule 4 after every tick, without the derived view: assigned beds inside a standing owned
/// house belong to its household or houseless sub-households.
fn owned_bed_violations(engine: &SimEngine) -> usize {
    let res = &engine.resources;
    let state = &res.colony.buildings.housing;
    let mut violations = 0;
    for (house, owner) in &state.ownership {
        // A physically lost house stops offering beds; its owner is released at the boundary.
        let Some(site) = res
            .colony
            .buildings
            .sites
            .get(&house.0)
            .filter(|site| site.status != BuildingStatus::Cancelled)
        else {
            continue;
        };
        let allowed = household_members(engine, *owner);
        violations += res
            .bed_of
            .iter()
            .filter(|(agent, (x, y))| {
                *x >= site.anchor.0
                    && *y >= site.anchor.1
                    && *x < site.anchor.0 + site.footprint.0
                    && *y < site.anchor.1 + site.footprint.1
                    && !allowed.contains(agent)
            })
            .count();
    }
    violations
}

#[test]
fn harness_house_identity_survives_upgrade_and_room_split() {
    let mut engine = scene();
    for (name, count) in [
        ("log", 100),
        ("stone", 20),
        ("clay", 60),
        ("mud_brick", 66),
        ("cut_stone", 70),
    ] {
        raw_stock(&mut engine, name, count);
    }
    let template = dwelling(true, false);
    let first = build(&mut engine, &template);
    let second = build(&mut engine, &template);
    after_housing_boundary(&mut engine);
    let before: BTreeMap<_, _> = [first, second]
        .into_iter()
        .map(|id| (id, view(&engine, id)))
        .collect();
    for (id, row) in &before {
        assert!(row.first_completed_tick.is_some(), "house {id} first date");
        assert!(
            !row.resident_ids.is_empty(),
            "house {id} has residents: {row:?}"
        );
        assert!(row.owner_household.is_some(), "house {id} owned: {row:?}");
        assert_eq!(row.room_ids.len(), 1, "house {id} has its interior room");
    }
    assert_eq!(residents_match_assignment(&engine), 0);
    assert_eq!(room_reference_errors(&engine), 0);
    command(&mut engine, Command::UpgradeBuilding { site: second });
    let target = engine.resources.colony.buildings.sites[&second]
        .template
        .clone();
    assert_ne!(target, template, "upgrade installs the next template");
    let mut saw_upgrading = false;
    let mut room_sets = BTreeSet::from([before[&second].room_ids.clone()]);
    while engine.current_tick() < spine_support::DEADLINE && !complete(&engine, second) {
        engine.tick();
        let row = view(&engine, second);
        room_sets.insert(row.room_ids.clone());
        if row.status == sim_core::colony::household::HouseStatus::Upgrading {
            saw_upgrading = true;
            assert_eq!(row.owner_household, before[&second].owner_household);
        }
        // Room detection splits/rejoins the interior while walls are replaced.
        assert_eq!(
            room_reference_errors(&engine),
            0,
            "tick {}",
            engine.current_tick()
        );
    }
    assert!(
        saw_upgrading && complete(&engine, second),
        "upgrade reached completion"
    );
    // The wall replacement really opened/split the interior: the house's room set changed.
    assert!(
        room_sets.len() >= 2,
        "the upgrade never changed the house's rooms: {room_sets:?}"
    );
    after_housing_boundary(&mut engine);
    let ids: BTreeSet<_> = house_views(&engine.world, &engine.resources)
        .iter()
        .map(|row| row.id.0)
        .collect();
    assert_eq!(ids, BTreeSet::from([first, second]), "same two root ids");
    let after = view(&engine, second);
    assert_eq!(after.template, target);
    assert_eq!(
        after.first_completed_tick,
        before[&second].first_completed_tick
    );
    assert_eq!(after.owner_household, before[&second].owner_household);
    assert_eq!(residents_match_assignment(&engine), 0);
    assert_eq!(room_reference_errors(&engine), 0);
    println!(
        "HOUSE1 identity: houses={ids:?} upgrade {template}->{target} residents={:?}/{:?}",
        view(&engine, first).resident_ids,
        after.resident_ids
    );
}

#[test]
fn harness_house_residents_match_reachable_beds() {
    let mut engine = scene();
    for (name, count) in [("log", 120), ("stone", 20), ("clay", 60), ("mud_brick", 66)] {
        raw_stock(&mut engine, name, count);
    }
    let multi = dwelling(false, true);
    let roomy = dwelling(true, false);
    let hut = build(&mut engine, &multi);
    let house = build(&mut engine, &roomy);
    after_housing_boundary(&mut engine);
    let hut_beds = engine.resources.rules.colony.expect("colony").hut_beds;
    let rows = house_views(&engine.world, &engine.resources);
    assert_eq!(rows.len(), 2);
    let mut occupied = 0;
    for row in &rows {
        // Independent recount of reachable physical slots inside the footprint.
        let site = &engine.resources.colony.buildings.sites[&row.id.0];
        let inside = |(x, y): (u32, u32)| {
            x >= site.anchor.0
                && y >= site.anchor.1
                && x < site.anchor.0 + site.footprint.0
                && y < site.anchor.1 + site.footprint.1
        };
        let formation =
            engine.resources.settlements[&row.settlement.expect("settlement")].formation_tile;
        let reach = |tile| engine.resources.nav.reachable(formation, tile);
        let bedrolls = engine
            .resources
            .colony
            .beds
            .keys()
            .filter(|tile| inside(**tile) && reach(**tile))
            .count() as u32;
        let huts = engine
            .resources
            .building_kinds
            .iter()
            .filter(|(tile, kind)| **kind == BuildingKind::Hut && inside(**tile) && reach(**tile))
            .count() as u32;
        assert_eq!(row.capacity, bedrolls + huts * hut_beds, "{row:?}");
        assert!(row.capacity > 0, "{row:?}");
        assert!(row.resident_ids.len() as u32 <= row.capacity, "{row:?}");
        // No double assignment beyond a tile's slots, no false bed, no false hearth.
        for bed in &row.bed_refs {
            let sleepers = engine
                .resources
                .bed_of
                .values()
                .filter(|t| **t == bed.tile)
                .count();
            assert!(sleepers as u32 <= bed.slots, "{bed:?}");
        }
        for tile in &row.hearth_refs {
            assert_eq!(
                engine.resources.building_kinds.get(tile),
                Some(&BuildingKind::Hearth)
            );
        }
        occupied += usize::from(!row.resident_ids.is_empty());
        let owner = row
            .owner_household
            .expect("both houses owned after the boundary");
        let allowed = household_members(&engine, owner);
        assert!(
            row.resident_ids.iter().all(|id| allowed.contains(id)),
            "{row:?} holds non-members of {owner:?}"
        );
        assert_eq!(
            row.primary_household.is_some(),
            !row.resident_ids.is_empty()
        );
        assert_eq!(row.household_groups.len(), row.resident_ids.len());
    }
    assert_ne!(
        view(&engine, hut).owner_household,
        view(&engine, house).owner_household,
        "two houses go to two households"
    );
    assert_eq!(
        view(&engine, hut).bed_refs.len(),
        1,
        "one multi-slot hut tile"
    );
    assert_eq!(view(&engine, hut).capacity, hut_beds);
    assert!(occupied >= 1, "at least one house is lived in");
    assert_eq!(residents_match_assignment(&engine), 0);
    assert_eq!(exclusivity_violations(&engine), 0);
}

#[test]
fn harness_house_legacy_dates_and_replay_are_exact() {
    let mut engine = scene();
    for (name, count) in [("log", 100), ("stone", 20), ("clay", 60), ("mud_brick", 66)] {
        raw_stock(&mut engine, name, count);
    }
    let template = dwelling(true, false);
    let house = build(&mut engine, &template);
    after_housing_boundary(&mut engine);
    let completed = engine
        .resources
        .chronicle
        .entries()
        .iter()
        .filter(|e| e.kind == ChronicleKind::BuildingComplete)
        .map(|e| e.tick)
        .min()
        .expect("completion entry");
    assert_eq!(view(&engine, house).first_completed_tick, Some(completed));
    // A real format25 save (the predecessor layout, no housing tail) migrates by rule 1.
    let legacy = sim_systems::save::save_format25(&mut engine).expect("format25 save");
    assert_eq!(&legacy[8..12], &25u32.to_le_bytes());
    let entries = engine.resources.chronicle.len();
    let migrated = sim_systems::save::load(&legacy).expect("format25 loads");
    assert_eq!(
        migrated.resources.chronicle.len(),
        entries,
        "migration emits no events"
    );
    let state = &migrated.resources.colony.buildings.housing;
    assert!(state.founded);
    assert_eq!(
        state.houses[&HouseId(house)].first_completed_tick,
        Some(completed),
        "known history recovered from the completion chronicle"
    );
    assert!(state.validate().is_ok());
    // Rule 1: the house belongs to the household holding most of its assigned beds (ties: larger
    // household, then lower id); every resident is that household or its sub-household.
    let site = &migrated.resources.colony.buildings.sites[&house];
    let inside = |(x, y): (u32, u32)| {
        x >= site.anchor.0
            && y >= site.anchor.1
            && x < site.anchor.0 + site.footprint.0
            && y < site.anchor.1 + site.footprint.1
    };
    let mut counts: BTreeMap<HouseholdId, usize> = BTreeMap::new();
    for (agent, bed) in &migrated.resources.bed_of {
        if inside(*bed) {
            *counts
                .entry(
                    state
                        .household_of(*agent)
                        .expect("resident has a household"),
                )
                .or_default() += 1;
        }
    }
    let expected = counts
        .iter()
        .max_by_key(|(id, count)| {
            (
                **count,
                state.households[*id].members.len(),
                std::cmp::Reverse(**id),
            )
        })
        .map(|(id, _)| *id);
    assert!(expected.is_some(), "the migrated house has residents");
    assert_eq!(state.ownership.get(&HouseId(house)).copied(), expected);
    // Lost history: the same migration over a chronicle without the evidence yields None.
    let mut lost = sim_systems::save::load(&legacy).expect("format25 loads");
    lost.resources.chronicle = Default::default();
    sim_systems::runtime::housing::migrate(&lost.world, &mut lost.resources);
    assert_eq!(
        lost.resources.colony.buildings.housing.houses[&HouseId(house)].first_completed_tick,
        None
    );
    // Current format: exact round trip, then save/load continuation equals an uninterrupted run.
    let saved = sim_systems::save::save(&mut engine).expect("format26 save");
    let mut loaded = sim_systems::save::load(&saved).expect("format26 load");
    assert_eq!(
        sim_systems::save::save(&mut loaded).expect("roundtrip"),
        saved
    );
    for _ in 0..3 * 1_440 {
        engine.tick();
        loaded.tick();
    }
    assert_eq!(
        sim_test::probe::colony::colony_fingerprint(&engine),
        sim_test::probe::colony::colony_fingerprint(&loaded)
    );
    assert_eq!(
        house_views(&engine.world, &engine.resources),
        house_views(&loaded.world, &loaded.resources)
    );
    assert_eq!(
        engine.resources.colony.buildings.housing,
        loaded.resources.colony.buildings.housing
    );
    // Command replay: two runs of the same log are identical, housing included.
    let mut twin = scene();
    for (name, count) in [("log", 100), ("stone", 20), ("clay", 60), ("mud_brick", 66)] {
        raw_stock(&mut twin, name, count);
    }
    let again = build(&mut twin, &template);
    assert_eq!(again, house);
    after_housing_boundary(&mut twin);
    for _ in 0..3 * 1_440 {
        twin.tick();
    }
    assert_eq!(twin.current_tick(), engine.current_tick());
    assert_eq!(
        sim_test::probe::colony::colony_fingerprint(&engine),
        sim_test::probe::colony::colony_fingerprint(&twin)
    );
    assert!(engine.resources.colony.buildings.housing.founded);
    assert_eq!(
        engine.resources.colony.buildings.housing,
        twin.resources.colony.buildings.housing
    );
    assert_eq!(
        house_views(&engine.world, &engine.resources),
        house_views(&twin.world, &twin.resources)
    );
}

#[test]
fn harness_household_ownership_exclusive_and_inherited() {
    let day = 1_440u64;
    let days = 48u64;
    let mut eligible_couples = 0usize;
    let mut couples_claimed = 0usize;
    let mut orphaned_houses = 0usize;
    for seed in 0..4 {
        let mut engine = sim_test::colony_engine(seed);
        command(
            &mut engine,
            Command::SetSteward {
                on: true,
                scope: sim_core::colony::steward::StewardScope::All,
            },
        );
        let mut violations = 0;
        let mut late_handovers = 0;
        let mut missed_vacancies = 0;
        let mut checks = 0u64;
        // Neolocal couples (households formed with a parent) and when they first owned a house.
        let mut couples: BTreeMap<HouseholdId, u64> = BTreeMap::new();
        let mut claimed: BTreeMap<HouseholdId, u64> = BTreeMap::new();
        // Owned houses whose owner has no living adult, and since when.
        let mut adultless: BTreeMap<HouseId, (HouseholdId, u64)> = BTreeMap::new();
        let mut orphaned_here = 0usize;
        let mut seen_entries = 0;
        while engine.current_tick() < days * day {
            engine.tick();
            let tick = engine.current_tick();
            let entries = engine.resources.chronicle.entries();
            for entry in &entries[seen_entries..] {
                match entry.kind {
                    ChronicleKind::HouseholdFormed if entry.value >= 0 => {
                        couples.insert(HouseholdId(entry.cause), entry.tick);
                    }
                    ChronicleKind::HouseOwnershipChanged => {
                        if let (_, Some(to), _) = unpack_ownership_change(entry.value) {
                            claimed.entry(to).or_insert(entry.tick);
                        }
                    }
                    _ => {}
                }
            }
            seen_entries = entries.len();
            if seed == 0 && tick == 20 * day {
                let (house, owner) =
                    kill_owning_adults(&mut engine).expect("seed 0 owns a house by day 20");
                adultless.insert(house, (owner, tick));
                orphaned_here += 1;
            }
            // Rule 4 holds after every tick, not only at the boundary.
            violations += owned_bed_violations(&engine);
            // Rule 5: a house whose owner lost its last living adult is inherited or vacant
            // within one day.
            let grown = adults(&engine);
            let state = &engine.resources.colony.buildings.housing;
            for (house, owner) in &state.ownership {
                let led = state
                    .households
                    .get(owner)
                    .is_some_and(|h| h.members.iter().any(|m| grown.contains(m)));
                if !led && !adultless.contains_key(house) {
                    adultless.insert(*house, (*owner, tick));
                    orphaned_here += 1;
                }
            }
            adultless.retain(|house, entry| {
                let (owner, since) = *entry;
                let still = state.ownership.get(house) == Some(&owner)
                    && !state
                        .households
                        .get(&owner)
                        .is_some_and(|h| h.members.iter().any(|m| grown.contains(m)));
                if still && tick > since + day {
                    late_handovers += 1;
                    return false;
                }
                still
            });
            if minute(&engine) == 1201 {
                checks += 1;
                missed_vacancies += homeless_beside_vacancy(&engine);
                assert_eq!(residents_match_assignment(&engine), 0, "seed {seed}");
                assert_eq!(dangling_owners(&engine), 0, "seed {seed}");
                assert!(engine.resources.colony.buildings.housing.validate().is_ok());
            }
        }
        let end = engine.current_tick();
        let eligible: Vec<_> = couples
            .iter()
            .filter(|(_, tick)| **tick + 2 * day <= end)
            .collect();
        let in_time = eligible
            .iter()
            .filter(|(id, tick)| claimed.get(*id).is_some_and(|at| *at <= **tick + 2 * day))
            .count();
        let owned = engine.resources.colony.buildings.housing.ownership.len();
        println!(
            "HOUSE1 seed {seed}: checks={checks} violations={violations} late_handovers={late_handovers} \
             orphaned_houses={orphaned_here} missed_vacancies={missed_vacancies} couples={} \
             eligible={} claimed_in_2d={in_time} owned={owned}",
            couples.len(),
            eligible.len()
        );
        assert!(
            checks + 1 >= days,
            "seed {seed}: a boundary check every day"
        );
        assert_eq!(
            violations, 0,
            "seed {seed}: owned beds are exclusive after every tick"
        );
        assert_eq!(
            late_handovers, 0,
            "seed {seed}: orphaned houses hand over within a day"
        );
        assert_eq!(
            missed_vacancies, 0,
            "seed {seed}: no adult-led houseless household beside a vacant house"
        );
        assert!(owned > 0, "seed {seed}: some house is owned");
        eligible_couples += eligible.len();
        couples_claimed += in_time;
        orphaned_houses += orphaned_here;
    }
    println!(
        "HOUSE1 seeds 0-3: eligible_couples={eligible_couples} claimed_in_2d={couples_claimed} \
         orphaned_houses={orphaned_houses}"
    );
    assert!(
        eligible_couples >= 1,
        "some neolocal couple formed with two days left"
    );
    assert!(
        couples_claimed >= 1,
        "a neolocal couple claimed a house within 2 days"
    );
    assert!(
        orphaned_houses >= 1,
        "a household lost its last adult while owning a house"
    );
}
