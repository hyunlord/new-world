//! Resolve construction references and bounded existing placement mechanics locally.
use super::{raw_blueprint::BlueprintRaw, *};

pub(super) fn resolve(r: BlueprintRaw, c: &Catalog) -> Result<BlueprintDef, CatalogError> {
    let context = &r.identity.id;
    validate::selector(r.footprint_rule.as_deref(), &["hut_footprint"], context)?;
    validate::selector(r.work_rule.as_deref(), &["hut_progress"], context)?;
    if r.footprint.0 == 0
        || r.footprint.1 == 0
        || r.footprint.0.checked_mul(r.footprint.1).is_none()
        || r.needs.len() > 2
        || (r.requires_space && r.footprint != (1, 1))
        || (r.placeable && r.work == 0)
        || r.style_id.is_empty()
    {
        return Err(CatalogError::new(
            context,
            "invalid footprint/needs/work/style",
        ));
    }
    if r.menu_order < -1
        || !(-1..=10).contains(&r.shortcut_slot)
        || (!r.placeable && (r.menu_order != -1 || r.shortcut_slot != -1))
    {
        return Err(CatalogError::new(context, "invalid menu/shortcut metadata"));
    }
    match r.completion {
        CompletionCategory::Wall | CompletionCategory::Door => {
            if r.render_wall_code.is_none_or(|code| code > 2) {
                return Err(CatalogError::new(
                    context,
                    "wall render category required in0..2",
                ));
            }
        }
        CompletionCategory::Dwelling
        | CompletionCategory::Workshop
        | CompletionCategory::Floor
        | CompletionCategory::Bed => {
            if r.render_wall_code.is_some() {
                return Err(CatalogError::new(
                    context,
                    "wall render category on a nonwall",
                ));
            }
        }
    }
    let material_name = match &r.material {
        CompletionMaterial::FirstDelivered { fallback } => fallback,
        CompletionMaterial::Fixed { material } => material,
    };
    validate::material(c, material_name)?;
    let needs = r
        .needs
        .into_iter()
        .map(|n| {
            validate::selector(n.count_rule.as_deref(), &["hut_logs"], context)?;
            if n.count == 0 {
                return Err(CatalogError::new(
                    context,
                    "authored material count must be positive",
                ));
            }
            let item = c.item_by_name(&n.item).ok_or_else(|| {
                CatalogError::new(context, format!("unknown need item {}", n.item))
            })?;
            if c.item(item).is_some_and(|definition| definition.food.is_some()) {
                return Err(CatalogError::new(
                    context,
                    format!("food item {} cannot be a construction need: delivered materials have no food-age owner", n.item),
                ));
            }
            Ok(BlueprintNeed {
                item,
                count: n.count,
                count_rule: n.count_rule,
            })
        })
        .collect::<Result<Vec<_>, CatalogError>>()?;
    let initial_bill = r
        .initial_bill
        .map(|b| {
            validate::selector(Some(&b.enabled_rule), &["society.firewood"], context)?;
            validate::selector(Some(&b.target_rule), &["hearth.auto_bill_count"], context)?;
            let recipe = c.recipe_by_name(&b.recipe).ok_or_else(|| {
                CatalogError::new(context, format!("unknown initial recipe {}", b.recipe))
            })?;
            Ok(InitialBillDef {
                recipe,
                enabled_rule: b.enabled_rule,
                target_rule: b.target_rule,
            })
        })
        .transpose()?;
    Ok(BlueprintDef {
        identity: r.identity,
        placeable: r.placeable,
        menu_order: r.menu_order,
        shortcut_slot: r.shortcut_slot,
        footprint: r.footprint,
        footprint_rule: r.footprint_rule,
        needs,
        work: r.work,
        work_rule: r.work_rule,
        placement: r.placement,
        completion: r.completion,
        workshop_tags: r.workshop_tags,
        requires_economy: r.requires_economy,
        requires_knowledge: r.requires_knowledge,
        requires_space: r.requires_space,
        planned_nav: r.planned_nav,
        material: r.material,
        style_id: r.style_id,
        render_wall_code: r.render_wall_code,
        completion_policy: r.completion_policy,
        initial_bill,
    })
}
