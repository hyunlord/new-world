use std::collections::HashMap;
use std::path::Path;

use serde::Deserialize;

use crate::error::{DataError, DataResult};
use crate::loader::{list_json_files, load_json};

#[derive(Debug, Clone, Deserialize)]
struct OccupationCategoriesFile {
    #[serde(rename = "_comment", default)]
    _comment: Option<String>,
    #[serde(default)]
    categories: HashMap<String, Vec<String>>,
    #[serde(default)]
    default: String,
}

#[derive(Debug, Clone)]
pub struct OccupationCategories {
    categories: HashMap<String, Vec<String>>,
    default: String,
}

impl OccupationCategories {
    pub fn len(&self) -> usize {
        self.categories.len()
    }

    pub fn is_empty(&self) -> bool {
        self.categories.is_empty()
    }

    pub fn default_job(&self) -> &str {
        &self.default
    }

    pub fn get(&self, category: &str) -> Option<&Vec<String>> {
        self.categories.get(category)
    }
}

#[derive(Debug, Clone, Deserialize)]
pub struct JobProfile {
    pub job_id: String,
    pub riasec: String,
    #[serde(default)]
    pub hexaco_ideal: HashMap<String, f64>,
    #[serde(default)]
    pub value_weights: HashMap<String, f64>,
    pub primary_skill: String,
    pub prestige: f64,
    pub autonomy_level: f64,
    pub danger_level: f64,
    pub creativity_level: f64,
}

#[derive(Debug, Clone)]
pub struct JobProfiles(HashMap<String, JobProfile>);

impl JobProfiles {
    pub fn len(&self) -> usize {
        self.0.len()
    }

    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }

    pub fn get(&self, job_id: &str) -> Option<&JobProfile> {
        self.0.get(job_id)
    }
}

#[derive(Debug, Clone)]
pub struct OccupationData {
    pub categories: OccupationCategories,
    pub jobs: JobProfiles,
}

/// Load occupation categories + job profiles from:
/// - `base_dir/occupations/occupation_categories.json`
/// - `base_dir/jobs/*.json`
pub fn load_occupation_data(base_dir: &Path) -> DataResult<OccupationData> {
    let categories = load_occupation_categories(base_dir)?;
    let jobs = load_job_profiles(base_dir)?;
    Ok(OccupationData { categories, jobs })
}

fn load_occupation_categories(base_dir: &Path) -> DataResult<OccupationCategories> {
    let path = base_dir
        .join("occupations")
        .join("occupation_categories.json");
    let raw: OccupationCategoriesFile = load_json(&path)?;
    validate_occupation_categories(&raw, &path)?;
    Ok(OccupationCategories {
        categories: raw.categories,
        default: raw.default,
    })
}

fn load_job_profiles(base_dir: &Path) -> DataResult<JobProfiles> {
    let jobs_dir = base_dir.join("jobs");
    let files = list_json_files(&jobs_dir)?;
    let mut profiles = HashMap::new();
    for path in files {
        let profile: JobProfile = load_json(&path)?;
        validate_job_profile(&profile, &path)?;
        profiles.insert(profile.job_id.clone(), profile);
    }
    Ok(JobProfiles(profiles))
}

fn validate_occupation_categories(raw: &OccupationCategoriesFile, path: &Path) -> DataResult<()> {
    let p = path.display().to_string();
    if raw.default.trim().is_empty() {
        return Err(DataError::MissingField {
            field: "default".to_string(),
            path: p.clone(),
        });
    }

    for (category, occupations) in &raw.categories {
        if occupations.is_empty() {
            return Err(DataError::InvalidField {
                field: format!("categories.{}", category),
                path: p.clone(),
                reason: "expected non-empty occupation list".to_string(),
            });
        }
        for occupation in occupations {
            if occupation.trim().is_empty() {
                return Err(DataError::InvalidField {
                    field: format!("categories.{}", category),
                    path: p.clone(),
                    reason: "contains empty occupation key".to_string(),
                });
            }
        }
    }

    Ok(())
}

fn validate_job_profile(profile: &JobProfile, path: &Path) -> DataResult<()> {
    let p = path.display().to_string();
    if profile.job_id.trim().is_empty() {
        return Err(DataError::MissingField {
            field: "job_id".to_string(),
            path: p.clone(),
        });
    }
    let file_job_id = path
        .file_stem()
        .and_then(|s| s.to_str())
        .unwrap_or_default()
        .to_owned();
    if !file_job_id.is_empty() && file_job_id != profile.job_id {
        return Err(DataError::InvalidField {
            field: "job_id".to_string(),
            path: p.clone(),
            reason: format!(
                "job_id '{}' does not match file name '{}'",
                profile.job_id, file_job_id
            ),
        });
    }
    if profile.riasec.trim().is_empty() {
        return Err(DataError::MissingField {
            field: "riasec".to_string(),
            path: p.clone(),
        });
    }
    if profile.primary_skill.trim().is_empty() {
        return Err(DataError::MissingField {
            field: "primary_skill".to_string(),
            path: p.clone(),
        });
    }

    validate_unit_range(profile.prestige, "prestige", &p)?;
    validate_unit_range(profile.autonomy_level, "autonomy_level", &p)?;
    validate_unit_range(profile.danger_level, "danger_level", &p)?;
    validate_unit_range(profile.creativity_level, "creativity_level", &p)?;

    for (axis, value) in &profile.hexaco_ideal {
        if !(-1.0..=1.0).contains(value) {
            return Err(DataError::InvalidField {
                field: format!("hexaco_ideal.{}", axis),
                path: p.clone(),
                reason: "must be in [-1, 1]".to_string(),
            });
        }
    }
    for (key, value) in &profile.value_weights {
        if !(0.0..=1.0).contains(value) {
            return Err(DataError::InvalidField {
                field: format!("value_weights.{}", key),
                path: p.clone(),
                reason: "must be in [0, 1]".to_string(),
            });
        }
    }

    Ok(())
}

fn validate_unit_range(value: f64, field: &str, path: &str) -> DataResult<()> {
    if !(0.0..=1.0).contains(&value) {
        return Err(DataError::InvalidField {
            field: field.to_string(),
            path: path.to_string(),
            reason: "must be in [0, 1]".to_string(),
        });
    }
    Ok(())
}
