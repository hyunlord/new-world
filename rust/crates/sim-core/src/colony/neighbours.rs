//! M5.5 (ADR 0031, 0032): the colony's neighbours — two off-screen aggregates — and the visitors they send: migrant
//! waves and trading caravans. Visitors are not ECS entities while they visit (DECISIONS M5.5 D16): a wave becomes
//! people only when the player accepts it. Logic lives in `sim_systems::runtime::neighbours`.
use serde::{Deserialize, Serialize};

use super::ItemKind;

/// A map edge a neighbour lies beyond.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub enum Edge {
    /// The top edge (y = 0).
    North,
    /// The right edge (x = width − 1).
    East,
    /// The bottom edge (y = height − 1).
    South,
    /// The left edge (x = 0).
    West,
}

impl Edge {
    /// Every edge in code order.
    pub const ALL: [Self; 4] = [Self::North, Self::East, Self::South, Self::West];

    /// Stable code (0 north, 1 east, 2 south, 3 west).
    pub const fn code(self) -> u8 {
        match self {
            Self::North => 0,
            Self::East => 1,
            Self::South => 2,
            Self::West => 3,
        }
    }

    /// Stable lowercase name.
    pub const fn name(self) -> &'static str {
        match self {
            Self::North => "north",
            Self::East => "east",
            Self::South => "south",
            Self::West => "west",
        }
    }

    /// The edge of [`Self::code`] (`North` for an unknown code).
    pub const fn from_code(code: u8) -> Self {
        match code % 4 {
            1 => Self::East,
            2 => Self::South,
            3 => Self::West,
            _ => Self::North,
        }
    }
}

/// One neighbouring group (ADR 0031): an aggregate, never simulated person by person.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Neighbour {
    /// Its id (0 or 1).
    pub id: u8,
    /// The index of its name in the locale's neighbour names (`NEIGHBOUR_NAME_<n>`).
    pub name_key: u8,
    /// The map edge it lies beyond.
    pub edge: Edge,
    /// Days of travel to it (2–5).
    pub distance_days: u8,
    /// Its people (20–60 at the start; drifts ±1 % a day).
    pub population: u16,
    /// Its attitude toward the colony, 0–100 (starts 40–60).
    pub attitude: u8,
    /// The knowledge nodes it knows (bit `i` = node `i`).
    pub known: u32,
    /// The display day of the last contact (a wave or a caravan reached the hearth), if any.
    pub last_contact_day: Option<u32>,
}

/// One person of a migrant wave before they join: what the dialog shows and what they bring.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct MigrantSpec {
    /// The `AgentId` they will have, reserved when the wave set out (the dialog names them by it).
    #[serde(default)]
    pub agent: crate::components::AgentId,
    /// The lineage key they will carry (keyed on the wave and their place in it).
    pub key: u64,
    /// Whether they are a woman (`Sex::from_lineage` of the key).
    pub female: bool,
    /// Their age in years at arrival (their `Life` follows from the key; reported for the dialog).
    pub age_years: u16,
    /// What they know: `(node id, proficiency ‰)`, in node order.
    pub knowledge: Vec<(u8, u16)>,
    /// Whether they arrive wearing a hide wrap.
    pub wrap: bool,
}

/// Where a wave or a caravan is in its visit.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum VisitState {
    /// Walking from its edge to the hearth.
    Arriving,
    /// At the hearth since `since` (a wave waits for the player's answer; a caravan trades).
    Waiting {
        /// The tick it arrived.
        since: u64,
    },
    /// Walking back to its edge (a declined or unanswered wave, a caravan that is done).
    Leaving,
    /// Gone (a wave joined the colony or left the map; a caravan left the map).
    Gone,
}

impl VisitState {
    /// Stable code for the bridge (0 arriving, 1 waiting, 2 leaving, 3 gone).
    pub const fn code(self) -> u8 {
        match self {
            Self::Arriving => 0,
            Self::Waiting { .. } => 1,
            Self::Leaving => 2,
            Self::Gone => 3,
        }
    }
}

/// How a wave's visit ended.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum WaveAnswer {
    /// The player accepted: they joined.
    Accepted,
    /// The player declined.
    Declined,
    /// Nobody answered within a day.
    Unanswered,
}

/// One migrant wave (ADR 0031).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Wave {
    /// Its number (1, 2, 3 … in arrival order).
    pub id: u32,
    /// The neighbour it comes from.
    pub neighbour: u8,
    /// Its people.
    pub people: Vec<MigrantSpec>,
    /// The group's tile.
    pub pos: (u32, u32),
    /// The path it follows (first step first).
    pub path: Vec<(u32, u32)>,
    /// Where it is in its visit.
    pub state: VisitState,
    /// The display day it set out (appeared at the edge).
    pub day: u32,
    /// The answer, once given or timed out.
    pub answer: Option<WaveAnswer>,
    /// The `AgentId`s of the people it became (accepted).
    pub joined: Vec<u64>,
}

