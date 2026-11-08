//! Authenticated format19 admission and varint proof for the 128-node map.
//!
//! Bincode's configured variable integers retain every u64 encoding when read
//! as u128, including the complete former neighbour word. No nested field moves.
use super::SaveError;
use sim_core::colony::content::{Catalog, RawSources};

const REVISION: &str = "catalog-v1-fnv1a64:3798fe7bf081a5e3:100534";

pub(super) fn accepts_catalog(revision: &str, catalog: &Catalog) -> Result<bool, SaveError> {
    if revision != REVISION {
        return Ok(false);
    }
    let bundled = Catalog::from_sources(RawSources::bundled(), None)
        .map_err(|error| SaveError::Decode(error.to_string()))?;
    if bundled.revision() != catalog.revision() {
        return Ok(false);
    }
    let old_templates = [
        "hut",
        "log_house",
        "stone_house",
        "workshop_shed",
        "storehouse",
        "drying_shed",
        "granary",
        "pen",
        "well",
        "smokehouse",
        "kiln",
        "mud_brick_house",
        "charcoal_clamp",
        "smelter",
        "casting_bench",
        "bloomery",
        "forge",
        "shrine",
        "chief_hall",
        "trading_post",
        "market",
        "warehouse",
    ];
    let templates_preserved = old_templates.iter().enumerate().all(|(ordinal, id)| {
        catalog
            .building_templates()
            .buildings
            .get(ordinal)
            .is_some_and(|row| row.id == *id)
    });
    if !templates_preserved || !super::legacy_buildings_v18::identities_preserved(catalog) {
        return Err(SaveError::Decode(
            "format19 identities changed without a migration map".into(),
        ));
    }
    Ok(true)
}

#[cfg(test)]
mod tests {
    #[test]
    fn every_predecessor_varint_boundary_keeps_its_bytes_and_following_field() {
        let mut values = vec![0, 1, 250, 251, 252, 253, 254, 255, 256, u64::MAX];
        for bit in 0..64 {
            let edge = 1_u64 << bit;
            values.extend([edge.saturating_sub(1), edge, edge.saturating_add(1)]);
        }
        for low in values {
            let old = super::super::encode(&(low, 123456789_u64)).expect("old wire");
            let new =
                super::super::encode(&(u128::from(low), 123456789_u64)).expect("widened wire");
            assert_eq!(old, new, "lower-word value {low}");
            let decoded: (u128, u64) = super::super::decode(&old).expect("old body field");
            assert_eq!(decoded, (u128::from(low), 123456789));
        }
        let high = (1_u128 << 127) | (1_u128 << 64) | u128::from(u64::MAX);
        let bytes = super::super::encode(&(high, 7_u64)).expect("upper-word wire");
        let decoded: (u128, u64) = super::super::decode(&bytes).expect("upper-word roundtrip");
        assert_eq!(decoded, (high, 7));
    }

    #[test]
    fn format19_rejects_unregistered_catalogue_revisions() {
        let catalog = sim_core::colony::content::Catalog::from_sources(
            sim_core::colony::content::RawSources::bundled(),
            None,
        )
        .expect("catalogue");
        assert!(!super::accepts_catalog("unknown", &catalog).expect("rejected"));
        assert!(super::accepts_catalog(super::REVISION, &catalog).expect("historical prefix"));
    }
}
