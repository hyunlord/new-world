//! T6.7 — Material RON 105 criterion benchmarks.
//!
//! Four benchmarks covering the material data layer hot paths:
//!   load_directory (cold start), registry register, by_category dispatch,
//!   iter_category (Stone 30).

use std::path::Path;

use criterion::{black_box, criterion_group, criterion_main, Criterion};
use sim_core::material::{load_directory, MaterialDef, MaterialRegistry};

const DATA_DIR: &str = "data/material";

fn load_all_materials() -> Vec<MaterialDef> {
    let mut all_materials = vec![];
    for category in &["stone", "wood", "animal", "mineral", "plant"] {
        let dir = Path::new(DATA_DIR).join(category);
        all_materials.extend(load_directory(&dir).unwrap());
    }
    all_materials
}

fn build_full_registry() -> MaterialRegistry {
    let mut registry = MaterialRegistry::default();
    for def in load_all_materials() {
        registry.register(def, None).unwrap();
    }
    registry
}

fn bench_load_directory_all_categories(c: &mut Criterion) {
    c.bench_function("load_directory_all_categories", |b| {
        b.iter(|| {
            for category in &["stone", "wood", "animal", "mineral", "plant"] {
                let dir = Path::new(DATA_DIR).join(category);
                let _ = black_box(load_directory(&dir).unwrap());
            }
        });
    });
}

fn bench_registry_register_105(c: &mut Criterion) {
    let all_materials = load_all_materials();

    c.bench_function("registry_register_105", |b| {
        b.iter(|| {
            let mut registry = MaterialRegistry::default();
            for def in all_materials.iter().cloned() {
                let _ = black_box(registry.register(def, None));
            }
        });
    });
}

fn bench_by_category_dispatcher(c: &mut Criterion) {
    let registry = build_full_registry();

    c.bench_function("by_category_dispatcher", |b| {
        b.iter(|| {
            black_box(registry.stones().count());
            black_box(registry.woods().count());
            black_box(registry.animals().count());
            black_box(registry.minerals().count());
            black_box(registry.plants().count());
        });
    });
}

fn bench_iter_category_stone_30(c: &mut Criterion) {
    let registry = build_full_registry();

    c.bench_function("iter_category_stone_30", |b| {
        b.iter(|| {
            let count: usize = black_box(registry.stones().count());
            assert_eq!(count, 30);
        });
    });
}

criterion_group!(
    benches,
    bench_load_directory_all_categories,
    bench_registry_register_105,
    bench_by_category_dispatcher,
    bench_iter_category_stone_30,
);
criterion_main!(benches);
