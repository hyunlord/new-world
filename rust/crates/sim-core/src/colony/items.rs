//! Physical items (M1, M2): kinds, ground stacks, carried loads, held tools and the conservation ledger.
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

use crate::material::MaterialId;

/// Validated item identity from the installed content catalog.
pub use super::content::ids::ItemKind;

impl ItemKind {
    /// Every installed item, in ascending stable code order.
    pub fn all() -> impl ExactSizeIterator<Item = Self> {
        super::content::installed_or_bundled().item_ids()
    }

    /// Definition belonging to this installed identity.
    pub fn definition(self) -> &'static super::content::ItemDef {
        super::content::installed_or_bundled()
            .item(self)
            .unwrap_or_else(|| panic!("installed item definition missing for code {}", self.code()))
    }

    /// Required, validated content role (not a content-name dispatch).
    pub fn role(role: &str) -> Self {
        super::content::installed_or_bundled()
            .item_role(role)
            .unwrap_or_else(|| panic!("required installed item role missing: {role}"))
    }

    /// Stable canonical name for logs, scripts and summaries.
    pub fn name(self) -> &'static str {
        &self.definition().identity.id
    }

    /// Resolve a canonical name or declared resolver alias.
    pub fn from_name(name: &str) -> Option<Self> {
        super::content::installed_or_bundled().item_by_name(name)
    }

    /// Default items per stack; world rule overrides are applied by ColonyRule.
    pub fn default_stack_max(self) -> u16 {
        self.definition().stack_max
    }

    /// Default items per carried load; world rule overrides are applied by ColonyRule.
    pub fn default_carry(self) -> u16 {
        self.definition().carry
    }
}

/// The quality of a raw material (goal 3.1).
pub const RAW_QUALITY: u8 = 50;

/// Quality bucket (0–10): stacks merge only within one bucket.
pub const fn quality_bucket(quality: u8) -> u8 {
    quality / 10
}

/// What one item is, beyond its count: kind, material and quality. Two
/// items may share a stack when [`ItemSpec::merges_with`] holds.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct ItemSpec {
    /// The kind.
    pub kind: ItemKind,
    /// The material (raw: the source's; crafted: the main input's), if any.
    pub material: Option<MaterialId>,
    /// Quality 0–100 (50 for raw materials).
    pub quality: u8,
}

impl ItemSpec {
    /// A raw item of `kind` and `material` at quality 50.
    pub const fn raw(kind: ItemKind, material: Option<MaterialId>) -> Self {
        Self {
            kind,
            material,
            quality: RAW_QUALITY,
        }
    }

    /// Same kind, same material and same quality bucket (goal 3.1).
    pub fn merges_with(&self, other: &Self) -> bool {
        self.kind == other.kind
            && self.material == other.material
            && quality_bucket(self.quality) == quality_bucket(other.quality)
    }
}

/// Items of one spec lying on one tile (one stack per tile).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct ItemStack {
    /// What the stack holds.
    pub kind: ItemKind,
    /// How many; a stored stack is never 0.
    pub count: u16,
    /// Quality 0–100: the count-weighted mean of what was merged in (it stays
    /// in one bucket, since only one bucket merges).
    pub quality: u8,
    /// The material, if any.
    pub material: Option<MaterialId>,
}

impl ItemStack {
    /// A stack of `count` items of `spec`.
    pub const fn of(spec: ItemSpec, count: u16) -> Self {
        Self {
            kind: spec.kind,
            count,
            quality: spec.quality,
            material: spec.material,
        }
    }

    /// The spec of this stack's items.
    pub const fn spec(&self) -> ItemSpec {
        ItemSpec {
            kind: self.kind,
            material: self.material,
            quality: self.quality,
        }
    }

    /// Merge `n` items of `spec` in (the caller checked `merges_with`).
    pub fn absorb(&mut self, spec: ItemSpec, n: u16) {
        let total = u32::from(self.count) + u32::from(n);
        if total > 0 {
            let weighted = u32::from(self.quality) * u32::from(self.count)
                + u32::from(spec.quality) * u32::from(n);
            self.quality = (weighted / total) as u8;
        }
        self.count = total.min(u32::from(u16::MAX)) as u16;
    }
}

/// Items an agent holds in its hands (an ECS component).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct Carry {
    /// What is carried.
    pub kind: ItemKind,
    /// How many; never 0 while the component exists.
    pub count: u16,
    /// Quality of the carried items.
    pub quality: u8,
    /// Material of the carried items.
    pub material: Option<MaterialId>,
}

impl Carry {
    /// `count` items of `spec`.
    pub const fn of(spec: ItemSpec, count: u16) -> Self {
        Self {
            kind: spec.kind,
            count,
            quality: spec.quality,
            material: spec.material,
        }
    }

