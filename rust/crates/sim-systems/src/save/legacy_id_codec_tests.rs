//! Frozen ca9f531ca unit-enum wire goldens for all forty historical content IDs.
use super::{decode, encode};
use serde::Serialize;
use sim_core::colony::content::{bootstrap, BootSelection};
use sim_core::colony::{BlueprintKind, ItemKind, RecipeKind};

#[derive(Clone, Copy, Serialize)]
enum OldItemKind {
    Log,
    Firewood,
    Stone,
    Flint,
    FlintBlade,
    StoneAxe,
    Hide,
    Clay,
    RawPot,
    ClayPot,
    GroundAxe,
    Meat,
    CuredHide,
    HideWrap,
    FurCloak,
    Spear,
    WolfHide,
}

#[test]
fn items_historical_discriminants_match_nested_bytes() {
    bootstrap(BootSelection::Bundled).expect("bundled catalog");
    let cases = [
        (OldItemKind::Log, ItemKind::Log),
        (OldItemKind::Firewood, ItemKind::Firewood),
        (OldItemKind::Stone, ItemKind::Stone),
        (OldItemKind::Flint, ItemKind::Flint),
        (OldItemKind::FlintBlade, ItemKind::FlintBlade),
        (OldItemKind::StoneAxe, ItemKind::StoneAxe),
        (OldItemKind::Hide, ItemKind::Hide),
        (OldItemKind::Clay, ItemKind::Clay),
        (OldItemKind::RawPot, ItemKind::RawPot),
        (OldItemKind::ClayPot, ItemKind::ClayPot),
        (OldItemKind::GroundAxe, ItemKind::GroundAxe),
        (OldItemKind::Meat, ItemKind::Meat),
        (OldItemKind::CuredHide, ItemKind::CuredHide),
        (OldItemKind::HideWrap, ItemKind::HideWrap),
        (OldItemKind::FurCloak, ItemKind::FurCloak),
        (OldItemKind::Spear, ItemKind::Spear),
        (OldItemKind::WolfHide, ItemKind::WolfHide),
    ];
    for (code, (old, current)) in cases.into_iter().enumerate() {
        assert_eq!(current.code(), u32::try_from(code).expect("small code"));
        let before = (0xfeed_u32, Some(old), vec![old, old], -912_i64);
        let after = (0xfeed_u32, Some(current), vec![current, current], -912_i64);
        let frozen = encode(&before).expect("old enum encoding");
        assert_eq!(frozen, encode(&after).expect("current ID encoding"));
        let decoded: (u32, Option<ItemKind>, Vec<ItemKind>, i64) =
            decode(&frozen).expect("decode frozen enum bytes");
        assert_eq!(decoded, after);
    }
}

#[derive(Clone, Copy, Serialize)]
enum OldBlueprintKind {
    Hut,
    KnappingSpot,
    WoodBench,
    DryingRack,
    LogWall,
    StoneWall,
    Door,
    PackedFloor,
    Bedroll,
    GrindingStone,
    PotteryBench,
    HearthFire,
}

#[test]
fn orders_historical_discriminants_match_nested_bytes() {
    bootstrap(BootSelection::Bundled).expect("bundled catalog");
    let cases = [
        (OldBlueprintKind::Hut, BlueprintKind::Hut),
        (OldBlueprintKind::KnappingSpot, BlueprintKind::KnappingSpot),
        (OldBlueprintKind::WoodBench, BlueprintKind::WoodBench),
        (OldBlueprintKind::DryingRack, BlueprintKind::DryingRack),
        (OldBlueprintKind::LogWall, BlueprintKind::LogWall),
        (OldBlueprintKind::StoneWall, BlueprintKind::StoneWall),
        (OldBlueprintKind::Door, BlueprintKind::Door),
        (OldBlueprintKind::PackedFloor, BlueprintKind::PackedFloor),
        (OldBlueprintKind::Bedroll, BlueprintKind::Bedroll),
        (
            OldBlueprintKind::GrindingStone,
            BlueprintKind::GrindingStone,
        ),
        (OldBlueprintKind::PotteryBench, BlueprintKind::PotteryBench),
        (OldBlueprintKind::HearthFire, BlueprintKind::HearthFire),
    ];
    for (code, (old, current)) in cases.into_iter().enumerate() {
        assert_eq!(current.code(), u32::try_from(code).expect("small code"));
        let before = (0xfeed_u32, Some(old), vec![old, old], -912_i64);
        let after = (0xfeed_u32, Some(current), vec![current, current], -912_i64);
        let frozen = encode(&before).expect("old enum encoding");
        assert_eq!(frozen, encode(&after).expect("current ID encoding"));
        let decoded: (u32, Option<BlueprintKind>, Vec<BlueprintKind>, i64) =
            decode(&frozen).expect("decode frozen enum bytes");
        assert_eq!(decoded, after);
    }
}

