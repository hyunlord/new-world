//! Historical nested low128 wire; Knowledge reconstructs its derived cache and
//! format22 saves carry authoritative neighbour upper words in an explicit tail.
//! Standalone KnowledgeMask serde is lossless across all four words.
use super::KnowledgeMask;
use serde::{de, Deserialize, Deserializer, Serialize, Serializer};
use std::fmt;

pub(crate) fn serialize<S: Serializer>(
    mask: &KnowledgeMask,
    serializer: S,
) -> Result<S::Ok, S::Error> {
    if !serializer.is_human_readable() {
        return serializer.serialize_u128(mask.low128());
    }
    let [low, high, upper_low, upper_high] = mask.words();
    if upper_low != 0 || upper_high != 0 {
        return (low, high, upper_low, upper_high).serialize(serializer);
    }
    if high == 0 {
        serializer.serialize_u64(low)
    } else {
        (low, high).serialize(serializer)
    }
}

pub(crate) fn deserialize<'de, D: Deserializer<'de>>(
    deserializer: D,
) -> Result<KnowledgeMask, D::Error> {
    if !deserializer.is_human_readable() {
        return u128::deserialize(deserializer).map(KnowledgeMask::from);
    }
    struct Words;
    impl<'de> de::Visitor<'de> for Words {
        type Value = KnowledgeMask;
        fn expecting(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
            formatter.write_str("a legacy unsigned mask, two u64 words, or four u64 words")
        }
        fn visit_u64<E: de::Error>(self, low: u64) -> Result<KnowledgeMask, E> {
            Ok(KnowledgeMask::from(u128::from(low)))
        }
        fn visit_seq<A: de::SeqAccess<'de>>(self, mut seq: A) -> Result<KnowledgeMask, A::Error> {
            let low: u64 = seq
                .next_element()?
                .ok_or_else(|| de::Error::invalid_length(0, &self))?;
            let high: u64 = seq
                .next_element()?
                .ok_or_else(|| de::Error::invalid_length(1, &self))?;
            let Some(upper_low) = seq.next_element::<u64>()? else {
                return Ok(KnowledgeMask::from_words([low, high, 0, 0]));
            };
            let upper_high = seq
                .next_element::<u64>()?
                .ok_or_else(|| de::Error::invalid_length(3, &self))?;
            if seq.next_element::<de::IgnoredAny>()?.is_some() {
                return Err(de::Error::invalid_length(5, &self));
            }
            Ok(KnowledgeMask::from_words([
                low, high, upper_low, upper_high,
            ]))
        }
    }
    deserializer.deserialize_any(Words)
}

#[cfg(test)]
mod tests {
    use serde::{Deserialize, Serialize};

    #[derive(Debug, PartialEq, Serialize, Deserialize)]
    struct Mask(#[serde(with = "super")] super::KnowledgeMask);

    #[test]
    fn c1_human_mask_preserves_legacy_numbers_and_rejects_malformed_words() {
        for mask in [
            0,
            u128::from(u64::MAX),
            1_u128 << 64,
            1_u128 << 127,
            u128::MAX,
        ] {
            let text = ron::to_string(&Mask(mask.into())).expect("human mask");
            assert_eq!(
                ron::from_str::<Mask>(&text).expect("mask roundtrip"),
                Mask(mask.into())
            );
        }
        assert_eq!(
            ron::to_string(&Mask(7u128.into())).expect("legacy shape"),
            "(7)"
        );
        for invalid in ["(-1)", "((1))", "((1,2,3))", "((1,-1))"] {
            assert!(ron::from_str::<Mask>(invalid).is_err(), "{invalid}");
        }
    }
}

#[cfg(test)]
mod full_width_tests {
    use super::KnowledgeMask;
    #[test]
    fn four_words_roundtrip_and_malformed_lengths_reject() {
        // Given every word boundary, including the top bit.
        let mask = KnowledgeMask::from_words([u64::MAX, 1 << 63, 1, 1 << 63]);
        // When serialized through the standalone human boundary.
        let text = ron::to_string(&mask).expect("four words");
        // Then every bit survives and ambiguous three/five-word values reject.
        assert_eq!(
            ron::from_str::<KnowledgeMask>(&text).expect("roundtrip"),
            mask
        );
        for invalid in ["(1,2,3)", "(1,2,3,4,5)", "(1,2,-1,4)"] {
            assert!(ron::from_str::<KnowledgeMask>(invalid).is_err());
        }
        let json = serde_json::to_string(&mask).expect("json");
        assert_eq!(
            serde_json::from_str::<KnowledgeMask>(&json).expect("json roundtrip"),
            mask
        );
    }
}
