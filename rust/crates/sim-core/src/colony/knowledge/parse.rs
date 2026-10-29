//! Raw knowledge validation and local catalog reference resolution; no bootstrap calls.
use super::*;
use crate::colony::content::Catalog;
use crate::colony::{Designation, SkillKind};
use serde::{Deserialize, Serialize};

#[derive(Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
struct FileRaw {
    schema_version: u32,
    absent_markers: Vec<(String, u16)>,
    knowledge: Vec<NodeRaw>,
}

#[derive(Deserialize, Serialize)]
enum CondRaw {
    Stockpile(String, u16),
    Skill(String, u8),
    Knows(String, u16),
    ClayBankNear(u32),
}

#[derive(Deserialize, Serialize)]
enum GateRaw {
    Recipe(String),
    Blueprint(String),
    Workshop(String),
    Designation(String),
    Channel(String),
    Lore(String),
}

#[derive(Deserialize, Serialize)]
enum SeedRaw {
    None,
    All,
    Count(u8),
}

#[derive(Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
struct NodeRaw {
    id: String,
    lane: String,
    era: String,
    threshold: u16,
    prereqs: Vec<String>,
    difficulty: u8,
    discover: Vec<CondRaw>,
    gates: Vec<GateRaw>,
    marker_weight: u16,
    seed: SeedRaw,
    name_key: String,
}

fn designation_from_name(name: &str) -> Option<Designation> {
    match name {
        "chop" => Some(Designation::Chop),
        "quarry" => Some(Designation::Quarry),
        "dig" => Some(Designation::Dig),
        "deconstruct" => Some(Designation::Deconstruct),
        "clay_pit" => Some(Designation::ClayPit),
        "hunt" => Some(Designation::Hunt),
        "harvest" => Some(Designation::Harvest),
        _ => None,
    }
}

fn raw_file(text: &str) -> Result<FileRaw, String> {
    let file: FileRaw = ron::from_str(text).map_err(|e| format!("knowledge.ron: {e}"))?;
    if file.schema_version != 1 || file.knowledge.len() > MAX_NODES {
        return Err("knowledge.ron: expected schema 1 and at most 32 nodes".into());
    }
    Ok(file)
}

/// Append extension records without replacing or reordering the base prefix.
pub(crate) fn merge_sources(base: &str, extension: &str) -> Result<String, String> {
    let mut base = raw_file(base)?;
    let extension = raw_file(extension)?;
    base.knowledge.extend(extension.knowledge);
    base.absent_markers.extend(extension.absent_markers);
    if base.knowledge.len() > MAX_NODES {
        return Err("knowledge.ron: appended map exceeds 32 nodes".into());
    }
    ron::to_string(&base).map_err(|e| format!("knowledge.ron merge: {e}"))
}

