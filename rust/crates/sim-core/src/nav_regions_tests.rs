//! Frozen region labels and connectivity-premise tests for navigation preparation.
use super::*;

fn reference(map: &CostMap) -> Vec<u32> {
    let mut labels = vec![0u32; map.cost.len()];
    let mut next = 1u32;
    let mut queue: VecDeque<Tile> = VecDeque::new();
    for y in 0..map.height {
        for x in 0..map.width {
            let i = (y as usize) * (map.width as usize) + (x as usize);
            if labels[i] != 0 || map.cost[i] == IMPASSABLE {
                continue;
            }
            labels[i] = next;
            queue.push_back((x, y));
            while let Some(t) = queue.pop_front() {
                for n in map.steps_from(t) {
                    let j = (n.1 as usize) * (map.width as usize) + (n.0 as usize);
                    if labels[j] == 0 {
                        labels[j] = next;
                        queue.push_back(n);
                    }
                }
            }
            next += 1;
        }
    }
    labels
}

fn assert_labels(map: &CostMap) {
    let expected = reference(map);
    assert_eq!(Regions::compute(map), expected, "map={map:?}");
    assert_eq!(Regions::compute(map), expected, "repeat");
}

#[test]
fn region_labels_match_frozen_all_4x4_masks_and_diagonal_premise() {
    let costs = [COST_PLAIN, COST_DOOR, COST_BANK, u8::MAX];
    for mask in 0u32..=u32::from(u16::MAX) {
        let mut map = CostMap::new(4, 4);
        for (i, cell) in map.cost.iter_mut().enumerate() {
            *cell = if mask & (1 << i) == 0 {
                IMPASSABLE
            } else {
                costs[i % costs.len()]
            };
        }
        assert_labels(&map);
        // Region BFS never visits an impassable source. Under that actual movement
        // premise, every accepted diagonal has both cardinal two-step alternatives.
        for y in 0..4 {
            for x in 0..4 {
                let a = (x, y);
                if !map.passable(a) {
                    continue;
                }
                for b in map.steps_from(a) {
                    if a.0 != b.0 && a.1 != b.1 {
                        let horizontal = (b.0, a.1);
                        let vertical = (a.0, b.1);
                        assert!(map.can_step(a, horizontal) && map.can_step(horizontal, b));
                        assert!(map.can_step(a, vertical) && map.can_step(vertical, b));
                    }
                }
            }
        }
    }
}

#[test]
fn region_labels_match_frozen_empty_edges_weighted_and_large_maps() {
    for (width, height) in [
        (0, 0),
        (0, 7),
        (7, 0),
        (1, 1),
        (1, 37),
        (37, 1),
        (7, 11),
        (192, 192),
    ] {
        for pattern in 0..8 {
            let mut map = CostMap::new(width, height);
            for y in 0..height {
                for x in 0..width {
                    let open = match pattern {
                        0 => false,
                        1 => true,
                        2 => (x + y) % 2 == 0,
                        3 => x == 0 || y == 0 || x + 1 == width || y + 1 == height,
                        4 => x % 3 != 1 || y % 7 == 0,
                        5 => y % 3 != 1 || x % 7 == 0,
                        6 => (x * 17 + y * 31) % 13 < 8,
                        _ => (x * 37 + y * 11) % 19 < 11,
                    };
                    let cost = if open {
                        [1, COST_PLAIN, COST_DOOR, COST_BANK, u8::MAX][((x + y) % 5) as usize]
                    } else {
                        IMPASSABLE
                    };
                    map.set((x, y), cost);
                }
            }
            assert_labels(&map);
        }
    }
}

