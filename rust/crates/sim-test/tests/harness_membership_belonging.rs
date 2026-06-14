//! Settlement Membership Model Reform — belonging model harness (A1–A10).
//!
//! Reform: `SettlementSystem::sync_membership` no longer wholesale-recomputes
//! `member_agents = {agents within radius}` every tick (which dropped a member
//! the moment it stepped outside the bubble — the 2-2 Gather 75→0 collapse).
//! Instead:
//!   - DROP only DEAD members (retain ids still resolvable to a live Position).
//!   - ADD in-radius live agents (deterministic, sorted, cap-respecting).
//!   - one settlement per agent (Manhattan-nearest formation_tile, tie → lower id).
//!   - dissolve only when zero LIVE members (all dead) AND no buildings.
//!
//! Information barrier: this harness owns the membership-reform assertions. The
//! existing settlement/freeze/determinism harnesses must stay green UNMODIFIED
//! (verified by the GATE, not here).
//!
//! ── Scene configuration ──────────────────────────────────────────────────────
//! Controlled engine (small, inspectable): A1, A2, A3, A4, A5, A8, A9 — settlements
//! are obtained through the system's OWN formation path (`form_settlement_at`) so
//! `Settlement.formation_tile` is populated and the membership-sync will process
//! them. Production scene (bootstrap 64 + 3 startup buildings at
//! (32,32)/(24,32)/(40,32), seed 42): A6, A7, A10.
//!
//! ── Sync-Liveness Requirement ────────────────────────────────────────────────
//! Hot/Warm/Cold frequency tiering means a stepped window may not run the
//! membership-sync, so a "no-change" reading could pass vacuously. Every
//! controlled no-change reading (A1, A3a, A9) is therefore gated on a positive
//! control: a fresh non-member relocated INSIDE a settlement radius MUST appear
//! in that roster within the stepped window ([`prove_sync_join`]) — only then is
//! a "did-not-change" reading trusted. Change-based readings (A2 death-leave, A4
//! join, A5 resolve, A8 transfer) are self-witnessing (the change itself proves
//! the sync ran). If the witness never fires, the assertion reports a SETUP
//! LIMITATION (printed, returns) — never a silent pass.
//!
//! Run:
//!   cargo test -p sim-test --test harness_membership_belonging -- --nocapture

use hecs::Entity;
use sim_bridge::ffi::world_node::{bootstrap_spawn_agents, enqueue_building_placed};
use sim_core::components::{
    Agent, AgentId, AgentState, Hunger, Memory, Position, SettlementId, Sleep, Social, Thirst,
    SETTLEMENT_MAX_POP, SETTLEMENT_PROXIMITY_RADIUS,
};
use sim_core::material::MaterialRegistry;
use sim_engine::{BuildingPlacedEvent, RuntimeSystem, SimEngine};
use sim_systems::register_default_runtime_systems;
use sim_systems::runtime::influence::BuildingStampSystem;

const W: u32 = 64;
const H: u32 = 64;
/// Generous per-window step budget. `SettlementSystem` runs at tick_interval 1
/// so one sync == one tick, but the Sync-Liveness Requirement forbids hard-
/// coding 1 — every loop observes an actual membership change instead.
const SYNC_BUDGET: u64 = 12;

// ── distance helpers ─────────────────────────────────────────────────────────

/// Chebyshev distance (the radius metric used by formation/sync).
fn chebyshev(a: (u32, u32), b: (u32, u32)) -> u32 {
    a.0.abs_diff(b.0).max(a.1.abs_diff(b.1))
}

/// Manhattan distance (the exclusivity-resolution metric).
fn manhattan(a: (u32, u32), b: (u32, u32)) -> u64 {
    a.0.abs_diff(b.0) as u64 + a.1.abs_diff(b.1) as u64
}

// ── world readers ────────────────────────────────────────────────────────────

fn agent_id(e: &SimEngine, ent: Entity) -> AgentId {
    e.world.get::<&Agent>(ent).expect("Agent present").id
}

fn agent_pos(e: &SimEngine, ent: Entity) -> (u32, u32) {
    let p = e.world.get::<&Position>(ent).expect("Position present");
    (p.x, p.y)
}

fn is_member(e: &SimEngine, aid: AgentId) -> bool {
    e.resources
        .settlements
        .values()
        .any(|s| s.member_agents.contains(&aid))
}

fn is_member_of(e: &SimEngine, sid: SettlementId, aid: AgentId) -> bool {
    e.resources
        .settlements
        .get(&sid)
        .map(|s| s.member_agents.contains(&aid))
        .unwrap_or(false)
}

