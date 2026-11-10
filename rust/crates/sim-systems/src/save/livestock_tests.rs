//! Livestock save continuity, width, historical slots and published20 regressions.
use super::*;
use sim_core::colony::knowledge::Knowledge;
use sim_core::colony::livestock::{CaptureAttempt, GrassState, LivestockOrderCause};
use sim_core::colony::{SkillKind, Skills};

fn bootstrap() {
    sim_core::colony::content::bootstrap(sim_core::colony::content::BootSelection::Bundled)
        .expect("bundled catalog");
}

fn recurring_engine(female: bool) -> (sim_engine::SimEngine, hecs::Entity, u32, (u32, u32)) {
    use crate::runtime::livestock::tests::{add_captured, fixture};
    use sim_core::colony::building_site::BuildingStatus;
    use sim_core::colony::knowledge::Source;
    use sim_core::colony::{Command, CommandRecord, CommandResult};
    use sim_core::components::{
        Agent, AgentState, BodyHealth, Hunger, Inventory, Lineage, Memory, Position, Sleep, Social,
        Temperament, Thirst,
    };

    bootstrap();
    let (mut world, mut resources, site, _) = fixture(0);
    resources.ticks_per_day = 240;
    resources.rules.clock_offset_ticks = 120;
    resources.colony.command_log.push(CommandRecord {
        tick: 0,
        command: Command::BuildTemplate {
            id: "pen".into(),
            anchor: (2, 2),
            rotation: 0,
            station: None,
        },
        result: CommandResult::ok(1),
    });
    // The shared fixture supplies physical pieces; the real lifecycle records completion.
    resources
        .colony
        .buildings
        .sites
        .get_mut(&site)
        .expect("pen")
        .status = BuildingStatus::Building;
    crate::runtime::building_templates::advance(&mut world, &mut resources);
    assert!(resources.colony.buildings.sites[&site]
        .completion_entry
        .is_some());
    let species = sim_core::colony::content::installed_or_bundled()
        .animal_by_name("wild_sheep")
        .expect("sheep");
    let policy = species.definition().tameable.as_ref().expect("policy");
    let animal = add_captured(
        &mut resources,
        site,
        species,
        female,
        policy.kid_age_days - 1,
    );
    let gate = crate::runtime::livestock::pen_layout(&resources, site)
        .expect("pen")
        .gates[0];
    let mut knowledge = Knowledge::default();
    for product in policy
        .products
        .milk
        .iter()
        .chain(policy.products.wool.iter())
    {
        knowledge.set(
            sim_core::colony::knowledge::defs()
                .id_of(&product.requires_knowledge)
                .expect("knowledge"),
            1000,
            Source::Seeded,
            0,
        );
    }
    let worker = world.spawn((
        Agent { id: 7 },
        Position {
            x: gate.0,
            y: gate.1,
        },
        AgentState::Idle,
        knowledge,
        crate::runtime::agent::movement::MovementRng::new(7),
        Hunger::new(0.0, 0.05),
        Thirst::new(0.0, 0.08),
        Sleep::new(0.0, 0.03),
        Social::new(0.0, 0.04),
        Memory::new(),
        BodyHealth::new(),
        Inventory::default(),
        Lineage { key: 7, births: 0 },
        Temperament::from_axes([500; 4]),
    ));
    resources.current_tick = u64::from(policy.products.wool.as_ref().expect("wool").every_days)
        * resources.ticks_per_day
        - resources.rules.clock_offset_ticks;
    let tick = resources.current_tick;
    let mut engine = sim_engine::SimEngine::new(1, 1, sim_core::MaterialRegistry::new());
    engine
        .install_saved(world, resources, tick, &[])
        .expect("fixture clock");
    crate::register_world_runtime_systems(&mut engine, sim_engine::registration::WorldKind::Colony);
    (engine, worker, animal, gate)
}

fn start_recurring(engine: &mut sim_engine::SimEngine, worker: hecs::Entity, gate: (u32, u32)) {
    use crate::runtime::jobs::livestock::{pick_milk, start, Offer};
    let practice = engine
        .world
        .get::<&Knowledge>(worker)
        .expect("knowledge")
        .practice();
    let offer = pick_milk(
        &engine.resources,
        practice,
        engine.resources.nav.region_of(gate),
        gate,
        u32::MAX,
    )
    .expect("ready recurring product");
    assert!(start(
        &mut engine.world,
        &mut engine.resources,
        worker,
        Offer::Milk(offer)
    ));
}

