//! Template content tests; runtime physical completion is separately covered by spine_chain_log_house.
use super::*;
use crate::colony::content::{Catalog, RawSources};
use crate::colony::knowledge::ContentStatus;

const DATA: &str = include_str!("../../../data/colony/buildings.ron");

fn family() -> (Catalog, TemplateCatalog) {
    let catalog = Catalog::from_sources(RawSources::bundled(), None).expect("catalogue");
    let family = TemplateCatalog::parse(DATA, None, &catalog).expect("template family");
    (catalog, family)
}

#[test]
fn c2_templates_resolve_fifteen_connected_and_keep_nine_future_buildings_visible() {
    // Given the bundled template family, including future missing blueprint identities.
    let (_, templates) = family();
    // When its explicit connection states are counted.
    let connected = templates
        .buildings
        .iter()
        .filter(|t| t.status == ContentStatus::Connected)
        .count();
    let inert = templates
        .buildings
        .iter()
        .filter(|t| t.status == ContentStatus::Inert)
        .count();
    // C2 connects five reserved plans and retains the unsupported hive as explicit future data.
    assert_eq!((connected, inert), (15, 9));
    assert!(templates
        .buildings
        .iter()
        .filter(|t| t.status == ContentStatus::Inert)
        .all(|t| t.debt.as_deref().is_some_and(|debt| !debt.is_empty())));
}

#[test]
fn spine_log_house_plan_overrides_ring_door_and_totals_real_piece_costs() {
    // Given a 4x4 external log-house footprint.
    let (catalog, templates) = family();
    let house = templates.get("log_house").expect("log house");
    // When its file-ordered blueprint plan is expanded.
    let plan = house.expand(&catalog, None, 0).expect("plan");
    // Then eleven walls, one door, four floors and two beds use25logs (not illustrative14).
    assert_eq!(plan.len(), 18);
    for (name, count) in [
        ("log_wall", 11),
        ("door", 1),
        ("packed_floor", 4),
        ("bedroll", 2),
    ] {
        let kind = catalog.blueprint_by_name(name).expect("kind");
        assert_eq!(
            plan.iter().filter(|piece| piece.kind == kind).count(),
            count
        );
    }
    assert_eq!(
        material_cost(&plan, &catalog).expect("cost"),
        std::collections::BTreeMap::from([("log".to_string(), 25)])
    );
    let door = catalog.blueprint_by_name("door").expect("door");
    assert_eq!(
        plan.iter()
            .find(|piece| piece.kind == door)
            .expect("door plan")
            .offset,
        (2, 3)
    );
    let wall = catalog.blueprint_by_name("log_wall").expect("wall");
    assert!(!plan
        .iter()
        .any(|piece| piece.kind == wall && piece.offset == (2, 3)));
}

#[test]
fn spine_station_fits_existing_physical_orientation_and_work_tile_after_rotation() {
    // Given the enlarged executable workshop footprint and each authored station choice.
    let (catalog, templates) = family();
    let shed = templates.get("workshop_shed").expect("shed");
    for station in &shed.stations {
        for rotation in 0..4 {
            // When the template is rotated, only existing nonrotated workshop mechanics are used.
            let plan = shed
                .expand(&catalog, Some(station), rotation)
                .expect("rotated station plan");
            // Then the station and its south work tile fit within the physical room interior.
            let piece = plan.last().expect("station");
            let bounds = shed.rotated_footprint(rotation).expect("bounds");
            assert_eq!(
                piece.kind,
                catalog
                    .blueprint_by_name(station)
                    .expect("station identity")
            );
            assert!(piece.offset.0 > 0 && piece.offset.1 > 0);
            assert!(piece.offset.0 + piece.size.0 < bounds.0);
            assert!(piece.offset.1 + piece.size.1 + 1 < bounds.1);
        }
    }
}

#[test]
fn spine_template_bad_costs_and_dangling_connected_pieces_are_rejected() {
    // Given valid data with one independent malformed connected field at a time.
    let (catalog, _) = family();
    for (text, expected) in [
        (
            DATA.replacen("\"log\": 25", "\"log\": 14", 1),
            "declared materials",
        ),
        (
            DATA.replacen("kind: \"log_wall\"", "kind: \"missing_wall\"", 1),
            "missing blueprint",
        ),
        (
            DATA.replacen("\"pottery_bench\"", "\"missing_station\"", 1),
            "unknown station",
        ),
    ] {
        // When the local parser resolves the altered family.
        let error = TemplateCatalog::parse(&text, None, &catalog).expect_err("invalid data");
        // Then no malformed connected content reaches runtime.
        assert!(error.contains(expected), "{error}");
    }
}