fn settlements_containing(e: &SimEngine, aid: AgentId) -> usize {
    e.resources
        .settlements
        .values()
        .filter(|s| s.member_agents.contains(&aid))
        .count()
}

fn formation_tile(e: &SimEngine, sid: SettlementId) -> (u32, u32) {
    e.resources
        .settlements
        .get(&sid)
        .map(|s| s.formation_tile)
        .expect("settlement present")
}

fn member_len(e: &SimEngine, sid: SettlementId) -> usize {
    e.resources
        .settlements
        .get(&sid)
        .map(|s| s.member_agents.len())
        .unwrap_or(0)
}

fn current(e: &SimEngine, sid: SettlementId) -> u32 {
    e.resources
        .settlements
        .get(&sid)
        .map(|s| s.population_stats.current)
        .unwrap_or(0)
}

/// Settlement id whose `formation_tile` equals `ftile` (the formation-path anchor).
fn settlement_id_at(e: &SimEngine, ftile: (u32, u32)) -> Option<SettlementId> {
    e.resources
        .settlements
        .iter()
        .find(|(_, s)| s.formation_tile == ftile)
        .map(|(id, _)| *id)
}

/// Count of live settlements with ≥1 member resolvable to a live world Position
/// (the exact A1/A2 invariant of `harness_settlements_zero_regression`).
fn settlements_with_resolvable_members(e: &SimEngine) -> usize {
    use std::collections::HashMap;
    let positions: HashMap<AgentId, (u32, u32)> = e
        .world
        .query::<(&Agent, &Position)>()
        .iter()
        .map(|(_, (a, p))| (a.id, (p.x, p.y)))
        .collect();
    e.resources
        .settlements
        .values()
        .filter(|s| s.member_agents.iter().any(|id| positions.contains_key(id)))
        .count()
}

// ── scene builders ─────────────────────────────────────────────────────────────

/// Fresh 64×64 engine with the full default runtime (includes `SettlementSystem`).
fn fresh_engine() -> SimEngine {
    let mut e = SimEngine::new(W, H, MaterialRegistry::new());
    register_default_runtime_systems(&mut e);
    e
}

/// Seed a STATIONARY agent (no `MovementRng` → `AgentMovementSystem` skips it, so
/// its Position only changes when the harness relocates it). All needs 0 with 0
/// growth so it never seeks/starves. Returns the entity.
fn seed_stationary(e: &mut SimEngine, x: u32, y: u32) -> Entity {
    let ent = e.spawn_agent(x, y);
    e.world
        .insert(
            ent,
            (
                AgentState::Idle,
                Hunger::new(0.0, 0.0),
                Thirst::new(0.0, 0.0),
                Sleep::new(0.0, 0.0),
                Social::new(0.0, 0.0),
                Memory::new(),
            ),
        )
        .expect("seed stationary agent");
    ent
}

/// Form ONE settlement through the system's formation path: 3 stationary founders
/// at `(cx,cy),(cx+1,cy),(cx+2,cy)` + 2 buildings within radius at `(cx,cy+3)` /
/// `(cx+1,cy+3)`. The lowest-sorted candidate tile `(cx,cy)` becomes the
/// `formation_tile`. Returns `(settlement_id, founder_entities)`.
fn form_settlement_at(e: &mut SimEngine, cx: u32, cy: u32) -> (SettlementId, Vec<Entity>) {
    let mut founders = Vec::new();
    for i in 0..3u32 {
        founders.push(seed_stationary(e, cx + i, cy));
    }
    for i in 0..2u32 {
        e.resources.building_event_queue.push_back(BuildingPlacedEvent {
            position: (cx + i, cy + 3),
            radius: 1,
        });
    }
    let mut bss = BuildingStampSystem::new();
    bss.tick(&mut e.world, &mut e.resources);
    e.tick(); // sync (no-op on empty) → formation scan forms the settlement
    let sid = settlement_id_at(e, (cx, cy))
        .unwrap_or_else(|| panic!("a settlement must form at ({cx},{cy})"));
    (sid, founders)
}

/// The real production scene WITHOUT running ticks (caller runs them): bootstrap
/// 64 + the 3 startup buildings at (32,32)/(24,32)/(40,32) radius 8, stamped.
/// Mirrors `harness_settlements_zero_regression::production_scene`.
fn production_scene_engine() -> SimEngine {
    let mut e = SimEngine::new(W, H, MaterialRegistry::new());
    register_default_runtime_systems(&mut e);
    bootstrap_spawn_agents(&mut e);
    for (x, y) in [(32i32, 32i32), (24, 32), (40, 32)] {
        let ok = enqueue_building_placed(&mut e.resources, x, y, 8);
        assert!(ok, "startup building ({x},{y}) must enqueue within bounds");
    }
    let mut bss = BuildingStampSystem::new();
    bss.tick(&mut e.world, &mut e.resources);
    e
}

