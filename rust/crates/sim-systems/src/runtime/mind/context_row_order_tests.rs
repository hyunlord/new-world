//! Differential small-index ordering against the original full-row sort.
use super::*;

fn rows(ids: &[AgentId]) -> Vec<PersonSnapshot> {
    let mut world = World::new();
    let entity = world.spawn((Agent { id: 0 }, Position::new(1, 2), AgentState::Idle));
    let prototype = PersonSnapshot::capture(&world, entity, 0, 0).unwrap();
    ids.iter()
        .enumerate()
        .map(|(index, id)| {
            let mut row = prototype.clone();
            row.agent = *id;
            row.position.x = u32::try_from(index).unwrap();
            row.memory_entries = Some(index);
            row.memory_weights = [f64::from_bits(0x3ff0_0000_0000_0000 + index as u64); 6];
            row.hunger = Some(if index % 2 == 0 { -0.0 } else { 1.0 });
            row
        })
        .collect()
}
fn original(rows: &mut [PersonSnapshot]) -> Result<(), ContextError> {
    rows.sort_unstable_by_key(|row| row.agent);
    for pair in rows.windows(2) {
        if pair[0].agent == pair[1].agent {
            return Err(ContextError::DuplicateAgent(pair[0].agent));
        }
    }
    Ok(())
}
fn compare(mut actual: Vec<PersonSnapshot>) {
    let mut expected = actual.clone();
    let old = original(&mut expected);
    let new = order_person_rows(&mut actual);
    assert_eq!(new, old);
    if new.is_ok() {
        assert_eq!(format!("{actual:?}"), format!("{expected:?}"));
        for (a, b) in actual.iter().zip(&expected) {
            assert_eq!(a.hunger.map(f64::to_bits), b.hunger.map(f64::to_bits));
            assert_eq!(
                a.memory_weights.map(f64::to_bits),
                b.memory_weights.map(f64::to_bits)
            );
        }
    }
}
fn permutations(values: &mut [AgentId], start: usize) {
    if start == values.len() {
        compare(rows(values));
        return;
    }
    for next in start..values.len() {
        values.swap(start, next);
        permutations(values, start + 1);
        values.swap(start, next);
    }
}
#[test]
fn ordering_matches_direct_sort_across_cycles_extrema_and_duplicates() {
    for length in 0..=7 {
        permutations(&mut (0..length).collect::<Vec<_>>(), 0);
    }
    compare(rows(&[u64::MAX, 0, 1, u64::MAX - 1, 42]));
    for ids in [
        vec![9, 2, 9, 1, 2],
        vec![u64::MAX, u64::MAX, 0, 0],
        vec![7, 7],
        vec![3, 2, 1, 3, 1],
    ] {
        compare(rows(&ids));
    }
    let mut ids: Vec<u64> = (0..2048).collect();
    let mut state = 7_u64;
    for index in (1..ids.len()).rev() {
        state = state.wrapping_mul(6364136223846793005).wrapping_add(1);
        ids.swap(index, (state % (index as u64 + 1)) as usize);
    }
    compare(rows(&ids));
}
#[test]
fn actual_capture_retains_sorted_payloads_and_smallest_duplicate_error() {
    let mut world = World::new();
    for (id, x) in [(9, 90), (2, 20), (7, 70)] {
        let entity = world.spawn((Agent { id }, Position::new(x, 1), AgentState::Idle));
        if id == 7 {
            world.insert_one(entity, Child { born_tick: 1 }).unwrap();
        }
    }
    let resources = SimResources::new(100, 100, sim_core::MaterialRegistry::new());
    let context = TickContext::capture(&world, &resources).unwrap();
    assert_eq!(
        context.rows.iter().map(|row| row.agent).collect::<Vec<_>>(),
        [2, 7, 9]
    );
    for id in [2, 7, 9] {
        let row = context.row(id).unwrap();
        assert_eq!(row.position.x, u32::try_from(id * 10).unwrap());
        assert_eq!(
            row.life_stage,
            if id == 7 {
                LifeStage::Child
            } else {
                LifeStage::Adult
            }
        );
    }
    world.spawn((Agent { id: 9 }, Position::new(1, 1), AgentState::Idle));
    world.spawn((Agent { id: 2 }, Position::new(1, 1), AgentState::Idle));
    assert!(matches!(
        TickContext::capture(&world, &resources),
        Err(ContextError::DuplicateAgent(2))
    ));
}
