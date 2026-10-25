//! Typed raw B4 records; no derived score feeds the simulation.
use serde::Serialize;

#[derive(Debug, Clone, Serialize)]
pub struct InvalidReason {
    pub code: String,
    pub executed_tick: Option<u64>,
    pub agent_id: Option<u64>,
    pub detail: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct TallyRow {
    pub agent_id: u64,
    pub recipe_code: u32,
    pub completed_count: u64,
    pub quality_sum: u64,
}

#[derive(Debug, Clone, Serialize)]
pub struct GatingNode {
    pub node_id: u8,
    pub entry_present: bool,
    pub proficiency: u16,
    pub last_used: Option<u16>,
    pub admission_threshold: u16,
}

#[derive(Debug, Clone, Serialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum MaterialTerm {
    None,
    Hardness { offset: i32, divisor: i32 },
}

#[derive(Debug, Clone, Serialize)]
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
    pub at: (u32, u32),
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
    pub gating_nodes: Vec<GatingNode>,
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

#[derive(Debug, Clone, Default, Serialize)]
pub struct ColdCounts {
    pub clothed_cold: u64,
    pub clothed_eligible: u64,
    pub unclothed_cold: u64,
    pub unclothed_eligible: u64,
}

#[derive(Debug, Serialize)]
pub struct GarmentCount {
    pub item_code: u32,
    pub count: u64,
    pub cold_permille: u32,
}

#[derive(Debug, Serialize)]
pub struct CancelCount {
    pub reason: &'static str,
    pub count: u64,
}

#[derive(Debug, Serialize)]
pub struct MeatCounts {
    pub dropped: u64,
    pub eaten: u64,
    pub spoiled: u64,
    pub hauls: u64,
}

#[derive(Debug, Serialize)]
pub struct DayRow {
    pub day: u32,
    pub executed_tick: u64,
    pub population: u64,
    pub insulating_wearers: u64,
    pub garment_counts: Vec<GarmentCount>,
    pub craft_tallies: Vec<TallyRow>,
    pub cold: ColdCounts,
    pub deaths_total: u64,
    pub jobs_done_total: u64,
    pub cancel_totals: Vec<CancelCount>,
    pub meat: MeatCounts,
    pub ledger_violation_ticks: u64,
    pub reservation_violation_ticks: u64,
}

#[derive(Debug, Default, Serialize)]
pub struct Conservation {
    pub ledger_violation_ticks: u64,
    pub reservation_violation_ticks: u64,
    pub unresolved_start_count: u64,
    pub unresolved_terminal_count: u64,
}