fn finish_recurring(engine: &mut sim_engine::SimEngine, worker: hecs::Entity, gate: (u32, u32)) {
    use sim_core::colony::{Job, JobStage};
    let mut job = *engine.world.get::<&Job>(worker).expect("job");
    job.stage = JobStage::Working { ticks_left: 0 };
    engine
        .world
        .insert_one(worker, job)
        .expect("completed work duration");
    crate::runtime::jobs::livestock::step(
        &mut engine.world,
        &mut engine.resources,
        7,
        worker,
        job,
        gate,
    );
    assert!(engine.world.get::<&Job>(worker).is_err());
}

#[test]
fn recurring_wool_full_save_restores_omitted_cooldown_before_continuation() {
    let (mut engine, worker, animal, gate) = recurring_engine(false);
    let wool = engine.resources.livestock.animals[&animal]
        .species
        .definition()
        .tameable
        .as_ref()
        .expect("policy")
        .products
        .wool
        .as_ref()
        .expect("wool")
        .item;
    start_recurring(&mut engine, worker, gate);
    finish_recurring(&mut engine, worker, gate);
    let cooldown = engine.resources.livestock.wool_last_day[&animal];
    assert!(cooldown > 0);
    assert_eq!(engine.resources.colony.item_ledger.created_of(wool), 1);
    let bytes = super::super::save(&mut engine).expect("full save with wool witness");
    let envelope: super::super::Envelope = super::super::decode(&bytes[12..]).expect("envelope");
    let body: super::super::Body = super::super::decode(&envelope.payload).expect("saved body");
    assert!(
        body.livestock.wool_last_day.is_empty(),
        "cache is omitted from actual save"
    );
    assert!(!body.livestock.witnesses.is_empty());
    let mut loaded = super::super::load(&bytes).expect("official full load");
    assert_eq!(loaded.resources.livestock, engine.resources.livestock);
    assert_eq!(
        super::super::save(&mut loaded).expect("immediate roundtrip"),
        bytes
    );
    let practice = loaded
        .world
        .get::<&Knowledge>(worker)
        .expect("restored knowledge")
        .practice();
    for _ in 0..60 {
        assert!(
            crate::runtime::jobs::livestock::pick_milk(
                &loaded.resources,
                practice,
                loaded.resources.nav.region_of(gate),
                gate,
                u32::MAX,
            )
            .is_none(),
            "restored wool cannot repeat before its cooldown"
        );
        engine.tick();
        loaded.tick();
        assert_eq!(loaded.resources.colony.item_ledger.created_of(wool), 1);
        assert_eq!(loaded.resources.livestock.wool_last_day[&animal], cooldown);
    }
    assert_eq!(
        super::super::save(&mut loaded).expect("continued restored save"),
        super::super::save(&mut engine).expect("continued original save")
    );
}

#[test]
fn recurring_legacy_milk_placeholder_survives_full_save_and_completes_as_milk() {
    use sim_core::colony::{ItemKind, Job};
    let (mut engine, worker, animal, gate) = recurring_engine(true);
    let milk = engine.resources.livestock.animals[&animal]
        .species
        .definition()
        .tameable
        .as_ref()
        .expect("policy")
        .products
        .milk
        .as_ref()
        .expect("milk")
        .item;
    engine
        .resources
        .livestock
        .animals
        .get_mut(&animal)
        .expect("animal")
        .milk_ready = true;
    start_recurring(&mut engine, worker, gate);
    // Reproduce the persisted pre-C2 Milk job field while retaining real admission/reservations.
    let placeholder = ItemKind::role("default_placeholder");
    engine.world.get::<&mut Job>(worker).expect("job").item = placeholder;
    let bytes = super::super::save(&mut engine).expect("legacy job full save");
    let mut loaded = super::super::load(&bytes).expect("legacy job full load");
    assert_eq!(
        loaded.world.get::<&Job>(worker).expect("restored job").item,
        placeholder
    );
    finish_recurring(&mut loaded, worker, gate);
    assert_eq!(loaded.resources.colony.item_ledger.created_of(milk), 1);
    assert!(loaded.resources.livestock.wool_last_day.is_empty());
    assert!(!loaded.resources.livestock.animals[&animal].milk_ready);
    super::super::save(&mut loaded).expect("completed milk remains save-valid");
}

