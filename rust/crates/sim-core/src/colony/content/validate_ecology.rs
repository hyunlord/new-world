//! Field-specific validation for supported animal and pressure mechanics.
use super::*;

fn selector(rule: &Option<String>, allowed: &[&str], context: &str) -> Result<(), CatalogError> {
    validate::selector(rule.as_deref(), allowed, context)
}
fn positive(value: u64, context: &str) -> Result<(), CatalogError> {
    if value == 0 {
        Err(CatalogError::new(
            context,
            "positive mechanic value required",
        ))
    } else {
        Ok(())
    }
}
fn number(v: &RuleU32, allowed: &[&str], context: &str) -> Result<(), CatalogError> {
    selector(&v.rule, allowed, context)
}
pub(super) fn animal(a: &AnimalDef<String>) -> Result<(), CatalogError> {
    let ctx = &a.identity.id;
    let (category, hp_rule) = match a.mechanic {
        AnimalMechanic::Grazer => ("grazer", "deer_hp"),
        AnimalMechanic::Predator => ("predator", "wolf_hp"),
    };
    if a.category != category
        || a.grazer.is_some() != (a.mechanic == AnimalMechanic::Grazer)
        || a.predator.is_some() != (a.mechanic == AnimalMechanic::Predator)
        || (a.huntable && a.mechanic == AnimalMechanic::Predator)
    {
        return Err(CatalogError::new(
            ctx,
            "category/mechanic/capability mismatch",
        ));
    }
    positive(u64::from(a.hp.value), ctx)?;
    selector(&a.hp.rule, &[hp_rule], ctx)?;
    if let Some(g) = &a.grazer {
        selector(&g.flee.rule, &["deer_flee"], ctx)?;
        for (v, key) in [
            (&g.flee_tiles, "deer_flee_tiles"),
            (&g.flee_speed, "deer_flee_speed"),
            (&g.flee_steps, "deer_flee_steps"),
            (&g.rest_steps, "deer_rest_steps"),
            (&g.home_tiles, "deer_home_tiles"),
            (&g.min_hearth_distance, "deer_min_hearth_distance"),
        ] {
            number(v, &[key], ctx)?;
        }
        positive(u64::from(g.flee_speed.value), ctx)?;
    }
    if let Some(p) = &a.predator {
        for (v, key) in [
            (&p.sight, "wolf_sight"),
            (&p.chase_tiles, "wolf_chase_tiles"),
            (&p.hit_ticks, "hit_ticks"),
        ] {
            number(v, &[key], ctx)?;
            positive(u64::from(v.value), ctx)?;
        }
        selector(&p.damage.rule, &["wolf_damage"], ctx)?;
        positive(u64::from(p.damage.value), ctx)?;
    }
    if let Some(m) = &a.loot.meat {
        selector(&m.with_cutter.rule, &["meat_with_knife"], ctx)?;
        selector(&m.without_cutter.rule, &["meat_without_knife"], ctx)?;
    }
    if let Some(h) = &a.loot.hide {
        selector(&h.enabled.rule, &["wolf_hides"], ctx)?;
        positive(u64::from(h.count), ctx)?;
    }
    Ok(())
}
pub(super) fn pressure(p: &PressureDef<String>) -> Result<(), CatalogError> {
    let ctx = &p.identity.id;
    let (category, duration, slot, enabled) = match p.mechanic {
        PressureMechanic::Cold => ("cold", "cold_snap_days", Some(2), None),
        PressureMechanic::Drought => ("drought", "drought_days", Some(3), None),
        PressureMechanic::Pack => ("pack", "beasts_max_days", None, Some("beasts")),
        PressureMechanic::Illness => ("illness", "sick_days", None, Some("illness")),
    };
    if p.category != category
        || p.cold.is_some() != (p.mechanic == PressureMechanic::Cold)
        || p.drought.is_some() != (p.mechanic == PressureMechanic::Drought)
        || p.pack.is_some() != (p.mechanic == PressureMechanic::Pack)
        || p.illness.is_some() != (p.mechanic == PressureMechanic::Illness)
        || p.major != (p.mechanic != PressureMechanic::Illness)
    {
        return Err(CatalogError::new(ctx, "category/mechanic/effect mismatch"));
    }
    selector(
        &p.enabled_rule,
        &enabled.into_iter().collect::<Vec<_>>(),
        ctx,
    )?;
    selector(&p.duration.rule, &[duration], ctx)?;
    if p.duration.value.0 == 0
        || p.duration.value.0 > p.duration.value.1
        || p.duration_draw_slot != slot
        || (slot.is_none() && p.duration.value.0 != p.duration.value.1)
    {
        return Err(CatalogError::new(
            ctx,
            "invalid duration or keyed draw slot",
        ));
    }
    for (v, allowed) in [
        (&p.mercy.first_day, &["mercy_day", "illness_day"][..]),
        (&p.mercy.first_winter_days, &["first_winter_mercy_days"][..]),
        (&p.mercy.repeat_gap_days, &["beasts_gap_days"][..]),
        (&p.mercy.winter_end_days, &["winter_end_mercy_days"][..]),
    ] {
        if let Some(v) = v {
            number(v, allowed, ctx)?;
        }
    }
    if let Some(v) = &p.due_days {
        number(v, &["cold_due_days", "beasts_due_days"], ctx)?;
        positive(u64::from(v.value), ctx)?;
    }
    if p.due_seasons.iter().any(|s| *s > 3)
        || p.due_seasons.iter().collect::<BTreeSet<_>>().len() != p.due_seasons.len()
        || p.due_days.is_some() == p.due_seasons.is_empty()
    {
        return Err(CatalogError::new(ctx, "invalid due seasons"));
    }
    match &p.intensity {
        PressureIntensity::None => {
            if p.mechanic != PressureMechanic::Illness {
                return Err(CatalogError::new(ctx, "missing intensity mechanics"));
            }
        }
        PressureIntensity::ExtendForAdults {
            threshold,
            extra_days,
        } => {
            if !matches!(
                p.mechanic,
                PressureMechanic::Cold | PressureMechanic::Drought
            ) {
                return Err(CatalogError::new(ctx, "invalid extension intensity"));
            }
            number(threshold, &["big_colony_adults"], ctx)?;
            positive(u64::from(threshold.value), ctx)?;
            positive(u64::from(*extra_days), ctx)?;
        }
        PressureIntensity::PackByWealth {
            base,
            per_items,
            max,
        } => {
            if p.mechanic != PressureMechanic::Pack {
                return Err(CatalogError::new(ctx, "invalid pack intensity"));
            }
            for (v, key) in [
                (base, "wolves_base"),
                (per_items, "wolves_per_items"),
                (max, "wolves_max"),
            ] {
                number(v, &[key], ctx)?;
                positive(u64::from(v.value), ctx)?;
            }
            if base.value > max.value {
                return Err(CatalogError::new(ctx, "pack maximum below base"));
            }
        }
    }
    if let Some(e) = &p.cold {
        for (v, key) in [
            (&e.burn, "snap_burn_permille"),
            (&e.exposure, "snap_exposure_permille"),
            (&e.outdoor_work, "snap_outdoor_work_permille"),
        ] {
            number(v, &[key], ctx)?;
            positive(u64::from(v.value), ctx)?;
        }
    }
    if let Some(e) = &p.drought {
        number(&e.regrowth, &["drought_regen_permille"], ctx)?;
        if e.regrowth.value > 1000 {
            return Err(CatalogError::new(ctx, "drought regrowth exceeds permille"));
        }
        number(&e.thirst, &["drought_thirst_permille"], ctx)?;
        positive(u64::from(e.thirst.value), ctx)?;
    }
    if let Some(e) = &p.illness {
        number(&e.work, &["sick_work_permille"], ctx)?;
        number(&e.sleep, &["sick_sleep_permille"], ctx)?;
        number(&e.spread, &["sick_spread_permille"], ctx)?;
        positive(u64::from(e.work.value), ctx)?;
        positive(u64::from(e.sleep.value), ctx)?;
        if e.spread.value > 1000 {
            return Err(CatalogError::new(
                ctx,
                "spread probability exceeds permille",
            ));
        }
    }
    Ok(())
}
