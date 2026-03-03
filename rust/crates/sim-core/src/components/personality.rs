use serde::{Deserialize, Serialize};
use crate::enums::{HexacoAxis, HexacoFacet, AttachmentType};

pub const AXIS_COUNT: usize = 6;
pub const FACET_COUNT: usize = 24;
pub const FACETS_PER_AXIS: usize = 4;

/// HEXACO 6-axis 24-facet personality model (Ashton & Lee)
/// All values 0.0..=1.0, normal distribution mean=0.5 std=0.15
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Personality {
    /// H, E, X, A, C, O axis values (derived from facet averages)
    pub axes: [f64; AXIS_COUNT],
    /// 24 facets: index = axis_index * 4 + facet_offset
    pub facets: [f64; FACET_COUNT],
    /// Bowlby (1969) attachment style
    pub attachment: AttachmentType,
}

impl Default for Personality {
    fn default() -> Self {
        Self {
            axes: [0.5; AXIS_COUNT],
            facets: [0.5; FACET_COUNT],
            attachment: AttachmentType::Secure,
        }
    }
}

impl Personality {
    #[inline]
    pub fn axis(&self, a: HexacoAxis) -> f64 {
        self.axes[a as usize]
    }

    #[inline]
    pub fn facet(&self, f: HexacoFacet) -> f64 {
        self.facets[f as usize]
    }

    /// Recalculate axis values from facet averages
    pub fn recalculate_axes(&mut self) {
        for i in 0..AXIS_COUNT {
            let start = i * FACETS_PER_AXIS;
            let sum: f64 = self.facets[start..start + FACETS_PER_AXIS].iter().sum();
            self.axes[i] = sum / FACETS_PER_AXIS as f64;
        }
    }
}
