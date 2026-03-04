use crate::loader::load_json;
use serde::Deserialize;
use std::collections::HashMap;
use std::path::Path;

use crate::DataResult;

#[derive(Debug, Clone, Deserialize)]
pub struct CorrelationMatrix {
    pub axes_order: Vec<String>,
    pub matrix: Vec<Vec<f64>>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct MaturationEntry {
    pub target_shift: f64,
    pub age_range: [f64; 2],
}

#[derive(Debug, Clone, Deserialize)]
pub struct OuParameters {
    pub theta: f64,
    pub sigma: f64,
}

/// Raw deserialization target — maturation maps axis keys to entries, but the
/// JSON object also contains a `"comment"` string key that serde cannot
/// deserialize as `MaturationEntry`. We capture the whole object as a
/// `HashMap<String, serde_json::Value>` and filter to only object-valued keys.
#[derive(Debug, Clone, Deserialize)]
struct PersonalityDistributionRaw {
    pub sd: f64,
    pub correlation_matrix: CorrelationMatrix,
    /// Heritability map also contains a `"comment"` key; unknown fields are
    /// silently ignored by serde (no `deny_unknown_fields`), but because the
    /// value is a string rather than f64 it would cause a parse error if the
    /// key were included. We capture as `HashMap<String, serde_json::Value>`
    /// and filter to numeric entries only.
    pub heritability: HashMap<String, serde_json::Value>,
    pub sex_difference_d: HashMap<String, serde_json::Value>,
    pub maturation: HashMap<String, serde_json::Value>,
    pub facet_spread: f64,
    pub ou_parameters: OuParameters,
}

#[derive(Debug, Clone)]
pub struct PersonalityDistribution {
    pub sd: f64,
    pub correlation_matrix: CorrelationMatrix,
    /// HEXACO axis → heritability coefficient (0.0–1.0).
    pub heritability: HashMap<String, f64>,
    /// HEXACO axis → Cohen's d sex difference (positive = female higher).
    pub sex_difference_d: HashMap<String, f64>,
    /// HEXACO axis → maturation parameters.
    pub maturation: HashMap<String, MaturationEntry>,
    pub facet_spread: f64,
    pub ou_parameters: OuParameters,
}

/// Load personality distribution from
/// `{base_dir}/species/human/personality/distribution.json`.
pub fn load_personality_distribution(base_dir: &Path) -> DataResult<PersonalityDistribution> {
    let path = base_dir.join("species/human/personality/distribution.json");
    let raw: PersonalityDistributionRaw = load_json(&path)?;

    // Filter heritability: keep only numeric values (skip "comment" strings).
    let heritability: HashMap<String, f64> = raw
        .heritability
        .into_iter()
        .filter_map(|(k, v)| v.as_f64().map(|f| (k, f)))
        .collect();

    // Filter sex_difference_d similarly.
    let sex_difference_d: HashMap<String, f64> = raw
        .sex_difference_d
        .into_iter()
        .filter_map(|(k, v)| v.as_f64().map(|f| (k, f)))
        .collect();

    // Filter maturation: keep only object values and deserialize each.
    let mut maturation: HashMap<String, MaturationEntry> = HashMap::new();
    for (k, v) in raw.maturation {
        if v.is_object() {
            let entry: MaturationEntry = serde_json::from_value(v).map_err(|e| {
                crate::DataError::Json {
                    path: base_dir
                        .join("species/human/personality/distribution.json")
                        .display()
                        .to_string(),
                    source: e,
                }
            })?;
            maturation.insert(k, entry);
        }
    }

    Ok(PersonalityDistribution {
        sd: raw.sd,
        correlation_matrix: raw.correlation_matrix,
        heritability,
        sex_difference_d,
        maturation,
        facet_spread: raw.facet_spread,
        ou_parameters: raw.ou_parameters,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    const MINIMAL_JSON: &str = r#"{
        "sd": 0.25,
        "comment_sd": "ignored",
        "correlation_matrix": {
            "comment": "ignored",
            "axes_order": ["H", "E", "X", "A", "C", "O"],
            "matrix": [
                [1.00, 0.12, -0.11, 0.26, 0.18, 0.21],
                [0.12, 1.00, -0.13, -0.08, 0.15, -0.10],
                [-0.11, -0.13, 1.00, 0.05, 0.10, 0.08],
                [0.26, -0.08, 0.05, 1.00, 0.01, 0.03],
                [0.18, 0.15, 0.10, 0.01, 1.00, 0.03],
                [0.21, -0.10, 0.08, 0.03, 0.03, 1.00]
            ]
        },
        "heritability": {
            "comment": "ignored",
            "H": 0.45, "E": 0.58, "X": 0.57, "A": 0.47, "C": 0.52, "O": 0.63
        },
        "sex_difference_d": {
            "comment": "ignored",
            "H": 0.41, "E": 0.96, "X": 0.10, "A": 0.28, "C": 0.00, "O": -0.04
        },
        "maturation": {
            "comment": "ignored",
            "H": {"target_shift": 1.0, "age_range": [18, 60]},
            "E": {"target_shift": 0.3, "age_range": [18, 60]},
            "X": {"target_shift": 0.3, "age_range": [18, 60]},
            "A": {"target_shift": 0.0, "age_range": [18, 60]},
            "C": {"target_shift": 0.0, "age_range": [18, 60]},
            "O": {"target_shift": 0.0, "age_range": [18, 60]}
        },
        "facet_spread": 0.75,
        "comment_facet_spread": "ignored",
        "ou_parameters": {
            "comment": "ignored",
            "theta": 0.03,
            "sigma": 0.03
        }
    }"#;

    fn parse_minimal() -> PersonalityDistribution {
        let raw: PersonalityDistributionRaw =
            serde_json::from_str(MINIMAL_JSON).expect("raw parse failed");

        let heritability: HashMap<String, f64> = raw
            .heritability
            .into_iter()
            .filter_map(|(k, v)| v.as_f64().map(|f| (k, f)))
            .collect();

        let sex_difference_d: HashMap<String, f64> = raw
            .sex_difference_d
            .into_iter()
            .filter_map(|(k, v)| v.as_f64().map(|f| (k, f)))
            .collect();

        let mut maturation: HashMap<String, MaturationEntry> = HashMap::new();
        for (k, v) in raw.maturation {
            if v.is_object() {
                let entry: MaturationEntry =
                    serde_json::from_value(v).expect("maturation entry parse failed");
                maturation.insert(k, entry);
            }
        }

        PersonalityDistribution {
            sd: raw.sd,
            correlation_matrix: raw.correlation_matrix,
            heritability,
            sex_difference_d,
            maturation,
            facet_spread: raw.facet_spread,
            ou_parameters: raw.ou_parameters,
        }
    }

    #[test]
    fn sd_is_correct() {
        let dist = parse_minimal();
        assert!(
            (dist.sd - 0.25).abs() < f64::EPSILON,
            "sd should be 0.25, got {}",
            dist.sd
        );
    }

    #[test]
    fn heritability_h_is_correct() {
        let dist = parse_minimal();
        let h = dist.heritability.get("H").copied().unwrap_or(0.0);
        assert!(
            (h - 0.45).abs() < f64::EPSILON,
            "heritability[H] should be 0.45, got {}",
            h
        );
    }

    #[test]
    fn heritability_comment_excluded() {
        let dist = parse_minimal();
        assert!(
            !dist.heritability.contains_key("comment"),
            "heritability map must not contain 'comment' key"
        );
    }

    #[test]
    fn correlation_matrix_is_6x6() {
        let dist = parse_minimal();
        let m = &dist.correlation_matrix.matrix;
        assert_eq!(m.len(), 6, "matrix should have 6 rows");
        for row in m {
            assert_eq!(row.len(), 6, "each matrix row should have 6 columns");
        }
    }

    #[test]
    fn facet_spread_is_correct() {
        let dist = parse_minimal();
        assert!(
            (dist.facet_spread - 0.75).abs() < f64::EPSILON,
            "facet_spread should be 0.75, got {}",
            dist.facet_spread
        );
    }

    #[test]
    fn maturation_has_six_axes() {
        let dist = parse_minimal();
        assert_eq!(
            dist.maturation.len(),
            6,
            "maturation should have 6 axis entries (comment key excluded)"
        );
        assert!(
            !dist.maturation.contains_key("comment"),
            "maturation map must not contain 'comment' key"
        );
    }

    #[test]
    fn ou_parameters_correct() {
        let dist = parse_minimal();
        assert!(
            (dist.ou_parameters.theta - 0.03).abs() < f64::EPSILON,
            "theta should be 0.03"
        );
        assert!(
            (dist.ou_parameters.sigma - 0.03).abs() < f64::EPSILON,
            "sigma should be 0.03"
        );
    }
}