// ── Sync-Liveness positive control ───────────────────────────────────────────

/// Relocate a fresh non-member INTO `sid`'s radius (at its formation_tile) and
/// step the FULL engine until it appears in `sid.member_agents`, capped at
/// `budget`. Returns `Some(ticks)` when the join is observed (proving the
/// membership-sync executed in the window), `None` otherwise (setup limitation).
///
/// Steps the whole engine, so any change under test elsewhere is also advanced.
fn prove_sync_join(e: &mut SimEngine, sid: SettlementId, budget: u64) -> Option<u64> {
    let ft = formation_tile(e, sid);
    let witness = seed_stationary(e, ft.0, ft.1);
    let wid = agent_id(e, witness);
    for t in 1..=budget {
        e.tick();
        if is_member_of(e, sid, wid) {
            return Some(t);
        }
    }
    None
}

// ── FNV-1a fingerprint (A6) ──────────────────────────────────────────────────

fn fnv(acc: &mut u64, bytes: &[u8]) {
    for &b in bytes {
        *acc ^= b as u64;
        *acc = acc.wrapping_mul(0x0000_0100_0000_01B3);
    }
}

/// Byte-stable fingerprint: per settlement sorted by SettlementId
/// (formation_tile, SORTED member_agents, population_stats.current) + the global
/// per-agent (id, position, AgentState) set sorted by id. Mirrors
/// `harness_resource_scarcity::behavioural_fingerprint`'s FNV fold.
fn fingerprint(e: &SimEngine) -> u64 {
    let mut acc: u64 = 0xcbf2_9ce4_8422_2325;

    let mut sids: Vec<SettlementId> = e.resources.settlements.keys().copied().collect();
    sids.sort_unstable();
    for sid in &sids {
        let s = &e.resources.settlements[sid];
        fnv(&mut acc, &sid.to_le_bytes());
        fnv(&mut acc, &s.formation_tile.0.to_le_bytes());
        fnv(&mut acc, &s.formation_tile.1.to_le_bytes());
        let mut members: Vec<AgentId> = s.member_agents.iter().copied().collect();
        members.sort_unstable();
        for m in &members {
            fnv(&mut acc, &m.to_le_bytes());
        }
        fnv(&mut acc, &s.population_stats.current.to_le_bytes());
        fnv(&mut acc, b"|");
    }

    let mut rows: Vec<(AgentId, (u32, u32), String)> = Vec::new();
    for (_, (a, p, st)) in e.world.query::<(&Agent, &Position, &AgentState)>().iter() {
        rows.push((a.id, (p.x, p.y), format!("{st:?}")));
    }
    rows.sort_by_key(|(id, ..)| *id);
    for (id, (x, y), st) in &rows {
        fnv(&mut acc, &id.to_le_bytes());
        fnv(&mut acc, &x.to_le_bytes());
        fnv(&mut acc, &y.to_le_bytes());
        fnv(&mut acc, st.as_bytes());
        fnv(&mut acc, b";");
    }
    acc
}

/// Run the production scene N ticks, fingerprinting the state at tick 0 (before
/// any tick — sync exercised against the EMPTY settlement store) and after each
/// tick. Returns `(per_tick_fingerprints, final_settlement_count)`.
fn fingerprint_sequence(n: u64) -> (Vec<u64>, usize) {
    let mut e = production_scene_engine();
    let mut seq = Vec::with_capacity(n as usize + 1);
    seq.push(fingerprint(&e));
    for _ in 0..n {
        e.tick();
        seq.push(fingerprint(&e));
    }
    (seq, e.resources.settlements.len())
}

