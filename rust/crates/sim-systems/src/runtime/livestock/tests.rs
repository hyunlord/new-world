//! Physical enclosure, day-clock, immutable event and natural-removal regressions.
use super::*;
use sim_core::colony::building_site::{
    BuildCause, BuildingPiece, BuildingSite, BuildingStatus, PiecePhase,
};
use sim_core::colony::content::{installed_or_bundled, AnimalKind, CompletionCategory};
use sim_core::colony::content_state::CompletedIdentity;
use sim_core::colony::livestock::{KnowledgeWitness, LivestockOrderCause};
use sim_core::colony::{
    Command, CommandRecord, CommandResult, ItemKind, ItemSpec, Job, JobKind, JobStage, ReserveKey,
};
use sim_core::components::{Agent, AgentState, Position, Sex};

/// Physical completed template shared by sibling job tests; no production bypass.
pub(crate) fn fixture(rotation: u8) -> (World, SimResources, u32, AnimalKind) {
    let catalog = installed_or_bundled();
    let template = catalog.building_templates().get("pen").expect("pen");
    let mut res = SimResources::new(12, 12, sim_core::MaterialRegistry::new());
    res.rules = sim_core::rules::WorldRules::colony(0);
    res.rules.clock_offset_ticks = 0;
    let plan = template
        .expand(catalog, None, rotation)
        .expect("pen expansion");
    let mut pieces = Vec::new();
    for (index, planned) in plan.iter().enumerate() {
        let tile = (2 + planned.offset.0, 2 + planned.offset.1);
        let piece = BuildingPiece {
            id: index as u32,
            kind: planned.kind,
            tile,
            size: planned.size,
            phase: PiecePhase::Complete,
        };
        let def = piece.kind.definition();
        let grid_index = res.tile_grid.idx(tile.0, tile.1);
        res.tile_grid.wall_material[grid_index] = Some(sim_core::colony::materials::OAK);
        res.tile_grid
            .set_door(grid_index, def.completion == CompletionCategory::Door);
        res.content_state.completed.insert(
            (
                tile,
                crate::runtime::jobs::space::completion_layer(piece.kind),
            ),
            CompletedIdentity {
                blueprint: piece.kind,
                style_id: def.style_id.clone(),
            },
        );
        pieces.push(piece);
    }
    res.colony.buildings.next_piece_id = pieces.len() as u32;
    res.colony.buildings.next_site_id = 2;
    res.colony.buildings.sites.insert(
        1,
        BuildingSite {
            id: 1,
            template: template.id.clone(),
            station: None,
            anchor: (2, 2),
            rotation,
            footprint: template.rotated_footprint(rotation).expect("footprint"),
            pieces,
            cause: BuildCause::Command,
            source_record: 0,
            placed_tick: 0,
            status: BuildingStatus::Complete,
            completion_entry: None,
        },
    );
    crate::runtime::nav::ensure(&mut res);
    (
        World::new(),
        res,
        1,
        catalog
            .animal_by_name("wild_goat")
            .expect("tameable species"),
    )
}

/// Retained captured provenance for focused lifecycle tests, without a job runner.
pub(crate) fn add_captured(
    res: &mut SimResources,
    site: u32,
    species: AnimalKind,
    female: bool,
    age: u32,
) -> u32 {
    let policy = species.definition().tameable.as_ref().expect("tameable");
    let (id, next) = res.livestock.checked_next_id().expect("id");
    let wild = id;
    res.colony.wildlife.next_id = res.colony.wildlife.next_id.max(wild + 1);
    let source_record = res.colony.command_log.len() as u32;
    res.colony.command_log.push(CommandRecord {
        tick: res.current_tick,
        command: Command::DesignateCapture { animal: wild },
        result: CommandResult::ok(1),
    });
    let knowledge = installed_or_bundled()
        .knowledge_defs()
        .id_of(&policy.capture_knowledge)
        .expect("capture knowledge");
    let creation_entry = next_event_index(res).expect("event");
    let day = day(res);
    res.livestock.next_id = next;
    res.livestock.animals.insert(
        id,
        Livestock {
            id,
            species,
            age_days: age,
            last_age_tick: res.current_tick,
            sex: Sex { female },
            hunger_days: 0,
            pen_site: site,
            milk_ready: false,
            last_milk_day: day,
            last_bred_day: day,
            created_day: day,
            creation_entry,
        },
    );
    if res.livestock.first_capture.is_none() {
        res.livestock.first_capture = Some(id);
    }
    let entry = ChronicleEntry::new(res.current_tick, ChronicleKind::LivestockCaptured).subject(7);
    emit_event(
        res,
        entry,
        LivestockEventWitness {
            livestock: id,
            species,
            pen_site: site,
            event: LivestockEvent::Captured {
                wild,
                attempt: 1,
                knowledge: KnowledgeWitness {
                    knowledge,
                    proficiency: 1000,
                    required: installed_or_bundled().knowledge_defs().threshold(knowledge),
                },
                tool: policy.capture_tool.map(|kind| ItemSpec::raw(kind, None)),
                origin: LivestockOrderCause {
                    source_record,
                    steward_rule: None,
                },
            },
        },
    );
    id
}

