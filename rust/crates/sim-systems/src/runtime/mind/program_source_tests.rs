//! Dynamic policy owns its bytes; bundled policy need not duplicate static data.
use super::MindProgram;

const SHIPPED: &str = include_str!("../../../../sim-core/data/colony/mind.ron");

#[test]
fn program_source_dynamic_load_owns_exact_bytes_beyond_callers_lifetime() {
    let expected = format!("{SHIPPED}\n// dynamic source lifetime regression\n");
    let program = {
        let mut temporary = expected.clone();
        let program = MindProgram::load(&temporary).expect("dynamic policy compiles");
        assert_ne!(program.source().as_ptr(), temporary.as_ptr());
        temporary.clear();
        program
    };
    assert_eq!(program.source().as_bytes(), expected.as_bytes());
    let reloaded = MindProgram::load(program.source()).expect("retained source recompiles");
    assert_eq!(program.policy.revision(), reloaded.policy.revision());
}

#[test]
fn program_source_static_load_borrows_same_bytes_with_identical_compilation() {
    let borrowed = MindProgram::load_static(SHIPPED).expect("static policy compiles");
    let owned = MindProgram::load(SHIPPED).expect("owned policy compiles");
    assert_eq!(borrowed.source().as_bytes(), SHIPPED.as_bytes());
    assert_eq!(borrowed.policy.revision(), owned.policy.revision());
    assert_eq!(borrowed.policy.nodes().len(), owned.policy.nodes().len());
    assert_eq!(
        borrowed.source().as_ptr(),
        SHIPPED.as_ptr(),
        "static source must not allocate a second policy text buffer"
    );
}

#[test]
fn saved_pre_livestock_registry_keeps_food_facts_and_excludes_new_fact() {
    use sim_core::colony::mind_records::MindHistory;
    use std::borrow::Cow;
    const SOURCE: &str = r#"(schema_version:1,bands:["critical","duty","need","work","leisure","idle"],nodes:[
      (id:"idle",band:"idle",behaviour:"wander",when:[],
       score:(constant:0,terms:[(fact:"job_food_transport",weight_permille:1),
         (fact:"harvest_window",weight_permille:1),(fact:"harvest_storage_available",weight_permille:1),
         (fact:"winter_stock_duty",weight_permille:1),(fact:"water_target_available",weight_permille:1)]),hysteresis:0,life_stages:["adult","child","elder"],tags:[],name_key:"IDLE")])"#;
    let old =
        MindProgram::compile_schema(Cow::Borrowed(SOURCE), super::PRE_LIVESTOCK_EXCLUDED_FACTS)
            .expect("published20 registration set");
    let history = MindHistory::new(&old.policy).expect("exact old dictionary");
    let restored = MindProgram::load_saved(SOURCE, &history).expect("authenticated old registry");
    assert_eq!(restored.policy.revision(), old.policy.revision());
    for fact in [
        "job_food_transport",
        "harvest_window",
        "harvest_storage_available",
        "winter_stock_duty",
        "water_target_available",
    ] {
        assert!(
            restored.policy.fact_id(fact).is_some(),
            "old food fact retained: {fact}"
        );
    }
    assert!(restored.policy.fact_id("job_livestock").is_none());
    assert!(
        MindProgram::load_saved(&format!("{SOURCE}\n"), &history).is_err(),
        "source bytes remain authenticated"
    );
    let changed = SOURCE.replace("job_food_transport", "job_livestock");
    assert!(
        MindProgram::load_saved(&changed, &history).is_err(),
        "new fact cannot enter old dictionary"
    );
}

#[test]
fn saved_format13_registry_excludes_all_later_registrations() {
    use sim_core::colony::mind_records::MindHistory;
    use std::borrow::Cow;
    const SOURCE: &str = r#"(schema_version:1,bands:["critical","duty","need","work","leisure","idle"],nodes:[
      (id:"idle",band:"idle",behaviour:"wander",when:[],score:(constant:0,terms:[(fact:"hunger",weight_permille:1)]),
       hysteresis:0,life_stages:["adult","child","elder"],tags:[],name_key:"IDLE")])"#;
    let old = MindProgram::compile_schema(Cow::Borrowed(SOURCE), super::FORMAT13_EXCLUDED_FACTS)
        .expect("published13 registration set");
    let history = MindHistory::new(&old.policy).expect("exact original dictionary");
    let restored = MindProgram::load_saved(SOURCE, &history).expect("original schema restored");
    assert_eq!(restored.policy.revision(), old.policy.revision());
    for fact in [
        "job_food_transport",
        "harvest_window",
        "harvest_storage_available",
        "winter_stock_duty",
        "water_target_available",
        "job_livestock",
    ] {
        assert!(
            restored.policy.fact_id(fact).is_none(),
            "later fact excluded: {fact}"
        );
    }
}
