//! Pure livestock requests through ordinary template/capture commands; no state writes or draws.
use hecs::World;
use sim_core::colony::building_site::BuildingStatus;
use sim_core::colony::content::installed_or_bundled;
use sim_core::colony::knowledge::{self, ContentStatus, Knowledge};
use sim_core::colony::{Command, WorkPriorities, WorkType};
use sim_core::components::{Agent, Child, Position};
use sim_engine::SimResources;

/// One deterministic request; capture offers share the ordinary job's live admission checks.
pub(super) fn plan(world: &World, res: &SimResources) -> Vec<Command> {
    if res.rules.colony.is_none() {
        return Vec::new();
    }
    let catalog = installed_or_bundled();
    let capture_mask = catalog
        .animals()
        .filter(|animal| animal.status == ContentStatus::Connected)
        .filter_map(|animal| animal.tameable.as_ref())
        .filter_map(|policy| catalog.knowledge_defs().id_of(&policy.capture_knowledge))
        .fold(
            sim_core::colony::knowledge::KnowledgeMask::ZERO,
            |mask, id| mask | knowledge::bit(id),
        );
    if capture_mask == 0 {
        return Vec::new();
    }
    let mut people: Vec<_> = world
        .query::<(&Agent, &Position, &Knowledge, Option<&WorkPriorities>)>()
        .without::<&Child>()
        .iter()
        .filter(|(_, (_, _, known, priorities))| {
            known.practice() & capture_mask != 0
                && priorities.is_none_or(|priorities| priorities.level(WorkType::Hunt) > 0)
        })
        .map(|(_, (agent, position, known, _))| {
            (agent.id, (position.x, position.y), known.practice())
        })
        .collect();
    people.sort_by_key(|person| person.0);
    if people.is_empty() {
        return Vec::new();
    }
    for &(_, position, practice) in &people {
        let region = res.nav.region_of(position);
        if let Some(offer) = crate::runtime::jobs::livestock::capture_candidate(
            res,
            practice,
            region,
            position,
            u32::MAX,
            false,
        ) {
            if !res.livestock.capture_orders.contains_key(&offer.wild) {
                return vec![Command::DesignateCapture { animal: offer.wild }];
            }
        }
    }
    // An existing or pending capacity-bearing template is already the requested pen.
    if res.colony.buildings.sites.values().any(|site| {
        site.status != BuildingStatus::Cancelled
            && catalog
                .building_templates()
                .get(&site.template)
                .is_some_and(|template| template.function.livestock_capacity > 0)
    }) {
        return Vec::new();
    }
    let Some(hearth) = super::super::jobs::hearth::hearth_tiles(res)
        .first()
        .copied()
    else {
        return Vec::new();
    };
    let mut templates: Vec<_> = catalog
        .building_templates()
        .buildings
        .iter()
        .filter(|template| {
            template.status == ContentStatus::Connected
                && template.function.livestock_capacity > 0
                && crate::runtime::building_templates::commands::known(
                    world,
                    &template.requires_knowledge,
                    catalog,
                )
        })
        .collect();
    templates.sort_by(|a, b| b.tier.cmp(&a.tier).then_with(|| a.id.cmp(&b.id)));
    if templates.is_empty() {
        return Vec::new();
    }
    let mut anchors: Vec<_> = (1..res.tile_grid.height.saturating_sub(1))
        .flat_map(|y| (1..res.tile_grid.width.saturating_sub(1)).map(move |x| (x, y)))
        .collect();
    anchors.sort_by_key(|&(x, y)| (x.abs_diff(hearth.0).max(y.abs_diff(hearth.1)), y, x));
    for template in templates {
        for &anchor in &anchors {
            if res.steward.vetoed(
                &sim_core::colony::steward::Target::Tile(anchor.0, anchor.1),
                res.current_tick,
            ) {
                continue;
            }
            for rotation in 0..4 {
                if crate::runtime::building_templates::commands::preview(
                    world,
                    res,
                    &template.id,
                    anchor,
                    rotation,
                    None,
                )
                .is_ok_and(|plan| {
                    plan.iter().all(|piece| {
                        let tile = (anchor.0 + piece.offset.0, anchor.1 + piece.offset.1);
                        crate::commands::camp_reaches(res, tile)
                    })
                }) {
                    return vec![Command::BuildTemplate {
                        id: template.id.clone(),
                        anchor,
                        rotation,
                        station: None,
                    }];
                }
            }
        }
    }
    Vec::new()
}