#[test]
fn rotated_pen_requires_actual_boundary_and_only_nonbarrier_door_is_gate() {
    for rotation in 0..4 {
        let (_, mut res, site, _) = fixture(rotation);
        let pen = pen_layout(&res, site).expect("physical pen");
        assert_eq!(
            (pen.capacity, pen.interior.len(), pen.gates.len()),
            (6, 9, 1)
        );
        assert!(pen
            .interior
            .windows(2)
            .all(|w| (w[0].1, w[0].0) < (w[1].1, w[1].0)));
        // All fence tiles are passable to people, but none masquerades as a gate.
        for piece in &res.colony.buildings.sites[&site].pieces {
            assert!(res.nav.passable(piece.tile));
            if piece.kind.definition().blocks_livestock {
                assert!(!pen.gates.contains(&piece.tile));
            }
        }
        let gate = pen.gates[0];
        let i = res.tile_grid.idx(gate.0, gate.1);
        res.tile_grid.set_door(i, false);
        assert_eq!(pen_layout(&res, site), Err(PenUnavailable::BrokenBoundary));
        res.tile_grid.set_door(i, true);
        res.tile_grid.wall_material[i] = None;
        assert_eq!(pen_layout(&res, site), Err(PenUnavailable::BrokenBoundary));
    }
}

#[test]
fn intact_phase_without_a_fence_hole_or_gate_does_not_prove_enclosure() {
    let (_, mut res, site, _) = fixture(0);
    let piece = res.colony.buildings.sites[&site]
        .pieces
        .iter()
        .find(|p| p.kind.definition().blocks_livestock)
        .expect("fence")
        .clone();
    // Removing a constituent from provenance can still leave is_intact true.
    res.colony
        .buildings
        .sites
        .get_mut(&site)
        .unwrap()
        .pieces
        .retain(|p| p.id != piece.id);
    assert!(crate::runtime::building_templates::is_intact(
        &res,
        &res.colony.buildings.sites[&site]
    ));
    assert_eq!(pen_layout(&res, site), Err(PenUnavailable::BrokenBoundary));
    let (_, mut res, site, _) = fixture(0);
    let gate_id = res.colony.buildings.sites[&site]
        .pieces
        .iter()
        .find(|p| !p.kind.definition().blocks_livestock)
        .unwrap()
        .id;
    let fence = res.colony.buildings.sites[&site]
        .pieces
        .iter()
        .find(|p| p.kind.definition().blocks_livestock)
        .unwrap()
        .kind;
    let gate = res
        .colony
        .buildings
        .sites
        .get_mut(&site)
        .unwrap()
        .pieces
        .iter_mut()
        .find(|p| p.id == gate_id)
        .unwrap();
    gate.kind = fence;
    let tile = gate.tile;
    res.content_state
        .completed
        .get_mut(&(tile, crate::runtime::jobs::space::completion_layer(fence)))
        .unwrap()
        .blueprint = fence;
    assert_eq!(pen_layout(&res, site), Err(PenUnavailable::NoGate));
}

#[test]
fn forage_area_excludes_physical_obstacles_and_own_capture_reservation_is_not_a_head() {
    let (_, mut res, site, species) = fixture(0);
    let pen = pen_layout(&res, site).unwrap();
    let tile = pen.interior[0];
    res.colony
        .trees
        .insert(tile, sim_core::colony::Tree { logs: 1 });
    assert_eq!(pen_layout(&res, site).unwrap().interior.len(), 8);
    res.colony.trees.remove(&tile);
    res.colony
        .reservations
        .insert(ReserveKey::PenCapture(site), 7);
    assert!(!has_capture_slot(&res, &pen, None));
    assert!(has_capture_slot(&res, &pen, Some(7)));
    assert!(!has_capture_slot(&res, &pen, Some(8)));
    for _ in 0..6 {
        add_captured(&mut res, site, species, false, 0);
    }
    assert!(!has_capture_slot(&res, &pen, Some(7)));
}

