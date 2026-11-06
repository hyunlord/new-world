//! Exact physical item wire leaves for published save formats 6–16.
use super::{
    Bill, BillId, BlueprintKind, Carry, ItemKind, ItemSpec, ItemStack, Tool, Workshop, WorkshopId,
};
use crate::material::MaterialId;
use serde::{Deserialize, Serialize};

/// Item identity before finite garment wear.
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct ItemSpecV16 {
    /// Item kind.
    pub kind: ItemKind,
    /// Physical material.
    pub material: Option<MaterialId>,
    /// Physical quality.
    pub quality: u8,
}
impl From<ItemSpecV16> for ItemSpec {
    fn from(old: ItemSpecV16) -> Self {
        Self {
            kind: old.kind,
            material: old.material,
            quality: old.quality,
            wear_remaining_hours: None,
        }
    }
}
impl TryFrom<ItemSpec> for ItemSpecV16 {
    type Error = &'static str;
    fn try_from(item: ItemSpec) -> Result<Self, Self::Error> {
        if item.wear_remaining_hours.is_some() {
            return Err("finite wear cannot be encoded before format17");
        }
        Ok(Self {
            kind: item.kind,
            material: item.material,
            quality: item.quality,
        })
    }
}
/// Ground/shelf stack before finite garment wear.
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct ItemStackV16 {
    /// Item kind.
    pub kind: ItemKind,
    /// Whole item count.
    pub count: u16,
    /// Physical quality.
    pub quality: u8,
    /// Physical material.
    pub material: Option<MaterialId>,
}
impl From<ItemStackV16> for ItemStack {
    fn from(old: ItemStackV16) -> Self {
        Self::of(
            ItemSpecV16 {
                kind: old.kind,
                material: old.material,
                quality: old.quality,
            }
            .into(),
            old.count,
        )
    }
}
impl TryFrom<ItemStack> for ItemStackV16 {
    type Error = &'static str;
    fn try_from(item: ItemStack) -> Result<Self, Self::Error> {
        let spec = ItemSpecV16::try_from(item.spec())?;
        Ok(Self {
            kind: spec.kind,
            count: item.count,
            quality: spec.quality,
            material: spec.material,
        })
    }
}
/// Carry has exactly the same historical wire fields as a stack.
pub type CarryV16 = ItemStackV16;
impl From<CarryV16> for Carry {
    fn from(old: CarryV16) -> Self {
        Self::of(
            ItemSpecV16 {
                kind: old.kind,
                material: old.material,
                quality: old.quality,
            }
            .into(),
            old.count,
        )
    }
}
/// Held tool before its nested item spec gained wear.
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct ToolV16 {
    /// Physical tool.
    pub item: ItemSpecV16,
    /// Work uses already accrued.
    pub uses: u32,
}
impl From<ToolV16> for Tool {
    fn from(old: ToolV16) -> Self {
        Self {
            item: old.item.into(),
            uses: old.uses,
        }
    }
}
/// The two historical enum discriminants, in their original order.
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum TopV16 {
    /// Discriminant zero.
    HideWrap,
    /// Discriminant one.
    FurCloak,
}
/// Worn clothing before its exact physical spec was retained.
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct ApparelV16 {
    /// Historical slot.
    pub top: TopV16,
    /// Original display day.
    pub since_day: u16,
}
impl From<ApparelV16> for super::people::Apparel {
    fn from(old: ApparelV16) -> Self {
        Self {
            top: match old.top {
                TopV16::HideWrap => super::people::Top::HideWrap,
                TopV16::FurCloak => super::people::Top::FurCloak,
            },
            since_day: old.since_day,
            material: None,
            quality: super::RAW_QUALITY,
            wear_remaining_hours: None,
        }
    }
}
/// Historical workshop with frozen physical shelf stacks.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkshopV16 {
    /// Stable workshop id.
    pub id: WorkshopId,
    /// Authored workshop kind.
    pub kind: BlueprintKind,
    /// Footprint origin.
    pub top_left: (u32, u32),
    /// Footprint size.
    pub size: (u32, u32),
    /// Ordered bills.
    pub bills: Vec<Bill>,
    /// Physical shelf items.
    pub shelf: Vec<ItemStackV16>,
    /// Active bill.
    pub active: Option<BillId>,
}
impl From<WorkshopV16> for Workshop {
    fn from(old: WorkshopV16) -> Self {
        Self {
            id: old.id,
            kind: old.kind,
            top_left: old.top_left,
            size: old.size,
            bills: old.bills,
            shelf: old.shelf.into_iter().map(Into::into).collect(),
            active: old.active,
        }
    }
}
impl TryFrom<Workshop> for WorkshopV16 {
    type Error = &'static str;
    fn try_from(shop: Workshop) -> Result<Self, Self::Error> {
        Ok(Self {
            id: shop.id,
            kind: shop.kind,
            top_left: shop.top_left,
            size: shop.size,
            bills: shop.bills,
            shelf: shop
                .shelf
                .into_iter()
                .map(TryInto::try_into)
                .collect::<Result<_, _>>()?,
            active: shop.active,
        })
    }
}

impl TryFrom<Carry> for CarryV16 {
    type Error = &'static str;
    fn try_from(item: Carry) -> Result<Self, Self::Error> {
        ItemStack::of(item.spec(), item.count).try_into()
    }
}
impl TryFrom<Tool> for ToolV16 {
    type Error = &'static str;
    fn try_from(tool: Tool) -> Result<Self, Self::Error> {
        Ok(Self {
            item: tool.item.try_into()?,
            uses: tool.uses,
        })
    }
}
impl TryFrom<super::people::Apparel> for ApparelV16 {
    type Error = &'static str;
    fn try_from(item: super::people::Apparel) -> Result<Self, Self::Error> {
        if item.material.is_some()
            || item.quality != super::RAW_QUALITY
            || item.wear_remaining_hours.is_some()
        {
            return Err("exact garment properties cannot be encoded before format17");
        }
        let top = match item.top {
            super::people::Top::HideWrap => TopV16::HideWrap,
            super::people::Top::FurCloak => TopV16::FurCloak,
            super::people::Top::Authored(_) => {
                return Err("authored garment cannot be encoded before format17")
            }
        };
        Ok(Self {
            top,
            since_day: item.since_day,
        })
    }
}
