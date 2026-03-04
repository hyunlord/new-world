//! Name generator: port of name_generator.gd.
//!
//! Generates culturally-appropriate, unique names for agents using either
//! syllabic (Markov-style) generation or pre-built name pools.
//! Tracks used names per settlement to avoid duplicates.

use std::collections::{HashMap, HashSet};
use rand::Rng;
use sim_core::enums::Sex;
use sim_data::NameCulture;

// Minimal fallback pools used when no cultures are loaded at all.
const FALLBACK_MALE: &[&str] = &["Oak", "Ash", "Elm", "Birch", "Cedar"];
const FALLBACK_FEMALE: &[&str] = &["Ivy", "Lily", "Rose", "Fern", "Dawn"];

/// Generates unique agent names using culture-specific rules.
///
/// Tracks used names per settlement so every agent in a settlement
/// has a distinct name. Names may still collide across settlements.
pub struct NameGenerator {
    cultures: HashMap<String, NameCulture>,
    /// `settlement_id` → set of names already in use.
    used_names: HashMap<u32, HashSet<String>>,
}

impl NameGenerator {
    pub fn new(cultures: HashMap<String, NameCulture>) -> Self {
        Self {
            cultures,
            used_names: HashMap::new(),
        }
    }

    /// Generate a unique name for the given sex, culture, and settlement.
    ///
    /// Algorithm (port of name_generator.gd::generate_name):
    /// 1. Look up the requested culture; fall back to "proto_nature" or
    ///    built-in fallback pools.
    /// 2. Try up to 20 times:
    ///    - 70 % syllabic (if `allow_markov_generation`) or
    ///    - 30 % pool pick — swap to syllabic if pool is empty.
    /// 3. Apply patronymic if the culture rule != "none" and a parent name is given.
    /// 4. Accept the first candidate that is not a duplicate for this settlement.
    /// 5. If all 20 attempts collide, append " II" and accept unconditionally.
    pub fn generate_name(
        &mut self,
        sex: Sex,
        culture_id: &str,
        settlement_id: u32,
        parent_name: Option<&str>,
        rng: &mut impl Rng,
    ) -> String {
        // Resolve culture (requested → proto_nature fallback → built-in).
        let culture = self
            .cultures
            .get(culture_id)
            .or_else(|| self.cultures.get("proto_nature"))
            .cloned();

        for _ in 0..20 {
            let candidate = if let Some(ref c) = culture {
                self.pick_candidate(c, sex, rng)
            } else {
                self.pick_fallback(sex, rng)
            };

            if candidate.is_empty() {
                continue;
            }

            let full = if let Some(ref c) = culture {
                if c.patronymic_rule != "none" {
                    if let Some(pname) = parent_name {
                        self.apply_patronymic(&candidate, pname, sex, c)
                    } else {
                        candidate
                    }
                } else {
                    candidate
                }
            } else {
                candidate
            };

            if !self.is_duplicate(&full, settlement_id) {
                self.register_name(&full, settlement_id);
                return full;
            }
        }

        // All 20 tries were duplicates — generate one more and force-accept with " II".
        let base = if let Some(ref c) = culture {
            self.generate_syllabic(c, sex, rng)
        } else {
            self.pick_fallback(sex, rng)
        };
        let fallback = if base.is_empty() {
            format!("Wanderer {}", rng.gen_range(100u32..999))
        } else {
            format!("{} II", base)
        };
        self.register_name(&fallback, settlement_id);
        fallback
    }

    /// Record a name as used in the given settlement.
    pub fn register_name(&mut self, name: &str, settlement_id: u32) {
        self.used_names
            .entry(settlement_id)
            .or_default()
            .insert(name.to_string());
    }

    /// Remove a name from the used-names set (e.g. on agent death/migration).
    pub fn unregister_name(&mut self, name: &str, settlement_id: u32) {
        if let Some(set) = self.used_names.get_mut(&settlement_id) {
            set.remove(name);
        }
    }

    // ── Private helpers ───────────────────────────────────────────────────────

