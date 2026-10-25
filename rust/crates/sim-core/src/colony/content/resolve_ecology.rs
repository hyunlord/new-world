//! Cross-family reference resolution for animal loot and pressure packs.
use super::*;
pub(super) fn animal(c: &Catalog, raw: AnimalDef<String>) -> Result<AnimalDef, CatalogError> {
    validate_ecology::animal(&raw)?;
    let context = &raw.identity.id;
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
        category: raw.category,
        mechanic: raw.mechanic,
        huntable: raw.huntable,
        hp: raw.hp,
        grazer: raw.grazer,
        predator: raw.predator,
        loot: AnimalLoot { meat, hide },
    })
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
