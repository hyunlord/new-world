//! Pure ecology-family loading after item identities and roles are resolved.
use super::*;
use serde::Deserialize;

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct AnimalFile {
    schema_version: u32,
    #[serde(default)]
    roles: BTreeMap<String, String>,
    animals: Vec<AnimalDef<String>>,
}
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct PressureFile {
    schema_version: u32,
    #[serde(default)]
    roles: BTreeMap<String, String>,
    pressures: Vec<PressureDef<String>>,
    schedule: Option<ScheduleRaw>,
}
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct ScheduleRaw {
    seasons: BTreeMap<u32, Vec<(u16, Option<String>)>>,
    due_order: Vec<String>,
    minor: Option<String>,
    minor_seasons: Vec<u32>,
    minor_draw_slot: u32,
}
fn parse<T: serde::de::DeserializeOwned>(
    map: &BTreeMap<String, Vec<u8>>,
    name: &str,
) -> Result<T, CatalogError> {
    ron::from_str(sources::text(map, name)?).map_err(|e| CatalogError::new(name, e.to_string()))
}
fn schema(n: u32, family: &str) -> Result<(), CatalogError> {
    if n == 1 {
        Ok(())
    } else {
        Err(CatalogError::new(family, "unsupported schema"))
    }
}
fn roles(
    base: &mut BTreeMap<String, String>,
    extra: BTreeMap<String, String>,
) -> Result<(), CatalogError> {
    for (key, value) in extra {
        if key.is_empty() || base.insert(key.clone(), value).is_some() {
            return Err(CatalogError::new(key, "duplicate/empty role"));
        }
    }
    Ok(())
}
pub(super) fn load(
    c: &mut Catalog,
    base: &BTreeMap<String, Vec<u8>>,
    ext: &BTreeMap<String, Vec<u8>>,
) -> Result<(), CatalogError> {
    let mut animals: AnimalFile = parse(base, "animals.ron")?;
    let mut pressures: PressureFile = parse(base, "pressures.ron")?;
    schema(animals.schema_version, "animals")?;
    schema(pressures.schema_version, "pressures")?;
    if ext.contains_key("animals.ron") {
        let extra: AnimalFile = parse(ext, "animals.ron")?;
        schema(extra.schema_version, "animals")?;
        roles(&mut animals.roles, extra.roles)?;
        animals.animals.extend(extra.animals);
    }
    if ext.contains_key("pressures.ron") {
        let extra: PressureFile = parse(ext, "pressures.ron")?;
        schema(extra.schema_version, "pressures")?;
        if extra.schedule.is_some() {
            return Err(CatalogError::new(
                "pressures",
                "extension cannot replace base schedule",
            ));
        }
        roles(&mut pressures.roles, extra.roles)?;
        pressures.pressures.extend(extra.pressures);
    }
    animals.animals.sort_by_key(|d| d.identity.code);
    pressures.pressures.sort_by_key(|d| d.identity.code);
    c.animal_index = Index::build("animals", animals.animals.iter().map(|d| &d.identity))?;
    c.pressure_index = Index::build("pressures", pressures.pressures.iter().map(|d| &d.identity))?;
    c.animals = animals
        .animals
        .into_iter()
        .map(|raw| resolve_ecology::animal(c, raw))
        .collect::<Result<_, _>>()?;
    c.pressures = pressures
        .pressures
        .into_iter()
        .map(|raw| resolve_ecology::pressure(c, raw))
        .collect::<Result<_, _>>()?;
    for (role, name) in animals.roles {
        let id = c
            .animal_by_name(&name)
            .ok_or_else(|| CatalogError::new(&role, "unknown animal role target"))?;
        c.animal_roles.insert(role, id);
    }
    for (role, name) in pressures.roles {
        let id = c
            .pressure_by_name(&name)
            .ok_or_else(|| CatalogError::new(&role, "unknown pressure role target"))?;
        c.pressure_roles.insert(role, id);
    }
    for (role, mechanic) in [
        ("default_grazer", AnimalMechanic::Grazer),
        ("default_predator", AnimalMechanic::Predator),
    ] {
        let id = c
            .animal_role(role)
            .ok_or_else(|| CatalogError::new(role, "missing required role"))?;
        if c.require_animal(id)?.mechanic != mechanic {
            return Err(CatalogError::new(role, "incorrect role mechanic"));
        }
    }
    for (role, mechanic) in [
        ("cold_default", PressureMechanic::Cold),
        ("drought_default", PressureMechanic::Drought),
        ("pack_default", PressureMechanic::Pack),
        ("illness_default", PressureMechanic::Illness),
    ] {
        let id = c
            .pressure_role(role)
            .ok_or_else(|| CatalogError::new(role, "missing required role"))?;
        if c.require_pressure(id)?.mechanic != mechanic {
            return Err(CatalogError::new(role, "incorrect role mechanic"));
        }
    }
    c.pressure_schedule = resolve_schedule(
        c,
        pressures
            .schedule
            .ok_or_else(|| CatalogError::new("pressures", "missing schedule"))?,
    )?;
    Ok(())
}
fn resolve_schedule(c: &Catalog, raw: ScheduleRaw) -> Result<PressureSchedule, CatalogError> {
    let resolve = |name: &str| {
        c.pressure_by_name(name)
            .ok_or_else(|| CatalogError::new("schedule", format!("unknown pressure {name}")))
    };
    if raw.seasons.keys().copied().collect::<Vec<_>>() != vec![0, 1, 2, 3]
        || raw.minor_seasons.iter().any(|s| *s > 3)
        || raw.minor_draw_slot != 4
    {
        return Err(CatalogError::new(
            "schedule",
            "invalid seasons/minor keyed slot",
        ));
    }
    let mut seasons = BTreeMap::new();
    for (season, rows) in raw.seasons {
        let mut previous = 0;
        let mut draws = Vec::new();
        for (upper, name) in rows {
            if upper <= previous || upper > 1000 {
                return Err(CatalogError::new("schedule", "unordered draw boundaries"));
            }
            previous = upper;
            draws.push(PressureDraw {
                upper_exclusive: upper,
                kind: name.as_deref().map(resolve).transpose()?,
            });
        }
        if previous != 1000 {
            return Err(CatalogError::new(
                "schedule",
                "draw boundaries must cover 1000",
            ));
        }
        seasons.insert(season, draws);
    }
    let due_order = raw
        .due_order
        .iter()
        .map(|s| resolve(s))
        .collect::<Result<Vec<_>, _>>()?;
    if due_order.iter().copied().collect::<BTreeSet<_>>().len() != due_order.len()
        || due_order
            .iter()
            .any(|id| c.pressure(*id).is_none_or(|d| d.due_days.is_none()))
    {
        return Err(CatalogError::new("schedule", "invalid due precedence"));
    }
    let minor = raw.minor.as_deref().map(resolve).transpose()?;
    if minor.is_some_and(|id| c.pressure(id).is_none_or(|d| d.major)) {
        return Err(CatalogError::new("schedule", "minor must not be major"));
    }
    Ok(PressureSchedule {
        seasons,
        due_order,
        minor,
        minor_seasons: raw.minor_seasons,
        minor_draw_slot: raw.minor_draw_slot,
    })
}
