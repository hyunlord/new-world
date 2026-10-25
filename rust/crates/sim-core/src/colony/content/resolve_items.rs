//! Resolve raw item references without installed catalogue access.
use super::{raw_item::ItemRaw, *};

pub(super) fn resolve(raw: ItemRaw, c: &Catalog) -> Result<ItemDef, CatalogError> {
    let context = &raw.identity.id;
    if raw.stack_max == 0 || raw.carry == 0 || raw.category.is_empty() || raw.value == 0 {
        return Err(CatalogError::new(
            context,
            "positive capacities/value and category required",
        ));
    }
    validate::selector(raw.stack_rule.as_deref(), &["stack_max_logs"], context)?;
    validate::selector(raw.carry_rule.as_deref(), &["carry_logs"], context)?;
    if raw.fuel_units.is_some_and(|units| units != 1) {
        return Err(CatalogError::new(context, "fuel conversion must be1:1"));
    }
    let chop = raw
        .chop
        .map(|r| {
            validate::selector(
                r.wear_rule.as_deref(),
                &["axe_uses", "ground_axe_uses"],
                context,
            )?;
            if r.quality_permille.contains(&0) {
                return Err(CatalogError::new(
                    context,
                    "chop multiplier must be positive",
                ));
            }
            let break_return = r
                .break_return
                .map(|(name, count)| {
                    if count == 0 {
                        return Err(CatalogError::new(
                            context,
                            "break return count must be positive",
                        ));
                    }
                    Ok((
                        c.item_by_name(&name).ok_or_else(|| {
                            CatalogError::new(context, format!("unknown return item {name}"))
                        })?,
                        count,
                    ))
                })
                .transpose()?;
            Ok(ChopDef {
                quality_permille: r.quality_permille,
                wear_rule: r.wear_rule,
                wear_default: r.wear_default,
                break_return,
                preserve_return_material: r.preserve_return_material,
            })
        })
        .transpose()?;
    if let Some(r) = &raw.weapon {
        validate::selector(
            r.damage_rule.as_deref(),
            &[
                "axe_damage",
                "ground_axe_damage",
                "spear_damage",
                "hand_damage",
            ],
            context,
        )?;
        validate::selector(r.wear_rule.as_deref(), &["spear_uses"], context)?;
        validate::selector(
            r.break_tally.as_deref(),
            &["spears_broken", "knives_broken"],
            context,
        )?;
    }
    if let Some(r) = &raw.cutter {
        validate::selector(r.wear_rule.as_deref(), &["knife_uses"], context)?;
        validate::selector(r.break_tally.as_deref(), &["knives_broken"], context)?;
    }
    if let Some(r) = &raw.food {
        validate::selector(r.units_rule.as_deref(), &["meat_food_units"], context)?;
        validate::selector(r.spoil_rule.as_deref(), &["meat_spoil_days"], context)?;
        if r.units == 0 || r.spoil_days == 0 {
            return Err(CatalogError::new(
                context,
                "food units/spoil days must be positive",
            ));
        }
    }
    if let Some(r) = &raw.apparel {
        if !(1..=2).contains(&r.legacy_top_code) || r.cold_permille > 1000 {
            return Err(CatalogError::new(
                context,
                "unsupported legacy top code/cold permille",
            ));
        }
    }
    Ok(ItemDef {
        identity: raw.identity,
        category: raw.category,
        material_format_key: raw.material_format_key,
        tags: raw.tags,
        stack_max: raw.stack_max,
        carry: raw.carry,
        stack_rule: raw.stack_rule,
        carry_rule: raw.carry_rule,
        value: raw.value,
        made_by: raw.made_by,
        clue_nodes: raw.clue_nodes,
        chop,
        weapon: raw.weapon,
        cutter: raw.cutter,
        food: raw.food,
        fuel_units: raw.fuel_units,
        apparel: raw.apparel,
    })
}