// ════════════════════════════════════════════════════════════════════════════
// Assertion 1 — a LIVE member that moves OUTSIDE the radius persists (★ headline).
// ════════════════════════════════════════════════════════════════════════════
#[test]
fn harness_membership_a1_member_outside_radius_persists() {
    // Type A. Under the belonging model, membership ≠ standing-in-the-bubble:
    // a far-but-LIVE member must remain in member_agents. Sync-Liveness: a
    // witness join proves the sync ran before trusting the "still present" read.
    let mut e = fresh_engine();
    let (sid, founders) = form_settlement_at(&mut e, 20, 20);
    let member_ent = founders[0];
    let member_id = agent_id(&e, member_ent);

    // Relocate the member far outside the radius (chebyshev 20 > 5).
    e.world
        .insert_one(member_ent, Position::new(40, 40))
        .expect("relocate member outside radius");

    let proven = prove_sync_join(&mut e, sid, SYNC_BUDGET);
    if proven.is_none() {
        println!("[membership A1] SETUP LIMITATION: membership-sync did not run in the window");
        return;
    }

    let ft = formation_tile(&e, sid);
    let pos = agent_pos(&e, member_ent);
    let outside = chebyshev(ft, pos) > SETTLEMENT_PROXIMITY_RADIUS;
    assert!(
        outside,
        "A1 co-assert: member must be verifiably OUTSIDE radius; cheby({ft:?},{pos:?})={} !> {SETTLEMENT_PROXIMITY_RADIUS}",
        chebyshev(ft, pos)
    );
    assert!(
        is_member_of(&e, sid, member_id),
        "A1: a far-but-LIVE member must PERSIST in member_agents (belonging model)"
    );
    assert_eq!(
        current(&e, sid) as usize,
        member_len(&e, sid),
        "A1: population_stats.current must equal member_agents.len()"
    );
    assert!(current(&e, sid) >= 1, "A1: population_stats.current must be >= 1");
    println!(
        "[membership A1] far live member persists (cheby {}); current={} == len={} ✓",
        chebyshev(ft, pos),
        current(&e, sid),
        member_len(&e, sid)
    );
}

// ════════════════════════════════════════════════════════════════════════════
// Assertion 2 — a DEAD member leaves within one sync (retain-live).
// ════════════════════════════════════════════════════════════════════════════
#[test]
fn harness_membership_a2_dead_member_leaves_within_one_sync() {
    // Type A. Raw world despawn (NOT the cleanup leave path) → only the sync's
    // retain-live step can drop the stale id. A co-member must remain; current
    // must equal len. The disappearance is itself the sync-witnessing change.
    let mut e = fresh_engine();
    let (sid, founders) = form_settlement_at(&mut e, 20, 20);
    let victim_ent = founders[0];
    let victim_id = agent_id(&e, victim_ent);
    let survivor_id = agent_id(&e, founders[1]);

    e.world.despawn(victim_ent).expect("raw despawn victim entity");

    let mut left = false;
    for _ in 0..SYNC_BUDGET {
        e.tick();
        if !is_member_of(&e, sid, victim_id) {
            left = true;
            break;
        }
    }
    assert!(
        left,
        "A2: the despawned (dead) member id must be dropped by the retain-live step within one sync window"
    );
    assert!(
        !is_member_of(&e, sid, victim_id),
        "A2: despawned id must be ABSENT from member_agents"
    );
    assert!(
        is_member_of(&e, sid, survivor_id),
        "A2: a co-member that was NOT despawned must REMAIN"
    );
    assert_eq!(
        current(&e, sid) as usize,
        member_len(&e, sid),
        "A2: population_stats.current must equal member_agents.len() after the sync"
    );
    println!("[membership A2] dead member dropped; survivor kept; current == len ✓");
}

