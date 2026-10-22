//! M2 Part 1: items, resource nodes and the per-kind ledger (goal 3.1, 3.2, §5).

mod colony_support;

use colony_support::tick_checked;
use sim_core::colony::materials::{FLINT, GRANITE, LIMESTONE};
use sim_core::colony::{Command, ItemKind, NodeKind};
use sim_core::TerrainType;
use sim_engine::SimEngine;
use sim_systems::commands::submit;
use sim_systems::runtime::jobs::invariants::ledger_of;
use sim_systems::worldgen::colony::hearth;
use sim_test::probe::colony::{nearest_nodes, stockpile_block, u_script_commands};

fn terrain(engine: &SimEngine, t: (u32, u32)) -> TerrainType {
    let g = &engine.resources.tile_grid;
    g.terrain_type[g.idx(t.0, t.1)]
}

/// The bank tiles as the goal defines them (not water or mountain, within 2 of
/// a river tile), recomputed here independently of the worldgen code.
fn bank_tiles(engine: &SimEngine) -> usize {
    let g = &engine.resources.tile_grid;
    let mut n = 0;
    for y in 0..g.height {
        for x in 0..g.width {
            if matches!(
                terrain(engine, (x, y)),
                TerrainType::River | TerrainType::Mountain
            ) {
                continue;
            }
            let near = (y.saturating_sub(2)..=(y + 2).min(g.height - 1)).any(|yy| {
                (x.saturating_sub(2)..=(x + 2).min(g.width - 1))
                    .any(|xx| terrain(engine, (xx, yy)) == TerrainType::River)
            });
            n += usize::from(near);
        }
    }
    n
}

#[test]
fn resource_nodes_follow_the_placement_rule() {
    for seed in [0u64, 3] {
        let engine = sim_test::colony_engine(seed);
        let r = &engine.resources;
        let camp = hearth(r).expect("hearth");
        let (mut hill, mut grass, mut outcrops) = (0usize, 0usize, 0usize);
        for (tile, node) in &r.colony.nodes {
            let d = tile.0.abs_diff(camp.0).max(tile.1.abs_diff(camp.1));
            assert!(d > 6, "seed {seed}: node {tile:?} inside the clearing");
            assert!(!r.colony.trees.contains_key(tile) && !r.building_kinds.contains_key(tile));
            match node.kind {
                NodeKind::Boulder => {
                    assert!((8..=20).contains(&node.left), "{node:?}");
                    match terrain(&engine, *tile) {
                        TerrainType::Hill => {
                            assert_eq!(node.material, GRANITE);
                            hill += 1;
                        }
                        TerrainType::Plain => {
                            assert_eq!(node.material, LIMESTONE);
                            grass += 1;
                        }
                        other => panic!("boulder on {other:?}"),
                    }
                }
                NodeKind::FlintOutcrop => {
                    assert!((6..=14).contains(&node.left), "{node:?}");
                    assert_eq!(node.material, FLINT);
                    outcrops += 1;
                }
            }
        }
        let count = |t: TerrainType| r.tile_grid.terrain_type.iter().filter(|x| **x == t).count();
        // 25 % of hills (DECISIONS D5) and 2 % of grass, minus excluded tiles:
        // the keyed draw is uniform, so the share lands within a few points.
        let (hills, plains) = (count(TerrainType::Hill), count(TerrainType::Plain));
        assert!(
            hill * 1000 >= hills * 200 && hill * 1000 <= hills * 300,
            "seed {seed}: {hill} boulders on {hills} hills"
        );
        assert!(
            grass * 1000 >= plains * 12 && grass * 1000 <= plains * 28,
            "seed {seed}: {grass} boulders on {plains} plains"
        );
        // Exactly one outcrop per 40 bank tiles (the free bank tiles are many more).
        let bank = bank_tiles(&engine);
        assert!(bank >= 400, "seed {seed}: {bank} bank tiles");
        assert_eq!(
            outcrops,
            bank / 40,
            "seed {seed}: outcrops on {bank} bank tiles"
        );
        assert_eq!(r.colony.materials.by_id.len(), 5);
    }
}