#[test]
fn region_queries_keep_exact_labels_and_lazy_rebuild_counters() {
    let mut nav = NavState::default();
    assert_eq!(nav.region_of((0, 0)), 0);
    assert_eq!(*nav.stats.borrow(), NavStats::default());
    let mut map = CostMap::new(7, 5);
    for y in 0..5 {
        map.set((3, y), IMPASSABLE);
    }
    nav.install(map.clone());
    let mut builds = 0;
    let mut expected_version = nav.version;
    for stage in 0..5 {
        match stage {
            1 => {
                assert!(nav.set_cost((3, 2), COST_DOOR));
                map.set((3, 2), COST_DOOR);
                expected_version += 1;
            }
            2 => {
                // Cost-only edits retain topology but must still invalidate once.
                assert!(nav.set_cost((3, 2), COST_BANK));
                map.set((3, 2), COST_BANK);
                expected_version += 1;
            }
            3 => {
                assert!(!nav.set_cost((3, 2), COST_BANK));
            }
            4 => nav.install(map.clone()),
            _ => {}
        }
        assert_eq!(nav.version, expected_version);
        assert_eq!(nav.map(), &map);
        assert_eq!(nav.region_of((99, 99)), 0);
        assert_eq!(
            nav.stats.borrow().region_builds,
            builds,
            "off-map must not warm cache"
        );
        let expected = reference(&map);
        let mut actual = Vec::new();
        for y in 0..5 {
            for x in 0..7 {
                actual.push(nav.region_of((x, y)));
            }
        }
        assert_eq!(actual, expected);
        if stage != 3 {
            builds += 1;
        }
        assert_eq!(
            *nav.stats.borrow(),
            NavStats {
                region_builds: builds,
                ..NavStats::default()
            }
        );
        // Repeat region and reach/goal queries must leave the already warm cache alone.
        for y in 0..5 {
            for x in 0..7 {
                let goal = (x, y);
                let mut goal_regions = Vec::new();
                if map.passable(goal) {
                    goal_regions.push(expected[(y * 7 + x) as usize]);
                } else {
                    for dy in -1i64..=1 {
                        for dx in -1i64..=1 {
                            let (nx, ny) = (i64::from(x) + dx, i64::from(y) + dy);
                            if (dx, dy) != (0, 0) && nx >= 0 && ny >= 0 && nx < 7 && ny < 5 {
                                let label = expected[(ny * 7 + nx) as usize];
                                if label != 0 {
                                    goal_regions.push(label);
                                }
                            }
                        }
                    }
                    goal_regions.sort_unstable();
                    goal_regions.dedup();
                }
                assert_eq!(nav.goal_regions(goal), goal_regions);
                for region in 0..=3 {
                    assert_eq!(
                        nav.reaches(region, goal),
                        goal_regions.contains(&region) && region != 0
                    );
                }
            }
        }
        assert_eq!(
            *nav.stats.borrow(),
            NavStats {
                region_builds: builds,
                ..NavStats::default()
            }
        );
    }
}

#[test]
fn preview_reachability_matches_regions_for_every_mask_and_goal_boundary() {
    for mask in 0u32..=u32::from(u16::MAX) {
        let mut map = CostMap::new(4, 4);
        for (i, cell) in map.cost.iter_mut().enumerate() {
            *cell = if mask & (1 << i) == 0 {
                IMPASSABLE
            } else {
                COST_DOOR
            };
        }
        let mut nav = NavState::default();
        nav.install(map);
        for start in [(0, 0), (3, 3)] {
            let region = nav.region_of(start);
            for goals in [
                vec![],
                vec![(1, 1)],
                vec![(0, 3), (3, 0), (3, 0)],
                vec![(4, 3)],
            ] {
                let expected = region != 0 && goals.iter().all(|&goal| nav.reaches(region, goal));
                let before = *nav.stats.borrow();
                assert_eq!(
                    nav.preview_reaches_all(start, goals.iter().copied()),
                    expected,
                    "mask={mask} start={start:?} goals={goals:?}"
                );
                assert_eq!(
                    *nav.stats.borrow(),
                    before,
                    "preview flood has no cache effects"
                );
            }
        }
    }
    for (width, height) in [(0, 0), (0, 7), (7, 0), (1, 9), (9, 1), (7, 5)] {
        let mut nav = NavState::default();
        nav.install(CostMap::new(width, height));
        for stage in 0..2 {
            if stage == 1 {
                nav.set_cost((0, 0), IMPASSABLE);
            }
            for start in [(0, 0), (1, 1), (u32::MAX, 0)] {
                let region = nav.region_of(start);
                for goals in [vec![], vec![(0, 0), (width, 0)], vec![(u32::MAX, u32::MAX)]] {
                    assert_eq!(
                        nav.preview_reaches_all(start, goals.iter().copied()),
                        region != 0 && goals.iter().all(|&goal| nav.reaches(region, goal)),
                        "size={width}x{height} stage={stage} start={start:?} goals={goals:?}"
                    );
                }
            }
        }
    }
}

