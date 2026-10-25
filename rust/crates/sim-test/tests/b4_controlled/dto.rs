//! Typed raw evidence shared only by the controlled B4 test fixtures.
use serde::{Deserialize, Serialize};

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Manifest {
    pub schema_version: u32,
    pub arm: String,
    pub arithmetic_manifest_sha256: String,
    pub policy: String,
    pub catalog_revision: String,
}

#[derive(Serialize)]
pub struct InvalidReason {
    pub code: &'static str,
    pub executed_tick: Option<u64>,
    pub agent_id: Option<u64>,
    pub detail: String,
}

#[derive(Serialize)]
pub struct Envelope<'a, T> {
    pub schema_version: u32,
    pub manifest_sha256: &'a str,
    pub arm: &'a str,
    pub fixture_kind: &'a str,
    pub status: &'a str,
    pub invalid_reasons: &'a [InvalidReason],
    pub rows: &'a [T],
}

#[derive(Serialize)]
pub struct ColdRow {
    pub seed: u64,
    pub agent_id: u64,
    pub lineage_key: u64,
    pub executed_tick: u64,
    pub garment_arm: &'static str,
    pub item_code: Option<u32>,
    pub cold_permille: u32,
    pub observed_cold: bool,
    pub member: bool,
    pub season: u8,
    pub in_cold_window: bool,
    pub burn_rate: u32,
    pub no_roof: bool,
    pub wood_zero: bool,
    pub excluded_markers_absent: bool,
    pub damage_disabled: bool,
}

#[derive(Serialize)]
pub struct TallyRow {
    pub agent_id: u64,
    pub recipe_code: u32,
    pub completed_count: u64,
    pub quality_sum: u64,
}

#[derive(Serialize)]
pub struct GateRow {
    pub node_id: u8,
    pub entry_present: bool,
    pub proficiency: u16,
    pub last_used: Option<u16>,
    pub admission_threshold: u16,
}

#[derive(Serialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum MaterialTerm {
    Hardness { offset: i32, divisor: i32 },
}

#[derive(Serialize)]
pub struct CraftEvent {
    pub seed: u64,
    pub agent_id: u64,
    pub executed_tick: u64,
    pub workshop_id: u32,
    pub bill_id: u32,
    pub recipe_code: u32,
    pub recipe_identity: String,
    pub catalog_revision: String,
    pub output_item_code: u32,
    pub output_count: u16,
    pub at: [u32; 2],
    pub job_stage: &'static str,
    pub ticks_left: u32,
    pub skill_code: u8,
    pub skill_present: bool,
    pub xp_before: u32,
    pub level_before: u8,
    pub material_id: Option<u32>,
    pub material_hardness: Option<u16>,
    pub hardness_percent: Option<i32>,
    pub temperament_present: bool,
    pub persistence: Option<u16>,
    pub knowledge_enabled: bool,
    pub gate_mask: u32,
    pub gating_nodes: Vec<GateRow>,
    pub recipe_quality_add: i32,
    pub material_term: MaterialTerm,
    pub tally_before_count: u64,
    pub tally_after_count: u64,
    pub tally_before_quality_sum: u64,
    pub tally_after_quality_sum: u64,
    pub completion_count_delta: u64,
    pub actual_quality: u8,
    pub alive_after_tick: bool,
    pub arithmetic_manifest_sha256: String,
}

#[derive(Serialize)]
pub struct LedgerRow {
    pub item_code: u32,
    pub created: u64,
    pub consumed: u64,
    pub traded_in: u64,
    pub traded_out: u64,
    pub ground: u64,
    pub carried: u64,
    pub held: u64,
    pub delivered: u64,
    pub shelved: u64,
}

#[derive(Serialize)]
pub struct CommandResult {
    pub accepted: bool,
    pub count: u32,
    pub reason: Option<String>,
}

#[derive(Serialize)]
pub struct CraftRow {
    #[serde(flatten)]
    pub event: CraftEvent,
    pub fixture_id: String,
    pub cap_arm: &'static str,
    pub target_level: u8,
    pub target_proficiency: u16,
    pub admission_eligible: bool,
    pub inputs_consumed: bool,
    pub ledger_before: Vec<LedgerRow>,
    pub ledger_after: Vec<LedgerRow>,
    pub output_delta: u64,
    pub bill_done_before: u32,
    pub bill_done_after: u32,
    pub setup_finished_tick: u64,
    pub add_bill_result: CommandResult,
    pub paid_working_observed: bool,
    pub ticks_left_at_paid_boundary: u32,
    pub primary_tally_before_payment: TallyRow,
    pub primary_tally_at_paid_boundary: TallyRow,
    pub tally_key_present_before_payment: bool,
    pub cleanup_complete: bool,
    pub ledger_balanced: bool,
    pub reservation_check_passed: bool,
}
