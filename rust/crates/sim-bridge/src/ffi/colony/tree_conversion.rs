//! Tree snapshot dictionary conversion and opt-in frozen conversion probe.

use super::TreeSnapshot;
use godot::prelude::*;

pub(super) fn tree_snapshot_to_dict(trees: TreeSnapshot) -> VarDictionary {
    if std::env::var("WORLDSIM_MUX2_TREE_CONVERSION_PROBE").as_deref() == Ok("1") {
        return probe_tree_conversion(trees);
    }
    converted_tree_to_dict(trees, |name| GString::from(name))
}

fn converted_tree_to_dict(
    trees: TreeSnapshot,
    convert: impl FnMut(&String) -> GString,
) -> VarDictionary {
    let mut dict = VarDictionary::new();
    dict.set("version", trees.version as i64);
    let xs: Vec<i32> = trees.tiles.iter().map(|t| t.0 as i32).collect();
    let ys: Vec<i32> = trees.tiles.iter().map(|t| t.1 as i32).collect();
    let species: Vec<GString> = tree_species_with(&trees.species, convert);
    dict.set("xs", PackedInt32Array::from(xs.as_slice()));
    dict.set("ys", PackedInt32Array::from(ys.as_slice()));
    dict.set("species", PackedStringArray::from(species.as_slice()));
    dict
}

// Frozen original dictionary packing; only the opt-in Godot probe calls this oracle.
fn trees_to_dict_reference(trees: TreeSnapshot) -> VarDictionary {
    let mut dict = VarDictionary::new();
    dict.set("version", trees.version as i64);
    let xs: Vec<i32> = trees.tiles.iter().map(|t| t.0 as i32).collect();
    let ys: Vec<i32> = trees.tiles.iter().map(|t| t.1 as i32).collect();
    let species: Vec<GString> = trees.species.iter().map(GString::from).collect();
    dict.set("xs", PackedInt32Array::from(xs.as_slice()));
    dict.set("ys", PackedInt32Array::from(ys.as_slice()));
    dict.set("species", PackedStringArray::from(species.as_slice()));
    dict
}

/// Conversion seam: output order is always the input order, including repeated empty identities.
fn tree_species_with<T: Clone>(
    species: &[String],
    mut convert: impl FnMut(&String) -> T,
) -> Vec<T> {
    let mut converted: std::collections::BTreeMap<&str, T> = std::collections::BTreeMap::new();
    species
        .iter()
        .map(|name| {
            converted
                .entry(name.as_str())
                .or_insert_with(|| convert(name))
                .clone()
        })
        .collect()
}

/// The explicit probe runs only inside Godot via the existing get_trees entry point.
fn probe_tree_conversion(trees: TreeSnapshot) -> VarDictionary {
    let mut conversions = 0usize;
    let actual = converted_tree_to_dict(trees.clone(), |name| {
        conversions += 1;
        GString::from(name)
    });
    let expected = trees_to_dict_reference(trees.clone());
    let unique = trees
        .species
        .iter()
        .collect::<std::collections::BTreeSet<_>>()
        .len();
    godot_print!(
        "MUX2_TREE_CONVERSION actual version={} rows={} unique={} conversions={} exact={}",
        trees.version,
        trees.species.len(),
        unique,
        conversions,
        actual == expected
    );
    for (index, species) in tree_conversion_cases().into_iter().enumerate() {
        let fixture = TreeSnapshot {
            version: 19 + index as u64,
            tiles: (0..species.len())
                .map(|i| (i as u32, (i % 17) as u32))
                .collect(),
            species,
        };
        let unique = fixture
            .species
            .iter()
            .collect::<std::collections::BTreeSet<_>>()
            .len();
        let rows = fixture.species.len();
        let mut conversions = 0usize;
        let candidate = converted_tree_to_dict(fixture.clone(), |name| {
            conversions += 1;
            GString::from(name)
        });
        let reference = trees_to_dict_reference(fixture);
        godot_print!(
            "MUX2_TREE_CONVERSION fixture={} rows={} unique={} conversions={} exact={}",
            index,
            rows,
            unique,
            conversions,
            candidate == reference
        );
    }
    let control = TreeSnapshot {
        version: 99,
        tiles: vec![(1, 2), (3, 4)],
        species: vec!["first".into(), "second".into()],
    };
    let mut reversed = control.clone();
    reversed.species.reverse();
    let detected = converted_tree_to_dict(reversed, |name| GString::from(name))
        != trees_to_dict_reference(control);
    godot_print!("MUX2_TREE_CONVERSION negative_order_detected={}", detected);
    actual
}

fn tree_conversion_cases() -> Vec<Vec<String>> {
    vec![
        vec![],
        vec!["one".into()],
        vec!["repeat".into(); 4096],
        (0..4096).map(|i| format!("species-{}", i % 3)).collect(),
        (0..128).map(|i| format!("unique-{i}")).collect(),
        vec!["".into(), "invalid".into(), "".into()],
        vec![
            "참나무".into(),
            "樹🌳".into(),
            "참나무".into(),
            "e\u{301}".into(),
            "é".into(),
        ],
    ]
}

#[cfg(test)]
mod tree_conversion_tests {
    use super::*;

    #[test]
    fn species_conversion_preserves_frozen_order_and_actual_work_counts() {
        let expect_unique = std::env::var("WORLDSIM_TREE_EXPECT_UNIQUE").as_deref() != Ok("0");
        for input in tree_conversion_cases() {
            // Given the original per-occurrence conversion, with no candidate cache in the oracle.
            let expected: Vec<String> = input.iter().map(|name| format!("<{name}>")).collect();
            let unique = input
                .iter()
                .collect::<std::collections::BTreeSet<_>>()
                .len();
            let mut conversions = 0usize;
            // When the actual conversion closure is invoked, count at its call site.
            let actual = tree_species_with(&input, |name| {
                conversions += 1;
                format!("<{name}>")
            });
            // Then output and work count must both match the selected baseline/candidate contract.
            assert_eq!(actual, expected);
            assert_eq!(
                conversions,
                if expect_unique { unique } else { input.len() }
            );
        }
    }

    #[test]
    fn invocation_cache_does_not_reuse_values_between_calls() {
        let input = vec!["same".to_owned(), "same".to_owned(), "".to_owned()];
        let first = tree_species_with(&input, |name| format!("first:{name}"));
        let second = tree_species_with(&input, |name| format!("second:{name}"));
        assert_eq!(first, vec!["first:same", "first:same", "first:"]);
        assert_eq!(second, vec!["second:same", "second:same", "second:"]);
        assert_ne!(first, second);
    }
}
