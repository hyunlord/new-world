//! Pass 20 (`game_v25`): read-only village relations and porters for the art
//! line. Pure collectors over `SimResources::settlement_relations` and the
//! world's `PorterTrip` markers; the `#[func]` wrappers in `world_node.rs`
//! only marshal. Both are empty in every scene without the relations rule.
//! (Until the art line reads `get_porter_snapshot`, a porter appears in the
//! existing snapshots as `Seeking { Agent(itself) }`.)

use godot::prelude::*;
use sim_core::components::{Agent, AgentId, PorterTrip, SettlementRelations};

/// One village pair (lower id first).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct RelationRow {
    /// Lower settlement id.
    pub a: u32,
    /// Higher settlement id.
    pub b: u32,
    /// Relation in −1,000..=1,000.
    pub value: i32,
    /// Estranged: no porters, bushes on the other's side avoided.
    pub estranged: bool,
    /// Kin base the value drifts toward.
    pub base: i32,
}

/// Every pair in key order.
pub fn collect_settlement_relations(relations: &SettlementRelations) -> Vec<RelationRow> {
    relations
        .pairs
        .iter()
        .map(|((a, b), state)| RelationRow {
            a: *a,
            b: *b,
            value: state.value,
            estranged: state.estranged,
            base: state.base,
        })
        .collect()
}

/// One porter on its trip.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct PorterRow {
    /// Entity bits, matching `get_agent_snapshot().ids`.
    pub entity_bits: u64,
    /// Simulation identity, used only to order rows.
    pub agent_id: AgentId,
    /// The trip.
    pub trip: PorterTrip,
}

/// Every porter, by AgentId.
pub fn collect_porter_snapshot(world: &hecs::World) -> Vec<PorterRow> {
    let mut rows: Vec<PorterRow> = world
        .query::<(&Agent, &PorterTrip)>()
        .iter()
        .map(|(entity, (agent, trip))| PorterRow {
            entity_bits: entity.to_bits().get(),
            agent_id: agent.id,
            trip: *trip,
        })
        .collect();
    rows.sort_by_key(|row| (row.agent_id, row.entity_bits));
    rows
}

/// `{a, b, value, estranged, base}` packed columns.
pub(super) fn relation_rows_to_dict(rows: &[RelationRow]) -> VarDictionary {
    let mut a = PackedInt64Array::new();
    let mut b = PackedInt64Array::new();
    let mut value = PackedInt32Array::new();
    let mut estranged = PackedByteArray::new();
    let mut base = PackedInt32Array::new();
    for row in rows {
        a.push(i64::from(row.a));
        b.push(i64::from(row.b));
        value.push(row.value);
        estranged.push(u8::from(row.estranged));
        base.push(row.base);
    }
    let mut dict = VarDictionary::new();
    dict.set("a", a);
    dict.set("b", b);
    dict.set("value", value);
    dict.set("estranged", estranged);
    dict.set("base", base);
    dict
}

/// `{ids, from, to, kind (0 food / 1 wood), amount, stage (0 out / 1 back)}`.
pub(super) fn porter_rows_to_dict(rows: &[PorterRow]) -> VarDictionary {
    let mut ids = PackedInt64Array::new();
    let mut from = PackedInt64Array::new();
    let mut to = PackedInt64Array::new();
    let mut kind = PackedByteArray::new();
    let mut amount = PackedInt32Array::new();
    let mut stage = PackedByteArray::new();
    for row in rows {
        ids.push(i64::from_ne_bytes(row.entity_bits.to_ne_bytes()));
        from.push(i64::from(row.trip.from));
        to.push(i64::from(row.trip.to));
        kind.push(row.trip.kind.as_u8());
        amount.push(i32::try_from(row.trip.amount).unwrap_or(i32::MAX));
        stage.push(row.trip.stage.as_u8());
    }
    let mut dict = VarDictionary::new();
    dict.set("ids", ids);
    dict.set("from", from);
    dict.set("to", to);
    dict.set("kind", kind);
    dict.set("amount", amount);
    dict.set("stage", stage);
    dict
}

#[cfg(test)]
mod tests {
    use super::*;
    use sim_core::components::{PorterKind, PorterStage, RelationState};

    #[test]
    fn relations_come_in_key_order_with_every_column() {
        let mut relations = SettlementRelations::default();
        relations.pairs.insert((3, 7), RelationState { value: -320, estranged: true, base: 150 });
        relations.pairs.insert((1, 3), RelationState { value: 340, estranged: false, base: 300 });
        let rows = collect_settlement_relations(&relations);
        assert_eq!(
            rows,
            vec![
                RelationRow { a: 1, b: 3, value: 340, estranged: false, base: 300 },
                RelationRow { a: 3, b: 7, value: -320, estranged: true, base: 150 },
            ]
        );
        assert!(collect_settlement_relations(&SettlementRelations::default()).is_empty());
    }

    #[test]
    fn porters_come_by_agent_id_with_entity_bits() {
        let mut world = hecs::World::new();
        let trip = |kind, stage| PorterTrip { from: 1, to: 2, kind, amount: 7, stage, depart_tick: 480 };
        let late = world.spawn((Agent { id: 9 }, trip(PorterKind::Wood, PorterStage::Return)));
        let early = world.spawn((Agent { id: 4 }, trip(PorterKind::Food, PorterStage::Outbound)));
        world.spawn((Agent { id: 5 },));
        let rows = collect_porter_snapshot(&world);
        assert_eq!(rows.len(), 2);
        assert_eq!((rows[0].agent_id, rows[0].entity_bits), (4, early.to_bits().get()));
        assert_eq!((rows[1].agent_id, rows[1].entity_bits), (9, late.to_bits().get()));
        assert_eq!((rows[0].trip.kind.as_u8(), rows[0].trip.stage.as_u8()), (0, 0));
        assert_eq!((rows[1].trip.kind.as_u8(), rows[1].trip.stage.as_u8()), (1, 1));
        assert!(collect_porter_snapshot(&hecs::World::new()).is_empty());
    }
}
