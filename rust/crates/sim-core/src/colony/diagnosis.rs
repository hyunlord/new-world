//! Read-only state diagnosis framework (M-READ §2.1, ADR 0055).
//! Checks are data-driven from `data/colony/diagnosis.ron`; code evaluates
//! predicates using the mind's operator set and follows `next` chains to
//! depth <= 5. Diagnosis never modifies simulation state.

use serde::Deserialize;
use std::collections::BTreeMap;

/// Embedded diagnosis data, parsed once at startup.
const DIAGNOSIS_RON: &str = include_str!("../../data/colony/diagnosis.ron");

// ── Raw RON types (serde) ──────────────────────────────────────────────

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct DiagnosisFile {
    schema_version: u32,
    subjects: Vec<RawSubjectBlock>,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct RawSubjectBlock {
    kind: String,
    checks: Vec<RawCheck>,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct RawCheck {
    id: String,
    when: Vec<RawPredicate>,
    text_key: String,
    #[serde(default)]
    params: BTreeMap<String, String>,
    severity: String,
    next: Option<RawNextSubject>,
    #[serde(default)]
    tags: Vec<String>,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct RawPredicate {
    fact: String,
    op: String,
    value: Option<i64>,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct RawNextSubject {
    kind: String,
    pick: String,
}

// ── Public resolved types ──────────────────────────────────────────────

/// Subject kinds the diagnosis framework can evaluate.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum SubjectKind {
    /// A living person.
    Person,
    /// A stockpile zone.
    Stockpile,
    /// A settlement hearth.
    Hearth,
    /// A room.
    Room,
    /// The whole settlement.
    Settlement,
}

impl SubjectKind {
    /// Parse from the RON string.
    pub fn from_name(name: &str) -> Option<Self> {
        match name {
            "person" => Some(Self::Person),
            "stockpile" => Some(Self::Stockpile),
            "hearth" => Some(Self::Hearth),
            "room" => Some(Self::Room),
            "settlement" => Some(Self::Settlement),
            _ => None,
        }
    }

    /// Stable string for bridge and locale use.
    pub const fn name(self) -> &'static str {
        match self {
            Self::Person => "person",
            Self::Stockpile => "stockpile",
            Self::Hearth => "hearth",
            Self::Room => "room",
            Self::Settlement => "settlement",
        }
    }

    /// Numeric code for the bridge (0-based).
    pub const fn code(self) -> u8 {
        match self {
            Self::Person => 0,
            Self::Stockpile => 1,
            Self::Hearth => 2,
            Self::Room => 3,
            Self::Settlement => 4,
        }
    }

    /// Reconstruct from the bridge code.
    pub fn from_code(code: u8) -> Option<Self> {
        match code {
            0 => Some(Self::Person),
            1 => Some(Self::Stockpile),
            2 => Some(Self::Hearth),
            3 => Some(Self::Room),
            4 => Some(Self::Settlement),
            _ => None,
        }
    }
}

/// Predicate operator, matching the mind's set.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DiagOp {
    /// `<`
    Lt,
    /// `<=`
    Le,
    /// `>`
    Gt,
    /// `>=`
    Ge,
    /// `==`
    Eq,
    /// Bool true (value == 1).
    Has,
    /// Bool false (value == 0).
    Not,
}

impl DiagOp {
    /// Parse from the authored string.
    fn from_str(s: &str) -> Option<Self> {
        match s {
            "<" => Some(Self::Lt),
            "<=" => Some(Self::Le),
            ">" => Some(Self::Gt),
            ">=" => Some(Self::Ge),
            "==" => Some(Self::Eq),
            "has" => Some(Self::Has),
            "not" => Some(Self::Not),
            _ => None,
        }
    }

    /// Stable display string.
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Lt => "<",
            Self::Le => "<=",
            Self::Gt => ">",
            Self::Ge => ">=",
            Self::Eq => "==",
            Self::Has => "has",
            Self::Not => "not",
        }
    }

    /// Evaluate the predicate: does `actual op threshold` hold?
    pub fn matches(self, actual: i64, threshold: i64) -> bool {
        match self {
            Self::Lt => actual < threshold,
            Self::Le => actual <= threshold,
            Self::Gt => actual > threshold,
            Self::Ge => actual >= threshold,
            Self::Eq => actual == threshold,
            Self::Has => actual == 1,
            Self::Not => actual == 0,
        }
    }
}