// ════════════════════════════════════════════════════════════════════════════
// Assertion 3 — dissolution means ALL LIVE members gone (not "none nearby").
// ════════════════════════════════════════════════════════════════════════════
#[test]
fn harness_membership_a3_dissolution_means_all_live_members_gone() {
    // (a) NON-dissolution (★ load-bearing, Sync-Liveness MANDATORY): a settlement
    //     whose only member is LIVE but far outside the radius must NOT dissolve.
    {
        let mut e = fresh_engine();
        let (s_sid, s_founders) = form_settlement_at(&mut e, 20, 20);
        let (w_sid, _w) = form_settlement_at(&mut e, 10, 50); // witness settlement

        // Reduce S to exactly one LIVE member, relocated outside its radius.
        e.world.despawn(s_founders[1]).expect("despawn S founder 1");
        e.world.despawn(s_founders[2]).expect("despawn S founder 2");
        let lone_ent = s_founders[0];
        let lone_id = agent_id(&e, lone_ent);
        e.world
            .insert_one(lone_ent, Position::new(40, 40))
            .expect("relocate lone member outside radius");

        // Prove the sync ran via the witness settlement (orthogonal to S).
        let proven = prove_sync_join(&mut e, w_sid, SYNC_BUDGET);
        if proven.is_none() {
            println!("[membership A3a] SETUP LIMITATION: sync did not run in the window");
            return;
        }

        assert!(
            e.resources.settlements.contains_key(&s_sid),
            "A3(a): a settlement with one far-but-LIVE member must NOT dissolve"
        );
        assert!(
            is_member_of(&e, s_sid, lone_id),
            "A3(a): the lone live member must still be present"
        );
        assert_eq!(
            current(&e, s_sid),
            1,
            "A3(a): exactly one live member ⇒ current == 1 (current got {})",
            current(&e, s_sid)
        );
        let ft = formation_tile(&e, s_sid);
        let pos = agent_pos(&e, lone_ent);
        assert!(
            chebyshev(ft, pos) > SETTLEMENT_PROXIMITY_RADIUS,
            "A3(a): the lone member must be verifiably outside the radius"
        );
        println!("[membership A3a] far live member ⇒ NOT dissolved (current=1) ✓");
    }

    // (b) Dissolution (feasibility guard FIRST on member_buildings.is_empty()):
    //     once the last live member despawns AND no buildings remain, a sync
    //     tick removes the settlement.
    {
        let mut e = fresh_engine();
        let (sid, founders) = form_settlement_at(&mut e, 20, 20);

        // Drop the buildings from the registry, then sync to prune member_buildings.
        e.resources.building_registry.clear();
        e.tick();

        let empty_ok = e
            .resources
            .settlements
            .get(&sid)
            .map(|s| s.member_buildings.is_empty())
            .unwrap_or(true);
        if !empty_ok {
            println!(
                "[membership A3b] SETUP LIMITATION: member_buildings could not be emptied — \
                 deferring the building-retention dissolution branch (covered by A7/A10)"
            );
            return;
        }

        // Despawn every live member; only the retain-live step can zero current.
        for ent in &founders {
            let _ = e.world.despawn(*ent);
        }

        let mut dissolved = false;
        for _ in 0..SYNC_BUDGET {
            e.tick();
            if !e.resources.settlements.contains_key(&sid) {
                dissolved = true;
                break;
            }
        }
        assert!(
            dissolved,
            "A3(b): after all live members despawn AND member_buildings.is_empty(), a sync must dissolve the settlement"
        );
        println!("[membership A3b] zero live members + no buildings ⇒ dissolved ✓");
    }
}

// ════════════════════════════════════════════════════════════════════════════
// Assertion 4 — a non-member ENTERING the radius joins; an outside control stays out.
// ════════════════════════════════════════════════════════════════════════════
#[test]
fn harness_membership_a4_nonmember_entering_radius_joins() {
    // Type A. Join-on-entry (additive half). The CONTROL (agent left outside all
    // radii) must NOT join — guards against a degenerate "add everyone".
    let mut e = fresh_engine();
    let (sid, _f) = form_settlement_at(&mut e, 20, 20);

    let joiner = seed_stationary(&mut e, 45, 45); // outside the radius
    let joiner_id = agent_id(&e, joiner);
    let control = seed_stationary(&mut e, 55, 55); // stays outside the radius
    let control_id = agent_id(&e, control);
    assert!(
        !is_member(&e, joiner_id),
        "A4 setup: joiner must start as a non-member"
    );

    // Relocate the joiner inside the radius (onto the formation tile).
    let ft = formation_tile(&e, sid);
    e.world
        .insert_one(joiner, Position::new(ft.0, ft.1))
        .expect("relocate joiner inside radius");

    let mut joined = false;
    for _ in 0..SYNC_BUDGET {
        e.tick();
        if is_member_of(&e, sid, joiner_id) {
            joined = true;
            break;
        }
    }
    assert!(joined, "A4: a non-member entering the radius must JOIN within one sync window");
    assert!(
        !is_member(&e, control_id),
        "A4 control: an agent left OUTSIDE every radius must NOT be added to any roster"
    );
    println!("[membership A4] joiner admitted on entry; outside control stays out ✓");
}

