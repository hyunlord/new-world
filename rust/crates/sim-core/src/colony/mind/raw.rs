//! Strict authored schema; string references resolve only against caller registries.

use serde::Deserialize;

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub(super) struct File {
    pub schema_version: u32,
    pub bands: Vec<String>,
    pub nodes: Vec<Node>,
}
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub(super) struct Node {
    pub id: String,
    pub band: String,
    pub behaviour: String,
    pub when: Vec<Predicate>,
    pub score: Score,
    pub hysteresis: i64,
    pub life_stages: Vec<String>,
    pub tags: Vec<String>,
    pub name_key: String,
    #[serde(default = "one")]
    pub every_ticks: u32,
    #[serde(default)]
    pub target: Option<String>,
    #[serde(default)]
    pub finish: Option<String>,
}
const fn one() -> u32 {
    1
}
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub(super) struct Predicate {
    pub fact: String,
    pub op: String,
    #[serde(default)]
    pub value: Option<i64>,
}
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub(super) struct Score {
    pub constant: i64,
    pub terms: Vec<Term>,
}
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub(super) struct Term {
    pub fact: String,
    pub weight_permille: i64,
}