#[test]
fn all_twenty_three_legacy_counters_and_skills_survive_without_new_completions() {
    assert_eq!(SkillKind::ALL.len(), 8);
    assert_eq!(JobKind::NAMES.len(), 26);
    let mut entries = [Skill::default(); 8];
    entries[7] = Skill { level: 4, xp: 137 };
    let mut completed = [0; 23];
    for (index, value) in completed.iter_mut().enumerate() {
        *value = u32::try_from(index).expect("small index") * 251 + 1;
    }
    completed[22] = u32::MAX;
    let old = vec![SkillsV20 { entries, completed }];
    let bytes = super::super::encode(&old).expect("actual old layout");
    let upgraded = skills_column(&bytes, 1).expect("widen");
    let current: Vec<Skills> = super::super::decode(&upgraded).expect("current column");
    assert_eq!(current[0].entries, entries);
    assert_eq!(&current[0].completed[..23], &completed);
    assert_eq!(&current[0].completed[23..], &[0; 3]);
    assert!(
        skills_column(&bytes, 2).is_err(),
        "entity count is authenticated"
    );
    assert!(
        skills_column(&upgraded, 1).is_err(),
        "current26 is never read as old23"
    );
}

#[test]
fn livestock_is_an_explicit_tail_and_never_changes_nested_resource_bytes() {
    bootstrap();
    let mut engine = sim_engine::SimEngine::new(8, 8, sim_core::MaterialRegistry::new());
    let before = super::super::encode(&engine.resources).expect("nested resources");
    engine.resources.livestock.next_id = 9;
    engine.resources.livestock.last_advanced_day = Some(3);
    engine
        .resources
        .livestock
        .grass
        .insert((1, 2), GrassState { next_regrow_day: 5 });
    engine.resources.livestock.capture_attempts.insert(
        3,
        CaptureAttempt {
            completed_attempts: 4,
            not_before_day: 6,
        },
    );
    engine.resources.livestock.capture_orders.insert(
        3,
        LivestockOrderCause {
            source_record: 7,
            steward_rule: Some(8),
        },
    );
    assert_eq!(
        super::super::encode(&engine.resources).expect("same wire"),
        before
    );
    let bytes = super::super::encode(&engine.resources.livestock).expect("tail");
    let restored: sim_core::colony::livestock::LivestockState =
        super::super::decode(&bytes).expect("tail restored");
    assert_eq!(restored, engine.resources.livestock);
    assert_eq!(super::super::encode(&restored).expect("same tail"), bytes);
}

fn additions(livestock: bool) -> super::super::LegacySystemAdditions {
    super::super::LegacySystemAdditions {
        steward: false,
        defence: false,
        food: false,
        buildings: false,
        livestock,
        ore: false,
        diagnosis: false,
    }
}

#[test]
fn only_historical_livestock_slot_inserts_exact_registered_bytes() {
    let registered = vec![
        ("Before".into(), vec![1]),
        ("LivestockSystem".into(), vec![251, 9]),
        ("After".into(), vec![2]),
    ];
    let original = vec![registered[0].clone(), registered[2].clone()];
    let mut saved = original.clone();
    super::super::insert_legacy_systems(&mut saved, &registered, additions(true))
        .expect("known slot");
    assert_eq!(saved, registered);
    assert!(super::super::insert_legacy_systems(&mut saved, &registered, additions(true)).is_err());
    let mut current_missing = original.clone();
    super::super::insert_legacy_systems(&mut current_missing, &registered, additions(false))
        .expect("no repair");
    assert_eq!(current_missing, original);
}

#[test]
fn format20_admission_is_revision_and_full_published_prefix_bound() {
    let catalog = Catalog::from_sources(RawSources::bundled(), None).expect("catalog");
    assert!(identities_preserved(&catalog));
    assert!(accepts_catalog(REVISION, &catalog).expect("genuine historical revision"));
    assert!(!accepts_catalog("unregistered20", &catalog).expect("unknown rejected"));
    let mut sources = RawSources::bundled();
    let source = sources
        .sources
        .iter_mut()
        .find(|source| source.name == "items.ron")
        .expect("items source");
    let text = std::str::from_utf8(&source.bytes).expect("utf8");
    source.bytes = text
        .replace("serde_name: \"Wool\"", "serde_name: \"OtherWool\"")
        .into_bytes();
    let changed = Catalog::from_sources(sources, None).expect("otherwise valid edited identity");
    assert!(
        !identities_preserved(&changed),
        "C1-a suffix is part of authenticated history"
    );
}