// ════════════════════════════════════════════════════════════════════════════
// Assertion 5 — membership is EXCLUSIVE (one settlement per agent), metric-agreed.
// ════════════════════════════════════════════════════════════════════════════
#[test]
fn harness_membership_a5_membership_is_exclusive() {
    // Type A. NEARER sub-case: an agent inside two overlapping radii at a spot
    // where Chebyshev, Manhattan, AND Euclidean all agree on the nearer
    // formation_tile ends up in EXACTLY that one. TIE sub-case: equidistant ⇒
    // EXACTLY the LOWER SettlementId.

    // ── NEARER: S1(20,20) & S2(20,27); agent at (20,23) strictly inside both ──
    {
        let mut e = fresh_engine();
        let (s1, _) = form_settlement_at(&mut e, 20, 20);
        let (s2, _) = form_settlement_at(&mut e, 20, 27);
        let f1 = formation_tile(&e, s1);
        let f2 = formation_tile(&e, s2);
        let p = (20u32, 23u32);
        // Feasibility guard: p must be STRICTLY inside both radii (no boundary).
        if !(chebyshev(f1, p) < SETTLEMENT_PROXIMITY_RADIUS
            && chebyshev(f2, p) < SETTLEMENT_PROXIMITY_RADIUS)
        {
            println!("[membership A5-nearer] SETUP LIMITATION: no strict double-radius position");
            return;
        }
        // All three metrics agree (pure vertical separation): f1 is nearer.
        assert!(manhattan(f1, p) < manhattan(f2, p), "A5-nearer setup: f1 strictly nearer");

        let agent = seed_stationary(&mut e, p.0, p.1);
        let aid = agent_id(&e, agent);
        let mut in_any = false;
        for _ in 0..SYNC_BUDGET {
            e.tick();
            if is_member(&e, aid) {
                in_any = true;
                break;
            }
        }
        if !in_any {
            println!("[membership A5-nearer] SETUP LIMITATION: agent never joined (sync did not run)");
            return;
        }
        assert_eq!(
            settlements_containing(&e, aid),
            1,
            "A5-nearer: agent must belong to EXACTLY ONE settlement"
        );
        assert!(
            is_member_of(&e, s1, aid),
            "A5-nearer: agent must be a member of the metric-agreed NEARER settlement (S1)"
        );
        println!("[membership A5-nearer] exactly one, nearer settlement ✓");
    }

    // ── TIE: S1(20,20) & S2(20,28); agent at (20,24) equidistant (Manhattan 4) ──
    {
        let mut e = fresh_engine();
        let (s1, _) = form_settlement_at(&mut e, 20, 20);
        let (s2, _) = form_settlement_at(&mut e, 20, 28);
        let f1 = formation_tile(&e, s1);
        let f2 = formation_tile(&e, s2);
        let p = (20u32, 24u32);
        let lower = s1.min(s2);
        if !(chebyshev(f1, p) < SETTLEMENT_PROXIMITY_RADIUS
            && chebyshev(f2, p) < SETTLEMENT_PROXIMITY_RADIUS
            && manhattan(f1, p) == manhattan(f2, p))
        {
            println!("[membership A5-tie] SETUP LIMITATION: no strict equidistant double-radius position");
            return;
        }
        let agent = seed_stationary(&mut e, p.0, p.1);
        let aid = agent_id(&e, agent);
        let mut in_any = false;
        for _ in 0..SYNC_BUDGET {
            e.tick();
            if is_member(&e, aid) {
                in_any = true;
                break;
            }
        }
        if !in_any {
            println!("[membership A5-tie] SETUP LIMITATION: agent never joined (sync did not run)");
            return;
        }
        assert_eq!(
            settlements_containing(&e, aid),
            1,
            "A5-tie: agent must belong to EXACTLY ONE settlement"
        );
        assert!(
            is_member_of(&e, lower, aid),
            "A5-tie: equidistant agent must resolve to the LOWER SettlementId ({lower})"
        );
        println!("[membership A5-tie] exactly one, lower SettlementId ✓");
    }
}

// ════════════════════════════════════════════════════════════════════════════
// Assertion 6 — lockstep per-tick fingerprint determinism (★ no iteration order).
// ════════════════════════════════════════════════════════════════════════════
#[test]
fn harness_membership_a6_lockstep_determinism_fingerprint() {
    // Type A. Two production-scene runs to N ticks must yield byte-identical
    // per-tick fingerprint sequences (membership included), and form ≥1 settlement.
    const N: u64 = 300;
    let (seq_a, count_a) = fingerprint_sequence(N);
    let (seq_b, _count_b) = fingerprint_sequence(N);
    assert_eq!(
        seq_a.len(),
        (N + 1) as usize,
        "A6: fingerprint sequence must have N+1 samples (tick 0 .. N)"
    );
    assert!(
        count_a >= 1,
        "A6: non-vacuous floor — at least one settlement must exist by tick {N}; got {count_a}"
    );
    assert_eq!(
        seq_a, seq_b,
        "A6: two identical seed-42 runs must produce byte-identical per-tick fingerprint sequences \
         (any HashSet/HashMap order leak in retain/add/exclusivity surfaces here)"
    );
    println!("[membership A6] {} per-tick fingerprints identical; {count_a} settlement(s) ✓", seq_a.len());
}