    /// The spec of the carried items.
    pub const fn spec(&self) -> ItemSpec {
        ItemSpec {
            kind: self.kind,
            material: self.material,
            quality: self.quality,
        }
    }
}

/// A tool an agent holds beside its hands (an ECS component, M2 goal 3.7):
/// one item, picked up from a stockpile on the way to a job.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct Tool {
    /// The tool item.
    pub item: ItemSpec,
    /// Jobs done with it.
    pub uses: u32,
}

/// Every item ever created and consumed, per kind (M1 7.6, M2 goal 3.1).
/// Conservation, per kind: `created == on the ground + carried + held as
/// tools + delivered to sites + on workshop shelves + consumed`.
///
/// A craft is a **transform**: its inputs are consumed (`consumed` and
/// `craft_inputs`) and its outputs created (`created` and `crafted`) in the
/// same function; `transforms` counts crafts. `burned` is the part of
/// `consumed` the hearth took (firewood fed to the settlement's wood).
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct ItemLedger {
    /// Items brought into the world (felled logs, quarried stone, crafted outputs).
    pub created: BTreeMap<ItemKind, u64>,
    /// Items used up (a finished building's materials, craft inputs, fed firewood).
    pub consumed: BTreeMap<ItemKind, u64>,
    /// The craft outputs among `created`.
    #[serde(default)]
    pub crafted: BTreeMap<ItemKind, u64>,
    /// The craft inputs among `consumed`.
    #[serde(default)]
    pub craft_inputs: BTreeMap<ItemKind, u64>,
    /// The firewood among `consumed` that fed a hearth.
    #[serde(default)]
    pub burned: BTreeMap<ItemKind, u64>,
    /// Crafts done.
    #[serde(default)]
    pub transforms: u64,
    /// M5.5 (ADR 0032): items a caravan gave the colony (they enter the world here, like `created`).
    #[serde(default)]
    pub traded_in: BTreeMap<ItemKind, u64>,
    /// M5.5 (ADR 0032): items the colony handed to a caravan (they leave the world here, like `consumed`).
    #[serde(default)]
    pub traded_out: BTreeMap<ItemKind, u64>,
}

impl ItemLedger {
    /// Record `n` new items of `kind`.
    pub fn create(&mut self, kind: ItemKind, n: u64) {
        *self.created.entry(kind).or_insert(0) += n;
    }

    /// Record `n` items of `kind` used up.
    pub fn consume(&mut self, kind: ItemKind, n: u64) {
        *self.consumed.entry(kind).or_insert(0) += n;
    }

    /// M5.5: record `n` items of `kind` a caravan gave the colony.
    pub fn trade_in(&mut self, kind: ItemKind, n: u64) {
        *self.traded_in.entry(kind).or_insert(0) += n;
    }

    /// M5.5: record `n` items of `kind` handed to a caravan.
    pub fn trade_out(&mut self, kind: ItemKind, n: u64) {
        *self.traded_out.entry(kind).or_insert(0) += n;
    }

    /// M5.5: items of `kind` a caravan gave the colony so far.
    pub fn traded_in_of(&self, kind: ItemKind) -> u64 {
        self.traded_in.get(&kind).copied().unwrap_or(0)
    }

    /// M5.5: items of `kind` handed to caravans so far.
    pub fn traded_out_of(&self, kind: ItemKind) -> u64 {
        self.traded_out.get(&kind).copied().unwrap_or(0)
    }

    /// Record one whole craft: `inputs` consumed, `outputs` created.
    pub fn transform(&mut self, inputs: &[(ItemKind, u16)], outputs: &[(ItemKind, u16)]) {
        self.craft_start(inputs);
        self.craft_end(outputs);
    }

    /// A craft starts (M2 goal 6): its inputs are consumed now (`consumed`,
    /// `craft_inputs`); a craft cancelled later loses them.
    pub fn craft_start(&mut self, inputs: &[(ItemKind, u16)]) {
        for &(kind, n) in inputs {
            self.consume(kind, u64::from(n));
            *self.craft_inputs.entry(kind).or_insert(0) += u64::from(n);
        }
    }

    /// A craft ends: its outputs are created (`created`, `crafted`) and the
    /// transform is counted. `craft_start` + `craft_end` = [`Self::transform`].
    pub fn craft_end(&mut self, outputs: &[(ItemKind, u16)]) {
        for &(kind, n) in outputs {
            self.create(kind, u64::from(n));
            *self.crafted.entry(kind).or_insert(0) += u64::from(n);
        }
        self.transforms += 1;
    }

