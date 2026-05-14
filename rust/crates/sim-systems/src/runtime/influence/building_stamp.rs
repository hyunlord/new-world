//! `BuildingStampSystem` — priority 90, every tick.
//!
//! Phase 0 Section 2.5.1 base. T7.7.B land (R1 event_queue):
//! drains `resources.building_event_queue` and calls
//! `InfluenceGrid::mark_dirty` for each stamped channel
//! (Warmth/Spiritual/Beauty/Light) using a Chebyshev box clamped to
//! grid bounds.
//!
//! Isolation invariant: this system writes to `dirty_regions` only — it
//! does **not** touch the influence `pending` buffers. Any value written
//! into `pending` before this system runs must still be present when it
//! exits. This invariant is tested by both `harness_building_stamp_isolation`
//! (T7.6) and `harness_ffi_isolation_invariant_survives_empty_queue_tick`
//! (T7.7.B).

use hecs::World;
use sim_core::causal::CausalEvent;
use sim_core::influence::{DirtyRegion, InfluenceChannel};
use sim_engine::{RuntimeSystem, SimResources};

/// Channels stamped by every building placement.
///
/// T7.7.B baseline: Warmth/Spiritual/Beauty/Light.
/// T7.10.C extension: Noise added so IUS can run `propagate_noise` from the
/// stamped region. A building placement counts as a transient acoustic event
/// for V7 Phase 2 (no agents yet), matching the on_building_placed → Warmth+Light
/// stamping convention.
/// T7.10.D extension: Danger added so IUS can run `propagate_danger` from the
/// stamped region. For V7 Phase 2 (no agents yet) the building placement is the
/// only Danger source — Phase 0 ISSUE 3 fix locks linear alpha=5 with a
/// sight-radius cap of 15 tiles (no wall blocking).
const STAMPED_CHANNELS: &[InfluenceChannel] = &[
    InfluenceChannel::Warmth,
    InfluenceChannel::Spiritual,
    InfluenceChannel::Beauty,
    InfluenceChannel::Light,
    InfluenceChannel::Noise,
    InfluenceChannel::Danger,
];

/// Phase 2 building → influence stamper (T7.7.B drains FFI queue).
pub struct BuildingStampSystem;

impl BuildingStampSystem {
    /// Construct a new stamper.
    pub fn new() -> Self {
        Self
    }
}

impl Default for BuildingStampSystem {
    fn default() -> Self {
        Self::new()
    }
}

impl RuntimeSystem for BuildingStampSystem {
    fn name(&self) -> &str {
        "BuildingStampSystem"
    }

    fn priority(&self) -> u32 {
        90
    }

    fn tick_interval(&self) -> u64 {
        1
    }

