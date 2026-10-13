//! A member of a departing fission group (Pass 11, `game_v17`).
use serde::{Deserialize, Serialize};

use crate::components::AgentId;

/// Marks a resident who left its settlement with a fission group and is
/// walking to the group's site. Attached by `FissionSystem` at departure and
/// removed when the daughter settlement is founded or the fission is
/// cancelled. While it is present the resident walks to `site`
/// (`Seeking { Agent(leader) }` with `SeekTarget = site`), takes no work,
/// forage, firewood, meal, social or building arm, follows no tether and is
/// never marked `Cold` at its parent's hearth. It never updates itself.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct Founding {
    /// The tile the group walks to (a wild-region centre).
    pub site: (u32, u32),
    /// The fission this resident belongs to (`FissionSystem` counter).
    pub fission_id: u32,
    /// The group's lowest-AgentId adult.
    pub leader: AgentId,
    /// Tick of departure.
    pub since_tick: u64,
}
