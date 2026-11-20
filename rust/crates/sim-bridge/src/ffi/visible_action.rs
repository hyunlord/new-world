//! Pack read-only visible action rows and the bounded stream of physical transfer cues.

use super::signed_or_none;
use godot::prelude::*;
use sim_core::colony::{
    content::installed_or_bundled,
    visible_action::{VisibleActionCues, VisibleActionRow, VisibleTarget},
};
use sim_engine::SimEngine;

#[derive(Default)]
struct TargetColumns {
    kinds: PackedInt32Array,
    xs: PackedInt32Array,
    ys: PackedInt32Array,
    ids: PackedInt64Array,
}

impl TargetColumns {
    fn push(&mut self, target: Option<VisibleTarget>) {
        self.kinds
            .push(target.map_or(-1, |t| i32::from(t.kind as u8)));
        let (x, y) = signed_tile(target.and_then(|t| t.tile));
        self.xs.push(x);
        self.ys.push(y);
        self.ids
            .push(signed_or_none(target.and_then(|t| t.object_id)));
    }

    fn target_to_dict(self, out: &mut VarDictionary) {
        out.set("target_kinds", self.kinds);
        out.set("target_xs", self.xs);
        out.set("target_ys", self.ys);
        out.set("target_ids", self.ids);
    }
}

fn signed_tile(tile: Option<(u32, u32)>) -> (i32, i32) {
    tile.map_or((-1, -1), |(x, y)| {
        (
            i32::try_from(x).unwrap_or(i32::MAX),
            i32::try_from(y).unwrap_or(i32::MAX),
        )
    })
}

fn signed_tick(tick: Option<u64>) -> i64 {
    tick.map_or(-1, |value| i64::try_from(value).unwrap_or(i64::MAX))
}

pub(super) fn visible_action_snapshot_to_dict(
    rows: &[VisibleActionRow],
    tick: u64,
    revision: i64,
) -> VarDictionary {
    let visuals = installed_or_bundled().action_visuals();
    let mut agent_ids = PackedInt64Array::new();
    let mut actions = PackedByteArray::new();
    let mut phases = PackedByteArray::new();
    let mut goal_xs = PackedInt32Array::new();
    let mut goal_ys = PackedInt32Array::new();
    let mut targets = TargetColumns::default();
    let mut partner_ids = PackedInt64Array::new();
    let mut held_items = PackedInt64Array::new();
    let mut held_materials = PackedInt64Array::new();
    let mut held_quantities = PackedInt32Array::new();
    let mut tool_items = PackedInt64Array::new();
    let mut tool_materials = PackedInt64Array::new();
    let mut tool_qualities = PackedInt32Array::new();
    let mut recipes = PackedInt64Array::new();
    let mut started_at_ticks = PackedInt64Array::new();
    let mut until_ticks = PackedInt64Array::new();
    let mut visual_keys = PackedStringArray::new();
    let mut fallback_keys = PackedStringArray::new();

    for row in rows {
        agent_ids.push(signed_or_none(Some(row.agent_id)));
        actions.push(row.action as u8);
        phases.push(row.phase as u8);
        let (x, y) = signed_tile(row.goal);
        goal_xs.push(x);
        goal_ys.push(y);
        targets.push(row.target);
        partner_ids.push(signed_or_none(row.partner));
        held_items.push(row.held.map_or(-1, |held| i64::from(held.item.code())));
        held_materials.push(
            row.held
                .and_then(|held| held.material)
                .map_or(-1, |material| i64::from(material.raw())),
        );
        held_quantities.push(row.held.map_or(-1, |held| i32::from(held.qty)));
        tool_items.push(row.tool.map_or(-1, |tool| i64::from(tool.item.kind.code())));
        tool_materials.push(
            row.tool
                .and_then(|tool| tool.item.material)
                .map_or(-1, |material| i64::from(material.raw())),
        );
        tool_qualities.push(row.tool.map_or(-1, |tool| i32::from(tool.item.quality)));
        recipes.push(row.recipe.map_or(-1, |recipe| i64::from(recipe.code())));
        started_at_ticks.push(signed_tick(row.started_at_tick));
        until_ticks.push(signed_tick(row.until_tick));
        let visual = visuals.resolve(row.action, row.phase, row.recipe);
        visual_keys.push(&GString::from(&visual.key));
        fallback_keys.push(&GString::from(&visual.fallback));
    }

    let mut out = VarDictionary::new();
    out.set("tick", signed_tick(Some(tick)));
    out.set("revision", revision);
    out.set("agent_ids", agent_ids);
    out.set("actions", actions);
    out.set("phases", phases);
    out.set("goal_xs", goal_xs);
    out.set("goal_ys", goal_ys);
    targets.target_to_dict(&mut out);
    out.set("partner_ids", partner_ids);
    out.set("held_items", held_items);
    out.set("held_materials", held_materials);
    out.set("held_quantities", held_quantities);
    out.set("tool_items", tool_items);
    out.set("tool_materials", tool_materials);
    out.set("tool_qualities", tool_qualities);
    out.set("recipes", recipes);
    out.set("started_at_ticks", started_at_ticks);
    out.set("until_ticks", until_ticks);
    out.set("visual_keys", visual_keys);
    out.set("fallback_keys", fallback_keys);
    out
}

pub(super) fn visible_action_cues_to_dict(
    engine: &SimEngine,
    revision: i64,
    since_tick: i64,
) -> VarDictionary {
    let visuals = installed_or_bundled().action_visuals();
    let since_tick = u64::try_from(since_tick).ok();
    let recent: &VisibleActionCues = &engine.resources.visible_action_cues;
    let mut ticks = PackedInt64Array::new();
    let mut sequences = PackedInt64Array::new();
    let mut agent_ids = PackedInt64Array::new();
    let mut kinds = PackedByteArray::new();
    let mut targets = TargetColumns::default();
    let mut items = PackedInt64Array::new();
    let mut materials = PackedInt64Array::new();
    let mut quantities = PackedInt32Array::new();
    let mut visual_keys = PackedStringArray::new();
    let mut fallback_keys = PackedStringArray::new();

    for cue in recent.since_tick(since_tick) {
        ticks.push(signed_tick(Some(cue.tick)));
        sequences.push(i64::from(cue.sequence));
        agent_ids.push(signed_or_none(Some(cue.agent_id)));
        kinds.push(cue.kind as u8);
        targets.push(Some(cue.target));
        items.push(i64::from(cue.item.item.code()));
        materials.push(
            cue.item
                .material
                .map_or(-1, |material| i64::from(material.raw())),
        );
        quantities.push(i32::from(cue.item.qty));
        let visual = visuals.cue(cue.kind);
        visual_keys.push(&GString::from(&visual.key));
        fallback_keys.push(&GString::from(&visual.fallback));
    }

    let mut out = VarDictionary::new();
    out.set("tick", signed_tick(Some(engine.current_tick())));
    out.set("revision", revision);
    out.set(
        "evicted_through_tick",
        signed_tick(recent.evicted_through_tick()),
    );
    out.set("ticks", ticks);
    out.set("sequences", sequences);
    out.set("agent_ids", agent_ids);
    out.set("kinds", kinds);
    targets.target_to_dict(&mut out);
    out.set("items", items);
    out.set("materials", materials);
    out.set("quantities", quantities);
    out.set("visual_keys", visual_keys);
    out.set("fallback_keys", fallback_keys);
    out
}
