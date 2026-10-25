//! Exact external inventory and immutable source/captured-command provenance.
use std::path::Path;
pub struct Fixture {
    pub path: &'static str,
    pub sha256: &'static str,
    pub version: u32,
    pub tick: u64,
    pub seed: u64,
    pub enabled: bool,
}
pub const FIXTURES: &[Fixture] = &[
    Fixture {
        path: "historical/m55/tick-2000.save",
        sha256: "5043fca3c5262693ea6040c9746fe8ea249ff9b6db6676d121879ebbd19c7efa",
        version: 6,
        tick: 2000,
        seed: 5,
        enabled: false,
    },
    Fixture {
        path: "historical/m55/tick-3000.save",
        sha256: "94616c6ded2e86989300d8b15b75159cd925242eb123c83bfd77fdfa414607b3",
        version: 6,
        tick: 3000,
        seed: 5,
        enabled: false,
    },
    Fixture {
        path: "historical/mart2/tick-2000.save",
        sha256: "5043fca3c5262693ea6040c9746fe8ea249ff9b6db6676d121879ebbd19c7efa",
        version: 6,
        tick: 2000,
        seed: 5,
        enabled: false,
    },
    Fixture {
        path: "historical/mart2/tick-3000.save",
        sha256: "94616c6ded2e86989300d8b15b75159cd925242eb123c83bfd77fdfa414607b3",
        version: 6,
        tick: 3000,
        seed: 5,
        enabled: false,
    },
    Fixture {
        path: "base/on-tick-5000.save",
        sha256: "b24605f9f8086a06f6de42597b4eb84a8d8dd3d535dd43afd41832efdfd46397",
        version: 7,
        tick: 5000,
        seed: 5,
        enabled: true,
    },
    Fixture {
        path: "base/on-tick-6000.save",
        sha256: "362a377d37a759e42c3b6c9d0e20f0c49b9dae793ec1a3653406a8e256d8ee53",
        version: 7,
        tick: 6000,
        seed: 5,
        enabled: true,
    },
    Fixture {
        path: "core8/on-tick-5000.save",
        sha256: "fed3fe2616fb94001858493aedb523421d3e93273e96e39c9db971e82130dadb",
        version: 8,
        tick: 5000,
        seed: 5,
        enabled: true,
    },
    Fixture {
        path: "core8/on-tick-6000.save",
        sha256: "c6ada78c90ca7420791fabc2245033645f950771b6d693cfa2e0ab8b5878ce7f",
        version: 8,
        tick: 6000,
        seed: 5,
        enabled: true,
    },
    Fixture {
        path: "b4/h-format9-fixture/tick4000.save",
        sha256: "e315e1c952b7e0504fd7cbe3b46c867b8c16d0735787e27e11ddab65f6894141",
        version: 9,
        tick: 4000,
        seed: 0,
        enabled: false,
    },
];
/// Use the existing macOS host utility, not a new dependency or home-grown digest.
pub fn check_hash(path: &Path, expected: &str) {
    let output = std::process::Command::new("shasum")
        .args(["-a", "256"])
        .arg(path)
        .output()
        .expect("required host shasum");
    assert!(
        output.status.success(),
        "hash {}: {}",
        path.display(),
        String::from_utf8_lossy(&output.stderr)
    );
    let text = String::from_utf8(output.stdout).expect("digest UTF8");
    assert_eq!(
        text.split_whitespace().next(),
        Some(expected),
        "{} hash",
        path.display()
    );
}
pub fn verify_provenance(root: &Path) {
    // Pins are original acquisition receipts, source identifiers, and the concrete replay inputs.
    for (relative, sha256) in [
        (
            "historical/acquisition-receipt.json",
            "d75171a4d51a9867e37c5bb670412511b16e05c394dbf47ae55f2ec05640508e",
        ),
        (
            "historical/m55/source-commit.txt",
            "898b1e64ef4bf3de5c94d1552e4430c7b17e6a377b0bf3492d29add961780709",
        ),
        (
            "historical/mart2/source-commit.txt",
            "0ece54b3e9ffc01873d2bea9885e48a03c77aced9516417f730274e85e672b16",
        ),
        (
            "historical/m55/continuation-2000-3000.json",
            "4a44834c6720e5e02619c0db6e9e08bf74b2bd9ac131f2929203be8af47357f6",
        ),
        (
            "historical/mart2/continuation-2000-3000.json",
            "4a44834c6720e5e02619c0db6e9e08bf74b2bd9ac131f2929203be8af47357f6",
        ),
        (
            "base/provenance.json",
            "109c3dda2e1b2d5583d0fab115d4df31677716b3d29f2bed566abb4213c17db4",
        ),
        (
            "core8/acquisition-receipt.json",
            "dac021d09b22ad8792b721ef84c54e094216366b3240e3f09b4c293a4d2b7aae",
        ),
        (
            "b4/h-format9-fixture/receipt.json",
            "2fdae7bcf9481347afaeda967f49144febadff727258f3f6eba8e38d2cf15bbc",
        ),
    ] {
        check_hash(&root.join(relative), sha256);
    }
}