#[test]
fn spine_inert_templates_invalid_rotations_and_station_choices_never_expand() {
    // Given a resolved template family.
    let (catalog, templates) = family();
    // When a caller requests inert content or an invalid command choice.
    let inert = templates
        .get("forge")
        .expect("forge")
        .expand(&catalog, None, 0);
    let rotated = templates
        .get("log_house")
        .expect("house")
        .expand(&catalog, None, 4);
    let station =
        templates
            .get("workshop_shed")
            .expect("shed")
            .expand(&catalog, Some("missing_station"), 0);
    // Then none produces a physical placement plan.
    assert!(inert.is_err());
    assert!(rotated.is_err());
    assert!(station.is_err());
}

#[test]
fn spine_multitile_same_layer_overlap_is_rejected_before_catalogue_installation() {
    // Given a 2x1 station selected at distinct adjacent interior anchors.
    let (catalog, templates) = family();
    let mut template = templates.get("workshop_shed").expect("shed").clone();
    template.stations.clear();
    template.pieces = vec![TemplatePiece {
        kind: "wood_bench".into(),
        at: PieceAt::Interior,
        count: Some(2),
    }];
    let station = catalog
        .blueprint(catalog.blueprint_by_name("wood_bench").expect("bench kind"))
        .expect("bench definition");
    assert_eq!(station.footprint, (2, 1));
    // Distinct anchors (1,1) and (2,1) share physical tile (2,1).
    let document = ron::to_string(&TemplateCatalog {
        schema_version: 1,
        zone_kinds: templates.zone_kinds.clone(),
        buildings: vec![template],
    })
    .expect("fixture ron");
    let error = TemplateCatalog::parse(&document, None, &catalog).expect_err("physical overlap");
    assert!(error.contains("overlapping physical footprint"), "{error}");
}

#[test]
fn spine_multitile_adjacent_pieces_and_floor_layer_overlap_remain_valid() {
    // Given 2x1 stations at x0 and x2 with an authored floor underneath.
    let (catalog, templates) = family();
    let mut template = templates.get("workshop_shed").expect("shed").clone();
    template.stations.clear();
    template.pieces = vec![
        TemplatePiece {
            kind: "packed_floor".into(),
            at: PieceAt::Anchor,
            count: None,
        },
        TemplatePiece {
            kind: "wood_bench".into(),
            at: PieceAt::Anchor,
            count: None,
        },
        TemplatePiece {
            kind: "wood_bench".into(),
            at: PieceAt::Edge(Direction::North),
            count: None,
        },
    ];
    let plan = template.expand(&catalog, None, 0).expect("adjacent pieces");
    assert_eq!(plan[1].offset, (0, 0));
    assert_eq!(plan[2].offset, (2, 0));
    assert_eq!(plan[1].size, (2, 1));
    template.materials = material_cost(&plan, &catalog).expect("fixture cost");
    let document = ron::to_string(&TemplateCatalog {
        schema_version: 1,
        zone_kinds: templates.zone_kinds.clone(),
        buildings: vec![template],
    })
    .expect("fixture ron");
    assert!(TemplateCatalog::parse(&document, None, &catalog).is_ok());
}

#[test]
fn spine_selected_station_reslots_after_occupied_multitile_pieces() {
    // Given one fixed 2x1 station at (1,1), plus the selected station choice.
    let (catalog, templates) = family();
    let mut template = templates.get("workshop_shed").expect("shed").clone();
    template.footprint = (6, 4);
    template.pieces = vec![TemplatePiece {
        kind: "wood_bench".into(),
        at: PieceAt::Interior,
        count: Some(1),
    }];
    template.stations = vec!["wood_bench".into()];
    // The next candidate (2,1) still overlaps; (3,1) is the first free physical footprint.
    let plan = template
        .expand(&catalog, None, 0)
        .expect("re-slotted station");
    assert_eq!(plan[0].offset, (1, 1));
    assert_eq!(plan[1].offset, (3, 1));
    assert_eq!(plan[1].size, (2, 1));
}

#[test]
fn spine_nonfloor_categories_share_one_physical_occupancy_layer() {
    let (catalog, templates) = family();
    let mut template = templates.get("workshop_shed").expect("shed").clone();
    template.footprint = (3, 4);
    template.stations.clear();
    // Different anchors/categories still overlap: station x0..1 and north wall x1.
    template.pieces = vec![
        TemplatePiece {
            kind: "wood_bench".into(),
            at: PieceAt::Anchor,
            count: None,
        },
        TemplatePiece {
            kind: "log_wall".into(),
            at: PieceAt::Edge(Direction::North),
            count: None,
        },
    ];
    let error = template
        .expand(&catalog, None, 0)
        .expect_err("wall overlaps structure");
    assert!(error.contains("overlapping physical footprint"), "{error}");
    // A hut's physical footprint also excludes furnishing, not just another structure.
    template.footprint = (3, 3);
    template.pieces = vec![
        TemplatePiece {
            kind: "hut".into(),
            at: PieceAt::Anchor,
            count: None,
        },
        TemplatePiece {
            kind: "bedroll".into(),
            at: PieceAt::Interior,
            count: Some(1),
        },
    ];
    assert!(template.expand(&catalog, None, 0).is_err());
}