#[test]
fn offset_daily_feeding_is_once_only_stable_and_three_heads_use_nine_cells() {
    let (mut world, mut res, site, species) = fixture(0);
    res.ticks_per_day = 240;
    res.rules.clock_offset_ticks = 120;
    for _ in 0..4 {
        add_captured(&mut res, site, species, false, 0);
    }
    advance(&mut world, &mut res);
    res.current_tick = 60;
    advance(&mut world, &mut res);
    assert!(res.livestock.grass.is_empty());
    res.current_tick = 120; // display day1, not raw tick/day1
    advance(&mut world, &mut res);
    assert_eq!(res.livestock.grass.len(), 9);
    assert_eq!(
        res.livestock
            .animals
            .values()
            .map(|a| a.hunger_days)
            .collect::<Vec<_>>(),
        [0, 0, 0, 1]
    );
    assert!(res.livestock.grass.values().all(|g| g.next_regrow_day == 2));
    let committed = res.livestock.clone();
    advance(&mut world, &mut res);
    assert_eq!(res.livestock, committed);
    res.current_tick = 360;
    advance(&mut world, &mut res);
    assert_eq!(res.livestock.animals[&4].hunger_days, 2);
    res.current_tick = 600;
    advance(&mut world, &mut res);
    assert!(!res.livestock.animals.contains_key(&4));
    assert_eq!(res.livestock.animals.len(), 3);
    assert!(matches!(
        res.livestock.witnesses.values().last().unwrap().event,
        LivestockEvent::Died {
            reason: LivestockDeathReason::Starvation
        }
    ));
    assert!(validate_state_at_boundary(&world, &res, res.current_tick).is_ok());
}

#[test]
fn natural_pair_breeds_at_creation_interval_and_newborn_does_not_eat_retroactively() {
    let (mut world, mut res, site, species) = fixture(0);
    res.ticks_per_day = 240;
    let mother = add_captured(&mut res, site, species, true, 29);
    let father = add_captured(&mut res, site, species, false, 29);
    advance(&mut world, &mut res);
    for day in 1..20 {
        res.current_tick = day * 240;
        advance(&mut world, &mut res);
    }
    assert_eq!(res.livestock.animals.len(), 2);
    res.current_tick = 20 * 240;
    advance(&mut world, &mut res);
    let kid = res.livestock.first_kid_id.expect("committed birth");
    assert_eq!(res.livestock.animals.len(), 3);
    assert_eq!(res.livestock.animals[&kid].age_days, 0);
    assert_eq!(res.livestock.animals[&kid].last_age_tick, res.current_tick);
    assert_eq!(res.livestock.animals[&mother].last_bred_day, 20);
    assert_eq!(res.livestock.animals[&kid].hunger_days, 0);
    assert!(!res.livestock.animals[&kid].milk_ready);
    assert_eq!(
        res.livestock
            .grass
            .values()
            .filter(|g| g.next_regrow_day == 21)
            .count(),
        6
    );
    let index = res.livestock.animals[&kid].creation_entry;
    assert_eq!(
        res.chronicle.cause_of(index),
        Some(CauseRef::Livestock { witness: index })
    );
    assert!(
        matches!(res.livestock.witnesses[&index].event,LivestockEvent::Born { mother:m,father:f } if m==mother && f==father)
    );
    assert!(validate_state_at_boundary(&world, &res, res.current_tick).is_ok());
    for day in 21..=40 {
        res.current_tick = day * 240;
        advance(&mut world, &mut res);
    }
    // Hard capacity6 does not manufacture the forage needed for a fourth head.
    assert_eq!(res.livestock.animals.len(), 3);
}

