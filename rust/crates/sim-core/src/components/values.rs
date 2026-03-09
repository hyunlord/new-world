// TODO(v3.1): Keep manual serde until the toolchain supports deriving for `[f64; 33]`.
use serde::{Deserialize, Deserializer, Serialize, Serializer};
use serde::de::Error;
use serde::ser::SerializeSeq;
use crate::enums::ValueType;

pub const VALUE_COUNT: usize = 33;

/// 33 personal values (-1.0..=+1.0)
/// Positive = value embraced, negative = value rejected
#[derive(Debug, Clone)]
pub struct Values {
    pub values: [f64; VALUE_COUNT],
}

impl Serialize for Values {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let mut seq = serializer.serialize_seq(Some(VALUE_COUNT))?;
        for value in self.values {
            seq.serialize_element(&value)?;
        }
        seq.end()
    }
}

impl<'de> Deserialize<'de> for Values {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let input = Vec::<f64>::deserialize(deserializer)?;
        if input.len() != VALUE_COUNT {
            return Err(D::Error::custom(format!(
                "expected {VALUE_COUNT} values, got {}",
                input.len()
            )));
        }

        let mut values = [0.0_f64; VALUE_COUNT];
        values.copy_from_slice(&input);
        Ok(Self { values })
    }
}

impl Default for Values {
    fn default() -> Self {
        Self { values: [0.0; VALUE_COUNT] }
    }
}

impl Values {
    #[inline]
    pub fn get(&self, v: ValueType) -> f64 {
        self.values[v as usize]
    }

    #[inline]
    pub fn get_mut(&mut self, v: ValueType) -> &mut f64 {
        &mut self.values[v as usize]
    }

    #[inline]
    pub fn set(&mut self, v: ValueType, val: f64) {
        self.values[v as usize] = val.clamp(-1.0, 1.0);
    }

    /// Value alignment score with another Values component (0.0..=1.0)
    pub fn alignment_with(&self, other: &Values) -> f64 {
        let dot: f64 = self.values.iter().zip(other.values.iter())
            .map(|(a, b)| a * b)
            .sum();
        (dot / VALUE_COUNT as f64 + 1.0) / 2.0
    }
}
