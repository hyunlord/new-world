//! Read-only domestic animals, physical pen capacity and immutable event evidence.
use godot::prelude::*;
use sim_core::colony::content::installed_or_bundled;
use sim_core::colony::livestock::{current_age, LivestockDeathReason, LivestockEvent};
use sim_core::colony::{Command, CommandResult, RejectReason};
use sim_engine::{SimEngine, SimResources};
use sim_systems::runtime::livestock::pen_layout;

/// One domestic animal in stable LivestockId order; never an AgentId or wild AnimalId.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct LivestockRow {
    /// Stable domestic identity, distinct from wild and human IDs.
    pub id: u32,
    /// Selected registry species code.
    pub kind: u32,
    /// Source building site, including a damaged enclosure.
    pub pen: u32,
    /// Current integer age in days.
    pub age: u32,
    /// Biological sex used by the lifecycle.
    pub female: bool,
    /// Consecutive days without sufficient forage.
    pub hunger: u32,
    /// Whether a milk job is currently eligible.
    pub milk_ready: bool,
    /// Stable display position inside the actual pen.
    pub tile: (u32, u32),
}
/// Physical pen facts distinguish grazing area from forage left after today's meal.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PenRow {
    /// Authored building site identity.
    pub site: u32,
    /// Authored occupancy limit.
    pub capacity: u16,
    /// Current domestic population.
    pub count: usize,
    /// Interior cells satisfying the permanent grazing predicate.
    pub grass_tiles: usize,
    /// Grazeable cells whose regrowth deadline has elapsed.
    pub available_grass: usize,
    /// Sum of the current population's authored daily requirement.
    pub grass_needed: u64,
    /// Whether the physical enclosure remains closed and complete.
    pub intact: bool,
}
/// Side-effect-free projection used by the renderer, chapters and cards.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct LivestockRows {
    /// False in supported baseline worlds.
    pub enabled: bool,
    /// Authoritative current simulation tick.
    pub tick: u64,
    /// Persisted livestock change counter.
    pub version: u64,
    /// Living animals in stable identity order.
    pub animals: Vec<LivestockRow>,
    /// Authored pen sites in stable site order.
    pub pens: Vec<PenRow>,
    /// Durable first successful capture chronicle entry.
    pub first_capture: Option<u32>,
    /// Durable first born identity, even after its removal.
    pub first_kid: Option<u32>,
}
/// Collect current animals and real authored enclosures without ticking or drawing RNG.
pub fn collect_livestock(res: &SimResources) -> LivestockRows {
    if res.rules.colony.is_none() {
        return LivestockRows::default();
    }
    let mut out = LivestockRows {
        enabled: true,
        tick: res.current_tick,
        version: res.livestock.version,
        first_capture: res.livestock.first_capture,
        first_kid: res.livestock.first_kid_id,
        ..LivestockRows::default()
    };
    let catalog = installed_or_bundled();
    let day =
        u32::try_from(sim_systems::runtime::resource_regen::day_index(res)).unwrap_or(u32::MAX);
    for site in res.colony.buildings.sites.values() {
        let Some(template) = catalog.building_templates().get(&site.template) else {
            continue;
        };
        if template.function.livestock_capacity == 0 {
            continue;
        }
        let animals: Vec<_> = res
            .livestock
            .animals
            .values()
            .filter(|a| a.pen_site == site.id)
            .collect();
        let layout = pen_layout(res, site.id).ok();
        let grass_tiles = layout.as_ref().map_or(0, |p| p.interior.len());
        let available_grass = layout.as_ref().map_or(0, |p| {
            p.interior
                .iter()
                .filter(|t| {
                    res.livestock
                        .grass
                        .get(t)
                        .is_none_or(|g| day >= g.next_regrow_day)
                })
                .count()
        });
        let grass_needed = animals
            .iter()
            .map(|a| {
                a.species
                    .definition()
                    .tameable
                    .as_ref()
                    .map_or(0, |p| u64::from(p.graze_tiles_per_head))
            })
            .sum();
        out.pens.push(PenRow {
            site: site.id,
            capacity: template.function.livestock_capacity,
            count: animals.len(),
            grass_tiles,
            available_grass,
            grass_needed,
            intact: layout.is_some(),
        });
        for (index, animal) in animals.into_iter().enumerate() {
            let tile = layout
                .as_ref()
                .and_then(|p| p.interior.get(index % p.interior.len().max(1)))
                .copied()
                .unwrap_or((
                    site.anchor.0.saturating_add(1),
                    site.anchor.1.saturating_add(1),
                ));
            out.animals.push(LivestockRow {
                id: animal.id,
                kind: animal.species.code(),
                pen: site.id,
                age: current_age(animal, res.current_tick, res.ticks_per_day),
                female: animal.sex.female,
                hunger: animal.hunger_days,
                milk_ready: animal.milk_ready,
                tile,
            });
        }
    }
    out.animals.sort_by_key(|a| a.id);
    out
}
fn ints(values: impl Iterator<Item = i64>) -> PackedInt64Array {
    PackedInt64Array::from(values.collect::<Vec<_>>().as_slice())
}
/// Marshal the additive schema; absent first identities use -1.
pub(super) fn livestock_to_dict(v: &LivestockRows) -> VarDictionary {
    let mut d = VarDictionary::new();
    d.set("schema_version", 1);
    d.set("enabled", v.enabled);
    d.set("tick", i64::try_from(v.tick).unwrap_or(i64::MAX));
    d.set("version", i64::try_from(v.version).unwrap_or(i64::MAX));
    d.set("first_capture", v.first_capture.map_or(-1, i64::from));
    d.set("first_kid_id", v.first_kid.map_or(-1, i64::from));
    d.set("ids", ints(v.animals.iter().map(|a| i64::from(a.id))));
    d.set("kinds", ints(v.animals.iter().map(|a| i64::from(a.kind))));
    d.set(
        "pen_sites",
        ints(v.animals.iter().map(|a| i64::from(a.pen))),
    );
    d.set("age_days", ints(v.animals.iter().map(|a| i64::from(a.age))));
    d.set(
        "female",
        ints(v.animals.iter().map(|a| i64::from(a.female))),
    );
    d.set(
        "hunger_days",
        ints(v.animals.iter().map(|a| i64::from(a.hunger))),
    );
    d.set(
        "milk_ready",
        ints(v.animals.iter().map(|a| i64::from(a.milk_ready))),
    );
    d.set("xs", ints(v.animals.iter().map(|a| i64::from(a.tile.0))));
    d.set("ys", ints(v.animals.iter().map(|a| i64::from(a.tile.1))));
    let mut pens: Array<VarDictionary> = Array::new();
    for p in &v.pens {
        let mut row = VarDictionary::new();
        row.set("site", i64::from(p.site));
        row.set("capacity", i64::from(p.capacity));
        row.set("count", p.count as i64);
        row.set("grass_tiles", p.grass_tiles as i64);
        row.set("available_grass", p.available_grass as i64);
        row.set(
            "grass_needed",
            i64::try_from(p.grass_needed).unwrap_or(i64::MAX),
        );
        row.set("intact", p.intact);
        row.set(
            "overstocked",
            p.grass_needed > p.grass_tiles as u64 || p.count > usize::from(p.capacity),
        );
        pens.push(&row);
    }
    d.set("pens", pens);
    d
}
/// Checked player command; native submit owns provenance and acceptance.
pub fn submit_capture(engine: &mut SimEngine, id: i64) -> CommandResult {
    u32::try_from(id).ok().map_or_else(
        || CommandResult::rejected(RejectReason::Unknown),
        |animal| sim_systems::commands::submit(engine, Command::DesignateCapture { animal }),
    )
}
/// Checked player slaughter command; never directly removes an animal.
pub fn submit_slaughter(engine: &mut SimEngine, id: i64) -> CommandResult {
    u32::try_from(id).ok().map_or_else(
        || CommandResult::rejected(RejectReason::Unknown),
        |animal| sim_systems::commands::submit(engine, Command::DesignateSlaughter { animal }),
    )
}
/// Terminal immutable evidence survives source removal, save/load and dead parents.
pub(super) fn evidence_to_dict(res: &SimResources, index: Option<u32>) -> VarDictionary {
    let mut d = VarDictionary::new();
    let Some(w) = index.and_then(|i| res.livestock.witnesses.get(&i)) else {
        return d;
    };
    d.set("livestock", i64::from(w.livestock));
    d.set("species", i64::from(w.species.code()));
    d.set("pen_site", i64::from(w.pen_site));
    match &w.event {
        LivestockEvent::Captured {
            wild,
            attempt,
            knowledge,
            origin,
            ..
        } => {
            d.set("event", "captured");
            d.set("wild", i64::from(*wild));
            d.set("attempt", i64::from(*attempt));
            d.set("knowledge", i64::from(knowledge.knowledge));
            d.set("proficiency", i64::from(knowledge.proficiency));
            d.set("required", i64::from(knowledge.required));
            d.set("source_record", i64::from(origin.source_record));
            d.set("steward_rule", origin.steward_rule.map_or(-1, i64::from));
        }
        LivestockEvent::Born { mother, father } => {
            d.set("event", "born");
            d.set("mother", i64::from(*mother));
            d.set("father", i64::from(*father));
        }
        LivestockEvent::Milk { knowledge, product } => {
            d.set("event", "milk");
            d.set("knowledge", i64::from(knowledge.knowledge));
            d.set("proficiency", i64::from(knowledge.proficiency));
            d.set("required", i64::from(knowledge.required));
            d.set("product", i64::from(product.item.kind.code()));
            d.set("count", i64::from(product.count));
        }
        LivestockEvent::Slaughter { origin, products } => {
            d.set("event", "slaughter");
            d.set("source_record", i64::from(origin.source_record));
            d.set("steward_rule", origin.steward_rule.map_or(-1, i64::from));
            d.set(
                "products",
                ints(products.iter().map(|p| i64::from(p.item.kind.code()))),
            );
            d.set("counts", ints(products.iter().map(|p| i64::from(p.count))));
        }
        LivestockEvent::Died { reason } => {
            d.set("event", "died");
            d.set(
                "reason",
                match reason {
                    LivestockDeathReason::Starvation => "starvation",
                    LivestockDeathReason::OldAge => "old_age",
                },
            );
        }
    }
    d
}
pub(super) fn livestock_chronicle_rows_to_dict(
    rows: super::chronicle::ChronicleRows,
    page: bool,
    res: &SimResources,
) -> VarDictionary {
    let evidence: Array<VarDictionary> = rows
        .index
        .iter()
        .map(|i| evidence_to_dict(res, u32::try_from(*i).ok()))
        .collect();
    let mut out = super::food::food_chronicle_rows_to_dict(rows, page, res);
    out.set("livestock_evidence", evidence);
    out
}
pub(super) fn livestock_why_to_dict(
    rows: &super::story::WhyRows,
    res: &SimResources,
) -> VarDictionary {
    let evidence: Array<VarDictionary> = rows
        .link
        .iter()
        .zip(&rows.code)
        .map(|(link, code)| {
            evidence_to_dict(
                res,
                if matches!(link, 0 | 4) {
                    u32::try_from(*code).ok()
                } else {
                    None
                },
            )
        })
        .collect();
    let mut out = super::food::food_why_to_dict(rows, res);
    out.set("livestock_evidence", evidence);
    out
}