    /// Pick a candidate name from a culture (syllabic or pool).
    fn pick_candidate(&self, culture: &NameCulture, sex: Sex, rng: &mut impl Rng) -> String {
        if culture.allow_markov_generation && rng.gen::<f64>() > 0.3 {
            self.generate_syllabic(culture, sex, rng)
        } else {
            let pool = match sex {
                Sex::Male => &culture.given_names.male,
                Sex::Female => &culture.given_names.female,
            };
            if pool.is_empty() {
                self.generate_syllabic(culture, sex, rng)
            } else {
                pool[rng.gen_range(0..pool.len())].clone()
            }
        }
    }

    /// Pick from built-in fallback pools (used when no cultures are loaded).
    fn pick_fallback(&self, sex: Sex, rng: &mut impl Rng) -> String {
        let pool = match sex {
            Sex::Male => FALLBACK_MALE,
            Sex::Female => FALLBACK_FEMALE,
        };
        pool[rng.gen_range(0..pool.len())].to_string()
    }

    /// Generate a syllabic name from culture pools.
    ///
    /// Port of name_generator.gd::generate_syllabic_name().
    ///
    /// Structure per syllable:
    ///  - onset (70 % chance after first syllable)
    ///  - nucleus (always)
    ///  - coda (60 % chance for non-final); coda_final for final syllable
    fn generate_syllabic(&self, culture: &NameCulture, sex: Sex, rng: &mut impl Rng) -> String {
        let pools = match &culture.syllable_pools {
            Some(p) => p,
            None => {
                // No syllable pools — fall back to the name pool.
                let pool = match sex {
                    Sex::Male => &culture.given_names.male,
                    Sex::Female => &culture.given_names.female,
                };
                return if pool.is_empty() {
                    String::new()
                } else {
                    pool[rng.gen_range(0..pool.len())].clone()
                };
            }
        };

        let (min, max) = culture
            .syllable_count
            .as_ref()
            .map(|sc| (sc.min as usize, sc.max as usize))
            .unwrap_or((2, 3));

        let count = if min >= max { min } else { rng.gen_range(min..=max) };

        let mut result = String::new();

        for i in 0..count {
            // Onset
            let onset_pool = match sex {
                Sex::Male if !pools.onset_male.is_empty() => &pools.onset_male,
                Sex::Female if !pools.onset_female.is_empty() => &pools.onset_female,
                Sex::Male => &pools.onset_female,
                Sex::Female => &pools.onset_male,
            };
            if !onset_pool.is_empty() && (i == 0 || rng.gen::<f64>() < 0.7) {
                result.push_str(&onset_pool[rng.gen_range(0..onset_pool.len())]);
            }

            // Nucleus (always)
            if !pools.nucleus.is_empty() {
                result.push_str(&pools.nucleus[rng.gen_range(0..pools.nucleus.len())]);
            }

            // Coda
            if i + 1 == count {
                // Final syllable: use coda_final
                if !pools.coda_final.is_empty() && rng.gen::<f64>() < 0.6 {
                    result.push_str(&pools.coda_final[rng.gen_range(0..pools.coda_final.len())]);
                }
            } else if !pools.coda.is_empty() && rng.gen::<f64>() < 0.6 {
                result.push_str(&pools.coda[rng.gen_range(0..pools.coda.len())]);
            }
        }

        capitalize_first(&result)
    }

    /// Apply patronymic rule to a given name.
    fn apply_patronymic(
        &self,
        given: &str,
        parent_name: &str,
        sex: Sex,
        culture: &NameCulture,
    ) -> String {
        let config = match &culture.patronymic_config {
            Some(c) => c,
            None => return given.to_string(),
        };

        // Use only the first word of the parent's name as the base.
        let parent_base = parent_name.split_whitespace().next().unwrap_or(parent_name);

        match culture.patronymic_rule.as_str() {
            "suffix" => {
                let suffix = match sex {
                    Sex::Male => config.male_suffix.as_deref().unwrap_or(""),
                    Sex::Female => config.female_suffix.as_deref().unwrap_or(""),
                };
                if suffix.is_empty() {
                    given.to_string()
                } else {
                    format!("{} {}{}", given, parent_base, suffix)
                }
            }
            "prefix" => {
                let prefix = match sex {
                    Sex::Male => config.male_prefix.as_deref().unwrap_or(""),
                    Sex::Female => config.female_prefix.as_deref().unwrap_or(""),
                };
                if prefix.is_empty() {
                    format!("{} {}", given, parent_base)
                } else {
                    format!("{} {}{}", given, prefix, parent_base)
                }
            }
            _ => given.to_string(),
        }
    }

