//! Open content identities with historical unit-enum wire and Debug compatibility.
use super::{installed, CatalogAccessError};
use serde::de::{self, EnumAccess, VariantAccess, Visitor};
use serde::ser::Error as _;
use serde::{Deserialize, Deserializer, Serialize, Serializer};
use std::fmt;

macro_rules! content_id {
    ($name:ident, $by_code:ident, $by_name:ident, [$($variant:ident = $code:literal),+ $(,)?]) => {
        /// An explicitly assigned, family-specific content code.
        #[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
        pub struct $name(u32);
        impl $name {
            /// Explicit stable code, never an array index.
            pub const fn code(self) -> u32 { self.0 }
            pub(super) const fn from_validated(code: u32) -> Self { Self(code) }
            /// Resolve a code only in the already installed catalogue.
            pub fn checked(code: u32) -> Result<Self, CatalogAccessError> {
                let catalog = installed()?;
                if catalog.$by_code(code).is_some() { Ok(Self(code)) }
                else { Err(CatalogAccessError::Unknown { family: stringify!($name), code }) }
            }
            // These spellings preserve historical test/codec APIs, not behavior dispatch.
            $(#[allow(non_upper_case_globals)]
              #[doc = concat!("Historical codec/test identity `", stringify!($variant), "`.")]
              pub const $variant: Self = Self($code);)+
            fn legacy_debug(self) -> Option<&'static str> {
                match self.0 { $($code => Some(stringify!($variant)),)+ _ => None }
            }
        }
        impl fmt::Debug for $name {
            fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
                if let Some(name) = self.legacy_debug() { return f.write_str(name); }
                let name = installed().ok().and_then(|c| c.$by_code(self.0)).map(|d| d.identity.id.as_str());
                let mut out = f.debug_struct(stringify!($name));
                if let Some(name) = name { out.field("id", &name); }
                out.field("code", &self.0).finish()
            }
        }
        impl Serialize for $name {
            fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok,S::Error> {
                if !serializer.is_human_readable() { return serializer.serialize_u32(self.0); }
                let catalog = installed().map_err(S::Error::custom)?;
                let def = catalog.$by_code(self.0).ok_or_else(|| S::Error::custom(
                    format!("unknown {} code {}", stringify!($name), self.0)))?;
                serializer.serialize_unit_variant(stringify!($name), self.0, def.identity.serde_name.as_str())
            }
        }
        impl<'de> Deserialize<'de> for $name {
            fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self,D::Error> {
                if !deserializer.is_human_readable() {
                    return Self::checked(u32::deserialize(deserializer)?).map_err(de::Error::custom);
                }
                struct UnitVisitor;
                impl<'de> Visitor<'de> for UnitVisitor {
                    type Value = $name;
                    fn expecting(&self,f:&mut fmt::Formatter<'_>)->fmt::Result {
                        write!(f,"a registered {} unit name", stringify!($name))
                    }
                    fn visit_enum<A:EnumAccess<'de>>(self, access:A)->Result<$name,A::Error> {
                        let (id,unit) = access.variant_seed(NameSeed)?;
                        unit.unit_variant()?;
                        Ok(id)
                    }
                }
                struct NameSeed;
                impl<'de> de::DeserializeSeed<'de> for NameSeed {
                    type Value = $name;
                    fn deserialize<D:Deserializer<'de>>(self,d:D)->Result<$name,D::Error> {
                        d.deserialize_identifier(NameVisitor)
                    }
                }
                struct NameVisitor;
                impl<'de> Visitor<'de> for NameVisitor {
                    type Value = $name;
                    fn expecting(&self,f:&mut fmt::Formatter<'_>)->fmt::Result {
                        write!(f,"a registered {} identifier", stringify!($name))
                    }
                    fn visit_str<E:de::Error>(self,name:&str)->Result<$name,E> {
                        installed().map_err(E::custom)?.$by_name(name)
                            .ok_or_else(|| E::custom(format!("unknown {} name {name:?}",stringify!($name))))
                    }
                }
                deserializer.deserialize_enum(stringify!($name), &[], UnitVisitor)
            }
        }
    };
}

content_id!(
    ItemKind,
    item_by_code,
    item_by_serde_name,
    [
        Log = 0,
        Firewood = 1,
        Stone = 2,
        Flint = 3,
        FlintBlade = 4,
        StoneAxe = 5,
        Hide = 6,
        Clay = 7,
        RawPot = 8,
        ClayPot = 9,
        GroundAxe = 10,
        Meat = 11,
        CuredHide = 12,
        HideWrap = 13,
        FurCloak = 14,
        Spear = 15,
        WolfHide = 16,
    ]
);
content_id!(
    BlueprintKind,
    blueprint_by_code,
    blueprint_by_serde_name,
    [
        Hut = 0,
        KnappingSpot = 1,
        WoodBench = 2,
        DryingRack = 3,
        LogWall = 4,
        StoneWall = 5,
        Door = 6,
        PackedFloor = 7,
        Bedroll = 8,
        GrindingStone = 9,
        PotteryBench = 10,
        HearthFire = 11,
    ]
);
content_id!(
    RecipeKind,
    recipe_by_code,
    recipe_by_serde_name,
    [
        KnapBlade = 0,
        HaftAxe = 1,
        SplitFirewood = 2,
        KnapFineBlade = 3,
        GrindAxe = 4,
        FormPot = 5,
        FirePot = 6,
        HaftSpear = 7,
        ScrapeHide = 8,
        MakeHideWrap = 9,
        MakeFurCloak = 10,
    ]
);

content_id!(
    PressureKind,
    pressure_by_code,
    pressure_by_serde_name,
    [ColdSnap = 0, Drought = 1, Beasts = 2, Illness = 3]
);
content_id!(
    AnimalKind,
    animal_by_code,
    animal_by_serde_name,
    [Wolf = 0, Deer = 1]
);
