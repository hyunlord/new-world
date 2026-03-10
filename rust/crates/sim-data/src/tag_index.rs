use std::collections::HashMap;
use std::sync::Arc;

use crate::{MaterialDef, TagRequirement};

/// Reverse index from tag to matching materials.
#[derive(Debug, Clone, Default)]
pub struct TagIndex {
    tag_to_materials: HashMap<String, Vec<Arc<MaterialDef>>>,
}

impl TagIndex {
    /// Build a reverse tag index from the material registry.
    pub fn build(materials: &HashMap<String, Arc<MaterialDef>>) -> Self {
        let mut tag_to_materials: HashMap<String, Vec<Arc<MaterialDef>>> = HashMap::new();
        for material in materials.values() {
            for tag in &material.tags {
                tag_to_materials
                    .entry(tag.clone())
                    .or_default()
                    .push(Arc::clone(material));
            }
        }

        for materials in tag_to_materials.values_mut() {
            materials.sort_by(|left, right| left.id.cmp(&right.id));
        }

        Self { tag_to_materials }
    }

    /// Query all materials with a tag.
    pub fn query(&self, tag: &str) -> &[Arc<MaterialDef>] {
        self.tag_to_materials
            .get(tag)
            .map(Vec::as_slice)
            .unwrap_or(&[])
    }

    /// Query materials by tag plus property thresholds.
    pub fn query_with_threshold(&self, req: &TagRequirement) -> Vec<Arc<MaterialDef>> {
        self.query(&req.tag)
            .iter()
            .filter(|material| {
                req.min_hardness
                    .is_none_or(|min| material.properties.hardness >= min)
                    && req
                        .min_density
                        .is_none_or(|min| material.properties.density >= min)
                    && req
                        .max_rarity
                        .is_none_or(|max| material.properties.rarity <= max)
            })
            .cloned()
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::{BTreeSet, HashMap};

    fn sample_material(
        id: &str,
        tags: &[&str],
        hardness: f64,
        density: f64,
        rarity: f64,
    ) -> MaterialDef {
        MaterialDef {
            id: id.to_string(),
            display_name_key: format!("MAT_{}", id.to_ascii_uppercase()),
            category: crate::MaterialCategory::Stone,
            tags: tags
                .iter()
                .map(|tag| (*tag).to_string())
                .collect::<BTreeSet<_>>(),
            properties: crate::MaterialProperties {
                hardness,
                density,
                melting_point: None,
                rarity,
                value: 1.0,
            },
        }
    }

    #[test]
    fn query_with_threshold_filters_materials() {
        let mut materials = HashMap::new();
        materials.insert(
            "flint".to_string(),
            Arc::new(sample_material("flint", &["sharp"], 7.0, 2.6, 0.3)),
        );
        materials.insert(
            "granite".to_string(),
            Arc::new(sample_material("granite", &["sharp"], 3.0, 2.7, 0.1)),
        );

        let index = TagIndex::build(&materials);
        let matches = index.query_with_threshold(&TagRequirement {
            tag: "sharp".to_string(),
            min_hardness: Some(4.0),
            min_density: None,
            max_rarity: None,
            amount: 1,
        });

        assert_eq!(matches.len(), 1);
        assert_eq!(matches[0].id, "flint");
    }
}
