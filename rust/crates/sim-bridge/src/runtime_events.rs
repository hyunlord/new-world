use godot::prelude::{Array, VarDictionary};
use sim_engine::GameEvent;

pub(crate) const EVENT_TYPE_ID_TICK_COMPLETED: i32 = 1;
pub(crate) const EVENT_TYPE_ID_SIMULATION_PAUSED: i32 = 2;
pub(crate) const EVENT_TYPE_ID_SIMULATION_RESUMED: i32 = 3;
pub(crate) const EVENT_TYPE_ID_SPEED_CHANGED: i32 = 4;
pub(crate) const EVENT_TYPE_ID_ENTITY_SPAWNED: i32 = 10;
pub(crate) const EVENT_TYPE_ID_ENTITY_DIED: i32 = 11;
pub(crate) const EVENT_TYPE_ID_ENTITY_REMOVED: i32 = 12;
pub(crate) const EVENT_TYPE_ID_STRESS_CHANGED: i32 = 20;
pub(crate) const EVENT_TYPE_ID_MENTAL_BREAK_TRIGGERED: i32 = 21;
pub(crate) const EVENT_TYPE_ID_JOB_ASSIGNED: i32 = 30;
pub(crate) const EVENT_TYPE_ID_RESOURCE_GATHERED: i32 = 31;
pub(crate) const EVENT_TYPE_ID_BUILDING_CONSTRUCTED: i32 = 32;
pub(crate) const EVENT_TYPE_ID_MIGRATION_OCCURRED: i32 = 33;
pub(crate) const EVENT_TYPE_ID_SETTLEMENT_FOUNDED: i32 = 34;
pub(crate) const EVENT_TYPE_ID_FAMILY_FORMED: i32 = 35;
pub(crate) const EVENT_TYPE_ID_BIRTH_OCCURRED: i32 = 36;
pub(crate) const EVENT_TYPE_ID_SOCIAL_EVENT_OCCURRED: i32 = 50;
pub(crate) const EVENT_TYPE_ID_RELATIONSHIP_CHANGED: i32 = 51;
pub(crate) const EVENT_TYPE_ID_TECH_DISCOVERED: i32 = 60;
pub(crate) const EVENT_TYPE_ID_ERA_ADVANCED: i32 = 61;
pub(crate) const EVENT_TYPE_ID_GENERIC: i32 = 9000;

pub(crate) fn game_event_type_id(event: &GameEvent) -> i32 {
    match event {
        GameEvent::TickCompleted { .. } => EVENT_TYPE_ID_TICK_COMPLETED,
        GameEvent::SimulationPaused => EVENT_TYPE_ID_SIMULATION_PAUSED,
        GameEvent::SimulationResumed => EVENT_TYPE_ID_SIMULATION_RESUMED,
        GameEvent::SpeedChanged { .. } => EVENT_TYPE_ID_SPEED_CHANGED,
        GameEvent::EntitySpawned { .. } => EVENT_TYPE_ID_ENTITY_SPAWNED,
        GameEvent::EntityDied { .. } => EVENT_TYPE_ID_ENTITY_DIED,
        GameEvent::EntityRemoved { .. } => EVENT_TYPE_ID_ENTITY_REMOVED,
        GameEvent::StressChanged { .. } => EVENT_TYPE_ID_STRESS_CHANGED,
        GameEvent::MentalBreakTriggered { .. } => EVENT_TYPE_ID_MENTAL_BREAK_TRIGGERED,
        GameEvent::JobAssigned { .. } => EVENT_TYPE_ID_JOB_ASSIGNED,
        GameEvent::ResourceGathered { .. } => EVENT_TYPE_ID_RESOURCE_GATHERED,
        GameEvent::BuildingConstructed { .. } => EVENT_TYPE_ID_BUILDING_CONSTRUCTED,
        GameEvent::MigrationOccurred { .. } => EVENT_TYPE_ID_MIGRATION_OCCURRED,
        GameEvent::SettlementFounded { .. } => EVENT_TYPE_ID_SETTLEMENT_FOUNDED,
        GameEvent::FamilyFormed { .. } => EVENT_TYPE_ID_FAMILY_FORMED,
        GameEvent::BirthOccurred { .. } => EVENT_TYPE_ID_BIRTH_OCCURRED,
        GameEvent::SocialEventOccurred { .. } => EVENT_TYPE_ID_SOCIAL_EVENT_OCCURRED,
        GameEvent::RelationshipChanged { .. } => EVENT_TYPE_ID_RELATIONSHIP_CHANGED,
        GameEvent::TechDiscovered { .. } => EVENT_TYPE_ID_TECH_DISCOVERED,
        GameEvent::EraAdvanced { .. } => EVENT_TYPE_ID_ERA_ADVANCED,
        _ => EVENT_TYPE_ID_GENERIC,
    }
}

