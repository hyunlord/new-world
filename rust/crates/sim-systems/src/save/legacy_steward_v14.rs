//! Published formats11–14, before M-LOOP inserted two steward fields.
use serde::{Deserialize, Serialize};
use sim_core::colony::steward::{DefenceState, OwnedOrder, StewardScope, StewardState, Target};
use sim_core::colony::Command;
use std::collections::BTreeMap;

#[derive(Default, Serialize, Deserialize)]
pub(super) struct LegacyStewardStateV14 {
    pub(super) enabled: bool,
    pub(super) scope: StewardScope,
    pub(super) cooldowns: BTreeMap<u16, u64>,
    pub(super) vetoes: BTreeMap<Target, u64>,
    pub(super) owned: Vec<OwnedOrder>,
    pub(super) sequence: u64,
    pub(super) last_rule: Option<u16>,
    pub(super) last_tick: u64,
    pub(super) replay: bool,
    pub(super) pending: Vec<(u64, Command)>,
    pub(super) defence: DefenceState,
}

impl From<LegacyStewardStateV14> for StewardState {
    fn from(old: LegacyStewardStateV14) -> Self {
        Self {
            enabled: old.enabled,
            scope: old.scope,
            extended: false,
            need_since: BTreeMap::new(),
            cooldowns: old.cooldowns,
            vetoes: old.vetoes,
            owned: old.owned,
            sequence: old.sequence,
            last_rule: old.last_rule,
            last_tick: old.last_tick,
            replay: old.replay,
            pending: old.pending,
            defence: old.defence,
        }
    }
}

#[cfg(test)]
impl From<StewardState> for LegacyStewardStateV14 {
    fn from(state: StewardState) -> Self {
        assert!(!state.extended && state.need_since.is_empty());
        Self {
            enabled: state.enabled,
            scope: state.scope,
            cooldowns: state.cooldowns,
            vetoes: state.vetoes,
            owned: state.owned,
            sequence: state.sequence,
            last_rule: state.last_rule,
            last_tick: state.last_tick,
            replay: state.replay,
            pending: state.pending,
            defence: state.defence,
        }
    }
}

#[derive(Serialize, Deserialize)]
pub(super) struct LegacyBodyV14 {
    pub(super) current_tick: u64,
    pub(super) resources: sim_engine::SimResources,
    pub(super) world: super::world::WorldSave,
    pub(super) systems: Vec<(String, Vec<u8>)>,
    pub(super) steward: LegacyStewardStateV14,
    pub(super) content_state: sim_core::colony::content_state::ContentState,
    pub(super) mind: sim_core::colony::mind_state::MindState,
    pub(super) food: sim_core::colony::food::FoodState,
}

pub(super) fn decode_published_v14(payload: &[u8]) -> Result<super::Body, super::SaveError> {
    // M-LOOP wrote the new layout with the old tag. Do not guess when that
    // layout decodes, even if the published layout could also consume it.
    if super::decode::<super::Body>(payload).is_ok() {
        return Err(super::SaveError::Decode(
            "format14 contains an unpublished M-LOOP or ambiguous steward layout; preserve the original file".into(),
        ));
    }
    let old: LegacyBodyV14 = super::decode(payload).map_err(super::SaveError::Decode)?;
    Ok(super::Body {
        current_tick: old.current_tick,
        resources: old.resources,
        world: old.world,
        systems: old.systems,
        steward: old.steward.into(),
        content_state: old.content_state,
        mind: old.mind,
        food: old.food,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn published_layout_preserves_nonempty_steward_and_defaults_new_fields() {
        // Given the frozen published layout with cooldown and defence state.
        let mut engine = sim_engine::SimEngine::new(4, 4, sim_core::MaterialRegistry::new());
        engine.resources.steward.enabled = true;
        engine.resources.steward.cooldowns.insert(2, 720);
        engine.resources.steward.sequence = 7;
        engine.resources.steward.defence.enabled = false;
        let expected = engine.resources.steward.clone();
        let old = LegacyBodyV14 {
            current_tick: 360,
            world: super::super::world::capture(&engine.world).expect("world"),
            systems: vec![],
            steward: std::mem::take(&mut engine.resources.steward).into(),
            content_state: std::mem::take(&mut engine.resources.content_state),
            mind: std::mem::take(&mut engine.resources.mind),
            food: std::mem::take(&mut engine.resources.food),
            resources: engine.resources,
        };
        let bytes = super::super::encode(&old).expect("historical wire fixture");
        // When the version-specific reader migrates it.
        let migrated = decode_published_v14(&bytes).expect("published layout");
        // Then all old fields survive and new fields stay at inert defaults.
        assert_eq!(migrated.current_tick, 360);
        assert_eq!(migrated.steward, expected);
        assert!(!migrated.steward.extended);
        assert!(migrated.steward.need_since.is_empty());
    }

    #[test]
    fn new_layout_cannot_claim_published_format14_even_with_default_fields() {
        // Given a current save, including the M-LOOP steward wire fields.
        sim_core::colony::content::bootstrap(sim_core::colony::content::BootSelection::Bundled)
            .expect("catalog");
        let mut engine = sim_engine::SimEngine::new(4, 4, sim_core::MaterialRegistry::new());
        let mut bytes = super::super::save(&mut engine).expect("current save");
        assert_eq!(&bytes[8..12], &super::super::FORMAT_VERSION.to_le_bytes());
        // When its header falsely advertises the published old layout.
        bytes[8..12].copy_from_slice(&14u32.to_le_bytes());
        let result = super::super::load_with(&bytes, |_| {});
        // Then the reader rejects rather than guessing or rewriting evidence.
        assert!(
            matches!(result, Err(super::super::SaveError::Decode(message))
            if message.contains("unpublished M-LOOP or ambiguous"))
        );
    }
}
