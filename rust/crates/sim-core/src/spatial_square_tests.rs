//! Allocation-free square queries compared with a full index scan.
use super::*;

#[test]
fn square_iterator_matches_full_scan_at_chunk_and_coordinate_boundaries() {
    let mut index = ChunkIndex::new();
    for y in [0, 1, 15, 16, 17, 31, 32, u32::MAX - 1, u32::MAX] {
        for x in [0, 1, 15, 16, 17, 31, 32, u32::MAX - 1, u32::MAX] {
            index.insert((x, y), 1_u8);
        }
    }
    index.insert((16, 16), 1);
    index.insert((16, 16), 2);
    for centre in [(0, 0), (16, 15), (31, 32), (u32::MAX, u32::MAX)] {
        for radius in [0, 1, 8, 16, 33, u32::MAX] {
            let expected: Vec<_> = index
                .iter()
                .filter(|(tile, _)| chebyshev(*tile, centre) <= radius)
                .collect();
            assert_eq!(
                index.within_square(centre, radius).collect::<Vec<_>>(),
                expected
            );
        }
    }
    assert_eq!(
        ChunkIndex::<()>::new()
            .within_square((0, 0), u32::MAX)
            .count(),
        0
    );
}

#[test]
fn bulk_construction_matches_insert_for_order_queries_duplicates_and_mutation() {
    let mut entries = Vec::new();
    for y in [0, 15, 16, 31, u32::MAX] {
        for x in [0, 15, 16, 31, u32::MAX] {
            entries.push(((x, y), (x % 3) as u8));
        }
    }
    entries.extend([((16, 16), 0), ((16, 16), 2), ((16, 16), 0)]);
    for order in 0..3 {
        let mut input = entries.clone();
        match order {
            0 => {}
            1 => input.reverse(),
            _ => input.rotate_left(7),
        }
        let mut inserted = ChunkIndex::new();
        for &(tile, value) in &input {
            inserted.insert(tile, value);
        }
        let mut bulk = ChunkIndex::from_entries(input);
        assert_eq!(bulk, inserted);
        assert_eq!(bulk.len(), entries.len());
        assert!(!bulk.is_empty());
        for centre in [(0, 0), (16, 15), (31, 32), (u32::MAX, u32::MAX)] {
            for radius in [0, 1, 16, 33, u32::MAX] {
                assert_eq!(
                    bulk.within_square(centre, radius).collect::<Vec<_>>(),
                    inserted.within_square(centre, radius).collect::<Vec<_>>()
                );
                let mut bulk_calls = Vec::new();
                let mut inserted_calls = Vec::new();
                let found = bulk.nearest_where(centre, radius, |tile, value| {
                    bulk_calls.push((tile, *value));
                    *value > 0
                });
                let expected = inserted.nearest_where(centre, radius, |tile, value| {
                    inserted_calls.push((tile, *value));
                    *value > 0
                });
                assert_eq!(found, expected);
                assert_eq!(bulk_calls, inserted_calls);
            }
            for ring in 0..=2 {
                assert_eq!(
                    bulk.ring(centre, ring).collect::<Vec<_>>(),
                    inserted.ring(centre, ring).collect::<Vec<_>>()
                );
            }
        }
        for _ in 0..4 {
            assert_eq!(bulk.remove((16, 16), 0), inserted.remove((16, 16), 0));
            assert_eq!(bulk, inserted);
        }
        bulk.insert((16, 16), 4);
        inserted.insert((16, 16), 4);
        assert_eq!(bulk, inserted);
        bulk.clear();
        assert_eq!(bulk, ChunkIndex::new());
    }
    let empty = ChunkIndex::<u8>::from_entries([]);
    assert!(empty.is_empty());
    assert_eq!(empty.len(), 0);
    assert_eq!(empty.iter().count(), 0);
}

#[test]
fn bulk_preserves_reverse_insertion_order_for_equal_ordered_payloads() {
    #[derive(Clone, Copy, Debug)]
    struct Ranked(u8, u8);
    impl PartialEq for Ranked {
        fn eq(&self, other: &Self) -> bool {
            self.0 == other.0
        }
    }
    impl Eq for Ranked {}
    impl PartialOrd for Ranked {
        fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
            Some(self.cmp(other))
        }
    }
    impl Ord for Ranked {
        fn cmp(&self, other: &Self) -> std::cmp::Ordering {
            self.0.cmp(&other.0)
        }
    }
    let entries = [
        ((1, 1), Ranked(1, 10)),
        ((1, 1), Ranked(0, 20)),
        ((1, 1), Ranked(1, 30)),
        ((1, 1), Ranked(1, 40)),
    ];
    let mut inserted = ChunkIndex::new();
    for (tile, value) in entries {
        inserted.insert(tile, value);
    }
    let bulk = ChunkIndex::from_entries(entries);
    let labels =
        |index: &ChunkIndex<Ranked>| index.iter().map(|(_, value)| value.1).collect::<Vec<_>>();
    assert_eq!(labels(&bulk), labels(&inserted));
    assert_eq!(labels(&bulk), vec![20, 40, 30, 10]);
}
