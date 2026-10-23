//! M5 (ADR 0028): animals, hunt marks and fights. Animals are not ECS entities (M5 DECISIONS D6): at most 16
//! live in `ColonyState::wildlife`. Logic lives in `sim_systems::runtime::wildlife`.
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, BTreeSet};

use crate::components::AgentId;

/// An animal's id (`Wildlife::next_id`, from 1).
pub type AnimalId = u32;

/// What an animal is.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub enum AnimalKind {
    /// A wolf: comes in packs with the Beasts pressure.
    Wolf,
    /// A deer: grazes in the forest, hunted for meat and hide.
    Deer,
}

impl AnimalKind {
    /// Stable code (0 wolf, 1 deer).
    pub const fn code(self) -> u8 {
        match self {
            Self::Wolf => 0,
            Self::Deer => 1,
        }
    }

    /// Stable lowercase name.
    pub const fn name(self) -> &'static str {
        match self {
            Self::Wolf => "wolf",
            Self::Deer => "deer",
        }
    }
}

/// What an animal is doing.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum AnimalState {
    /// Grazing or roaming.
    Wander,
    /// Running from people (`steps` left before it tires; deer).
    Flee {
        /// Flee steps left.
        steps: u32,
    },
    /// Resting after a flight (`steps` left; deer).
    Rest {
        /// Rest steps left.
        steps: u32,
    },
    /// Walking toward the camp or a person (wolves).
    Hunt,
    /// Biting a person (wolves).
    Attack {
        /// The person.
        target: AgentId,
    },
    /// Running off the map (a wolf pack that lost half its number).
    Retreat,
}

impl AnimalState {
    /// Stable code for the bridge (0 wander, 1 flee, 2 rest, 3 hunt, 4 attack, 5 retreat).
    pub const fn code(self) -> u8 {
        match self {
            Self::Wander => 0,
            Self::Flee { .. } => 1,
            Self::Rest { .. } => 2,
            Self::Hunt => 3,
            Self::Attack { .. } => 4,
            Self::Retreat => 5,
        }
    }
}

/// One animal.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Animal {
    /// Its id.
    pub id: AnimalId,
    /// What it is.
    pub kind: AnimalKind,
    /// Hit points left.
    pub hp: u16,
    /// What it is doing.
    pub state: AnimalState,
    /// Its tile.
    pub pos: (u32, u32),
    /// The pack (a wolf: the day its Beasts pressure began).
    pub pack: Option<u32>,
    /// The tick it appeared.
    pub born_tick: u64,
    /// The path it follows (first step first) and the goal it was planned for.
    pub path: Vec<(u32, u32)>,
    /// The goal of `path`.
    pub goal: Option<(u32, u32)>,
    /// A deer's home ground: it wanders within `deer_home_tiles` of it (fleeing may take it farther).
    pub home: Option<(u32, u32)>,
}

/// One fighter of a fight: a person or an animal.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub enum Fighter {
    /// A person.
    Person(AgentId),
    /// An animal.
    Animal(AnimalId),
}

/// One side's attack in a fight (`ColonyState::fights`, keyed `(attacker, defender)`):
/// `CombatSystem` resolves it every `ticks` ticks with `damage`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct Fight {
    /// Ticks since the last resolved hit.
    pub progress: u32,
    /// Ticks between resolved hits.
    pub ticks: u32,
    /// Damage of each hit.
    pub damage: u16,
    /// Resolved hits after which the fight ends (0 = until one side falls or leaves; a quarrel's fight: 1).
    pub hits_left: u32,
    /// The quarrel cause code of a fight between people (`None` for beasts).
    pub quarrel: Option<u8>,
}

/// A hunt zone: every deer inside the rect `(x0, y0, x1, y1)` (inclusive) may be hunted.
pub type HuntZone = (u32, u32, u32, u32);