    fn tick(&mut self, _world: &mut World, resources: &mut SimResources) {
        let w = resources.influence_grid.width;
        let h = resources.influence_grid.height;

        // Zero-dimension guard: drain queue without any grid access.
        if w == 0 || h == 0 {
            resources.building_event_queue.clear();
            return;
        }

        let tick = resources.current_tick;

        // Drain all queued events in FIFO order.
        while let Some(ev) = resources.building_event_queue.pop_front() {
            let (cx, cy) = ev.position;

            // OOB guard: consume the event but emit no dirty region.
            if cx >= w || cy >= h {
                continue;
            }

            let r = ev.radius;
            let x1 = cx.saturating_sub(r);
            let y1 = cy.saturating_sub(r);
            let x2 = cx.saturating_add(r).min(w - 1);
            let y2 = cy.saturating_add(r).min(h - 1);

            // Phase 3-α: record the BuildingPlaced event onto the centre
            // tile so the "왜?" UI can trace any downstream stamp/influence
            // back to this FFI arrival.
            let centre_idx = resources.influence_grid.idx(cx, cy) as u32;
            resources.causal_log.push(
                centre_idx,
                CausalEvent::BuildingPlaced {
                    position: ev.position,
                    radius: r,
                    tick,
                },
            );

            for ch in STAMPED_CHANNELS {
                let region = DirtyRegion::new(x1, y1, x2, y2);
                resources.influence_grid.mark_dirty(*ch, region.clone());

                // Phase 3-α: record the BSS dirty-mark on the same centre
                // tile, once per stamped channel. Region is captured by
                // value so the "왜?" UI can reproduce the BFS box even
                // after IUS drains the dirty list.
                resources.causal_log.push(
                    centre_idx,
                    CausalEvent::StampDirty {
                        channel: *ch,
                        region,
                        tick,
                    },
                );
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use sim_core::material::MaterialRegistry;
    use sim_engine::{BuildingPlacedEvent, SimEngine};

    fn engine() -> SimEngine {
        SimEngine::new(32, 32, MaterialRegistry::new())
    }

    #[test]
    fn metadata() {
        let s = BuildingStampSystem::new();
        assert_eq!(s.name(), "BuildingStampSystem");
        assert_eq!(s.priority(), 90);
        assert_eq!(s.tick_interval(), 1);
    }

    #[test]
    fn empty_queue_is_no_op() {
        let mut e = engine();
        let mut s = BuildingStampSystem::new();
        s.tick(&mut e.world, &mut e.resources);
        for ch in InfluenceChannel::all() {
            assert!(e.resources.influence_grid.dirty_regions[*ch as usize].is_empty());
        }
    }

    #[test]
    fn single_event_marks_stamped_channels_dirty() {
        let mut e = engine();
        e.resources
            .building_event_queue
            .push_back(BuildingPlacedEvent {
                position: (10, 10),
                radius: 2,
            });
        let mut s = BuildingStampSystem::new();
        s.tick(&mut e.world, &mut e.resources);
        for ch in STAMPED_CHANNELS {
            let regs = &e.resources.influence_grid.dirty_regions[*ch as usize];
            assert_eq!(regs.len(), 1, "{ch:?} should have 1 dirty region");
        }
        // Non-stamped channels untouched (FoodAroma, Social).
        for ch in [InfluenceChannel::FoodAroma, InfluenceChannel::Social] {
            assert!(
                e.resources.influence_grid.dirty_regions[ch as usize].is_empty(),
                "{ch:?} must remain empty (not in STAMPED_CHANNELS)"
            );
        }
        // Queue drained.
        assert!(e.resources.building_event_queue.is_empty());
    }

    #[test]
    fn out_of_bounds_event_is_skipped() {
        let mut e = engine();
        e.resources
            .building_event_queue
            .push_back(BuildingPlacedEvent {
                position: (999, 999),
                radius: 1,
            });
        let mut s = BuildingStampSystem::new();
        s.tick(&mut e.world, &mut e.resources);
        for ch in STAMPED_CHANNELS {
            assert!(
                e.resources.influence_grid.dirty_regions[*ch as usize].is_empty(),
                "{ch:?} must have no dirty regions for OOB event"
            );
        }
        // Queue still drained.
        assert!(e.resources.building_event_queue.is_empty());
    }

    #[test]
    fn radius_clamps_to_grid() {
        let mut e = engine(); // 32×32
        e.resources
            .building_event_queue
            .push_back(BuildingPlacedEvent {
                position: (1, 1),
                radius: 100, // huge — must clamp to (0,0)..(31,31)
            });
        let mut s = BuildingStampSystem::new();
        s.tick(&mut e.world, &mut e.resources);
        let regs =
            &e.resources.influence_grid.dirty_regions[InfluenceChannel::Warmth as usize];
        assert_eq!(regs.len(), 1);
        let r = &regs[0];
        assert!(r.max_x <= 31, "max_x {} must be ≤ 31", r.max_x);
        assert!(r.max_y <= 31, "max_y {} must be ≤ 31", r.max_y);
    }

    #[test]
    fn tick_does_not_panic() {
        let mut e = SimEngine::new(32, 32, MaterialRegistry::new());
        e.register_system(Box::new(BuildingStampSystem::new()));
        for _ in 0..5 {
            e.tick();
        }
        assert_eq!(e.current_tick(), 5);
    }

    #[test]
    fn shell_does_not_mutate_tile_grid() {
        let mut e = SimEngine::new(32, 32, MaterialRegistry::new());
        let before = e.resources.tile_grid.len();
        e.register_system(Box::new(BuildingStampSystem::new()));
        for _ in 0..3 {
            e.tick();
        }
        assert_eq!(e.resources.tile_grid.len(), before);
    }
}