/// Resolve raw names exclusively against the supplied, locally built catalog.
pub(crate) fn parse_with_catalog(text: &str, catalog: &Catalog) -> Result<KnowledgeDefs, String> {
    let file = raw_file(text)?;
    if file.knowledge.is_empty() {
        return Err("knowledge.ron: the selected map must contain nodes".into());
    }
    let names: Vec<&str> = file.knowledge.iter().map(|n| n.id.as_str()).collect();
    let id_of = |name: &str| -> Result<KnowledgeId, String> {
        names
            .iter()
            .position(|n| *n == name)
            .and_then(|i| u8::try_from(i).ok())
            .ok_or_else(|| format!("knowledge.ron: unknown knowledge {name}"))
    };
    let mut nodes = Vec::with_capacity(names.len());
    for (i, raw) in file.knowledge.iter().enumerate() {
        if raw.id.is_empty()
            || names[..i].contains(&raw.id.as_str())
            || raw.threshold == 0
            || raw.threshold > 1000
            || !(1..=3).contains(&raw.difficulty)
        {
            return Err(format!(
                "knowledge.ron: invalid or duplicate node {}",
                raw.id
            ));
        }
        let discover = raw
            .discover
            .iter()
            .map(|c| {
                Ok(match c {
                    CondRaw::Stockpile(item, n) => Cond::Stockpile(
                        catalog
                            .item_by_name(item)
                            .ok_or_else(|| format!("unknown item {item}"))?,
                        *n,
                    ),
                    CondRaw::Skill(skill, level) => Cond::Skill(
                        SkillKind::from_name(skill)
                            .ok_or_else(|| format!("unknown skill {skill}"))?,
                        *level,
                    ),
                    CondRaw::Knows(node, p) => Cond::Knows(id_of(node)?, *p),
                    CondRaw::ClayBankNear(d) => Cond::ClayBankNear(*d),
                })
            })
            .collect::<Result<Vec<_>, String>>()?;
        let gates = raw
            .gates
            .iter()
            .map(|gate| {
                Ok(match gate {
                    GateRaw::Recipe(name) => Gate::Recipe(
                        catalog
                            .recipe_by_name(name)
                            .ok_or_else(|| format!("unknown recipe {name}"))?,
                    ),
                    GateRaw::Blueprint(name) => Gate::Blueprint(
                        catalog
                            .blueprint_by_name(name)
                            .ok_or_else(|| format!("unknown blueprint {name}"))?,
                    ),
                    GateRaw::Workshop(name) => Gate::Workshop(
                        catalog
                            .blueprint_by_name(name)
                            .ok_or_else(|| format!("unknown workshop {name}"))?,
                    ),
                    GateRaw::Designation(name) => Gate::Designation(
                        designation_from_name(name)
                            .ok_or_else(|| format!("unknown designation {name}"))?,
                    ),
                    GateRaw::Channel(name) => Gate::Channel(match name.as_str() {
                        "apprenticeship" => Channel::Apprenticeship,
                        "oral" => Channel::Oral,
                        _ => return Err(format!("unknown channel {name}")),
                    }),
                    GateRaw::Lore(name) => Gate::Lore(match name.as_str() {
                        "hearth" => Lore::Hearth,
                        "forage" => Lore::Forage,
                        "water" => Lore::Water,
                        _ => return Err(format!("unknown lore {name}")),
                    }),
                })
            })
            .collect::<Result<Vec<_>, String>>()?;
        nodes.push(KnowledgeDef {
            id: u8::try_from(i).map_err(|e| e.to_string())?,
            name: raw.id.clone(),
            lane: Lane::from_name(&raw.lane).ok_or_else(|| format!("unknown lane {}", raw.lane))?,
            era: Era::from_name(&raw.era).ok_or_else(|| format!("unknown era {}", raw.era))?,
            threshold: raw.threshold,
            prereqs: raw
                .prereqs
                .iter()
                .map(|p| id_of(p))
                .collect::<Result<_, _>>()?,
            difficulty: raw.difficulty,
            discover,
            gates,
            marker_weight: raw.marker_weight,
            seed: match raw.seed {
                SeedRaw::None => Seeding::Nobody,
                SeedRaw::All => Seeding::All,
                SeedRaw::Count(n) => Seeding::Count(n),
            },
            name_key: raw.name_key.clone(),
        });
    }
    let defs = KnowledgeDefs {
        nodes,
        absent_markers: file.absent_markers,
    };
    validate_graph(&defs)?;
    Ok(defs)
}

fn validate_graph(defs: &KnowledgeDefs) -> Result<(), String> {
    let mut complete = 0u32;
    while complete.count_ones() < u32::try_from(defs.nodes.len()).unwrap_or(u32::MAX) {
        let before = complete;
        for node in &defs.nodes {
            if node.prereqs.iter().all(|id| complete & bit(*id) != 0) {
                complete |= bit(node.id);
            }
        }
        if before == complete {
            return Err("knowledge.ron: cyclic prerequisites".into());
        }
    }
    Ok(())
}
