//! Pure keyed livestock traits and completed-attempt draws; no RNG stream state.
use super::content::{AnimalKind, TameableDef};
use super::wildlife::{Animal, AnimalId};
use crate::components::Sex;
use crate::rng::{keyed_below, STREAM_LIVESTOCK};

/// Biological traits projected from the original wild appearance.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct WildDemographics {
    /// Whole biological days, including the immutable appearance-age offset.
    pub age_days: u32,
    /// Immutable keyed sex copied on capture.
    pub sex: Sex,
    /// Last whole-day boundary, preserving the wild animal's age phase.
    pub last_age_tick: u64,
}

fn subject(species: AnimalKind, id: u32) -> u64 {
    (u64::from(species.code()) << 32) | u64::from(id)
}

/// The same read-only projection is used by offers, capture commits and UI.
pub fn wild_demographics(
    seed: u64,
    animal: &Animal,
    policy: &TameableDef,
    now_tick: u64,
    ticks_per_day: u64,
) -> WildDemographics {
    let key = subject(animal.kind, animal.id);
    let (lo, hi) = policy.initial_age_days;
    let span = u64::from(hi.saturating_sub(lo)) + 1;
    let initial = u64::from(lo) + keyed_below(seed, STREAM_LIVESTOCK, key, 0, 1, 0, span);
    let elapsed = now_tick.saturating_sub(animal.born_tick);
    let day = ticks_per_day.max(1);
    WildDemographics {
        age_days: u32::try_from(initial.saturating_add(elapsed / day)).unwrap_or(u32::MAX),
        sex: Sex {
            female: keyed_below(seed, STREAM_LIVESTOCK, key, 0, 0, 0, 2) == 0,
        },
        last_age_tick: now_tick.saturating_sub(elapsed % day),
    }
}

/// One chance in0..999 for a persisted completed-attempt ordinal, never an offer.
pub fn capture_draw(seed: u64, species: AnimalKind, wild: AnimalId, attempt: u32) -> u16 {
    keyed_below(
        seed,
        STREAM_LIVESTOCK,
        subject(species, wild),
        u64::from(attempt),
        2,
        0,
        1000,
    ) as u16
}

/// Newborn identity trait, independent of frame, worker and unrelated draws.
pub fn newborn_sex(seed: u64, species: AnimalKind, newborn: u32) -> Sex {
    Sex {
        female: keyed_below(
            seed,
            STREAM_LIVESTOCK,
            subject(species, newborn),
            0,
            3,
            0,
            2,
        ) == 0,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::colony::content::{Catalog, RawSources};
    use crate::colony::wildlife::AnimalState;

    fn fixture() -> (Animal, TameableDef) {
        let catalog = Catalog::from_sources(RawSources::bundled(), None).expect("catalog");
        let kind = catalog.animal_by_name("wild_goat").expect("species");
        let policy = catalog
            .animal(kind)
            .expect("animal")
            .tameable
            .clone()
            .expect("policy");
        (
            Animal {
                id: 7,
                kind,
                hp: 30,
                state: AnimalState::Wander,
                pos: (2, 3),
                pack: None,
                born_tick: 123,
                path: Vec::new(),
                goal: None,
                home: Some((2, 3)),
            },
            policy,
        )
    }

    #[test]
    fn immutable_wild_traits_advance_only_whole_biological_days() {
        let (animal, policy) = fixture();
        let first = wild_demographics(17, &animal, &policy, 123, 1440);
        assert!(first.age_days <= 29);
        assert_eq!(first.last_age_tick, 123);
        let before = wild_demographics(17, &animal, &policy, 1562, 1440);
        assert_eq!(before, first);
        let day = wild_demographics(17, &animal, &policy, 1563, 1440);
        assert_eq!(day.age_days, first.age_days + 1);
        assert_eq!(day.sex, first.sex);
        assert_eq!(day.last_age_tick, 1563);
        let encoded = ron::to_string(&animal).expect("wild save");
        let loaded: Animal = ron::from_str(&encoded).expect("wild load");
        assert_eq!(wild_demographics(17, &loaded, &policy, 1600, 1440), day);
    }

    #[test]
    fn keys_are_repeatable_bounded_and_do_not_consume_unrelated_draws() {
        let (animal, mut policy) = fixture();
        let unrelated = crate::rng::keyed_u64(17, crate::rng::STREAM_MOVE_DX, 8, 99, 0, 0);
        for attempt in 1..100 {
            let draw = capture_draw(17, animal.kind, animal.id, attempt);
            assert!(draw < 1000);
            assert_eq!(draw, capture_draw(17, animal.kind, animal.id, attempt));
            assert_eq!(
                newborn_sex(17, animal.kind, attempt),
                newborn_sex(17, animal.kind, attempt)
            );
        }
        assert_eq!(
            unrelated,
            crate::rng::keyed_u64(17, crate::rng::STREAM_MOVE_DX, 8, 99, 0, 0)
        );
        policy.initial_age_days = (u32::MAX, u32::MAX);
        let edge = wild_demographics(17, &animal, &policy, u64::MAX, 0);
        assert_eq!(edge.age_days, u32::MAX);
        assert_eq!(edge.last_age_tick, u64::MAX);
    }
}
