//! M2 economy bridge: bill, priority and quarry commands and the workshop,
//! priority, recipe and per-person economy reads behind `WorldSimNode`.
//!
//! Commands go through `sim_systems::commands::submit` (logged with the next
//! tick, ADR 0012); names arrive as strings (`"knap_blade"`, `"make"`,
//! `"chop"`) and are parsed here, at the FFI boundary. Reads are pure
//! collectors: [`collect_workshops`] (workshop id order, bills in list order),
//! [`collect_work_priorities`] (adults in `AgentId` order), [`collect_recipes`]
//! (recipe code order) and [`collect_agent_economy`] (one person by `AgentId`).
use godot::prelude::*;
use sim_core::colony::skills::tier;
use sim_core::colony::{
    recipes, BillMode, Carry, Command, CommandResult, ItemSpec, RecipeKind, RejectReason,
    SkillKind, Skills, Tool, WorkPriorities, WorkType,
};
use sim_core::components::{Agent, AgentId, Child, Elder};
use sim_engine::{SimEngine, SimResources};
use sim_systems::commands::submit;

fn mode_of(mode: &str, count: i64) -> Option<BillMode> {
    let n = u16::try_from(count.max(0)).ok()?;
    match mode {
        "make" => Some(BillMode::Make(n)),
        "maintain" => Some(BillMode::Maintain(n)),
        _ => None,
    }
}

fn id_of(v: i64) -> Option<u32> {
    u32::try_from(v).ok()
}

/// One M2 bill or priority command as the bridge receives it.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum EconomyCommand {
    /// `AddBill` (recipe by name, mode `"make"`/`"maintain"`).
    AddBill {
        /// Workshop id.
        workshop: i64,
        /// Recipe name.
        recipe: String,
        /// Mode name.
        mode: String,
        /// Mode count.
        count: i64,
    },
    /// `RemoveBill`.
    RemoveBill {
        /// Workshop id.
        workshop: i64,
        /// Bill id.
        bill: i64,
    },
    /// `SetBillMode`.
    SetBillMode {
        /// Workshop id.
        workshop: i64,
        /// Bill id.
        bill: i64,
        /// Mode name.
        mode: String,
        /// Mode count.
        count: i64,
    },
    /// `MoveBill`.
    MoveBill {
        /// Workshop id.
        workshop: i64,
        /// Bill id.
        bill: i64,
        /// Towards the front.
        up: bool,
    },
    /// `SetBillSuspended`.
    SetBillSuspended {
        /// Workshop id.
        workshop: i64,
        /// Bill id.
        bill: i64,
        /// Suspended or not.
        suspended: bool,
    },
    /// `SetWorkPriority` (work type by name).
    SetWorkPriority {
        /// The person (`AgentId`).
        agent: i64,
        /// Work type name.
        work: String,
        /// Level 0–4.
        level: i64,
    },
}

/// Parse an [`EconomyCommand`] into a `Command`; `None` for an unknown name
/// or a value out of range (rejected without a log entry).
pub fn parse_economy(cmd: &EconomyCommand) -> Option<Command> {
    Some(match cmd {
        EconomyCommand::AddBill {
            workshop,
            recipe,
            mode,
            count,
        } => Command::AddBill {
            workshop: id_of(*workshop)?,
            recipe: RecipeKind::from_name(recipe)?,
            mode: mode_of(mode, *count)?,
        },
        EconomyCommand::RemoveBill { workshop, bill } => Command::RemoveBill {
            workshop: id_of(*workshop)?,
            bill: id_of(*bill)?,
        },
        EconomyCommand::SetBillMode {
            workshop,
            bill,
            mode,
            count,
        } => Command::SetBillMode {
            workshop: id_of(*workshop)?,
            bill: id_of(*bill)?,
            mode: mode_of(mode, *count)?,
        },
        EconomyCommand::MoveBill { workshop, bill, up } => Command::MoveBill {
            workshop: id_of(*workshop)?,
            bill: id_of(*bill)?,
            up: *up,
        },
        EconomyCommand::SetBillSuspended {
            workshop,
            bill,
            suspended,
        } => Command::SetBillSuspended {
            workshop: id_of(*workshop)?,
            bill: id_of(*bill)?,
            suspended: *suspended,
        },
        EconomyCommand::SetWorkPriority { agent, work, level } => Command::SetWorkPriority {
            agent: u64::try_from(*agent).ok()?,
            work: WorkType::from_name(work)?,
            level: u8::try_from(*level).ok()?,
        },
    })
}

/// Apply and log an [`EconomyCommand`] (the body of the `cmd_*` funcs).
pub fn submit_economy(engine: &mut SimEngine, cmd: &EconomyCommand) -> CommandResult {
    match parse_economy(cmd) {
        Some(command) => submit(engine, command),
        None => CommandResult::rejected(RejectReason::Unknown),
    }
}

