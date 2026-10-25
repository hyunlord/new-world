//! M5.5 (ADR 0031, 0032): the neighbours, their migrant waves and their caravans. `None` outside the M5.5 colony.
//! The values, the names, the goods table and the guaranteed knowledge live in `data/colony/neighbours.ron`.
use serde::{Deserialize, Serialize};
use std::sync::OnceLock;

/// Tunables of the neighbours (`ColonyRule::neighbours`; goal M5.5 §3.2, §3.3).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct NeighbourRule {
    /// Days of travel to a neighbour (goal: 2–5).
    pub distance_days: (u8, u8),
    /// A neighbour's people at the start (goal: 20–60).
    pub population: (u16, u16),
    /// A neighbour's attitude at the start (goal: 40–60).
    pub attitude_start: (u8, u8),
    /// Neolithic nodes a neighbour knows (goal: 2–4; every Paleolithic node besides).
    pub neolithic_known: (u8, u8),
    /// Wave 1's day window and size (goal: day 8–12, 2–4 people).
    pub wave1_days: (u32, u32),
    /// See [`Self::wave1_days`].
    pub wave1_size: (u8, u8),
    /// Wave 2's day window and size (goal: day 20–26, 2–6 people).
    pub wave2_days: (u32, u32),
    /// See [`Self::wave2_days`].
    pub wave2_size: (u8, u8),
    /// A seasonal wave's size (goal: 1–4).
    pub roll_size: (u8, u8),
    /// The seasonal roll's chance ‰ = min(`roll_max_permille`, `surplus_permille` · food-surplus days
    /// + stockpile value / `value_divisor` + attitude · `attitude_permille`) (goal: 800, 50, 50, 2).
    pub roll_max_permille: u32,
    /// See [`Self::roll_max_permille`].
    pub surplus_permille: u32,
    /// See [`Self::roll_max_permille`].
    pub value_divisor: u32,
    /// See [`Self::roll_max_permille`].
    pub attitude_permille: u32,
    /// Food units an adult eats in a day (the surplus days' denominator; M5.5 DECISIONS: 3 meals of 1 unit).
    pub food_per_adult_day: u32,
    /// No wave when the colony holds this many people (adults and children; goal: 40).
    pub population_cap: u32,
    /// Days a wave waits at the hearth for an answer (goal: 1).
    pub wait_days: u32,
    /// Attitude lost when a wave is declined or left unanswered (goal: 5; droppable 7.5).
    pub decline_attitude: u8,
    /// A migrant's proficiency in the neighbour's Paleolithic nodes, ‰ (goal: 450–700).
    pub paleo_permille: (u16, u16),
    /// A migrant's chance of each of the neighbour's Neolithic nodes, ‰ (goal: 500), and its proficiency (420–650).
    pub neo_chance_permille: u32,
    /// See [`Self::neo_chance_permille`].
    pub neo_permille: (u16, u16),
    /// A migrant's chance of arriving in a hide wrap, ‰ (goal: 500).
    pub wrap_permille: u32,
    /// Whether caravans come (ADR 0032).
    pub caravans: bool,
    /// A neighbour sends caravans at this attitude or above (goal: 30).
    pub caravan_attitude: u8,
    /// No caravan before this day (goal: 6).
    pub caravan_first_day: u32,
    /// Days a caravan stays at the hearth (goal: 1).
    pub caravan_stay_days: u32,
    /// Stacks of goods a caravan brings and their quality (goal: 6–10 stacks, quality 30–70).
    pub goods_stacks: (u8, u8),
    /// See [`Self::goods_stacks`].
    pub goods_quality: (u8, u8),
    /// Draw weight of an item of a node the neighbour knows and the colony does not, and of a staple (goal: 3, 1).
    pub lack_weight: u32,
    /// See [`Self::lack_weight`].
    pub staple_weight: u32,
    /// Traders buy at this ‰ of value and sell at that ‰ (goal: 800, 1,200).
    pub buy_permille: u32,
    /// See [`Self::buy_permille`].
    pub sell_permille: u32,
    /// Attitude gained per trade (goal: 3).
    pub trade_attitude: u8,
    /// A gift gains one attitude per this much value (goal: 10; droppable 8.4).
    pub gift_value_per_attitude: u32,
    /// A held artifact multiplies the discovery chance of its producing node, ‰ (goal: ×3 = 3,000).
    pub eureka_permille: u32,
}

/// The neighbours' data file (`data/colony/neighbours.ron`; owner directive 2026-10-24: content lives in data).
#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct NeighbourData {
    /// File schema.
    pub schema_version: u32,
    /// The M5.5 rule.
    pub rule: NeighbourRule,
    /// Locale keys a neighbour's name is drawn from (`Neighbour::name_key` indexes this list).
    pub names: Vec<String>,
    /// Knowledge node ids at least one neighbour knows (M5.5 DECISIONS D23).
    #[serde(default)]
    pub one_neighbour_knows: Vec<String>,
    /// The caravans' draw table, in draw order.
    #[serde(default)]
    pub goods: Vec<GoodsEntry>,
    /// The item a migrant may arrive wearing (`rule.wrap_permille`; an apparel item name in values.ron).
    #[serde(default)]
    pub migrant_top: Option<String>,
}