fn game_event_tick(event: &GameEvent) -> i64 {
    match event {
        GameEvent::TickCompleted { tick } => *tick as i64,
        _ => -1,
    }
}

fn game_event_payload(event: &GameEvent) -> VarDictionary {
    let mut payload = VarDictionary::new();
    match event {
        GameEvent::TickCompleted { tick } => {
            payload.set("tick", *tick as i64);
        }
        GameEvent::EntityDied { entity_id, cause } => {
            payload.set("entity_id", entity_id.0 as i64);
            payload.set("cause", cause.clone());
        }
        GameEvent::EntitySpawned { entity_id } => {
            payload.set("entity_id", entity_id.0 as i64);
        }
        GameEvent::SpeedChanged { speed_index } => {
            payload.set("speed_index", *speed_index as i64);
        }
        GameEvent::EntityRemoved { entity_id } => {
            payload.set("entity_id", entity_id.0 as i64);
        }
        GameEvent::StressChanged { entity_id, stress } => {
            payload.set("entity_id", entity_id.0 as i64);
            payload.set("stress", *stress);
        }
        GameEvent::MentalBreakTriggered {
            entity_id,
            break_type,
        } => {
            payload.set("entity_id", entity_id.0 as i64);
            payload.set("break_type", break_type.clone());
        }
        GameEvent::JobAssigned { entity_id, job } => {
            payload.set("entity_id", entity_id.0 as i64);
            payload.set("job", job.clone());
        }
        GameEvent::ResourceGathered {
            entity_id,
            resource,
            amount,
        } => {
            payload.set("entity_id", entity_id.0 as i64);
            payload.set("resource", resource.clone());
            payload.set("amount", *amount);
        }
        GameEvent::BuildingConstructed {
            building_id,
            building_type,
        } => {
            payload.set("building_id", building_id.0 as i64);
            payload.set("building_type", building_type.clone());
        }
        GameEvent::MigrationOccurred {
            entity_id,
            from_settlement,
            to_settlement,
        } => {
            payload.set("entity_id", entity_id.0 as i64);
            payload.set("from_settlement", from_settlement.0 as i64);
            payload.set("to_settlement", to_settlement.0 as i64);
        }
        GameEvent::SettlementFounded { settlement_id } => {
            payload.set("settlement_id", settlement_id.0 as i64);
        }
        GameEvent::FamilyFormed { entity_a, entity_b } => {
            payload.set("entity_a", entity_a.0 as i64);
            payload.set("entity_b", entity_b.0 as i64);
        }
        GameEvent::BirthOccurred {
            parent_a,
            parent_b,
            child_id,
        } => {
            payload.set("parent_a", parent_a.0 as i64);
            payload.set("parent_b", parent_b.0 as i64);
            payload.set("child_id", child_id.0 as i64);
        }
        GameEvent::SocialEventOccurred {
            event_type,
            participants,
        } => {
            payload.set("event_type", event_type.clone());
            let mut out: Array<i64> = Array::new();
            for participant in participants {
                out.push(participant.0 as i64);
            }
            payload.set("participants", out);
        }
        GameEvent::RelationshipChanged {
            entity_a,
            entity_b,
            affinity,
        } => {
            payload.set("entity_a", entity_a.0 as i64);
            payload.set("entity_b", entity_b.0 as i64);
            payload.set("affinity", *affinity);
        }
        GameEvent::TechDiscovered {
            settlement_id,
            tech_id,
        } => {
            payload.set("settlement_id", settlement_id.0 as i64);
            payload.set("tech_id", tech_id.clone());
        }
        GameEvent::EraAdvanced {
            settlement_id,
            new_era,
        } => {
            payload.set("settlement_id", settlement_id.0 as i64);
            payload.set("new_era", new_era.clone());
        }
        _ => {}
    }
    payload
}

pub(crate) fn game_event_to_v2_dict(event: &GameEvent) -> VarDictionary {
    let mut dict = VarDictionary::new();
    dict.set("event_type_id", game_event_type_id(event));
    dict.set("event_name", event.name());
    dict.set("tick", game_event_tick(event));
    dict.set("payload", game_event_payload(event));
    dict
}