/// Check severity level.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum Severity {
    /// No problem (check did not fire).
    Ok,
    /// Informational.
    Notice,
    /// Needs attention.
    Warning,
    /// Urgent problem.
    Danger,
}

impl Severity {
    /// Parse from the authored string.
    fn from_str(s: &str) -> Option<Self> {
        match s {
            "ok" => Some(Self::Ok),
            "notice" => Some(Self::Notice),
            "warning" => Some(Self::Warning),
            "danger" => Some(Self::Danger),
            _ => None,
        }
    }

    /// Stable string for the bridge.
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Ok => "ok",
            Self::Notice => "notice",
            Self::Warning => "warning",
            Self::Danger => "danger",
        }
    }

    /// Numeric code (0 ok, 1 notice, 2 warning, 3 danger).
    pub const fn code(self) -> u8 {
        match self {
            Self::Ok => 0,
            Self::Notice => 1,
            Self::Warning => 2,
            Self::Danger => 3,
        }
    }
}

/// One predicate in a check's `when` list.
#[derive(Debug, Clone)]
pub struct DiagPredicate {
    /// Named fact to read.
    pub fact: String,
    /// Comparison operator.
    pub op: DiagOp,
    /// Threshold value; 0 for Has/Not.
    pub threshold: i64,
}

/// How to pick the next subject for chaining.
#[derive(Debug, Clone)]
pub struct NextSubject {
    /// Target subject kind.
    pub kind: SubjectKind,
    /// Selection strategy (e.g. "nearest_home", "home_hearth", "parent").
    pub pick: String,
}

/// One authored check within a subject.
#[derive(Debug, Clone)]
pub struct DiagCheck {
    /// Unique check id.
    pub id: String,
    /// Ordered predicates; all must hold for the check to fire.
    pub when: Vec<DiagPredicate>,
    /// Locale key for the check text.
    pub text_key: String,
    /// Named parameters: display name → fact name.
    pub params: BTreeMap<String, String>,
    /// Severity when this check fires.
    pub severity: Severity,
    /// Optional chain to another subject.
    pub next: Option<NextSubject>,
    /// Tags for matching requests to steward actions.
    pub tags: Vec<String>,
}

/// All diagnosis checks, indexed by subject kind.
#[derive(Debug, Clone)]
pub struct DiagnosisData {
    /// Checks per subject kind, in authored order.
    pub subjects: BTreeMap<SubjectKind, Vec<DiagCheck>>,
}

/// Parse the embedded diagnosis data file.
pub fn parse_diagnosis(text: &str) -> Option<DiagnosisData> {
    let file: DiagnosisFile = ron::from_str(text).ok()?;
    if file.schema_version != 1 {
        return None;
    }
    let mut subjects = BTreeMap::new();
    let mut seen_ids = std::collections::BTreeSet::new();
    for block in file.subjects {
        let kind = SubjectKind::from_name(&block.kind)?;
        if subjects.contains_key(&kind) {
            return None; // duplicate subject kind
        }
        let mut checks = Vec::with_capacity(block.checks.len());
        for raw in block.checks {
            if !seen_ids.insert(raw.id.clone()) {
                return None; // duplicate check id
            }
            let severity = Severity::from_str(&raw.severity)?;
            let next = match raw.next {
                Some(n) => Some(NextSubject {
                    kind: SubjectKind::from_name(&n.kind)?,
                    pick: n.pick,
                }),
                None => None,
            };
            let mut when = Vec::with_capacity(raw.when.len());
            for pred in raw.when {
                let op = DiagOp::from_str(&pred.op)?;
                let threshold = match op {
                    DiagOp::Has | DiagOp::Not => {
                        if pred.value.is_some() {
                            return None; // Has/Not must have value: None
                        }
                        0
                    }
                    _ => pred.value?,
                };
                when.push(DiagPredicate {
                    fact: pred.fact,
                    op,
                    threshold,
                });
            }
            checks.push(DiagCheck {
                id: raw.id,
                when,
                text_key: raw.text_key,
                params: raw.params,
                severity,
                next,
                tags: raw.tags,
            });
        }
        subjects.insert(kind, checks);
    }
    Some(DiagnosisData { subjects })
}

