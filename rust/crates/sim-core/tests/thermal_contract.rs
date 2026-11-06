//! Colony thermal authoring remains independent of historical catalog/save identities.
use sim_core::colony::thermal::ThermalRules;
#[test]
fn bundled_thermal_rules_resolve_real_material_ids() {
    let rules = ThermalRules::bundled().expect("valid authored thermal rules");
    let oak = sim_core::material::MaterialId::from_str_hash("oak");
    assert_eq!(rules.material_insulation_permille.get(&oak), Some(&900));
    assert!(rules.validate().is_ok());
}

#[test]
fn thermal_profile_rejects_unknown_duplicate_and_out_of_range_materials() {
    let catalog = sim_core::colony::content::installed_or_bundled();
    let materials = sim_core::colony::materials::embedded_definitions().expect("registry");
    for source in [
        ThermalRules::SOURCE.replace("(\"oak\",900)", "(\"missing\",900)"),
        ThermalRules::SOURCE.replace("(\"oak\",900)", "(\"oak\",900),(\"oak\",800)"),
        ThermalRules::SOURCE.replace("(\"oak\",900)", "(\"oak\",1000)"),
        ThermalRules::SOURCE.replace("capacity_per_tile: 750", "capacity_per_tile: 0"),
    ] {
        assert!(ThermalRules::from_source(&source, catalog, &materials).is_err());
    }
}

#[test]
fn alternate_profile_resolves_without_mutating_historical_catalog_identity() {
    let catalog = sim_core::colony::content::installed_or_bundled();
    let before = catalog.revision().to_owned();
    let materials = sim_core::colony::materials::embedded_definitions().expect("registry");
    let source = ThermalRules::SOURCE.replace("(\"oak\",900)", "(\"oak\",700)");
    let rules =
        ThermalRules::from_source(&source, catalog, &materials).expect("selected thermal profile");
    assert_eq!(
        rules.material_insulation_permille[&sim_core::material::MaterialId::from_str_hash("oak")],
        700
    );
    assert_eq!(catalog.revision(), before);
}

#[test]
fn selected_thermal_boot_validates_before_install_and_is_sticky() {
    use sim_core::colony::content::{self, BootSelection, NamedSource, RawSources};
    if let Ok(mode) = std::env::var("THERMAL_BOOT_TEST_CHILD") {
        let value = if mode == "invalid" { "1000" } else { "700" };
        let source = ThermalRules::SOURCE.replace("(\"oak\",900)", &format!("(\"oak\",{value})"));
        let selection = BootSelection::Explicit {
            request_id: "thermal-test-v1".into(),
            sources: RawSources {
                sources: vec![NamedSource {
                    name: "thermal".into(),
                    bytes: source.into_bytes(),
                }],
            },
        };
        let result = content::bootstrap(selection);
        if mode == "invalid" {
            assert!(result.is_err());
            assert!(content::bootstrap(BootSelection::Bundled).is_err());
            assert!(content::installed().is_err());
        } else {
            let catalog = result.expect("selected boot");
            let id = sim_core::material::MaterialId::from_str_hash("oak");
            assert_eq!(
                catalog.thermal_rules().material_insulation_permille[&id],
                700
            );
            assert_eq!(
                ThermalRules::bundled().expect("installed getter"),
                *catalog.thermal_rules()
            );
            assert!(content::bootstrap(BootSelection::Bundled).is_err());
        }
        return;
    }
    for mode in ["selected", "invalid"] {
        let status = std::process::Command::new(std::env::current_exe().expect("test executable"))
            .args([
                "--exact",
                "selected_thermal_boot_validates_before_install_and_is_sticky",
                "--nocapture",
            ])
            .env("THERMAL_BOOT_TEST_CHILD", mode)
            .status()
            .expect("isolated boot");
        assert!(status.success(), "boot mode {mode}");
    }
}