#[test]
fn relabeled_current_catalog_cannot_acquire_historical_schema_migrations() {
    bootstrap();
    let catalog = sim_core::colony::content::installed().expect("selected catalog");
    assert_ne!(
        catalog.revision(),
        REVISION,
        "this test requires the new21 catalog"
    );
    for version in [19_u32, 20_u32] {
        let envelope = super::super::Envelope {
            revision: catalog.revision().to_owned(),
            // It must be rejected before attempting even malformed body decoding.
            payload: vec![255],
        };
        let mut bytes = super::super::MAGIC.to_vec();
        bytes.extend_from_slice(&version.to_le_bytes());
        bytes.extend(super::super::encode(&envelope).expect("relabeled envelope"));
        let mut registered = false;
        let error = super::super::load_with(&bytes, |_| registered = true)
            .err()
            .expect("current revision cannot authenticate historical schema");
        assert!(
            matches!(error, SaveError::Decode(ref message)
            if message.starts_with("save content revision ")),
            "revision rejection precedes payload decode: {error}"
        );
        assert!(
            !registered,
            "no historical system insertion or registration"
        );
    }
}

// Mirrors only the unchanged opaque world-container wire to inspect original
// component bytes; production WorldSave fields remain encapsulated.
#[derive(Deserialize)]
struct InspectWorld {
    components: Vec<String>,
    archetypes: Vec<InspectArchetype>,
    _meta_len: u32,
    _free: Vec<u64>,
}
#[derive(Deserialize)]
struct InspectArchetype {
    types: Vec<u16>,
    entities: Vec<u64>,
    columns: Vec<Vec<u8>>,
}

