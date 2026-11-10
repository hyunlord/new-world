//! Cross-family reference resolution for animal loot and pressure packs.
use super::*;
pub(super) fn animal(c: &Catalog, raw: AnimalDef<String>) -> Result<AnimalDef, CatalogError> {
    validate_ecology::animal(&raw)?;
    let context = &raw.identity.id;
    validate::material(c, &raw.art.tint_material)?;
    let tameable = raw
        .tameable
        .map(|policy| resolve_tameable(c, context, policy))
        .transpose()?;
    let meat = raw
        .loot
        .meat
        .map(|m| {
            let item = c
                .item_by_name(&m.item)
                .ok_or_else(|| CatalogError::new(context, "unknown meat item"))?;
            if Some(item) != c.item_role("grazer_meat") {
                return Err(CatalogError::new(
                    context,
                    "unsupported perishable meat identity",
                ));
            }
            Ok(MeatLoot {
                item,
                with_cutter: m.with_cutter,
                without_cutter: m.without_cutter,
            })
        })
        .transpose()?;
    let hide = raw
        .loot
        .hide
        .map(|h| {
            let item = c
                .item_by_name(&h.item)
                .ok_or_else(|| CatalogError::new(context, "unknown hide item"))?;
            if c.require_item(item)?.food.is_some() {
                return Err(CatalogError::new(
                    context,
                    "hide loot must be nonperishable",
                ));
            }
            Ok(HideLoot {
                item,
                count: h.count,
                enabled: h.enabled,
            })
        })
        .transpose()?;
    Ok(AnimalDef {
        identity: raw.identity,
        status: raw.status,
        debt: raw.debt,
        category: raw.category,
        origin: raw.origin,
        population: raw.population,
        tameable,
        domestic_name_key: raw.domestic_name_key,
        art: raw.art,
        mechanic: raw.mechanic,
        huntable: raw.huntable,
        hp: raw.hp,
        grazer: raw.grazer,
        predator: raw.predator,
        loot: AnimalLoot { meat, hide },
    })
}

fn resolve_tameable(
    c: &Catalog,
    context: &str,
    raw: TameableDef<String>,
) -> Result<TameableDef, CatalogError> {
    let item = |name: &str, food: bool| -> Result<ItemKind, CatalogError> {
        let kind = c
            .item_by_name(name)
            .ok_or_else(|| CatalogError::new(context, format!("unknown livestock item {name}")))?;
        if c.require_item(kind)?.food.is_some() != food {
            return Err(CatalogError::new(
                context,
                "livestock item food capability mismatch",
            ));
        }
        Ok(kind)
    };
    let recurring =
        |product: RecurringProduct<String>, food: bool| -> Result<RecurringProduct, CatalogError> {
            Ok(RecurringProduct {
                item: item(&product.item, food)?,
                every_days: product.every_days,
                requires_knowledge: product.requires_knowledge,
            })
        };
    let resolved = TameableDef {
        capture_knowledge: raw.capture_knowledge,
        capture_tool: raw
            .capture_tool
            .map(|name| item(&name, false))
            .transpose()?,
        capture_job: raw.capture_job,
        capture_range_tiles: raw.capture_range_tiles,
        capture_work_ticks: raw.capture_work_ticks,
        capture_success_permille: raw.capture_success_permille,
        capture_retry_days: raw.capture_retry_days,
        initial_age_days: raw.initial_age_days,
        kid_age_days: raw.kid_age_days,
        breeding_interval_days: raw.breeding_interval_days,
        max_age_days: raw.max_age_days,
        graze_tiles_per_head: raw.graze_tiles_per_head,
        starvation_days: raw.starvation_days,
        grass_regrow_days: raw.grass_regrow_days,
        milk_work_ticks: raw.milk_work_ticks,
        slaughter_work_ticks: raw.slaughter_work_ticks,
        products: LivestockProducts {
            milk: raw.products.milk.map(|p| recurring(p, true)).transpose()?,
            wool: raw.products.wool.map(|p| recurring(p, false)).transpose()?,
            meat: (item(&raw.products.meat.0, true)?, raw.products.meat.1),
            hide: (item(&raw.products.hide.0, false)?, raw.products.hide.1),
        },
    };
    if resolved.products.milk.is_some()
        && resolved
            .products
            .wool
            .as_ref()
            .is_some_and(|wool| Some(wool.item) == c.item_role("default_placeholder"))
    {
        return Err(CatalogError::new(
            context,
            "wool output conflicts with legacy milk placeholder",
        ));
    }
    Ok(resolved)
}
pub(super) fn pressure(c: &Catalog, raw: PressureDef<String>) -> Result<PressureDef, CatalogError> {
    validate_ecology::pressure(&raw)?;
    let pack = raw
        .pack
        .map(|p| {
            let animal = c
                .animal_by_name(&p.animal)
                .ok_or_else(|| CatalogError::new(&raw.identity.id, "unknown pack animal"))?;
            if c.require_animal(animal)?.mechanic != AnimalMechanic::Predator {
                return Err(CatalogError::new(
                    &raw.identity.id,
                    "pack requires predator animal",
                ));
            }
            Ok(PackEffects { animal })
        })
        .transpose()?;
    Ok(PressureDef {
        identity: raw.identity,
        category: raw.category,
        mechanic: raw.mechanic,
        major: raw.major,
        enabled_rule: raw.enabled_rule,
        duration: raw.duration,
        duration_draw_slot: raw.duration_draw_slot,
        mercy: raw.mercy,
        due_days: raw.due_days,
        due_seasons: raw.due_seasons,
        intensity: raw.intensity,
        cold: raw.cold,
        drought: raw.drought,
        illness: raw.illness,
        pack,
    })
}
