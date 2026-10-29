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