/// One stack of a caravan's goods.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct Goods {
    /// What.
    pub kind: ItemKind,
    /// How many are left.
    pub count: u16,
    /// Their quality (30–70).
    pub quality: u8,
}

/// One trading caravan (ADR 0032).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Caravan {
    /// Its number (1, 2, 3 … in arrival order).
    pub id: u32,
    /// The neighbour it comes from.
    pub neighbour: u8,
    /// Its goods, in drawn order (the `stack` of a `Trade` command is an index here).
    pub goods: Vec<Goods>,
    /// The group's tile (two traders stand on it and the next tile).
    pub pos: (u32, u32),
    /// The path it follows (first step first).
    pub path: Vec<(u32, u32)>,
    /// Where it is in its visit.
    pub state: VisitState,
    /// The display day it set out.
    pub day: u32,
    /// The tile at the hearth where traded goods are handed over (set on arrival).
    pub spot: Option<(u32, u32)>,
    /// Items the colony still owes it from accepted trades: `(stack tile, kind, count)`, carried there by haulers.
    pub owed: Vec<((u32, u32), ItemKind, u16)>,
    /// Trades made with it.
    pub trades: u32,
}

/// Counters of the neighbours (measurement and the year summary).
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct NeighbourTallies {
    /// Waves that set out.
    pub waves: u64,
    /// Waves accepted, declined, unanswered.
    pub accepted: u64,
    /// See [`Self::accepted`].
    pub declined: u64,
    /// See [`Self::accepted`].
    pub unanswered: u64,
    /// People who joined from waves.
    pub migrants_joined: u64,
    /// Seasonal wave rolls made and won.
    pub rolls: u64,
    /// See [`Self::rolls`].
    pub roll_wins: u64,
    /// Waves not sent because the colony was at its population cap.
    pub capped: u64,
    /// Caravans that set out, and caravan dates skipped (a pressure or the season's end).
    pub caravans: u64,
    /// See [`Self::caravans`].
    pub caravans_skipped: u64,
    /// Trades accepted and refused, gifts given.
    pub trades: u64,
    /// See [`Self::trades`].
    pub trades_refused: u64,
    /// See [`Self::trades`].
    pub gifts: u64,
    /// Value given and taken over all trades (the traders' 80 % / 120 % applied).
    pub value_given: u64,
    /// See [`Self::value_given`].
    pub value_taken: u64,
    /// Discoveries helped by an artifact (eureka).
    pub eurekas: u64,
    /// Items taken in trades whose producing node no living colonist practised at the trade (F8).
    #[serde(default)]
    pub unknown_bought: u64,
}

/// The colony's neighbours and their visitors (`ColonyState::neighbours`; ADR 0031, 0032).
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct Neighbours {
    /// The two neighbours (empty until generated on the first day).
    pub list: Vec<Neighbour>,
    /// Every wave so far, in id order (the gone ones stay for the record).
    pub waves: Vec<Wave>,
    /// Every caravan so far, in id order.
    pub caravans: Vec<Caravan>,
    /// The display days of the planned waves not yet sent: `(day, size)`.
    pub planned_waves: Vec<(u32, u8)>,
    /// The display days of the planned caravans not yet sent: `(day, neighbour)`.
    pub planned_caravans: Vec<(u32, u8)>,
    /// Seasons planned so far (the next season index to plan).
    pub seasons_planned: u32,
    /// The first display day the daily step has not run yet.
    pub next_day: u32,
    /// The neighbour that sent the last caravan (they alternate when both may send).
    pub last_caravan_from: Option<u8>,
    /// Counters.
    pub tallies: NeighbourTallies,
    /// Bumped when anything the renderers show changes.
    pub version: u64,
}

impl Neighbours {
    /// The wave waiting at the hearth for an answer, if any (the lowest id).
    pub fn waiting_wave(&self) -> Option<&Wave> {
        self.waves
            .iter()
            .find(|w| matches!(w.state, VisitState::Waiting { .. }))
    }

    /// The caravan trading at the hearth, if any (the lowest id).
    pub fn trading_caravan(&self) -> Option<&Caravan> {
        self.caravans
            .iter()
            .find(|c| matches!(c.state, VisitState::Waiting { .. }))
    }

    /// The neighbour `id`.
    pub fn neighbour(&self, id: u8) -> Option<&Neighbour> {
        self.list.iter().find(|n| n.id == id)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn codes_are_stable() {
        let codes: Vec<u8> = Edge::ALL.iter().map(|e| e.code()).collect();
        assert_eq!(codes, vec![0, 1, 2, 3]);
        for e in Edge::ALL {
            assert_eq!(Edge::from_code(e.code()), e);
        }
        assert_eq!(VisitState::Waiting { since: 5 }.code(), 1);
        assert_eq!(VisitState::Gone.code(), 3);
    }
}