/// The colony's animals and hunting orders (`ColonyState::wildlife`).
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct Wildlife {
    /// Living animals by id.
    pub animals: BTreeMap<AnimalId, Animal>,
    /// The next animal id (from 1).
    pub next_id: AnimalId,
    /// Whether the deer were placed (worldgen runs once, on the first step).
    pub seeded: bool,
    /// The display day of the last deer birth.
    pub last_deer_day: u32,
    /// The deer the map holds (the worldgen count; births stop there).
    pub deer_cap: u32,
    /// The day of the Beasts pressure whose pack has come (its `start_day`).
    pub pack_day: Option<u32>,
    /// Wolves that came with that pack.
    pub pack_size: u32,
    /// The pack's map-edge tile, found once when it came (its door and its way out; M5 code review m4).
    #[serde(default)]
    pub pack_edge: Option<(u32, u32)>,
    /// Deer marked to be hunted.
    pub marks: BTreeSet<AnimalId>,
    /// Hunt zones in the order they were added.
    pub zones: Vec<HuntZone>,
    /// Meat on the map by the display day it was killed: `(day, count)`, oldest first.
    pub meat_lots: Vec<(u32, u32)>,
    /// Counters (measurement and UI).
    pub tallies: WildlifeTallies,
    /// Bumped whenever animals appear, move, change state or die (renderers).
    pub version: u64,
}

impl Wildlife {
    /// The living wolves.
    pub fn wolves(&self) -> impl Iterator<Item = &Animal> + '_ {
        self.animals.values().filter(|a| a.kind == AnimalKind::Wolf)
    }

    /// The living deer.
    pub fn deer(&self) -> impl Iterator<Item = &Animal> + '_ {
        self.animals.values().filter(|a| a.kind == AnimalKind::Deer)
    }

    /// Whether deer `id` may be hunted: marked, or inside a hunt zone.
    pub fn huntable(&self, a: &Animal) -> bool {
        a.kind == AnimalKind::Deer
            && (self.marks.contains(&a.id)
                || self.zones.iter().any(|&(x0, y0, x1, y1)| {
                    (x0..=x1).contains(&a.pos.0) && (y0..=y1).contains(&a.pos.1)
                }))
    }
}

/// Wildlife counters (`Wildlife::tallies`).
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct WildlifeTallies {
    /// Deer born after worldgen.
    pub deer_born: u64,
    /// Deer killed by hunters.
    pub deer_killed: u64,
    /// Wolves that appeared.
    pub wolves_spawned: u64,
    /// Wolves killed.
    pub wolves_killed: u64,
    /// Wolves that left (retreated or stayed their longest).
    pub wolves_left: u64,
    /// Bites wolves landed on people.
    pub bites: u64,
    /// Damage wolves did to people.
    pub bite_damage: u64,
    /// People a wolf ever bit (distinct per pack).
    pub bitten: u64,
    /// Hunting attacks made, and hits.
    pub hunt_attacks: u64,
    /// See [`Self::hunt_attacks`].
    pub hunt_hits: u64,
    /// Meat dropped by kills.
    pub meat_dropped: u64,
    /// Meat put into the settlement's food (eaten as foraged food is).
    pub meat_eaten: u64,
    /// Meat that spoiled.
    pub meat_spoiled: u64,
    /// Hides dropped by kills (deer and wolf).
    pub hides_dropped: u64,
    /// Knife uses (scraping, butchering) and knives broken.
    pub knife_uses: u64,
    /// See [`Self::knife_uses`].
    pub knives_broken: u64,
    /// Spears broken.
    pub spears_broken: u64,
    /// People drafted (commands), and self-undrafts at a critical need.
    pub drafts: u64,
    /// See [`Self::drafts`].
    pub auto_undrafts: u64,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn codes_are_stable_and_zones_mark_deer() {
        assert_eq!((AnimalKind::Wolf.code(), AnimalKind::Deer.code()), (0, 1));
        assert_eq!(AnimalState::Attack { target: 3 }.code(), 4);
        let mut w = Wildlife::default();
        let deer = Animal {
            id: 1,
            kind: AnimalKind::Deer,
            hp: 30,
            state: AnimalState::Wander,
            pos: (5, 5),
            pack: None,
            born_tick: 0,
            path: Vec::new(),
            goal: None,
            home: None,
        };
        assert!(!w.huntable(&deer));
        w.zones.push((4, 4, 6, 6));
        assert!(w.huntable(&deer));
        let wolf = Animal {
            kind: AnimalKind::Wolf,
            ..deer.clone()
        };
        assert!(!w.huntable(&wolf), "wolves are fought, not hunted");
    }
}