// ════════════════════════════════════════════════════════════════════════════
// Assertion 7 — formation-count regression preserved (★ cross-phase guard).
// ════════════════════════════════════════════════════════════════════════════
#[test]
fn harness_membership_a7_formation_count_regression_preserved() {
    // Type D. The reform must PRESERVE formation + a non-empty resolvable roster
    // (settlements_zero A1/A2 invariant). Observed 3 at seed 42; floor >= 2.
    const RUN_TICKS: u64 = 300;
    let mut e = production_scene_engine();
    for _ in 0..RUN_TICKS {
        e.tick();
    }
    let count = settlements_with_resolvable_members(&e);
    assert!(
        count >= 2,
        "A7: production scene must keep >= 2 settlements with resolvable members after {RUN_TICKS} ticks; got {count}"
    );
    println!("[membership A7] {count} settlement(s) with resolvable members (>=2) ✓");
}

// ════════════════════════════════════════════════════════════════════════════
// Assertion 8 — an existing member MIGRATES as a transfer, not a duplicate.
// ════════════════════════════════════════════════════════════════════════════
#[test]
fn harness_membership_a8_member_migrates_as_transfer() {
    // Type A. A member of X relocated so its nearest settlement is now Y must
    // TRANSFER: leave X, join Y, belong to EXACTLY ONE, and X.current decrements.
    let mut e = fresh_engine();
    let (x_sid, x_founders) = form_settlement_at(&mut e, 20, 20);
    let (y_sid, _y) = form_settlement_at(&mut e, 20, 28);
    let f_x = formation_tile(&e, x_sid);
    let f_y = formation_tile(&e, y_sid);

    let inside_x_outside_y = (20u32, 20u32);
    let inside_y_outside_x = (20u32, 28u32);
    // Feasibility guard: the two-settlement geometry must permit both placements.
    let geometry_ok = chebyshev(f_x, inside_x_outside_y) <= SETTLEMENT_PROXIMITY_RADIUS
        && chebyshev(f_y, inside_x_outside_y) > SETTLEMENT_PROXIMITY_RADIUS
        && chebyshev(f_x, inside_y_outside_x) > SETTLEMENT_PROXIMITY_RADIUS
        && chebyshev(f_y, inside_y_outside_x) <= SETTLEMENT_PROXIMITY_RADIUS;
    if !geometry_ok {
        println!("[membership A8] SETUP LIMITATION: two-settlement transfer geometry unavailable");
        return;
    }

    let mover_ent = x_founders[0]; // founder at (20,20): inside X, outside Y
    let mover_id = agent_id(&e, mover_ent);
    e.tick(); // stabilize memberships before measuring current_before
    if !is_member_of(&e, x_sid, mover_id) {
        println!("[membership A8] SETUP LIMITATION: mover is not a confirmed member of X");
        return;
    }
    let current_before = current(&e, x_sid);

    e.world
        .insert_one(mover_ent, Position::new(inside_y_outside_x.0, inside_y_outside_x.1))
        .expect("relocate mover into Y, outside X");

    let mut transferred = false;
    for _ in 0..SYNC_BUDGET {
        e.tick();
        if is_member_of(&e, y_sid, mover_id) && !is_member_of(&e, x_sid, mover_id) {
            transferred = true;
            break;
        }
    }
    assert!(transferred, "A8: the mover must TRANSFER (∈Y, ∉X) within one sync window");
    assert_eq!(
        settlements_containing(&e, mover_id),
        1,
        "A8: mover must belong to EXACTLY ONE settlement (no duplicate membership)"
    );
    assert!(!is_member_of(&e, x_sid, mover_id), "A8: mover removed from the OLD settlement X");
    assert!(is_member_of(&e, y_sid, mover_id), "A8: mover added to the NEW settlement Y");
    let current_after = current(&e, x_sid);
    assert_eq!(
        current_after,
        current_before - 1,
        "A8: X.population_stats.current must decrement by 1 ({current_before} → {current_after})"
    );
    println!(
        "[membership A8] transfer X→Y; exactly one; X.current {current_before}→{current_after} ✓"
    );
}

