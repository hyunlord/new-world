//! Independent frozen format13 wire oracle and format14 boundary checks.
use super::*;
use sim_core::colony::food::*;
use std::collections::BTreeMap;

#[derive(Serialize, Deserialize, Default)]
struct FrozenFoodV13 {
    /// Exact admitted food owner retained until its action ends or becomes invalid.
    active_targets: BTreeMap<sim_core::components::AgentId, FoodOwner>,
    /// Tile-indexed plant state.
    plants: BTreeMap<FoodTile, PlantState>,
    /// Exactly one age for every physical food owner.
    ages: BTreeMap<FoodOwner, FoodAge>,
    /// Real deployed pot vessels and their contents.
    pots: BTreeMap<u32, PotStore>,
    /// Next pot ID; allocation starts at one.
    next_pot_id: u32,
    /// Bounded per-person meal and group histories.
    meals: BTreeMap<u64, MealHistory>,
    /// Day last processed by the daily system.
    last_daily_day: Option<u32>,
    /// Conserved food-flow evidence.
    tallies: FoodTallies,
    /// Once-only narrative evidence.
    firsts: FoodFirsts,
    /// Mutation generation for pure derived readers.
    version: u64,
    /// Passive preservation batches; outputs are not yet physical items.
    pending_drying: Vec<DryingBatch>,
    /// Plan observations keyed by their actual chronicle entry index.
    winter_plans: BTreeMap<u32, WinterPlanEvidence>,
}

#[derive(Serialize, Deserialize)]
struct FrozenBodyV13 {
    current_tick: u64,
    resources: SimResources,
    world: world::WorldSave,
    systems: Vec<(String, Vec<u8>)>,
    steward: sim_core::colony::steward::StewardState,
    content_state: sim_core::colony::content_state::ContentState,
    mind: sim_core::colony::mind_state::MindState,
    food: FrozenFoodV13,
}

fn baseline13() -> Vec<u8> {
    sim_core::colony::content::bootstrap(sim_core::colony::content::BootSelection::Bundled)
        .unwrap();
    let mut engine = SimEngine::new(4, 4, MaterialRegistry::new());
    let body = FrozenBodyV13 {
        current_tick: engine.current_tick(),
        world: world::capture(&engine.world).unwrap(),
        systems: engine.system_states().unwrap(),
        steward: std::mem::take(&mut engine.resources.steward),
        content_state: std::mem::take(&mut engine.resources.content_state),
        mind: std::mem::take(&mut engine.resources.mind),
        resources: engine.resources,
        food: Default::default(),
    };
    let envelope = Envelope {
        revision: sim_core::colony::content::installed()
            .unwrap()
            .revision()
            .to_owned(),
        payload: encode(&body).unwrap(),
    };
    let mut bytes = MAGIC.to_vec();
    bytes.extend_from_slice(&13u32.to_le_bytes());
    bytes.extend(encode(&envelope).unwrap());
    bytes
}

#[test]
fn format13_baseline_migrates_only_empty_evidence_and_writes14() {
    let mut engine = load_with(&baseline13(), |_| {}).unwrap();
    assert_eq!(engine.resources.food, FoodState::default());
    assert!(engine.system_names().is_empty());
    let bytes = save(&mut engine).unwrap();
    assert_eq!(&bytes[8..12], &14u32.to_le_bytes());
    let mut again = load_with(&bytes, |_| {}).unwrap();
    assert_eq!(save(&mut again).unwrap(), bytes);
}

#[test]
fn relabelled13_body_is_not_a_complete14_body() {
    let mut bytes = baseline13();
    bytes[8..12].copy_from_slice(&14u32.to_le_bytes());
    assert!(load_with(&bytes, |_| {}).is_err());
}

#[test]
fn format13_cannot_use_prefood_catalog_exception() {
    let bytes = baseline13();
    let mut envelope: Envelope = decode(&bytes[12..]).unwrap();
    envelope.revision = "catalog-v1-fnv1a64:1e4d7fded25139b4:45408".into();
    assert_ne!(
        envelope.revision,
        sim_core::colony::content::installed().unwrap().revision()
    );
    let mut changed = bytes[..12].to_vec();
    changed.extend(encode(&envelope).unwrap());
    assert!(load_with(&changed, |_| {}).is_err());
}

#[test]
#[ignore = "requires original external format13 fixture directory; never substitutes generated fixtures"]
fn authenticated_original13_fixtures_keep_all_physical_food_and_system_slots() {
    sim_core::colony::content::bootstrap(sim_core::colony::content::BootSelection::Bundled)
        .unwrap();
    let base = std::path::PathBuf::from(
        std::env::var_os("M6A_FORMAT13_FIXTURE_DIR")
            .expect("explicit original fixture directory required"),
    );
    for (name, size, fingerprint) in [
        (
            "pot-fixture-1/pot-food.save",
            1594546,
            9411634032397233449u64,
        ),
        (
            "hunger-fixture-3/hunger-chain.save",
            1559437,
            5248365819841405844u64,
        ),
    ] {
        let bytes = std::fs::read(base.join(name)).unwrap();
        assert_eq!(bytes.len(), size);
        assert_eq!(&bytes[..8], &MAGIC);
        assert_eq!(&bytes[8..12], &13u32.to_le_bytes());
        let observed = bytes.iter().fold(14695981039346656037u64, |hash, byte| {
            (hash ^ u64::from(*byte)).wrapping_mul(1099511628211)
        });
        assert_eq!(observed, fingerprint, "original fixture bytes changed");
        let envelope: Envelope = decode(&bytes[12..]).unwrap();
        let original: FrozenBodyV13 = decode(&envelope.payload).unwrap();
        let old_food = encode(&original.food).unwrap();
        let mut loaded = load(&bytes).unwrap();
        assert_eq!(loaded.resources.food.evidence, Default::default());
        // Same field order and bytes for EVERY old physical/tally/history field.
        let new_food = encode(&loaded.resources.food).unwrap();
        let mut expected = old_food;
        expected.extend(encode(&loaded.resources.food.evidence).unwrap());
        assert_eq!(new_food, expected, "format13 physical state was remigrated");
        assert_eq!(loaded.current_tick(), original.current_tick);
        assert_eq!(loaded.system_states().unwrap(), original.systems);
        crate::runtime::food::evidence::observe_day(&mut loaded.resources);
        assert_ne!(loaded.resources.food.evidence, Default::default());
        let current = save(&mut loaded).unwrap();
        assert_eq!(&current[8..12], &14u32.to_le_bytes());
        let mut again = load(&current).unwrap();
        assert_eq!(
            again.resources.food.evidence,
            loaded.resources.food.evidence
        );
        assert_eq!(save(&mut again).unwrap(), current);
    }
}