/// One bill as the workshop panel shows it.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct BillRow {
    /// Bill id.
    pub id: u32,
    /// `RecipeKind::code`.
    pub recipe: u8,
    /// `BillMode::code` (0 make, 1 maintain).
    pub mode: u8,
    /// The mode's count.
    pub count: u16,
    /// Crafts done under it.
    pub done: u32,
    /// Suspended.
    pub suspended: bool,
    /// The hearth's auto-bill.
    pub auto: bool,
    /// The workshop's active bill.
    pub active: bool,
}

/// One workshop.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WorkshopRow {
    /// Workshop id.
    pub id: u32,
    /// `BlueprintKind::code`.
    pub kind: u8,
    /// Footprint top-left and size.
    pub rect: (u32, u32, u32, u32),
    /// Work tile.
    pub work: (u32, u32),
    /// Bills in work order.
    pub bills: Vec<BillRow>,
    /// Shelf: `(ItemKind::code, count)` per stack.
    pub shelf: Vec<(u8, u16)>,
}

/// Every finished workshop, in id order.
pub fn collect_workshops(resources: &SimResources) -> Vec<WorkshopRow> {
    resources
        .colony
        .workshops
        .values()
        .map(|shop| WorkshopRow {
            id: shop.id,
            kind: shop.kind.code(),
            rect: (shop.top_left.0, shop.top_left.1, shop.size.0, shop.size.1),
            work: shop.work_tile(),
            bills: shop
                .bills
                .iter()
                .map(|b| BillRow {
                    id: b.id,
                    recipe: b.recipe.code(),
                    mode: b.mode.code(),
                    count: b.mode.count(),
                    done: b.done,
                    suspended: b.suspended,
                    auto: b.auto,
                    active: shop.active == Some(b.id),
                })
                .collect(),
            shelf: shop
                .shelf
                .iter()
                .map(|s| (s.kind.code(), s.count))
                .collect(),
        })
        .collect()
}

fn i32s(values: impl Iterator<Item = i64>) -> PackedInt32Array {
    let v: Vec<i32> = values.map(|x| x as i32).collect();
    PackedInt32Array::from(v.as_slice())
}

pub(super) fn workshops_to_dict(rows: &[WorkshopRow]) -> VarDictionary {
    let mut dict = VarDictionary::new();
    dict.set("ids", i32s(rows.iter().map(|r| i64::from(r.id))));
    dict.set("kinds", i32s(rows.iter().map(|r| i64::from(r.kind))));
    dict.set("xs", i32s(rows.iter().map(|r| i64::from(r.rect.0))));
    dict.set("ys", i32s(rows.iter().map(|r| i64::from(r.rect.1))));
    dict.set("ws", i32s(rows.iter().map(|r| i64::from(r.rect.2))));
    dict.set("hs", i32s(rows.iter().map(|r| i64::from(r.rect.3))));
    dict.set("work_xs", i32s(rows.iter().map(|r| i64::from(r.work.0))));
    dict.set("work_ys", i32s(rows.iter().map(|r| i64::from(r.work.1))));
    // Bills flattened: `bill_shops[i]` is the row index of bill i's workshop.
    let bills: Vec<(usize, BillRow)> = rows
        .iter()
        .enumerate()
        .flat_map(|(i, r)| r.bills.iter().map(move |b| (i, *b)))
        .collect();
    dict.set("bill_shops", i32s(bills.iter().map(|(i, _)| *i as i64)));
    dict.set("bill_ids", i32s(bills.iter().map(|(_, b)| i64::from(b.id))));
    dict.set(
        "bill_recipes",
        i32s(bills.iter().map(|(_, b)| i64::from(b.recipe))),
    );
    dict.set(
        "bill_modes",
        i32s(bills.iter().map(|(_, b)| i64::from(b.mode))),
    );
    dict.set(
        "bill_counts",
        i32s(bills.iter().map(|(_, b)| i64::from(b.count))),
    );
    dict.set(
        "bill_done",
        i32s(bills.iter().map(|(_, b)| i64::from(b.done))),
    );
    dict.set(
        "bill_suspended",
        i32s(bills.iter().map(|(_, b)| i64::from(b.suspended))),
    );
    dict.set(
        "bill_auto",
        i32s(bills.iter().map(|(_, b)| i64::from(b.auto))),
    );
    dict.set(
        "bill_active",
        i32s(bills.iter().map(|(_, b)| i64::from(b.active))),
    );
    let shelf: Vec<(usize, (u8, u16))> = rows
        .iter()
        .enumerate()
        .flat_map(|(i, r)| r.shelf.iter().map(move |s| (i, *s)))
        .collect();
    dict.set("shelf_shops", i32s(shelf.iter().map(|(i, _)| *i as i64)));
    dict.set(
        "shelf_kinds",
        i32s(shelf.iter().map(|(_, s)| i64::from(s.0))),
    );
    dict.set(
        "shelf_counts",
        i32s(shelf.iter().map(|(_, s)| i64::from(s.1))),
    );
    dict
}