/// One row of the caravans' draw table: an item name, its category tag (`lacked` or `staple`) and a stack's size.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct GoodsEntry {
    /// The item's name (`ItemKind::from_name`).
    pub item: String,
    /// Its category: `lacked` (drawn when the neighbour can make it and the colony cannot) or `staple`.
    pub tag: String,
    /// Items in one drawn stack, `lo..=hi` (goal: an artifact 1–2, a staple 2–6).
    pub count: (u16, u16),
}

const NEIGHBOURS_RON: &str = include_str!("../../data/colony/neighbours.ron");

/// Parse a neighbours data file; `None` when malformed, of another schema or naming an unknown field.
pub fn parse_data(text: &str) -> Option<NeighbourData> {
    let d: NeighbourData = ron::from_str(text).ok()?;
    (d.schema_version == 1 && super::data::rule_fields_known(text, &d.rule)).then_some(d)
}

/// The embedded neighbours data (`None` only if the file were malformed; a unit test parses it).
pub fn data() -> Option<&'static NeighbourData> {
    static DATA: OnceLock<Option<NeighbourData>> = OnceLock::new();
    DATA.get_or_init(|| parse_data(NEIGHBOURS_RON)).as_ref()
}

impl NeighbourRule {
    /// The M5.5 values (goal §3.2, §3.3; M5.5 DECISIONS), read from `data/colony/neighbours.ron`.
    pub fn m5_5() -> Option<Self> {
        data().map(|d| d.rule)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_data_file_parses_with_the_goal_values() {
        let d = data().expect("neighbours.ron parses");
        let r = d.rule;
        // Goal §3.2–3.3 numbers, read back from the file.
        assert_eq!(
            (r.wave1_days, r.wave2_days, r.population_cap),
            ((8, 12), (20, 26), 40)
        );
        assert_eq!(
            (r.buy_permille, r.sell_permille, r.eureka_permille),
            (800, 1_200, 3_000)
        );
        assert_eq!(
            (r.paleo_permille, r.neo_chance_permille, r.neo_permille),
            ((450, 700), 500, (420, 650))
        );
        assert_eq!(d.names.len(), 12);
        assert_eq!(d.one_neighbour_knows, ["pot_firing"]);
        assert_eq!(d.migrant_top.as_deref(), Some("hide_wrap"));
        assert_eq!(d.goods.iter().filter(|g| g.tag == "lacked").count(), 6);
        assert_eq!(d.goods.iter().filter(|g| g.tag == "staple").count(), 5);
        assert!(
            parse_data("(schema_version: 1)").is_none(),
            "a file without a rule is refused"
        );
        let typo = NEIGHBOURS_RON.replacen("wait_days:", "wait_dayz:", 1);
        assert!(
            parse_data(&typo).is_none(),
            "a misspelt rule field is refused"
        );
        let other = NEIGHBOURS_RON.replace("schema_version: 1", "schema_version: 99");
        assert!(parse_data(&other).is_none(), "another schema is refused");
        let extra = NEIGHBOURS_RON.replacen("tag: \"staple\"", "tag: \"staple\", colour: 3", 1);
        assert!(
            parse_data(&extra).is_none(),
            "an unknown goods field is refused"
        );
        // Stack sizes: an artifact 1–2, a staple 2–6 (goal §3.3).
        assert!(d
            .goods
            .iter()
            .all(|g| g.count == if g.tag == "lacked" { (1, 2) } else { (2, 6) }));
    }

    /// Every name the data files use resolves: goods items and the migrants' top are items, `made_by`, `clue_nodes`
    /// and `one_neighbour_knows` are knowledge nodes, and every neighbour name key is in both locale files.
    #[test]
    fn every_name_in_the_data_resolves() {
        use crate::colony::knowledge::id;
        use crate::colony::ItemKind;
        let d = data().expect("neighbours.ron parses");
        for g in &d.goods {
            assert!(
                ItemKind::from_name(&g.item).is_some(),
                "goods item {}",
                g.item
            );
            assert!(
                matches!(g.tag.as_str(), "lacked" | "staple"),
                "goods tag {}",
                g.tag
            );
            assert!(
                g.count.0 >= 1 && g.count.0 <= g.count.1,
                "goods count {:?}",
                g.count
            );
        }
        let top = d.migrant_top.as_deref().and_then(ItemKind::from_name);
        assert!(
            top.and_then(crate::colony::people::Top::of_item).is_some(),
            "the migrants' top is an apparel item"
        );
        for n in &d.one_neighbour_knows {
            assert!(id(n).is_some(), "one_neighbour_knows {n}");
        }
        for kind in ItemKind::all() {
            let item = crate::colony::values::item_data(kind).expect("every item has data");
            for n in item.made_by.iter().chain(item.clue_nodes.iter()) {
                assert!(id(n).is_some(), "{kind:?} names node {n}");
            }
        }
        let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../../localization");
        for lang in ["ko", "en"] {
            let text =
                std::fs::read_to_string(root.join(lang).join("ui.json")).expect("locale file");
            let keys: serde_json::Value = serde_json::from_str(&text).expect("locale json");
            for k in &d.names {
                assert!(keys.get(k).is_some(), "{lang} lacks {k}");
            }
        }
    }
}