#[test]
fn quarrying_a_boulder_exhausts_it_in_batches_of_four() {
    let mut engine = sim_test::colony_engine(1);
    let boulder = nearest_nodes(&engine, NodeKind::Boulder, 1)[0];
    let node = engine.resources.colony.nodes[&boulder];
    let (x0, y0, x1, y1) = stockpile_block(&engine, 4, 3).expect("stockpile");
    for _ in 0..360 {
        tick_checked(&mut engine);
    }
    assert!(submit(&mut engine, Command::AddStockpile { x0, y0, x1, y1 }).accepted);
    let quarry = Command::DesignateQuarry {
        x0: boulder.0,
        y0: boulder.1,
        x1: boulder.0,
        y1: boulder.1,
    };
    assert_eq!(submit(&mut engine, quarry).count, 1);
    let mut gone_at = None;
    for tick in 0..3 * 1_440u64 {
        tick_checked(&mut engine);
        if gone_at.is_none() && !engine.resources.colony.nodes.contains_key(&boulder) {
            gone_at = Some(tick);
        }
    }
    let c = &engine.resources.colony;
    assert!(
        gone_at.is_some(),
        "boulder of {} not exhausted in 3 days",
        node.left
    );
    assert!(
        !c.designations.contains_key(&boulder),
        "the mark goes with the node"
    );
    let stone = ledger_of(&engine.world, &engine.resources, ItemKind::Stone);
    assert_eq!(
        stone.created,
        u64::from(node.left),
        "every stone of the boulder, no more"
    );
    assert_eq!(c.tallies.quarry_batches, u64::from(node.left).div_ceil(4));
    assert_eq!(c.tallies.nodes_exhausted, 1);
    // The stone is limestone or granite and reached the stockpile.
    let stocked: u64 = c
        .ground_items
        .iter()
        .filter(|(t, s)| s.kind == ItemKind::Stone && c.zone_of_tile.contains_key(t))
        .map(|(_, s)| {
            assert_eq!(s.material, Some(node.material));
            u64::from(s.count)
        })
        .sum();
    assert_eq!(stocked, stone.created, "all stone hauled to the stockpile");
}

/// Goal §5: the per-kind ledger, reservations and stacks over 8 seeds × 10
/// days with chop and quarry orders (the U-script plus the 2 nearest boulders
/// and the nearest outcrop). Logs, stone and flint must all be created.
#[test]
fn per_kind_ledger_holds_over_8_seeds_x_10_days_with_quarry_and_chop() {
    for seed in 0u64..8 {
        let mut engine = sim_test::colony_engine(seed);
        for tick in 0..10 * 1_440u64 {
            if tick == 360 {
                let mut cmds = u_script_commands(&engine);
                for (kind, n) in [(NodeKind::Boulder, 2), (NodeKind::FlintOutcrop, 1)] {
                    for (x, y) in nearest_nodes(&engine, kind, n) {
                        cmds.push(Command::DesignateQuarry {
                            x0: x,
                            y0: y,
                            x1: x,
                            y1: y,
                        });
                    }
                }
                for cmd in cmds {
                    assert!(
                        submit(&mut engine, cmd).accepted,
                        "seed {seed}: an order was rejected"
                    );
                }
            }
            tick_checked(&mut engine);
        }
        for kind in [ItemKind::Log, ItemKind::Stone, ItemKind::Flint] {
            let t = ledger_of(&engine.world, &engine.resources, kind);
            assert!(t.balanced());
            assert!(t.created > 0, "seed {seed}: no {} created", kind.name());
        }
        assert!(
            engine.resources.colony.tallies.nodes_exhausted >= 1,
            "seed {seed}"
        );
    }
}