#[test]
#[ignore = "mandatory genuine published-a20 suite: WORLDSIM_C1_FORMAT20_FIXTURES"]
fn genuine_published_a20_keeps_counters_upper_masks_physical_state_and_slots() {
    bootstrap();
    let root = std::path::PathBuf::from(
        std::env::var_os("WORLDSIM_C1_FORMAT20_FIXTURES")
            .expect("explicit immutable published-a fixture directory"),
    );
    for (name, size, expected_hash) in [
        ("ordinary20.wsave", 1_327_523, 0xf6b9d4b71c0355be_u64),
        ("rich20.wsave", 1_306_448, 0x90a49388e31160e3_u64),
        ("build20.wsave", 1_584_439, 0x4ad9a6bfe28b1918_u64),
    ] {
        let bytes = std::fs::read(root.join(name)).expect("actual published20 bytes");
        assert_eq!(bytes.len(), size);
        assert_eq!(&bytes[8..12], &20_u32.to_le_bytes());
        let observed_hash = bytes.iter().fold(0xcbf29ce484222325_u64, |hash, byte| {
            (hash ^ u64::from(*byte)).wrapping_mul(0x100000001b3)
        });
        assert_eq!(
            observed_hash, expected_hash,
            "immutable published fixture identity"
        );
        let envelope: super::super::Envelope =
            super::super::decode(&bytes[12..]).expect("envelope");
        assert_eq!(envelope.revision, REVISION);
        let old: LegacyBodyV20 =
            super::super::decode(&envelope.payload).expect("frozen actual20 body");
        assert!(!old.systems.iter().any(|row| row.0 == "LivestockSystem"));
        let inspected: InspectWorld =
            super::super::decode(&super::super::encode(&old.world).expect("opaque world"))
                .expect("container only");
        let source = old
            .mind
            .policy_source
            .as_deref()
            .expect("genuine saved mind source");
        let history = old.mind.history.as_ref().expect("genuine saved dictionary");
        let restored_mind = crate::runtime::mind::MindProgram::load_saved(source, history)
            .expect("genuine published-a policy registry");
        for fact in [
            "job_food_transport",
            "harvest_window",
            "harvest_storage_available",
            "winter_stock_duty",
            "water_target_available",
        ] {
            assert!(
                restored_mind.policy.fact_id(fact).is_some(),
                "genuine20 food fact: {fact}"
            );
        }
        assert!(restored_mind.policy.fact_id("job_livestock").is_none());
        let mut loaded = super::super::load(&bytes).expect("production historical load");
        assert_eq!(loaded.current_tick(), old.current_tick);
        assert_eq!(loaded.resources.livestock, Default::default());
        assert_eq!(loaded.resources.colony.buildings, old.buildings);
        assert_eq!(
            super::super::encode(&loaded.resources.colony.workshops).expect("restored stations"),
            super::super::encode(&old.resources.colony.workshops).expect("original stations"),
            "every physical station, shelf and bill survives exactly"
        );
        assert_eq!(
            loaded.resources.colony.ground_items,
            old.resources.colony.ground_items
        );
        assert_eq!(
            loaded.resources.colony.item_ledger,
            old.resources.colony.item_ledger
        );
        let registered = loaded.system_states().expect("states");
        assert_eq!(
            registered
                .iter()
                .filter(|row| row.0 == "LivestockSystem")
                .count(),
            1
        );
        let old_slots = registered
            .iter()
            .filter(|row| row.0 != "LivestockSystem" && row.0 != "OreDiscoverySystem")
            .cloned()
            .collect::<Vec<_>>();
        assert_eq!(
            old_slots, old.systems,
            "all historical state bytes and order survive"
        );
        let mut total_completed = 0_u64;
        let mut upper_mask = false;
        for archetype in inspected.archetypes {
            for (index, column) in archetype.types.into_iter().zip(archetype.columns) {
                match inspected.components[usize::from(index)].as_str() {
                    "Skills" => {
                        let original: Vec<SkillsV20> =
                            super::super::decode(&column).expect("genuine23 column");
                        for (bits, original) in archetype.entities.iter().zip(original) {
                            let entity = hecs::Entity::from_bits(*bits).expect("entity identity");
                            let current = loaded
                                .world
                                .get::<&Skills>(entity)
                                .expect("same entity skills");
                            assert_eq!(current.entries, original.entries);
                            assert_eq!(&current.completed[..23], &original.completed);
                            assert_eq!(&current.completed[23..], &[0; 3]);
                            total_completed +=
                                original.completed.into_iter().map(u64::from).sum::<u64>();
                        }
                    }
                    "Knowledge" => {
                        let original: Vec<Knowledge> =
                            super::super::decode(&column).expect("genuine128 column");
                        for (bits, original) in archetype.entities.iter().zip(original) {
                            let entity = hecs::Entity::from_bits(*bits).expect("entity identity");
                            assert_eq!(
                                *loaded
                                    .world
                                    .get::<&Knowledge>(entity)
                                    .expect("same knowledge"),
                                original
                            );
                            upper_mask |= original.practice().words()[1..]
                                .iter()
                                .any(|word| *word != 0);
                        }
                    }
                    _ => {}
                }
            }
        }
        if name == "rich20.wsave" {
            assert!(total_completed > 0, "genuine nonzero original counters");
            assert!(upper_mask, "genuine upper practitioner word");
            assert!(
                !old.resources.colony.workshops.is_empty(),
                "genuine physical crafting stations"
            );
        }
        if name == "build20.wsave" {
            use sim_core::colony::building_site::{BuildingStatus, PiecePhase};
            assert!(
                old.buildings
                    .sites
                    .values()
                    .any(|site| site.status == BuildingStatus::Complete
                        && site.completion_entry.is_some()
                        && !site.pieces.is_empty()
                        && site
                            .pieces
                            .iter()
                            .all(|piece| piece.phase == PiecePhase::Complete)),
                "genuine completed template tail with physical pieces and completion witness"
            );
        }
        let current = super::super::save(&mut loaded).expect("migrated21 save");
        assert_eq!(&current[8..12], &super::super::FORMAT_VERSION.to_le_bytes());
        let mut roundtrip = super::super::load(&current).expect("current21 load");
        assert_eq!(
            super::super::save(&mut roundtrip).expect("immediate21 roundtrip"),
            current
        );
        for _ in 0..60 {
            loaded.tick();
            roundtrip.tick();
        }
        assert_eq!(
            super::super::save(&mut loaded).expect("continued migrated"),
            super::super::save(&mut roundtrip).expect("continued restored")
        );
    }
}