/// Load the bundled diagnosis data; `None` when the data is malformed.
pub fn diagnosis_data() -> Option<&'static DiagnosisData> {
    use std::sync::OnceLock;
    static DATA: OnceLock<Option<DiagnosisData>> = OnceLock::new();
    DATA.get_or_init(|| parse_diagnosis(DIAGNOSIS_RON)).as_ref()
}

/// Maximum chain depth for diagnosis evaluation.
pub const MAX_CHAIN_DEPTH: usize = 5;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn bundled_diagnosis_parses() {
        let data =
            parse_diagnosis(DIAGNOSIS_RON).expect("diagnosis.ron must parse with schema version 1");
        // All five subject kinds present.
        assert!(data.subjects.contains_key(&SubjectKind::Person));
        assert!(data.subjects.contains_key(&SubjectKind::Stockpile));
        assert!(data.subjects.contains_key(&SubjectKind::Hearth));
        assert!(data.subjects.contains_key(&SubjectKind::Room));
        assert!(data.subjects.contains_key(&SubjectKind::Settlement));
        // Every check has at least one predicate.
        for checks in data.subjects.values() {
            for check in checks {
                assert!(
                    !check.when.is_empty(),
                    "check {} has no predicates",
                    check.id
                );
            }
        }
    }

    #[test]
    fn all_check_ids_unique() {
        let data = parse_diagnosis(DIAGNOSIS_RON).unwrap();
        let mut ids = std::collections::BTreeSet::new();
        for checks in data.subjects.values() {
            for check in checks {
                assert!(ids.insert(&check.id), "duplicate check id: {}", check.id);
            }
        }
    }

    #[test]
    fn severity_ordering() {
        assert!(Severity::Danger > Severity::Warning);
        assert!(Severity::Warning > Severity::Notice);
        assert!(Severity::Notice > Severity::Ok);
    }

    #[test]
    fn op_matches() {
        assert!(DiagOp::Ge.matches(750, 750));
        assert!(DiagOp::Ge.matches(800, 750));
        assert!(!DiagOp::Ge.matches(749, 750));
        assert!(DiagOp::Has.matches(1, 0));
        assert!(!DiagOp::Has.matches(0, 0));
        assert!(DiagOp::Not.matches(0, 0));
        assert!(!DiagOp::Not.matches(1, 0));
        assert!(DiagOp::Eq.matches(0, 0));
        assert!(!DiagOp::Eq.matches(1, 0));
        assert!(DiagOp::Lt.matches(5, 10));
        assert!(!DiagOp::Lt.matches(10, 10));
    }

    #[test]
    fn data_only_check_addition() {
        // Add a check by data only in a test copy and show it appears.
        let mut text = String::from(DIAGNOSIS_RON);
        // Insert a new check into the person subject after the last existing one.
        let insert_point = "tags: [\"water\"],\n        ),\n    ]),";
        let new_check = r#"tags: ["water"],
        ),
        (
            id: "person_test_custom",
            when: [(fact: "hunger_tenths", op: ">=", value: Some(100))],
            text_key: "DIAG_TEST_CUSTOM",
            params: {},
            severity: "notice",
            next: None,
            tags: ["test"],
        ),
    ]),"#;
        text = text.replace(insert_point, new_check);
        let data = parse_diagnosis(&text).expect("modified diagnosis must parse");
        let person_checks = data.subjects.get(&SubjectKind::Person).unwrap();
        assert!(
            person_checks.iter().any(|c| c.id == "person_test_custom"),
            "data-only check must appear"
        );
        let custom = person_checks
            .iter()
            .find(|c| c.id == "person_test_custom")
            .unwrap();
        assert_eq!(custom.text_key, "DIAG_TEST_CUSTOM");
        assert_eq!(custom.severity, Severity::Notice);
        assert_eq!(custom.tags, vec!["test"]);
    }
}