/// One adult's work priorities.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PriorityRow {
    /// The person.
    pub agent: AgentId,
    /// An elder (quarrying defaults to 0).
    pub elder: bool,
    /// Levels by `WorkType::code` (0 never, 1 highest … 4 lowest).
    pub levels: [u8; WorkType::ALL.len()],
}

/// Every adult (no `Child`), in `AgentId` order, with their priorities (the
/// defaults when they have no `WorkPriorities`).
pub fn collect_work_priorities(world: &hecs::World) -> Vec<PriorityRow> {
    let mut rows: Vec<PriorityRow> = world
        .query::<(&Agent, Option<&WorkPriorities>, Option<&Elder>)>()
        .without::<&Child>()
        .iter()
        .map(|(_, (agent, p, elder))| PriorityRow {
            agent: agent.id,
            elder: elder.is_some(),
            levels: p
                .copied()
                .unwrap_or(WorkPriorities::defaults(elder.is_some()))
                .levels,
        })
        .collect();
    rows.sort_by_key(|r| r.agent);
    rows
}

/// A row's levels in `WorkType::ALL` order, the order of `work_types` (M3:
/// mining's code is 6 but it sits fourth, after quarrying).
pub fn levels_in_work_order(row: &PriorityRow) -> [u8; WorkType::ALL.len()] {
    WorkType::ALL.map(|w| row.levels[w.code() as usize])
}

pub(super) fn priorities_to_dict(rows: &[PriorityRow]) -> VarDictionary {
    let mut dict = VarDictionary::new();
    let ids: Vec<i64> = rows.iter().map(|r| r.agent as i64).collect();
    dict.set("agent_ids", PackedInt64Array::from(ids.as_slice()));
    dict.set("elders", i32s(rows.iter().map(|r| i64::from(r.elder))));
    let levels: Vec<u8> = rows.iter().flat_map(levels_in_work_order).collect();
    dict.set("levels", PackedByteArray::from(levels.as_slice()));
    let works: Vec<GString> = WorkType::ALL
        .iter()
        .map(|w| GString::from(w.name()))
        .collect();
    dict.set("work_types", PackedStringArray::from(works.as_slice()));
    dict
}

/// Every recipe in code order: `(code, name, workshop kind code, output kind code, output count, work ticks, skill code)`.
pub fn collect_recipes() -> Vec<(u8, &'static str, u8, u8, u16, u32, u8)> {
    recipes()
        .iter()
        .map(|r| {
            (
                r.id.code(),
                r.id.name(),
                r.workshop.code(),
                r.output.0.code(),
                r.output.1,
                r.work_ticks,
                r.skill.code(),
            )
        })
        .collect()
}

