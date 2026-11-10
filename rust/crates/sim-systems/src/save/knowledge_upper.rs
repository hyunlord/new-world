//! Format22 authoritative neighbour upper words beside the unchanged nested low128.
use serde::{de, Deserializer};
use sim_core::colony::knowledge::KnowledgeMask;
use sim_engine::SimResources;
use std::{
    collections::{BTreeMap, BTreeSet},
    fmt,
};

pub(super) type UpperWords = BTreeMap<u8, [u64; 2]>;

pub(super) fn capture(resources: &SimResources) -> Result<UpperWords, String> {
    let mut ids = BTreeSet::new();
    let mut upper = BTreeMap::new();
    for neighbour in &resources.colony.neighbours.list {
        if !ids.insert(neighbour.id) {
            return Err("duplicate neighbour id at knowledge save boundary".into());
        }
        if neighbour.known.upper() != [0, 0] {
            upper.insert(neighbour.id, neighbour.known.upper());
        }
    }
    Ok(upper)
}

pub(super) fn restore(resources: &mut SimResources, upper: UpperWords) -> Result<(), String> {
    let mut ids = BTreeSet::new();
    for neighbour in &resources.colony.neighbours.list {
        if !ids.insert(neighbour.id) {
            return Err("duplicate neighbour id at knowledge load boundary".into());
        }
    }
    for (id, words) in upper {
        if words == [0, 0] {
            return Err("zero neighbour upper words are not a canonical save row".into());
        }
        let neighbour = resources
            .colony
            .neighbours
            .list
            .iter_mut()
            .find(|n| n.id == id)
            .ok_or_else(|| format!("upper knowledge words reference missing neighbour {id}"))?;
        let [low, high, _, _] = neighbour.known.words();
        neighbour.known = KnowledgeMask::from_words([low, high, words[0], words[1]]);
    }
    Ok(())
}

/// Reject duplicate or unordered rows before BTreeMap could silently normalize them.
pub(super) fn deserialize<'de, D: Deserializer<'de>>(
    deserializer: D,
) -> Result<UpperWords, D::Error> {
    struct Ordered;
    impl<'de> de::Visitor<'de> for Ordered {
        type Value = UpperWords;
        fn expecting(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
            f.write_str("strictly ordered neighbour ids with nonzero upper words")
        }
        fn visit_map<A: de::MapAccess<'de>>(self, mut map: A) -> Result<UpperWords, A::Error> {
            let mut rows = BTreeMap::new();
            let mut previous = None;
            while let Some((id, words)) = map.next_entry::<u8, [u64; 2]>()? {
                if previous.is_some_and(|last| last >= id) || words == [0, 0] {
                    return Err(de::Error::custom(
                        "noncanonical neighbour upper knowledge row",
                    ));
                }
                previous = Some(id);
                rows.insert(id, words);
            }
            Ok(rows)
        }
    }
    deserializer.deserialize_map(Ordered)
}