#[cfg(test)]
mod tests {
    use super::*;
    use sim_core::chronicle::{CauseRef, Chronicle, ChronicleEntry, ChronicleKind};
    use sim_core::colony::wildlife::{Animal, AnimalState};

    #[test]
    fn baseline_livestock_read_is_empty_and_does_not_mutate() {
        let engine = SimEngine::new(4, 4, sim_core::MaterialRegistry::new());
        let before = engine.resources.livestock.clone();
        assert_eq!(
            collect_livestock(&engine.resources),
            LivestockRows::default()
        );
        assert_eq!(engine.resources.livestock, before);
    }

    #[test]
    fn wild_read_uses_capture_demographics_without_mutation() {
        let mut engine = super::super::engine_build::build_game_engine(0);
        engine.resources.colony.wildlife.animals.clear();
        let catalog = installed_or_bundled();
        let policy_species = catalog
            .animals()
            .find(|a| a.tameable.is_some())
            .expect("bundled tameable species");
        let kind = sim_core::colony::wildlife::AnimalKind::from_name(&policy_species.identity.id)
            .expect("species id");
        let animal = Animal {
            id: 7,
            kind,
            hp: 20,
            state: AnimalState::Wander,
            pos: (3, 4),
            pack: None,
            born_tick: 11,
            path: Vec::new(),
            goal: None,
            home: None,
        };
        engine.resources.current_tick = 3700;
        engine
            .resources
            .colony
            .wildlife
            .animals
            .insert(7, animal.clone());
        let before = engine.resources.colony.wildlife.clone();
        let rows = super::super::story::collect_animals(&engine.resources);
        let expected = sim_core::colony::livestock_demographics::wild_demographics(
            0,
            &animal,
            policy_species.tameable.as_ref().expect("policy"),
            3700,
            engine.resources.ticks_per_day,
        );
        assert_eq!(rows.id, [7]);
        assert_eq!(rows.age_days, [i64::from(expected.age_days)]);
        assert_eq!(rows.female, [i64::from(expected.sex.female)]);
        assert_eq!(engine.resources.colony.wildlife, before);
    }

    #[test]
    fn livestock_why_is_terminal_and_never_a_person_or_recursive_entry() {
        let mut chronicle = Chronicle::new();
        let entry = chronicle.push_caused(
            ChronicleEntry::new(12, ChronicleKind::LivestockBorn),
            Some(CauseRef::Livestock { witness: 0 }),
        );
        let rows = super::super::story::collect_chronicle_why(&chronicle, i64::from(entry));
        assert_eq!(rows.link, [0, 4]);
        assert_eq!(rows.code, [i64::from(entry), 0]);
        assert_eq!(rows.subject, [-1, -1]);
        assert_eq!(rows.other, [-1, -1]);
        assert_eq!(
            super::super::story::cause_link(&chronicle, entry),
            (4, -1, 0)
        );
    }
}
