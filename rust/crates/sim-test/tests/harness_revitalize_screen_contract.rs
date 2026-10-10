//! Truthful-screen source contracts complement runtime smoke and visual acceptance.

#[cfg(test)]
mod tests {
    #[test]
    fn harness_building_renderer_reads_snapshot_only() {
        let world = include_str!("../../../../scripts/ui/world_renderer.gd");
        let buildings = include_str!("../../../../scripts/ui/building_renderer.gd");
        let terrain = include_str!("../../../../scripts/ui/terrain_renderer.gd");
        for source in [world, buildings, terrain] {
            for forbidden in [
                "on_building_placed(",
                "RESOURCE_SEED",
                "VILLAGE_FIXTURE",
                "SOURCE_KIND_COLORS",
            ] {
                assert!(!source.contains(forbidden), "renderer contains {forbidden}");
            }
        }
        assert!(buildings.contains("get_building_snapshot()"));
        assert!(buildings.contains("get_construction_snapshot()"));
        assert!(buildings.contains("get_resource_snapshot()"));
    }

    #[test]
    fn harness_territory_terrain_renderer_uses_ffi() {
        let source = include_str!("../../../../scripts/ui/terrain_renderer.gd");
        assert!(source.contains("\"rock\": return code == 2"));
        assert!(source.contains("get_terrain_snapshot()"));
        assert!(!source.contains("RandomNumberGenerator"));
        assert!(
            source.contains("\"hill\": return code == 2 or code == 3"),
            "Hill and Mountain atlas rows differ from Rust codes"
        );
    }

    #[test]
    fn harness_economy_hud_reads_sim_time_and_tallies() {
        let topbar = include_str!("../../../../scripts/ui/panels/hud_topbar.gd");
        let status = include_str!("../../../../scripts/ui/panels/hud_status_panel.gd");
        assert!(topbar.contains(r#"_snapshot("get_time_snapshot")"#));
        assert!(topbar.contains(r#"_snapshot("get_tallies")"#));
        assert!(topbar.contains("_world_sim.call(method)"));
        for source in [topbar, status] {
            for forbidden in ["TICKS_PER_DAY", "\"Berry", "\"Wood", "\"Tick %d"] {
                assert!(!source.contains(forbidden), "HUD contains {forbidden}");
            }
        }
    }
}