#[test]
fn spine_floor_after_nonfloor_overlap_is_rejected_but_floor_first_is_valid() {
    let (catalog, templates) = family();
    let mut template = templates.get("workshop_shed").expect("shed").clone();
    template.stations.clear();
    let bench = TemplatePiece {
        kind: "wood_bench".into(),
        at: PieceAt::Anchor,
        count: None,
    };
    let floor = TemplatePiece {
        kind: "packed_floor".into(),
        at: PieceAt::Anchor,
        count: None,
    };
    template.pieces = vec![bench.clone(), floor.clone()];
    let error = template.expand(&catalog, None, 0).expect_err("late floor");
    assert!(error.contains("floor must precede"), "{error}");
    template.pieces = vec![floor, bench];
    assert!(template.expand(&catalog, None, 0).is_ok());
}

#[test]
fn spine_work_tile_excludes_nonfloor_pieces_in_both_authored_orders() {
    let (catalog, templates) = family();
    let mut template = templates.get("workshop_shed").expect("shed").clone();
    template.stations.clear();
    // Bench at(0,0), size2x1: actual work_tile_of is(1,1), outside its own body.
    let bench = TemplatePiece {
        kind: "wood_bench".into(),
        at: PieceAt::Anchor,
        count: None,
    };
    let bed = TemplatePiece {
        kind: "bedroll".into(),
        at: PieceAt::Interior,
        count: Some(1),
    };
    for pieces in [vec![bench.clone(), bed.clone()], vec![bed, bench.clone()]] {
        template.pieces = pieces;
        let error = template
            .expand(&catalog, None, 0)
            .expect_err("blocked work tile");
        assert!(error.contains("work tile"), "{error}");
    }
    // A floor beneath that same work tile is explicitly permitted, if it comes first.
    template.pieces = vec![
        TemplatePiece {
            kind: "packed_floor".into(),
            at: PieceAt::Interior,
            count: Some(1),
        },
        bench,
    ];
    assert!(template.expand(&catalog, None, 0).is_ok());
}

#[test]
fn spine_selected_station_reslots_around_prior_station_work_tile() {
    let (catalog, templates) = family();
    let mut template = templates.get("workshop_shed").expect("shed").clone();
    template.footprint = (6, 4);
    template.pieces = vec![TemplatePiece {
        kind: "wood_bench".into(),
        at: PieceAt::Anchor,
        count: None,
    }];
    template.stations = vec!["wood_bench".into()];
    // Candidate(1,1) overlaps only the first bench's south work tile, not its body.
    let plan = template
        .expand(&catalog, None, 0)
        .expect("station skips reserved work tile");
    assert_eq!(plan[0].offset, (0, 0));
    assert_eq!(plan[1].offset, (2, 1));
}

#[test]
fn spine_unknown_storage_selector_is_rejected_before_installation() {
    let (catalog, _) = family();
    let text = DATA.replacen("storage: Some(\"all\")", "storage: Some(\"grain_only\")", 1);
    let error = TemplateCatalog::parse(&text, None, &catalog).expect_err("unsupported storage");
    assert!(error.contains("unsupported storage selector"), "{error}");
}

#[test]
fn spine_zone_definitions_reserve_only_unwired_growth_drivers() {
    let (catalog, templates) = family();
    let zones = templates.zone_kinds();
    assert_eq!(
        zones.iter().map(|z| z.id.as_str()).collect::<Vec<_>>(),
        ["housing", "storage", "workshop"]
    );
    assert_eq!(zones[0].status, ContentStatus::Connected);
    assert!(zones[1..]
        .iter()
        .all(|zone| zone.status == ContentStatus::Inert
            && zone
                .debt
                .as_deref()
                .is_some_and(|debt| !debt.trim().is_empty())));
    let extension = r#"(schema_version:1, buildings:[], zone_kinds:[
        (id:"future_zone",name_key:"EXT_FUTURE_ZONE",status:Inert,debt:Some("Missing future driver; C2"))])"#;
    let merged =
        TemplateCatalog::parse(DATA, Some(extension), &catalog).expect("inert extension zone");
    assert_eq!(
        merged.zone_kinds().last().expect("appended zone").id,
        "future_zone"
    );
    let building_only = "(schema_version:1, buildings:[])";
    assert_eq!(
        TemplateCatalog::parse(DATA, Some(building_only), &catalog)
            .expect("old extension shape")
            .zone_kinds(),
        zones
    );
}

