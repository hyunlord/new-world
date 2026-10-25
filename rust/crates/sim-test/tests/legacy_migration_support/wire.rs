//! Independent historical body layouts for migration assertions, never production readers.
use serde::{Deserialize, Serialize};
use sim_core::colony::{
    content_state::{CompletedIdentity, CompletedLayer, ContentState},
    steward::{OwnedOrder, StewardScope, StewardState, Target},
    story::PressureKind,
    Command, ItemKind,
};
use sim_core::components::AgentId;
use sim_engine::{
    legacy_v7::{LegacySimResources, LegacySimResourcesV8},
    SimResources,
};
use sim_systems::save::{self, world::WorldSave};
use std::collections::BTreeMap;

type Systems = Vec<(String, Vec<u8>)>;
type Completed = BTreeMap<((u32, u32), CompletedLayer), CompletedIdentity>;
type Cutters = BTreeMap<u32, ItemKind>;
type Illness = BTreeMap<AgentId, PressureKind>;
type Body6 = (u64, LegacySimResources, WorldSave, Systems);
type Body7 = (u64, LegacySimResources, WorldSave, Systems, OldSteward);
type Body8 = (
    u64,
    LegacySimResourcesV8,
    WorldSave,
    Systems,
    OldSteward,
    (Completed, Cutters),
);
type Body9 = (
    u64,
    SimResources,
    WorldSave,
    Systems,
    OldSteward,
    (Completed, Cutters, Illness),
);

#[derive(Deserialize, Serialize)]
pub struct OldSteward {
    enabled: bool,
    scope: StewardScope,
    cooldowns: BTreeMap<u16, u64>,
    vetoes: BTreeMap<Target, u64>,
    owned: Vec<OwnedOrder>,
    sequence: u64,
    last_rule: Option<u16>,
    last_tick: u64,
    replay: bool,
    pending: Vec<(u64, Command)>,
}
impl OldSteward {
    pub fn check(&self, current: &StewardState) {
        let projected = (
            &current.enabled,
            &current.scope,
            &current.cooldowns,
            &current.vetoes,
            &current.owned,
            &current.sequence,
            &current.last_rule,
            &current.last_tick,
            &current.replay,
            &current.pending,
        );
        assert_eq!(
            save::encode(self).expect("old steward"),
            save::encode(&projected).expect("preserved steward fields")
        );
    }
}
#[derive(Deserialize)]
struct Envelope {
    revision: String,
    payload: Vec<u8>,
}
pub struct Expected {
    pub tick: u64,
    pub resources: SimResources,
    pub steward: Option<OldSteward>,
    pub content: ContentState,
}
pub fn read(bytes: &[u8], version: u32) -> Expected {
    match version {
        6 => {
            let (tick, resources, _, _): Body6 =
                save::decode(&bytes[12..]).expect("frozen v6 body");
            Expected {
                tick,
                resources: resources.into(),
                steward: None,
                content: ContentState::default(),
            }
        }
        7 => {
            let (tick, resources, _, _, steward): Body7 =
                save::decode(&bytes[12..]).expect("frozen v7 body");
            Expected {
                tick,
                resources: resources.into(),
                steward: Some(steward),
                content: ContentState::default(),
            }
        }
        8 => {
            let envelope: Envelope = save::decode(&bytes[12..]).expect("v8 envelope");
            assert_eq!(
                envelope.revision,
                "catalog-v1-fnv1a64:12e812cb24fbd015:38733"
            );
            let (tick, resources, _, _, steward, (completed, workshop_cutters)): Body8 =
                save::decode(&envelope.payload).expect("frozen v8 body");
            Expected {
                tick,
                resources: resources.into(),
                steward: Some(steward),
                content: ContentState {
                    completed,
                    workshop_cutters,
                    ..Default::default()
                },
            }
        }
        9 => {
            let envelope: Envelope = save::decode(&bytes[12..]).expect("v9 envelope");
            assert_eq!(
                envelope.revision,
                "catalog-v1-fnv1a64:10279805eb0560ff:44709"
            );
            let (tick, resources, _, _, steward, (completed, workshop_cutters, illness_origins)): Body9 = save::decode(&envelope.payload).expect("frozen v9 body");
            Expected {
                tick,
                resources,
                steward: Some(steward),
                content: ContentState {
                    completed,
                    workshop_cutters,
                    illness_origins,
                    ..Default::default()
                },
            }
        }
        other => panic!("unregistered historical version {other}"),
    }
}