#[derive(Clone, Copy, Serialize)]
enum OldRecipeKind {
    KnapBlade,
    HaftAxe,
    SplitFirewood,
    KnapFineBlade,
    GrindAxe,
    FormPot,
    FirePot,
    HaftSpear,
    ScrapeHide,
    MakeHideWrap,
    MakeFurCloak,
}

#[test]
fn workshop_historical_discriminants_match_nested_bytes() {
    bootstrap(BootSelection::Bundled).expect("bundled catalog");
    let cases = [
        (OldRecipeKind::KnapBlade, RecipeKind::KnapBlade),
        (OldRecipeKind::HaftAxe, RecipeKind::HaftAxe),
        (OldRecipeKind::SplitFirewood, RecipeKind::SplitFirewood),
        (OldRecipeKind::KnapFineBlade, RecipeKind::KnapFineBlade),
        (OldRecipeKind::GrindAxe, RecipeKind::GrindAxe),
        (OldRecipeKind::FormPot, RecipeKind::FormPot),
        (OldRecipeKind::FirePot, RecipeKind::FirePot),
        (OldRecipeKind::HaftSpear, RecipeKind::HaftSpear),
        (OldRecipeKind::ScrapeHide, RecipeKind::ScrapeHide),
        (OldRecipeKind::MakeHideWrap, RecipeKind::MakeHideWrap),
        (OldRecipeKind::MakeFurCloak, RecipeKind::MakeFurCloak),
    ];
    for (code, (old, current)) in cases.into_iter().enumerate() {
        assert_eq!(current.code(), u32::try_from(code).expect("small code"));
        let before = (0xfeed_u32, Some(old), vec![old, old], -912_i64);
        let after = (0xfeed_u32, Some(current), vec![current, current], -912_i64);
        let frozen = encode(&before).expect("old enum encoding");
        assert_eq!(frozen, encode(&after).expect("current ID encoding"));
        let decoded: (u32, Option<RecipeKind>, Vec<RecipeKind>, i64) =
            decode(&frozen).expect("decode frozen enum bytes");
        assert_eq!(decoded, after);
    }
}

#[derive(Clone, Copy, Serialize)]
enum OldPressureKind {
    ColdSnap,
    Drought,
    Beasts,
    Illness,
}
#[derive(Clone, Copy, Serialize)]
enum OldAnimalKind {
    Wolf,
    Deer,
}

#[test]
fn pressure_and_animal_historical_unit_enum_bytes() {
    use sim_core::colony::{story::PressureKind, wildlife::AnimalKind};
    bootstrap(BootSelection::Bundled).expect("bundled catalog");
    for (old, new) in [
        (OldPressureKind::ColdSnap, PressureKind::ColdSnap),
        (OldPressureKind::Drought, PressureKind::Drought),
        (OldPressureKind::Beasts, PressureKind::Beasts),
        (OldPressureKind::Illness, PressureKind::Illness),
    ] {
        let bytes = encode(&(251u32, old, -123i64)).expect("old pressure");
        assert_eq!(
            bytes,
            encode(&(251u32, new, -123i64)).expect("new pressure")
        );
        assert_eq!(
            decode::<(u32, PressureKind, i64)>(&bytes).expect("decode pressure"),
            (251, new, -123)
        );
    }
    for (old, new) in [
        (OldAnimalKind::Wolf, AnimalKind::Wolf),
        (OldAnimalKind::Deer, AnimalKind::Deer),
    ] {
        let bytes = encode(&(251u32, old, -123i64)).expect("old animal");
        assert_eq!(bytes, encode(&(251u32, new, -123i64)).expect("new animal"));
        assert_eq!(
            decode::<(u32, AnimalKind, i64)>(&bytes).expect("decode animal"),
            (251, new, -123)
        );
    }
}