#[test]
fn broken_pen_stops_feeding_products_and_breeding_but_remains_valid_state() {
    let (mut world, mut res, site, species) = fixture(0);
    res.ticks_per_day = 240;
    let animal = add_captured(&mut res, site, species, true, 29);
    advance(&mut world, &mut res);
    let gate = pen_layout(&res, site).unwrap().gates[0];
    let i = res.tile_grid.idx(gate.0, gate.1);
    res.tile_grid.wall_material[i] = None;
    res.current_tick = 240;
    advance(&mut world, &mut res);
    assert_eq!(res.livestock.animals[&animal].hunger_days, 1);
    assert!(!res.livestock.animals[&animal].milk_ready);
    assert!(res.livestock.grass.is_empty());
    assert!(validate_state_at_boundary(&world, &res, res.current_tick).is_ok());
}

#[test]
fn natural_death_cancels_completed_waiting_job_and_preserves_typed_history() {
    let (mut world, mut res, site, species) = fixture(0);
    res.ticks_per_day = 240;
    let animal = add_captured(&mut res, site, species, true, 29);
    let gate = pen_layout(&res, site).unwrap().gates[0];
    let worker = world.spawn((
        Agent { id: 7 },
        Position::new(gate.0, gate.1),
        AgentState::Idle,
    ));
    let job = Job {
        kind: JobKind::Milk { animal, at: gate },
        stage: JobStage::Working { ticks_left: 0 },
        taken_at: 0,
        amount: 0,
        item: ItemKind::role("tree_output"),
        material: None,
        fetch: None,
        best_distance: 0,
        progress_tick: 0,
    };
    world.insert_one(worker, job).unwrap();
    res.colony
        .reservations
        .insert(ReserveKey::Livestock(animal), 7);
    advance(&mut world, &mut res);
    res.livestock.animals.get_mut(&animal).unwrap().age_days =
        species.definition().tameable.as_ref().unwrap().max_age_days;
    res.current_tick = 240;
    advance(&mut world, &mut res);
    assert!(world.get::<&Job>(worker).is_err());
    assert!(!res
        .colony
        .reservations
        .contains_key(&ReserveKey::Livestock(animal)));
    assert!(!res.livestock.animals.contains_key(&animal));
    assert_eq!(res.colony.tallies.jobs_done, 0);
    assert_eq!(res.livestock.first_capture, Some(animal));
    assert_eq!(res.livestock.witnesses.len(), 2);
    assert!(validate_state_at_boundary(&world, &res, res.current_tick).is_ok());
    let removed = res.livestock.witnesses.pop_last().unwrap();
    assert!(validate_state_at_boundary(&world, &res, res.current_tick).is_err());
    res.livestock.witnesses.insert(removed.0, removed.1);
    assert!(validate_state_at_boundary(&world, &res, res.current_tick).is_ok());
}

#[test]
fn insufficient_daily_meal_consumes_no_partial_cells_or_shortens_saved_regrowth() {
    let (mut world, mut res, site, species) = fixture(0);
    res.ticks_per_day = 240;
    for _ in 0..3 {
        add_captured(&mut res, site, species, false, 0);
    }
    let cells = pen_layout(&res, site).unwrap().interior;
    res.livestock
        .grass
        .insert(cells[0], GrassState { next_regrow_day: 2 });
    advance(&mut world, &mut res);
    res.current_tick = 240;
    advance(&mut world, &mut res);
    assert_eq!(res.livestock.grass[&cells[0]].next_regrow_day, 2);
    assert_eq!(res.livestock.grass.len(), 7); // reserved cell + two complete three-cell meals
    assert_eq!(res.livestock.animals[&3].hunger_days, 1);
    assert!(!res.livestock.grass.contains_key(&cells[7]));
    assert!(!res.livestock.grass.contains_key(&cells[8]));
    res.current_tick = 480;
    advance(&mut world, &mut res);
    assert_eq!(res.livestock.animals[&3].hunger_days, 0);
    assert_eq!(res.livestock.grass.len(), 9);
}

#[test]
fn exhausted_birth_allocator_preserves_parents_and_does_not_emit_a_birth() {
    let (mut world, mut res, site, species) = fixture(0);
    res.ticks_per_day = 240;
    let mother = add_captured(&mut res, site, species, true, 29);
    add_captured(&mut res, site, species, false, 29);
    advance(&mut world, &mut res);
    res.livestock.next_id = u32::MAX;
    for at in 1..=20 {
        res.current_tick = at * 240;
        advance(&mut world, &mut res);
    }
    assert_eq!(res.livestock.animals.len(), 2);
    assert_eq!(res.livestock.animals[&mother].last_bred_day, 0);
    assert_eq!(res.livestock.witnesses.len(), 2);
    assert_eq!(res.livestock.first_kid_id, None);
    assert_eq!(res.livestock.next_id, u32::MAX);
    assert!(validate_state_at_boundary(&world, &res, res.current_tick).is_ok());
}