    fn is_duplicate(&self, name: &str, settlement_id: u32) -> bool {
        self.used_names
            .get(&settlement_id)
            .map(|s| s.contains(name))
            .unwrap_or(false)
    }
}

/// Capitalize the first Unicode character of a string.
fn capitalize_first(s: &str) -> String {
    let mut chars = s.chars();
    match chars.next() {
        None => String::new(),
        Some(c) => c.to_uppercase().collect::<String>() + chars.as_str(),
    }
}

// ── Tests ──────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use rand::SeedableRng;
    use rand::rngs::SmallRng;
    use sim_data::load_name_cultures;

    fn data_dir() -> std::path::PathBuf {
        std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
            .parent()
            .unwrap()
            .parent()
            .unwrap()
            .parent()
            .unwrap()
            .join("data")
    }

    fn load_cultures() -> HashMap<String, NameCulture> {
        let dir = data_dir();
        if dir.exists() { load_name_cultures(&dir) } else { HashMap::new() }
    }

    #[test]
    fn generates_20_unique_names() {
        let cultures = load_cultures();
        if cultures.is_empty() { return; }

        let mut gen = NameGenerator::new(cultures);
        let mut rng = SmallRng::seed_from_u64(42);
        let mut names = HashSet::new();

        for _ in 0..20 {
            let name = gen.generate_name(Sex::Male, "proto_nature", 1, None, &mut rng);
            assert!(!name.is_empty(), "Name should not be empty");
            assert!(names.insert(name.clone()), "Duplicate name generated: '{}'", name);
        }
    }

    #[test]
    fn names_start_with_uppercase() {
        let cultures = load_cultures();
        if cultures.is_empty() { return; }

        let mut gen = NameGenerator::new(cultures);
        let mut rng = SmallRng::seed_from_u64(99);

        for _ in 0..20 {
            let name = gen.generate_name(Sex::Female, "proto_nature", 2, None, &mut rng);
            let first_char = name.chars().next();
            assert!(
                first_char.map(|c| c.is_uppercase()).unwrap_or(false),
                "Name '{}' should start with uppercase",
                name
            );
        }
    }

    #[test]
    fn fallback_produces_nonempty_names_without_data() {
        let mut gen = NameGenerator::new(HashMap::new());
        let mut rng = SmallRng::seed_from_u64(42);
        let name = gen.generate_name(Sex::Male, "proto_nature", 0, None, &mut rng);
        assert!(!name.is_empty(), "Fallback should produce a non-empty name");
    }

    #[test]
    fn syllabic_culture_generates_nonempty_names() {
        let cultures = load_cultures();
        if !cultures.contains_key("proto_syllabic") { return; }

        let mut gen = NameGenerator::new(cultures);
        let mut rng = SmallRng::seed_from_u64(7);

        for _ in 0..10 {
            let name = gen.generate_name(Sex::Female, "proto_syllabic", 3, None, &mut rng);
            assert!(!name.is_empty(), "Syllabic culture should produce non-empty names");
        }
    }

    #[test]
    fn register_and_unregister_work() {
        let mut gen = NameGenerator::new(HashMap::new());
        gen.register_name("TestName", 1);
        assert!(gen.is_duplicate("TestName", 1));
        gen.unregister_name("TestName", 1);
        assert!(!gen.is_duplicate("TestName", 1));
    }

    #[test]
    fn different_settlements_can_have_same_name() {
        let cultures = load_cultures();
        if cultures.is_empty() { return; }

        let mut gen = NameGenerator::new(cultures);
        // Manually register "Oak" in settlement 1
        gen.register_name("Oak", 1);
        // It should not be a duplicate in settlement 2
        assert!(!gen.is_duplicate("Oak", 2));
    }
}