    /// Record `n` items of `kind` fed to a hearth (consumed).
    pub fn burn(&mut self, kind: ItemKind, n: u64) {
        self.consume(kind, n);
        *self.burned.entry(kind).or_insert(0) += n;
    }

    /// Items of `kind` created so far.
    pub fn created_of(&self, kind: ItemKind) -> u64 {
        self.created.get(&kind).copied().unwrap_or(0)
    }

    /// Items of `kind` consumed so far.
    pub fn consumed_of(&self, kind: ItemKind) -> u64 {
        self.consumed.get(&kind).copied().unwrap_or(0)
    }

    /// Items of `kind` crafted so far.
    pub fn crafted_of(&self, kind: ItemKind) -> u64 {
        self.crafted.get(&kind).copied().unwrap_or(0)
    }

    /// Items of `kind` fed to a hearth so far.
    pub fn burned_of(&self, kind: ItemKind) -> u64 {
        self.burned.get(&kind).copied().unwrap_or(0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn catalog_capacities_keep_world_overrides_and_nonlog_limits() {
        let mut rule = crate::rules::ColonyRule::M1;
        rule.stack_max_logs = 73;
        rule.carry_logs = 19;
        assert_eq!(
            (rule.stack_max(ItemKind::Log), rule.carry(ItemKind::Log)),
            (73, 19)
        );
        assert_eq!(
            (
                rule.stack_max(ItemKind::Firewood),
                rule.carry(ItemKind::Firewood)
            ),
            (50, 10)
        );
        assert_eq!(
            (
                rule.stack_max(ItemKind::GroundAxe),
                rule.carry(ItemKind::GroundAxe)
            ),
            (5, 2)
        );
        assert!(ItemKind::from_name("unknown_item").is_none());
    }

    #[test]
    fn supported_saved_apparel_maps_both_directions_and_rejects_tools() {
        use super::super::people::Top;
        for (top, item, cold, rank, immune) in [
            (Top::HideWrap, ItemKind::HideWrap, 700, 1, false),
            (Top::FurCloak, ItemKind::FurCloak, 500, 2, true),
        ] {
            assert_eq!(top.item(), item);
            assert_eq!(Top::of_item(item), Some(top));
            assert_eq!(top.cold_permille(), cold);
            assert_eq!(top.warmth_rank(), rank);
            assert_eq!(top.cold_snap_work_immune(), immune);
        }
        assert_eq!(Top::of_item(ItemKind::StoneAxe), None);
    }

    #[test]
    fn ledger_counts_per_kind_and_codes_are_stable() {
        let mut ledger = ItemLedger::default();
        assert_eq!(ledger.created_of(ItemKind::Log), 0);
        ledger.create(ItemKind::Log, 4);
        ledger.create(ItemKind::Log, 3);
        ledger.consume(ItemKind::Log, 2);
        assert_eq!(ledger.created_of(ItemKind::Log), 7);
        assert_eq!(ledger.consumed_of(ItemKind::Log), 2);
        ledger.transform(&[(ItemKind::Flint, 1)], &[(ItemKind::FlintBlade, 1)]);
        assert_eq!(ledger.consumed_of(ItemKind::Flint), 1);
        assert_eq!(ledger.crafted_of(ItemKind::FlintBlade), 1);
        assert_eq!(ledger.created_of(ItemKind::FlintBlade), 1);
        assert_eq!(ledger.transforms, 1);
        ledger.burn(ItemKind::Firewood, 3);
        assert_eq!(
            (
                ledger.consumed_of(ItemKind::Firewood),
                ledger.burned_of(ItemKind::Firewood)
            ),
            (3, 3)
        );
        let codes: Vec<u32> = ItemKind::all().map(|k| k.code()).collect();
        assert_eq!(
            codes,
            vec![0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16]
        );
        assert_eq!((ItemKind::Log.code(), ItemKind::Log.name()), (0, "log"));
        for kind in ItemKind::all() {
            assert_eq!(ItemKind::from_name(kind.name()), Some(kind));
        }
    }

    #[test]
    fn stacks_merge_within_one_quality_bucket_and_keep_a_weighted_mean() {
        let a = ItemSpec {
            kind: ItemKind::FlintBlade,
            material: None,
            quality: 41,
        };
        let b = ItemSpec { quality: 49, ..a };
        let c = ItemSpec { quality: 50, ..a };
        assert!(a.merges_with(&b));
        assert!(!a.merges_with(&c));
        assert!(!a.merges_with(&ItemSpec::raw(ItemKind::Flint, None)));
        let mut stack = ItemStack::of(a, 3);
        stack.absorb(b, 1);
        // (41·3 + 49·1) / 4 = 43.
        assert_eq!((stack.count, stack.quality), (4, 43));
        assert_eq!(quality_bucket(stack.quality), quality_bucket(41));
    }
}