#[test]
fn preview_paths_match_normal_exact_vectors_on_weighted_and_blocked_maps() {
    for pattern in 0..8 {
        let mut map = CostMap::new(7, 5);
        for y in 0..5 {
            for x in 0..7 {
                let cost = if (x * 17 + y * 31 + pattern) % 7 < 2 {
                    IMPASSABLE
                } else {
                    [1, COST_PLAIN, COST_DOOR, COST_BANK, u8::MAX][((x + y) % 5) as usize]
                };
                map.set((x, y), cost);
            }
        }
        let mut normal = NavState::default();
        normal.install(map.clone());
        let mut preview = NavState::default();
        preview.install(map);
        // Bounded off-map endpoints avoid the normal path API's existing octile overflow.
        // Extreme-coordinate wrapping is checked by the reachability oracle above.
        let endpoints = [(0, 0), (3, 2), (6, 4), (7, 4), (8, 4)];
        for start in endpoints {
            for goal in endpoints {
                assert_eq!(
                    preview.preview_path(start, goal),
                    normal.path(start, goal),
                    "pattern={pattern} {start:?}->{goal:?}"
                );
            }
        }
        normal.set_cost((3, 2), COST_DOOR);
        preview.set_cost((3, 2), COST_DOOR);
        assert_eq!(
            preview.preview_path((0, 0), (6, 4)),
            normal.path((0, 0), (6, 4))
        );
    }
}

#[test]
fn preview_subset_preserves_order_without_warming_or_replacing_regions() {
    fn check(nav: &NavState) {
        let mut oracle = NavState::default();
        oracle.install(nav.map().clone());
        let targets = [
            (2, 1),
            (0, 0),
            (2, 1),
            (1, 1),
            (0, 2),
            (3, 0),
            (u32::MAX, u32::MAX),
            (u32::MAX, 0),
        ];
        for start in [(0, 0), (1, 1), (2, 2), (3, 0), (u32::MAX, 0)] {
            let region = oracle.region_of(start);
            let expected: Vec<_> = targets
                .into_iter()
                .filter(|&tile| oracle.reaches(region, tile))
                .collect();
            let before = (
                nav.regions.borrow().version,
                nav.regions.borrow().labels.clone(),
            );
            let stats = *nav.stats.borrow();
            assert_eq!(
                nav.preview_reachable_tiles(start, targets),
                expected,
                "start={start:?} map={:?}",
                nav.map().cost
            );
            assert_eq!(nav.preview_reachable_tiles(start, []), Vec::<Tile>::new());
            assert_eq!(
                (
                    nav.regions.borrow().version,
                    nav.regions.borrow().labels.clone()
                ),
                before
            );
            assert_eq!(*nav.stats.borrow(), stats);
        }
    }
    for mask in 0u16..512 {
        let mut map = CostMap::new(3, 3);
        for (i, cost) in map.cost.iter_mut().enumerate() {
            *cost = if mask & (1 << i) == 0 {
                COST_DOOR
            } else {
                IMPASSABLE
            };
        }
        let mut nav = NavState::default();
        nav.install(map);
        check(&nav); // Cold cache stays cold.
        nav.region_of((0, 0));
        check(&nav); // Warm cache is borrowed unchanged.
        nav.set_cost(
            (1, 1),
            if nav.passable((1, 1)) {
                IMPASSABLE
            } else {
                COST_PLAIN
            },
        );
        check(&nav); // Stale labels remain untouched while local labels are used.
        nav.install(CostMap::new(1, 9));
        check(&nav); // Same cell count, different dimensions and discarded cache.
        nav.install(CostMap::new(0, 9));
        check(&nav);
    }
}
