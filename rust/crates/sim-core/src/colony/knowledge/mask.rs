//! Fixed-width knowledge bits for every stable u8 node ordinal.
use serde::{Deserialize, Deserializer, Serialize, Serializer};
use std::{
    cmp::Ordering,
    fmt,
    ops::{BitAnd, BitAndAssign, BitOr, BitOrAssign, Not},
};

/// A 256-node knowledge set, stored as four little-endian u64 words.
#[derive(Clone, Copy, Default, PartialEq, Eq, Hash)]
pub struct KnowledgeMask([u64; 4]);

impl KnowledgeMask {
    /// No knowledge nodes.
    pub const ZERO: Self = Self([0; 4]);
    /// Every representable knowledge node.
    pub const MAX: Self = Self([u64::MAX; 4]);
    /// Construct from explicit little-endian words at a serialization boundary.
    pub const fn from_words(words: [u64; 4]) -> Self {
        Self(words)
    }
    /// Explicit little-endian words for DTOs and versioned save tails.
    pub const fn words(self) -> [u64; 4] {
        self.0
    }
    /// The historical low 128 bits; authoritative upper bits need their save tail.
    pub const fn low128(self) -> u128 {
        self.0[0] as u128 | ((self.0[1] as u128) << 64)
    }
    /// The upper half, in little-endian word order.
    pub const fn upper(self) -> [u64; 2] {
        [self.0[2], self.0[3]]
    }
    /// A single node; u8 ordinals cover exactly the entire capacity.
    pub const fn bit(id: u8) -> Self {
        let mut words = [0; 4];
        words[(id / 64) as usize] = 1u64 << (id % 64);
        Self(words)
    }
    /// Whether this set contains every node in `other`.
    pub const fn contains_all(self, other: Self) -> bool {
        self.0[0] & other.0[0] == other.0[0]
            && self.0[1] & other.0[1] == other.0[1]
            && self.0[2] & other.0[2] == other.0[2]
            && self.0[3] & other.0[3] == other.0[3]
    }
    /// Whether no node is set.
    pub const fn is_empty(self) -> bool {
        self.0[0] | self.0[1] | self.0[2] | self.0[3] == 0
    }
    /// Number of nodes set across all four words.
    pub const fn count_ones(self) -> u32 {
        self.0[0].count_ones()
            + self.0[1].count_ones()
            + self.0[2].count_ones()
            + self.0[3].count_ones()
    }
}

impl From<u128> for KnowledgeMask {
    fn from(value: u128) -> Self {
        Self([value as u64, (value >> 64) as u64, 0, 0])
    }
}

// Historical zero-mask comparisons remain readable without implicit narrowing.
impl PartialEq<u128> for KnowledgeMask {
    fn eq(&self, other: &u128) -> bool {
        self.upper() == [0, 0] && self.low128() == *other
    }
}
impl Ord for KnowledgeMask {
    fn cmp(&self, other: &Self) -> Ordering {
        self.0.iter().rev().cmp(other.0.iter().rev())
    }
}
impl PartialOrd for KnowledgeMask {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}
impl fmt::Debug for KnowledgeMask {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        // Preserve historical fingerprints when the additional words are unused.
        if self.upper() == [0, 0] {
            self.low128().fmt(f)
        } else {
            self.0.fmt(f)
        }
    }
}
impl fmt::Display for KnowledgeMask {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::Debug::fmt(self, f)
    }
}
impl BitOr for KnowledgeMask {
    type Output = Self;
    fn bitor(self, rhs: Self) -> Self {
        Self(std::array::from_fn(|i| self.0[i] | rhs.0[i]))
    }
}
impl BitAnd for KnowledgeMask {
    type Output = Self;
    fn bitand(self, rhs: Self) -> Self {
        Self(std::array::from_fn(|i| self.0[i] & rhs.0[i]))
    }
}
impl BitOrAssign for KnowledgeMask {
    fn bitor_assign(&mut self, rhs: Self) {
        *self = *self | rhs;
    }
}
impl BitAndAssign for KnowledgeMask {
    fn bitand_assign(&mut self, rhs: Self) {
        *self = *self & rhs;
    }
}
impl Not for KnowledgeMask {
    type Output = Self;
    fn not(self) -> Self {
        Self(self.0.map(|word| !word))
    }
}
impl Serialize for KnowledgeMask {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        if serializer.is_human_readable() {
            super::mask_serde::serialize(self, serializer)
        } else {
            self.0.serialize(serializer)
        }
    }
}
impl<'de> Deserialize<'de> for KnowledgeMask {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        if deserializer.is_human_readable() {
            super::mask_serde::deserialize(deserializer)
        } else {
            <[u64; 4]>::deserialize(deserializer).map(Self)
        }
    }
}
