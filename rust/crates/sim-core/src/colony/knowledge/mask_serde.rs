//! Human-readable knowledge words with unchanged binary u128 varints.
use serde::{de, Deserialize, Deserializer, Serialize, Serializer};
use std::fmt;

pub(crate) fn serialize<S: Serializer>(mask: &u128, serializer: S) -> Result<S::Ok, S::Error> {
    if !serializer.is_human_readable() {
        return serializer.serialize_u128(*mask);
    }
    let low = *mask as u64;
    let high = (*mask >> 64) as u64;
    if high == 0 {
        serializer.serialize_u64(low)
    } else {
        (low, high).serialize(serializer)
    }
}

pub(crate) fn deserialize<'de, D: Deserializer<'de>>(deserializer: D) -> Result<u128, D::Error> {
    if !deserializer.is_human_readable() {
        return u128::deserialize(deserializer);
    }
    struct Words;
    impl<'de> de::Visitor<'de> for Words {
        type Value = u128;
        fn expecting(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
            formatter.write_str("a legacy unsigned mask or a (low, high) u64 pair")
        }
        fn visit_u64<E: de::Error>(self, low: u64) -> Result<u128, E> {
            Ok(u128::from(low))
        }
        fn visit_seq<A: de::SeqAccess<'de>>(self, mut seq: A) -> Result<u128, A::Error> {
            let low: u64 = seq
                .next_element()?
                .ok_or_else(|| de::Error::invalid_length(0, &self))?;
            let high: u64 = seq
                .next_element()?
                .ok_or_else(|| de::Error::invalid_length(1, &self))?;
            if seq.next_element::<de::IgnoredAny>()?.is_some() {
                return Err(de::Error::invalid_length(3, &self));
            }
            Ok(u128::from(low) | (u128::from(high) << 64))
        }
    }
    deserializer.deserialize_any(Words)
}

#[cfg(test)]
mod tests {
    use serde::{Deserialize, Serialize};

    #[derive(Debug, PartialEq, Serialize, Deserialize)]
    struct Mask(#[serde(with = "super")] u128);

    #[test]
    fn c1_human_mask_preserves_legacy_numbers_and_rejects_malformed_words() {
        for mask in [
            0,
            u128::from(u64::MAX),
            1_u128 << 64,
            1_u128 << 127,
            u128::MAX,
        ] {
            let text = ron::to_string(&Mask(mask)).expect("human mask");
            assert_eq!(
                ron::from_str::<Mask>(&text).expect("mask roundtrip"),
                Mask(mask)
            );
        }
        assert_eq!(ron::to_string(&Mask(7)).expect("legacy shape"), "(7)");
        for invalid in ["(-1)", "((1))", "((1,2,3))", "((1,-1))"] {
            assert!(ron::from_str::<Mask>(invalid).is_err(), "{invalid}");
        }
    }
}