#[test]
fn immutable_history_rejects_rejected_sources_wrong_wrapper_and_retargeted_first_fact() {
    let (world, mut res, site, species) = fixture(0);
    let animal = add_captured(&mut res, site, species, false, 0);
    assert!(validate_state_at_boundary(&world, &res, 0).is_ok());
    res.colony.command_log[0].result =
        CommandResult::rejected(sim_core::colony::RejectReason::Nothing);
    assert!(validate_state_at_boundary(&world, &res, 0).is_err());
    res.colony.command_log[0].result = CommandResult::ok(1);
    res.colony.command_log[0].command = Command::Steward {
        rule: 6,
        command: Box::new(Command::DesignateCapture { animal: 1 }),
    };
    assert!(validate_state_at_boundary(&world, &res, 0).is_err());
    let index = res.livestock.animals[&animal].creation_entry;
    if let LivestockEvent::Captured { origin, .. } =
        &mut res.livestock.witnesses.get_mut(&index).unwrap().event
    {
        origin.steward_rule = Some(6);
    }
    assert!(validate_state_at_boundary(&world, &res, 0).is_ok());
    res.livestock.animals.get_mut(&animal).unwrap().age_days = u32::MAX;
    assert!(validate_state_at_boundary(&world, &res, 0).is_err());
    res.livestock.animals.get_mut(&animal).unwrap().age_days = 0;
    res.livestock.first_capture = Some(animal + 1);
    assert!(validate_state_at_boundary(&world, &res, 0).is_err());
}

#[test]
fn capture_retry_clock_rejects_corruption_for_live_and_recently_removed_sources() {
    use sim_core::colony::livestock::CaptureAttempt;
    use sim_core::colony::wildlife::{Animal, AnimalState};

    let (world, mut res, _, species) = fixture(0);
    res.current_tick = 3 * res.ticks_per_day;
    let today = day(&res);
    let retry = species
        .definition()
        .tameable
        .as_ref()
        .unwrap()
        .capture_retry_days;
    res.colony.wildlife.next_id = 2;
    res.colony.wildlife.animals.insert(
        1,
        Animal {
            id: 1,
            kind: species,
            hp: 30,
            state: AnimalState::Wander,
            pos: (1, 1),
            pack: None,
            born_tick: 0,
            path: Vec::new(),
            goal: None,
            home: None,
        },
    );
    res.livestock.capture_attempts.insert(
        1,
        CaptureAttempt {
            completed_attempts: 1,
            not_before_day: today + retry,
        },
    );
    assert!(validate_state_at_boundary(&world, &res, 0).is_ok());
    res.livestock
        .capture_attempts
        .get_mut(&1)
        .unwrap()
        .not_before_day = today.checked_add(retry).unwrap().checked_add(1).unwrap();
    assert!(validate_state_at_boundary(&world, &res, 0).is_err());
    res.livestock
        .capture_attempts
        .get_mut(&1)
        .unwrap()
        .not_before_day = u32::MAX;
    assert!(validate_state_at_boundary(&world, &res, 0).is_err());

    res.colony.wildlife.animals.remove(&1);
    let maximum = installed_or_bundled()
        .animals()
        .filter_map(|definition| definition.tameable.as_ref())
        .map(|policy| policy.capture_retry_days)
        .max()
        .unwrap();
    res.livestock
        .capture_attempts
        .get_mut(&1)
        .unwrap()
        .not_before_day = today + maximum;
    assert!(validate_state_at_boundary(&world, &res, 0).is_ok());
    res.livestock
        .capture_attempts
        .get_mut(&1)
        .unwrap()
        .not_before_day = today.checked_add(maximum).unwrap().checked_add(1).unwrap();
    assert!(validate_state_at_boundary(&world, &res, 0).is_err());
    res.livestock
        .capture_attempts
        .get_mut(&1)
        .unwrap()
        .not_before_day = u32::MAX;
    assert!(validate_state_at_boundary(&world, &res, 0).is_err());
}
