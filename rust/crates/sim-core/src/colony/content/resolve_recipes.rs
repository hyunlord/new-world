//! Resolve recipes after local item and blueprint definitions exist.
use super::{raw_recipe::RecipeRaw, *};
use crate::colony::{SkillKind, WorkType};

pub(super) fn resolve(r: RecipeRaw, c: &Catalog) -> Result<RecipeDef, CatalogError> {
    let context = &r.identity.id;
    if r.work_ticks == 0
        || r.output.1 == 0
        || r.inputs.is_empty()
        || r.inputs.iter().any(|(_, n)| *n == 0)
        || r.workshop_tags.is_empty()
    {
        return Err(CatalogError::new(
            context,
            "positive recipe quantities/ticks and workshop tags required",
        ));
    }
    if matches!(r.material_term,MaterialTerm::Hardness{divisor,..} if divisor<=0) {
        return Err(CatalogError::new(
            context,
            "material divisor must be positive",
        ));
    }
    let item = |name: &str| {
        c.item_by_name(name)
            .ok_or_else(|| CatalogError::new(context, format!("unknown recipe item {name}")))
    };
    let inputs = r
        .inputs
        .iter()
        .map(|(name, count)| Ok((item(name)?, *count)))
        .collect::<Result<Vec<_>, CatalogError>>()?;
    let output = (item(&r.output.0)?, r.output.1);
    let workshop = c
        .blueprint_by_name(&r.workshop)
        .ok_or_else(|| CatalogError::new(context, "unknown preferred workshop"))?;
    let compatible_workshops = c
        .blueprints
        .iter()
        .filter(|b| {
            b.completion == CompletionCategory::Workshop
                && r.workshop_tags
                    .iter()
                    .all(|tag| b.workshop_tags.contains(tag))
        })
        .map(|b| BlueprintKind::from_validated(b.identity.code))
        .collect::<Vec<_>>();
    if !compatible_workshops.contains(&workshop) {
        return Err(CatalogError::new(
            context,
            "preferred workshop lacks required capabilities",
        ));
    }
    let skill = SkillKind::from_name(&r.skill)
        .ok_or_else(|| CatalogError::new(context, "unknown skill mechanic"))?;
    let work = WorkType::from_name(&r.work)
        .ok_or_else(|| CatalogError::new(context, "unknown work mechanic"))?;
    let tool = r
        .tool
        .map(|t| {
            if t.capability != "cutter" || t.count != 1 || !t.reusable {
                return Err(CatalogError::new(
                    context,
                    "only reusable cutter count1 is supported",
                ));
            }
            let preferred_item = item(&t.preferred_item)?;
            if c.item(preferred_item).is_none_or(|d| d.cutter.is_none()) {
                return Err(CatalogError::new(
                    context,
                    "preferred tool lacks cutter capability",
                ));
            }
            Ok(RecipeToolDef {
                capability: t.capability,
                preferred_item,
                count: t.count,
                reusable: t.reusable,
            })
        })
        .transpose()?;
    validate::selector(r.auto_bill_role.as_deref(), &["firewood"], context)?;
    Ok(RecipeDef {
        id: RecipeKind::from_validated(r.identity.code),
        identity: r.identity,
        workshop,
        compatible_workshops,
        workshop_tags: r.workshop_tags,
        inputs,
        output,
        work_ticks: r.work_ticks,
        skill,
        work,
        quality_add: r.quality_add,
        material_term: r.material_term,
        tool,
        auto_bill_role: r.auto_bill_role,
    })
}