#[test]
fn spine_invalid_zone_rows_and_connected_zone_references_are_rejected() {
    let (catalog, _) = family();
    for (data, expected) in [
        (
            DATA.replacen("id: \"storage\"", "id: \" \"", 1),
            "invalid or duplicate identity",
        ),
        (
            DATA.replacen("name_key: \"ZONE_STORAGE\"", "name_key: \" \"", 1),
            "invalid or duplicate identity",
        ),
        (
            DATA.replacen("id: \"storage\"", "id: \"housing\"", 1),
            "duplicate identity",
        ),
        (
            DATA.replacen(
                "Missing automatic storage-zone growth planner; C2 content pass",
                " ",
                1,
            ),
            "inert zone needs debt",
        ),
        (
            DATA.replacen(
                "name_key: \"ZONE_STORAGE\", status: Inert",
                "name_key: \"ZONE_STORAGE\", status: Connected",
                1,
            ),
            "no connected zone driver",
        ),
        (
            DATA.replacen(
                "zone_kind: Some(\"housing\")",
                "zone_kind: Some(\"missing_zone\")",
                1,
            ),
            "unknown zone",
        ),
        (
            DATA.replacen(
                "zone_kind: Some(\"housing\")",
                "zone_kind: Some(\"storage\")",
                1,
            ),
            "requires connected zone",
        ),
    ] {
        let error = TemplateCatalog::parse(&data, None, &catalog).expect_err("invalid zone data");
        assert!(error.contains(expected), "{error}");
    }
    let duplicate = r#"(schema_version:1, buildings:[], zone_kinds:[
        (id:"housing",name_key:"EXT_HOUSING",status:Connected,debt:None)])"#;
    assert!(TemplateCatalog::parse(DATA, Some(duplicate), &catalog)
        .expect_err("duplicate across sources")
        .contains("duplicate identity"));
}

#[test]
fn c1_templates_keep_physical_costs_station_access_and_open_pen_capacity() {
    let (catalog, templates) = family();
    for id in ["granary", "pen", "smokehouse", "oven_house"] {
        let template = templates.get(id).expect("C1 template");
        assert_eq!(template.status, ContentStatus::Connected, "{id}");
        for rotation in 0..4 {
            let plan = template
                .expand(&catalog, None, rotation)
                .expect("physical plan");
            assert_eq!(
                material_cost(&plan, &catalog).expect("cost"),
                template.materials
            );
        }
    }
    let pen = templates.get("pen").expect("pen");
    assert_eq!(pen.function.livestock_capacity, 6);
    assert_eq!(pen.function.warmth, ShelterKind::Open);
    let plan = pen.expand(&catalog, None, 0).expect("pen plan");
    let fence = catalog.blueprint_by_name("fence").expect("fence");
    assert_eq!(plan.iter().filter(|piece| piece.kind == fence).count(), 15);
    let fence = catalog.blueprint(fence).expect("fence definition");
    assert_eq!(fence.footprint, (1, 1));
    assert_eq!(fence.planned_nav, crate::colony::content::PlannedNav::Door);
    assert!(fence.blocks_livestock);
    assert!(!fence.supports_roof);
    for (template, station, tag) in [
        ("smokehouse", "smoking_rack", "smoking"),
        ("oven_house", "clay_oven", "baking"),
    ] {
        let template = templates.get(template).expect("station house");
        let plan = template.expand(&catalog, None, 0).expect("station plan");
        let last = plan.last().expect("station");
        assert_eq!(
            last.kind,
            catalog.blueprint_by_name(station).expect("station id")
        );
        assert_eq!(
            catalog
                .blueprint(last.kind)
                .expect("station definition")
                .workshop_tags,
            [tag]
        );
        assert!(last.offset.1 + last.size.1 + 1 < template.footprint.1);
    }
}

#[test]
fn c1_granary_storage_lifetime_is_authored_and_does_not_protect_other_groups() {
    use crate::colony::content::FoodStorageClass;
    let (catalog, _) = family();
    let grain = catalog
        .item(catalog.item_by_name("wild_grain").expect("grain"))
        .expect("grain item")
        .food
        .as_ref()
        .expect("food");
    assert_eq!(
        grain.lifetime_days(FoodStorageClass::Granary),
        2 * grain.lifetime_days(FoodStorageClass::RoofedStore)
    );
    let mut non_grain = grain.clone();
    non_grain.groups = vec!["fruit".to_owned()];
    assert_eq!(
        non_grain.lifetime_days(FoodStorageClass::Granary),
        non_grain.lifetime_days(FoodStorageClass::RoofedStore)
    );
    let mut historical = grain.clone();
    historical
        .storage_spoil_days
        .as_mut()
        .expect("storage")
        .granary = None;
    assert_eq!(
        historical.lifetime_days(FoodStorageClass::Granary),
        historical.lifetime_days(FoodStorageClass::RoofedStore)
    );
}