pub(super) fn recipes_to_dict(rows: &[(u8, &'static str, u8, u8, u16, u32, u8)]) -> VarDictionary {
    let mut dict = VarDictionary::new();
    dict.set("codes", i32s(rows.iter().map(|r| i64::from(r.0))));
    let names: Vec<GString> = rows.iter().map(|r| GString::from(r.1)).collect();
    dict.set("names", PackedStringArray::from(names.as_slice()));
    dict.set("workshop_kinds", i32s(rows.iter().map(|r| i64::from(r.2))));
    dict.set("output_kinds", i32s(rows.iter().map(|r| i64::from(r.3))));
    dict.set("output_counts", i32s(rows.iter().map(|r| i64::from(r.4))));
    dict.set("work_ticks", i32s(rows.iter().map(|r| i64::from(r.5))));
    dict.set("skills", i32s(rows.iter().map(|r| i64::from(r.6))));
    dict
}

/// One person's economy: skills, tool, load.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct AgentEconomyRow {
    /// Levels by `SkillKind::code`.
    pub levels: [u8; SkillKind::ALL.len()],
    /// XP by `SkillKind::code`.
    pub xp: [u32; SkillKind::ALL.len()],
    /// Held tool: `(ItemKind::code, quality, material name)`.
    pub tool: Option<(u8, u8, String)>,
    /// Carried load: `(ItemKind::code, count, quality, material name)`.
    pub carry: Option<(u8, u16, u8, String)>,
    /// M3: the assigned bed tile, if any.
    pub bed: Option<(u32, u32)>,
    /// M3: the bed's room: `(RoomId, RoomRole::code, roofed)`.
    pub bed_room: Option<(u32, u8, bool)>,
}

fn material_name(resources: &SimResources, spec: ItemSpec) -> String {
    spec.material
        .map(|m| resources.colony.materials.name(m).to_string())
        .unwrap_or_default()
}

/// The economy of the person `agent` (`AgentId`), or `None`.
pub fn collect_agent_economy(
    world: &hecs::World,
    resources: &SimResources,
    agent: AgentId,
) -> Option<AgentEconomyRow> {
    let mut q = world.query::<(&Agent, Option<&Skills>, Option<&Tool>, Option<&Carry>)>();
    let (_, (_, skills, tool, carry)) = q.iter().find(|(_, (a, _, _, _))| a.id == agent)?;
    let skills = skills.copied().unwrap_or_default();
    let mut row = AgentEconomyRow::default();
    for kind in SkillKind::ALL {
        let s = skills.get(kind);
        row.levels[kind.code() as usize] = s.level;
        row.xp[kind.code() as usize] = s.xp;
    }
    row.tool = tool.map(|t| {
        (
            t.item.kind.code(),
            t.item.quality,
            material_name(resources, t.item),
        )
    });
    row.carry = carry.map(|c| {
        (
            c.kind.code(),
            c.count,
            c.quality,
            material_name(resources, c.spec()),
        )
    });
    row.bed = resources.bed_of.get(&agent).copied();
    row.bed_room = row.bed.and_then(|bed| {
        let room = sim_systems::runtime::rooms::room_at(resources, bed)?;
        Some((room.id, room.role.code(), room.roofed))
    });
    Some(row)
}

pub(super) fn agent_economy_to_dict(row: Option<AgentEconomyRow>) -> VarDictionary {
    let found = row.is_some();
    let row = row.unwrap_or_default();
    let mut dict = VarDictionary::new();
    dict.set("found", found);
    dict.set(
        "skill_levels",
        i32s(row.levels.iter().map(|l| i64::from(*l))),
    );
    dict.set("skill_xp", i32s(row.xp.iter().map(|x| i64::from(*x))));
    dict.set(
        "skill_tiers",
        i32s(row.levels.iter().map(|l| i64::from(tier(*l)))),
    );
    let skills: Vec<GString> = SkillKind::ALL
        .iter()
        .map(|s| GString::from(s.name()))
        .collect();
    dict.set("skill_names", PackedStringArray::from(skills.as_slice()));
    let (tk, tq, tm) = row.tool.map_or((-1, 0, String::new()), |t| {
        (i64::from(t.0), i64::from(t.1), t.2)
    });
    dict.set("tool_kind", tk);
    dict.set("tool_quality", tq);
    dict.set("tool_material", GString::from(tm.as_str()));
    let (ck, cn, cq, cm) = row.carry.map_or((-1, 0, 0, String::new()), |c| {
        (i64::from(c.0), i64::from(c.1), i64::from(c.2), c.3)
    });
    let (bx, by) = row
        .bed
        .map_or((-1, -1), |b| (i64::from(b.0), i64::from(b.1)));
    dict.set("bed_x", bx);
    dict.set("bed_y", by);
    let (rid, rrole, rroof) = row
        .bed_room
        .map_or((-1, -1, false), |r| (i64::from(r.0), i64::from(r.1), r.2));
    dict.set("bed_room", rid);
    dict.set("bed_room_role", rrole);
    dict.set("bed_roofed", rroof);
    dict.set("carry_kind", ck);
    dict.set("carry_count", cn);
    dict.set("carry_quality", cq);
    dict.set("carry_material", GString::from(cm.as_str()));
    dict
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn names_parse_and_bad_values_are_refused() {
        let add = EconomyCommand::AddBill {
            workshop: 2,
            recipe: "knap_blade".into(),
            mode: "maintain".into(),
            count: 10,
        };
        assert_eq!(
            parse_economy(&add),
            Some(Command::AddBill {
                workshop: 2,
                recipe: RecipeKind::KnapBlade,
                mode: BillMode::Maintain(10)
            })
        );
        let bad = EconomyCommand::AddBill {
            workshop: 2,
            recipe: "bronze".into(),
            mode: "make".into(),
            count: 1,
        };
        assert_eq!(parse_economy(&bad), None);
        let neg = EconomyCommand::SetWorkPriority {
            agent: 3,
            work: "chop".into(),
            level: -1,
        };
        assert_eq!(parse_economy(&neg), None);
        let ok = EconomyCommand::SetWorkPriority {
            agent: 3,
            work: "chop".into(),
            level: 0,
        };
        assert_eq!(
            parse_economy(&ok),
            Some(Command::SetWorkPriority {
                agent: 3,
                work: WorkType::Chop,
                level: 0
            })
        );
    }
}