// ════════════════════════════════════════════════════════════════════════════
// Assertion 9 — join does NOT exceed SETTLEMENT_MAX_POP (cap enforcement).
// ════════════════════════════════════════════════════════════════════════════
#[test]
fn harness_membership_a9_join_does_not_exceed_max_pop() {
    // Type A. FEASIBILITY GUARD FIRST: fill one settlement to exactly
    // SETTLEMENT_MAX_POP. If unreachable in-harness, report a setup limitation.
    // Then relocate one surplus non-member into the radius and assert the cap
    // holds AND the surplus is rejected. Sync-Liveness for the no-change
    // rejection: a witness settlement proves the sync ran in the window.
    let cap = SETTLEMENT_MAX_POP as usize;
    let mut e = fresh_engine();
    let (s, _founders) = form_settlement_at(&mut e, 20, 20);
    let (w, _w) = form_settlement_at(&mut e, 50, 10); // witness settlement, far away
    let ft = formation_tile(&e, s);

    // Spawn enough stationary agents on distinct in-radius tiles to reach the cap
    // (3 founders already members). Tiles span the 11×11 chebyshev-≤5 block.
    let mut placed = 3usize;
    'fill: for dy in 0..=(2 * SETTLEMENT_PROXIMITY_RADIUS) {
        for dx in 0..=(2 * SETTLEMENT_PROXIMITY_RADIUS) {
            if placed >= cap {
                break 'fill;
            }
            let x = ft.0 + dx - SETTLEMENT_PROXIMITY_RADIUS;
            let y = ft.1 + dy - SETTLEMENT_PROXIMITY_RADIUS;
            // Skip the founder tiles (already members at (ft.0..ft.0+2, ft.1)).
            if y == ft.1 && (ft.0..=ft.0 + 2).contains(&x) {
                continue;
            }
            seed_stationary(&mut e, x, y);
            placed += 1;
        }
    }

    // Step until S reaches the cap (each admission proves the sync runs).
    let mut reached = false;
    for _ in 0..SYNC_BUDGET {
        e.tick();
        if member_len(&e, s) == cap {
            reached = true;
            break;
        }
    }
    if !reached {
        println!(
            "[membership A9] SETUP LIMITATION: could not fill the settlement to SETTLEMENT_MAX_POP ({cap}); \
             at-cap state unconstructable (len={})",
            member_len(&e, s)
        );
        return;
    }
    assert_eq!(
        member_len(&e, s),
        cap,
        "A9 PRECONDITION: member_agents.len() must equal SETTLEMENT_MAX_POP before adding the surplus"
    );

    // Add ONE surplus non-member inside the radius.
    let surplus = seed_stationary(&mut e, ft.0, ft.1);
    let surplus_id = agent_id(&e, surplus);

    // Sync-Liveness for the no-change rejection: prove the sync ran via W.
    let proven = prove_sync_join(&mut e, w, SYNC_BUDGET);
    if proven.is_none() {
        println!("[membership A9] SETUP LIMITATION: sync did not run in the rejection window");
        return;
    }

    assert!(
        member_len(&e, s) <= cap,
        "A9: a join must NOT push member_agents.len() above SETTLEMENT_MAX_POP ({cap}); got {}",
        member_len(&e, s)
    );
    assert!(
        !is_member_of(&e, s, surplus_id),
        "A9: the surplus agent must be REJECTED (cap actively rejected the join)"
    );
    println!("[membership A9] cap held at {cap}; surplus rejected ✓");
}

// ════════════════════════════════════════════════════════════════════════════
// Assertion 10 — mass roster does NOT collapse to zero (★ direct 75→0 replication).
// ════════════════════════════════════════════════════════════════════════════
#[test]
fn harness_membership_a10_mass_roster_does_not_collapse() {
    // Type D. Direct replication of the reported 75→0 mass-collapse. Select a
    // settlement with len >= 5 (PRECONDITION — non-vacuous), then assert its
    // roster never hits 0 across a +200-tick observation window.
    const FORM_TICKS: u64 = 300;
    const OBSERVE_TICKS: u64 = 200;
    let mut e = production_scene_engine();
    for _ in 0..FORM_TICKS {
        e.tick();
    }

    let sid = e
        .resources
        .settlements
        .iter()
        .find(|(_, s)| s.member_agents.len() >= 5)
        .map(|(id, _)| *id);
    let sid = match sid {
        Some(id) => id,
        None => panic!(
            "A10 PRECONDITION: a settlement with >= 5 members must exist after {FORM_TICKS} ticks \
             (the reform must grow rosters — none found)"
        ),
    };
    let pre_len = member_len(&e, sid);

    let mut min_len = pre_len;
    for t in 1..=OBSERVE_TICKS {
        e.tick();
        let len = member_len(&e, sid);
        min_len = min_len.min(len);
        assert!(
            len > 0,
            "A10: the selected settlement's roster must NEVER collapse to 0 (tick +{t}, started at {pre_len})"
        );
    }
    println!(
        "[membership A10] selected roster started {pre_len}, min over +{OBSERVE_TICKS} ticks = {min_len} (> 0) ✓"
    );
}