#[cfg(test)]
mod tests {
    use super::*;
    use sim_core::colony::knowledge::{KnowledgeEntry, Source};

    #[test]
    fn requests_real_capacity_template_once_and_never_in_baseline() {
        sim_core::colony::content::bootstrap(sim_core::colony::content::BootSelection::Bundled)
            .expect("catalog");
        let mut engine = crate::worldgen::build_world_with(
            &crate::worldgen::WorldGenConfig::colony(29),
            sim_core::rules::WorldRules::colony(29),
        );
        assert!(
            plan(&engine.world, &engine.resources).is_empty(),
            "unlearned taming cannot request a pen"
        );
        let all = Knowledge::from_entries(
            (0..knowledge::defs().nodes.len())
                .map(|id| KnowledgeEntry::new(id as u8, Source::Oral, 1000, 0))
                .collect(),
        );
        let people: Vec<_> = engine
            .world
            .query::<&Agent>()
            .without::<&Child>()
            .iter()
            .map(|(entity, _)| entity)
            .collect();
        for entity in people {
            engine
                .world
                .insert_one(entity, all.clone())
                .expect("knowledge");
        }
        let commands = plan(&engine.world, &engine.resources);
        assert_eq!(commands.len(), 1);
        let Command::BuildTemplate { ref id, .. } = commands[0] else {
            panic!("ordinary template request")
        };
        assert!(
            installed_or_bundled()
                .building_templates()
                .get(id)
                .expect("template")
                .function
                .livestock_capacity
                > 0
        );
        let command = Command::Steward {
            rule: 11,
            command: Box::new(commands[0].clone()),
        };
        assert!(
            crate::commands::apply_and_log(&mut engine.world, &mut engine.resources, command, 0)
                .accepted
        );
        assert!(
            plan(&engine.world, &engine.resources).is_empty(),
            "pending template must not duplicate"
        );
        engine.resources.rules.colony = None;
        assert!(
            plan(&engine.world, &engine.resources).is_empty(),
            "baseline has no requests"
        );
    }

    #[test]
    fn dairy_discovery_needs_real_capture_practice_and_fired_pots_without_milk_stock() {
        use sim_core::colony::{JobKind, Skills};
        sim_core::colony::content::bootstrap(sim_core::colony::content::BootSelection::Bundled)
            .expect("catalog");
        let mut res = SimResources::new(8, 8, sim_core::MaterialRegistry::new());
        res.rules = sim_core::rules::WorldRules::colony(0);
        let mut world = World::new();
        let dairy = knowledge::id("dairy").expect("dairy");
        let pot = knowledge::id("pot_firing").expect("pot firing");
        let mut known = Knowledge::default();
        known.set(pot, 400, Source::Discovered, 0);
        let entity = world.spawn((known, Skills::default()));
        let mask = knowledge::bit(dairy);
        assert_eq!(
            crate::runtime::knowledge::discover::conditions(&world, &res) & mask,
            0
        );
        world
            .get::<&mut Skills>(entity)
            .expect("skills")
            .complete(JobKind::Capture { animal: 1, pen: 1 });
        assert_ne!(
            crate::runtime::knowledge::discover::conditions(&world, &res) & mask,
            0,
            "capture and fired-pot witness suffice before any milk exists"
        );
        world.get::<&mut Knowledge>(entity).expect("knowledge").set(
            pot,
            399,
            Source::Discovered,
            0,
        );
        assert_eq!(
            crate::runtime::knowledge::discover::conditions(&world, &res) & mask,
            0,
            "ordinary pot threshold must still hold"
        );
        assert!(res.colony.ground_items.is_empty());
    }
}
