# T2: Material Schema — Types + Derivation + Registry + Loader

## 1. Header

- **Feature**: `material_schema`
- **Phase**: V7 Week 1.1, ticket T2 (covers original T2~T5 of the Phase 1 plan)
- **Branch**: `lead/main` — base HEAD = `1588df25` (post T1 STRUCTURAL-COMMIT `77764531`)
- **Crate**: `sim-core` only (zero touches to other crates)
- **Module root**: `rust/crates/sim-core/src/material/`
- **Existing scaffolding** (T1, do not re-create or modify):
  - `rust/crates/sim-core/src/lib.rs` already exports `MATERIAL_SCHEMA_VERSION: u32 = 1` and `pub mod material;`
  - `rust/crates/sim-core/src/material/mod.rs` is an empty barrel with Phase 0 reference doc-comments
  - Cargo deps already wired: `serde`, `ron`, `glam`, `ahash`, `thiserror` (workspace inherited)
- **Reference docs (project knowledge)** — read these first, do not paraphrase:
  - `MATERIAL_PHASE0_DESIGN.md` v0.1 Section 2 (intent, hybrid rationale)
  - `MATERIAL_PHASE0_v0.2_PARTIAL.md` Sections 1 + 6 (DF + RimWorld justification)
  - `MATERIAL_PHASE0_v0.3.md` Sections 3 + 4 + 5 + 7 — **Section 4 is the binding spec**
  - `PHASE1_MATERIAL_INTEGRATION_COMMAND.md` Section 2 — **integration spec, locks lib.rs re-exports + TerrainType ordering**
- **External primary sources** (must be cited in derivation/explanation source strings):
  - DF wiki `Material_science` (absolute physical values, formula provenance)
  - RimWorld wiki `Stuff` / `StuffProperties` (factor multiplier pattern)
  - CRC Handbook of Chemistry and Physics, 102nd ed. (density / melting sanity)
  - Mohs scale 1812 (hardness)
  - Callister, *Materials Science and Engineering* (yield, fracture toughness)

## 2. Implementation Order — Strict Sequential, Single File per Step

> **Mandatory**: complete and verify each step before starting the next. Do not pre-write code from a later step. If any step fails verification, stop and report — do not proceed.

### Step 2.1 — Foundation types (~85 LOC, 4 files)
- `material/id.rs` — see **§3.A MaterialId lock**
- `material/category.rs` — see **§3.B MaterialCategory lock**
- `material/terrain.rs` — see **§3.C TerrainType lock**
- `material/error.rs` — see **§3.D MaterialError lock**

**Step 2.1 verification (mandatory before Step 2.2):**
```
cd rust && cargo build -p sim-core
cd rust && cargo test -p sim-core
cd rust && cargo clippy -p sim-core --all-targets -- -D warnings
```
No internal coupling between the 4 files allowed at this stage.

### Step 2.2 — Properties + Definition (~150 LOC, 2 files)
- `material/properties.rs` — see **§3.E MaterialProperties lock**, includes `validate()`
- `material/definition.rs` — see **§3.F MaterialDef lock**

**Step 2.2 verification (mandatory):** unit tests in `properties.rs` covering boundary cases — for each of the 14 numeric fields one min-edge OK, one max-edge OK, one OOR `PropertyOutOfRange`. `cargo test` + clippy clean.

### Step 2.3 — Derivation (~280 LOC, 1 file)
- `material/derivation.rs` — see **§3.G AutoDerivedStats lock + §3.H DerivedStatKind lock**
- 25 distinct formulas per Phase 0 v0.3 Section 3.2 (verbatim — do not "improve")
- **Iron baseline constants** locked at file top: see **§3.I**
- All formulas must produce finite `f64` for valid input properties — verify via unit tests on granite, oak, obsidian fixtures.

**Step 2.3 verification (mandatory):** unit tests asserting `derive_all` outputs are all finite for the 3 fixtures, and that iron-baseline-driven outputs (axe_damage_cut etc. for an iron-equivalent property fixture) match the published constants within 1e-6 tolerance.

### Step 2.4 — Explanation (~80 LOC, 1 file)
- `material/explanation.rs` — see **§3.J PropertyKind lock + §3.K Explanation lock**
- 23 explain match arms (one per `DerivedStatKind`); some reference 1 input `PropertyKind`, some reference 2 (per Section 3.2 dependencies)
- **Source attribution rule (binding):** every `source` string MUST contain at least one of the substrings `"DF"`, `"wiki"`, `"RimWorld"`, or `"CRC"`. The harness will assert this.

**Step 2.4 verification (mandatory):** unit test confirming `explain(DerivedStatKind::AxeDamageCut)` returns exactly 2 input `PropertyKind` entries (ShearYield + Hardness) and a non-empty source containing one of the four required substrings; same shape check for all 23 variants in a loop.

### Step 2.5 — Registry (~180 LOC, 1 file)
- `material/registry.rs` — see **§3.L MaterialRegistry lock**
- Public API names and signatures fixed per §3.L; do not rename
- All lookups must be O(1) average (AHashMap). No quadratic loops anywhere in this file.

**Step 2.5 verification (mandatory):** unit tests — register 100 synthetic defs, assert `count() == 100`; assert `derive(id)` second call hits cache (verify by checking `derive_cache.contains_key` after first call); assert `unload_mod("test_mod")` removes only that mod's defs and returns the right count.

### Step 2.6 — Loader + barrel + lib.rs re-exports (~150 LOC, 2 files + 1 edit)
- `material/loader.rs` — see **§3.M MaterialFile / MaterialDefRaw / loader lock**
- `material/mod.rs` — see **§3.N module barrel lock**
- `lib.rs` — see **§3.O lib.rs 11 re-exports lock** (exact names, exact order)

**Step 2.6 verification (mandatory):**
- Unit test asserting `load_ron` on a synthetic file with `schema_version: 999` returns `Err(MaterialError::SchemaMismatch { .. })`
- Unit test asserting `load_ron` on a synthetic valid file returns the expected `Vec<MaterialDef>` length and ID hashes
- `cargo build -p sim-core`, `cargo test --workspace`, `cargo clippy --workspace --all-targets -- -D warnings` all clean

---

## 3. LOCK된 명세 (Generator는 결정 X, 명세를 그대로 옮긴다)

> **Axiom anchor**: "Generator의 자유도 = 새 결정 권한이 아니라 명세의 충실한 옮김". v6 cognition.rs 2342 LOC monolith는 이 axiom 위반의 결과. V7 Hard Gate 5 (Solo Dev Mental Model 단순화)도 같은 정신. 아래 §3.A~§3.O는 **모두 lock**. 변형 X, 추가 X, 제거 X, 순서 변경 X, "개선" 제안 X.

### §3.A MaterialId (in `material/id.rs`)

```rust
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(transparent)]
pub struct MaterialId(u32);

impl MaterialId {
    /// Const djb2 hash of the input string. Identical bytes → identical id.
    pub const fn from_str_hash(s: &str) -> Self {
        let mut hash: u32 = 5381;
        let bytes = s.as_bytes();
        let mut i = 0;
        while i < bytes.len() {
            hash = hash.wrapping_mul(33).wrapping_add(bytes[i] as u32);
            i += 1;
        }
        Self(hash)
    }

    pub const fn raw(self) -> u32 { self.0 }
}

impl std::fmt::Display for MaterialId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "MaterialId({:#010x})", self.0)
    }
}
```

### §3.B MaterialCategory (in `material/category.rs`) — exactly 6 variants in this order

```rust
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum MaterialCategory {
    Stone,
    Wood,
    Animal,
    Mineral,
    Plant,
    Mod(u8),
}
```
`Mod(u8)` carries an inner mod-defined category id (0..=255).

### §3.C TerrainType (in `material/terrain.rs`) — exactly 10 variants in this order

```rust
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum TerrainType {
    Plain,
    Forest,
    Mountain,
    Hill,
    River,
    Coast,
    Desert,
    Tundra,
    Swamp,
    Cave,
}
```
No `Wetland`, no `Glacier`, no extras. Source: PHASE1_MATERIAL_INTEGRATION_COMMAND.md Section 2.

### §3.D MaterialError (in `material/error.rs`) — exactly 6 variants in this order

```rust
#[derive(Debug, thiserror::Error)]
pub enum MaterialError {
    #[error("RON parse error: {0}")]
    ParseError(String),

    #[error("io error: {0}")]
    Io(#[from] std::io::Error),

    #[error("schema mismatch: file declares version {file_version}, supported = {supported}")]
    SchemaMismatch { file_version: u32, supported: u32 },

    #[error("property `{property}` value {value} out of range; expected {expected}")]
    PropertyOutOfRange { property: &'static str, value: f64, expected: &'static str },

    #[error("duplicate material id: {0}")]
    DuplicateId(MaterialId),

    #[error("unknown terrain type: {0}")]
    UnknownTerrainType(String),
}
```

### §3.E MaterialProperties (in `material/properties.rs`) — exactly 15 fields (14 numeric + 1 distribution)

> **Cardinality nuance**: 14 `f64` + 1 `Vec<TerrainType>` (the `distribution` field). Earlier draft "15 f64" was incorrect; final spec is **14 f64 fields + 1 Vec field**. Total 15 named fields.

Field list, exact order, exact name, exact type, exact range:

| # | Name | Type | Unit | Range |
|---|------|------|------|-------|
| 1 | `density` | `f64` | kg/m³ | `100..=25000` |
| 2 | `hardness` | `f64` | Mohs | `1.0..=10.0` |
| 3 | `shear_yield` | `f64` | kPa | `1000..=600_000` |
| 4 | `impact_yield` | `f64` | kPa | `1000..=1_500_000` |
| 5 | `fracture_toughness` | `f64` | kPa | `1000..=800_000` |
| 6 | `melting_point` | `f64` | °C | `0.0..=3500.0` |
| 7 | `flammability` | `f64` | unitless | `0.0..=1.0` |
| 8 | `thermal_conductivity` | `f64` | W/m·K | `0.04..=400.0` |
| 9 | `cultural_value` | `f64` | unitless | `0.0..=1.0` |
| 10 | `rarity` | `f64` | unitless | `0.0..=1.0` |
| 11 | `distribution` | `Vec<TerrainType>` | — | `#[serde(default)]`, may be empty |
| 12 | `work_difficulty` | `f64` | unitless | `0.0..=1.0` |
| 13 | `aesthetic_value` | `f64` | unitless | `0.0..=1.0` |
| 14 | `workability` | `f64` | unitless | `0.0..=1.0` |
| 15 | `preservation` | `f64` | unitless | `0.0..=1.0` |

Derives: `Debug, Clone, Serialize, Deserialize` (no `Copy` — `Vec` makes it non-`Copy`).

`validate()` signature:
```rust
pub fn validate(&self) -> Result<(), MaterialError>;
```
Emits `MaterialError::PropertyOutOfRange { property, value, expected }` for any of the 14 numeric fields outside its inclusive range. `distribution` has no range check (empty allowed).

### §3.F MaterialDef (in `material/definition.rs`) — exactly 7 fields, order mandatory

User confirmed (post-review): MaterialDef = exactly 7 fields. Earlier "(8개)" header
was a typo. v0.3 Section 4 + Phase 1 integration command both verified: 7 fields.

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MaterialDef {
    pub id: MaterialId,
    pub name: String,
    pub category: MaterialCategory,
    pub properties: MaterialProperties,
    pub tier: u8,
    #[serde(default)]
    pub natural_in: Vec<TerrainType>,
    #[serde(default)]
    pub mod_source: Option<String>,
}
```
Add/remove/reorder X. Derive change X. No `Copy` (contains `String`).

### §3.G AutoDerivedStats (in `material/derivation.rs`) — exactly 23 fields in this order

Weapons (8):
1. `axe_damage_blunt: f64`
2. `axe_damage_cut: f64`
3. `axe_durability: f64`
4. `axe_speed: f64`
5. `sword_damage_cut: f64`
6. `sword_durability: f64`
7. `spear_damage_pierce: f64`
8. `dagger_damage_cut: f64`

Armor (3):
9. `armor_blunt: f64`
10. `armor_sharp: f64`
11. `armor_heat: f64`

Building (4):
12. `wall_strength: f64`
13. `wall_insulation: f64`
14. `wall_aesthetic: f64`
15. `floor_aesthetic: f64`

Influence blocking (3):
16. `blocking_warmth: f64`
17. `blocking_light: f64`
18. `blocking_noise: f64`

Crafting (2):
19. `craft_time_factor: f64`
20. `craft_quality_factor: f64`

RimWorld factor (3):
21. `sharp_damage_factor: f64`
22. `blunt_damage_factor: f64`
23. `max_hit_points_factor: f64`

Derives: `Debug, Clone, Copy, Serialize`.

### §3.H DerivedStatKind (in `material/derivation.rs`) — exactly 23 variants, 1:1 with §3.G

PascalCase mapping of §3.G field names:
```rust
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum DerivedStatKind {
    AxeDamageBlunt, AxeDamageCut, AxeDurability, AxeSpeed,
    SwordDamageCut, SwordDurability, SpearDamagePierce, DaggerDamageCut,
    ArmorBlunt, ArmorSharp, ArmorHeat,
    WallStrength, WallInsulation, WallAesthetic, FloorAesthetic,
    BlockingWarmth, BlockingLight, BlockingNoise,
    CraftTimeFactor, CraftQualityFactor,
    SharpDamageFactor, BluntDamageFactor, MaxHitPointsFactor,
}

impl DerivedStatKind {
    pub const fn all_variants() -> &'static [DerivedStatKind] {
        use DerivedStatKind::*;
        &[
            AxeDamageBlunt, AxeDamageCut, AxeDurability, AxeSpeed,
            SwordDamageCut, SwordDurability, SpearDamagePierce, DaggerDamageCut,
            ArmorBlunt, ArmorSharp, ArmorHeat,
            WallStrength, WallInsulation, WallAesthetic, FloorAesthetic,
            BlockingWarmth, BlockingLight, BlockingNoise,
            CraftTimeFactor, CraftQualityFactor,
            SharpDamageFactor, BluntDamageFactor, MaxHitPointsFactor,
        ]
    }
}
```

### §3.I Iron baseline constants (at top of `material/derivation.rs`, lock)

```rust
/// Iron axe cutting damage baseline (RimWorld factor normalisation reference).
pub const IRON_AXE_CUT: f64 = 11.6;
/// Iron axe blunt damage baseline.
pub const IRON_AXE_BLUNT: f64 = 62.96;
/// Iron axe durability baseline (HP per blade lifecycle).
pub const IRON_AXE_DURABILITY: f64 = 6745.0;
```
These constants live at the top of `derivation.rs` and are referenced inside `derive_all` for the RW-factor formulas. Generator does **not** decide their location.

### §3.J PropertyKind (in `material/explanation.rs`) — exactly 15 variants, 1:1 with §3.E

PascalCase mapping of §3.E field names:
```rust
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum PropertyKind {
    Density, Hardness, ShearYield, ImpactYield, FractureToughness,
    MeltingPoint, Flammability, ThermalConductivity, CulturalValue, Rarity,
    Distribution, WorkDifficulty, AestheticValue, Workability, Preservation,
}
```

### §3.K Explanation struct (in `material/explanation.rs`)

```rust
#[derive(Debug, Clone)]
pub struct Explanation {
    pub stat: DerivedStatKind,
    pub formula: &'static str,
    pub inputs: Vec<PropertyKind>,
    pub source: &'static str,
}

pub fn explain(stat: DerivedStatKind) -> Explanation;
```
`source` MUST contain `"DF"`, `"wiki"`, `"RimWorld"`, or `"CRC"`.

### §3.L MaterialRegistry (in `material/registry.rs`)

```rust
pub struct MaterialRegistry {
    defs: ahash::AHashMap<MaterialId, MaterialDef>,
    by_mod: ahash::AHashMap<String, Vec<MaterialId>>,
    derive_cache: ahash::AHashMap<MaterialId, AutoDerivedStats>,
    explain_cache: ahash::AHashMap<DerivedStatKind, Explanation>,
}

impl MaterialRegistry {
    pub fn new() -> Self;
    pub fn register(&mut self, def: MaterialDef, mod_id: Option<&str>) -> Result<(), MaterialError>;
    pub fn get(&self, id: MaterialId) -> Option<&MaterialDef>;
    pub fn derive(&mut self, id: MaterialId) -> Option<&AutoDerivedStats>;
    pub fn explain(&mut self, stat: DerivedStatKind) -> &Explanation;
    pub fn warm_cache(&mut self);
    pub fn unload_mod(&mut self, mod_id: &str) -> usize;
    pub fn all_ids(&self) -> impl Iterator<Item = MaterialId> + '_;
    pub fn count(&self) -> usize;
}
```
Names, signatures, return types: lock. O(1) average lookups.

### §3.M Loader (in `material/loader.rs`)

```rust
pub const CURRENT_SCHEMA_VERSION: u32 = 1;
// Must equal crate::MATERIAL_SCHEMA_VERSION; assert via const_assert or test.

#[derive(Debug, Deserialize)]
pub struct MaterialFile {
    pub schema_version: u32,
    pub materials: Vec<MaterialDefRaw>,
}

#[derive(Debug, Deserialize)]
pub struct MaterialDefRaw {
    pub id: String,
    pub name: String,
    pub category: MaterialCategory,
    pub properties: MaterialProperties,
    pub tier: u8,
    #[serde(default)]
    pub natural_in: Vec<TerrainType>,
    #[serde(default)]
    pub mod_source: Option<String>,
}

pub fn load_ron(path: &std::path::Path) -> Result<Vec<MaterialDef>, MaterialError>;
pub fn load_directory(path: &std::path::Path) -> Result<Vec<MaterialDef>, MaterialError>;
```
`MaterialDefRaw → MaterialDef` conversion: `id: String` becomes `MaterialId::from_str_hash(&raw.id)`. `load_ron` validates `schema_version == CURRENT_SCHEMA_VERSION` (else `SchemaMismatch`) and calls `MaterialProperties::validate()` for each. `load_directory` globs `*.ron` non-recursively.

### §3.N module barrel (in `material/mod.rs`)

```rust
pub mod id;
pub mod category;
pub mod terrain;
pub mod error;
pub mod properties;
pub mod definition;
pub mod derivation;
pub mod explanation;
pub mod registry;
pub mod loader;

pub use id::MaterialId;
pub use category::MaterialCategory;
pub use terrain::TerrainType;
pub use error::MaterialError;
pub use properties::MaterialProperties;
pub use definition::MaterialDef;
pub use derivation::{AutoDerivedStats, DerivedStatKind, IRON_AXE_BLUNT, IRON_AXE_CUT, IRON_AXE_DURABILITY};
pub use explanation::{Explanation, PropertyKind, explain};
pub use registry::MaterialRegistry;
pub use loader::{CURRENT_SCHEMA_VERSION, MaterialDefRaw, MaterialFile, load_directory, load_ron};
```

### §3.O lib.rs 11 top-level re-exports — exact 11 names, exact order

In `rust/crates/sim-core/src/lib.rs`, immediately after the existing `pub mod material;` line, add:
```rust
pub use material::{
    MaterialId, MaterialCategory, MaterialDef, MaterialProperties,
    MaterialRegistry, AutoDerivedStats, DerivedStatKind, MaterialError,
    Explanation, PropertyKind, TerrainType,
};
```
**Exactly 11 names**. Order locked. No additions (no `MaterialFile`, no `load_ron`, no `IRON_*`). No removals. No re-orderings.

---

## 4. Per-Step Verification — Generator's Own Duty

After **every** step, before starting the next:
```
cd rust && cargo build -p sim-core
cd rust && cargo test -p sim-core
cd rust && cargo clippy -p sim-core --all-targets -- -D warnings
```
Failures block progression. Do not bundle multiple steps' implementations into a single batch and verify only at the end — that violates Memory #2 (monolithic-prompt failure pattern).

## 5. Harness Pass Requirements (Evaluator gates)

- `cargo test --workspace` — zero regressions vs. baseline (T1 baseline: 2 passing tests)
- `cargo clippy --workspace --all-targets -- -D warnings` — clean
- All 6 sub-steps completed and committed in implementation order
- Phase 0 v0.3 Section 4 + Phase 1 integration Section 2 spec compliance — 100%
- **Cardinality contracts** (all must hold):
  - `MaterialProperties` named fields = 15 (14 `f64` + 1 `Vec<TerrainType>`)
  - `MaterialCategory` variants = 6
  - `TerrainType` variants = 10 (exact list per §3.C)
  - `MaterialError` variants = 6 (exact list per §3.D)
  - `AutoDerivedStats` fields = 23
  - `DerivedStatKind` variants = 23 (1:1 with `AutoDerivedStats`)
  - `PropertyKind` variants = 15 (1:1 with `MaterialProperties`)
  - `MaterialDef` fields = 7 (per §3.F lock)
  - **Other cardinality strings forbidden in spec compliance**: appearance of "(8개)", "(9개)", "(6개)" etc. for MaterialDef anywhere in code or comments → prompt-spec mismatch, BLOCK
  - lib.rs `pub use material::{...}` = exactly 11 names per §3.O
- **Order contracts** (all must hold):
  - `TerrainType` variant order = §3.C order
  - `MaterialCategory` variant order = §3.B order
  - `AutoDerivedStats` field order = §3.G order
  - `DerivedStatKind` variant order = §3.H order
  - lib.rs 11 re-exports order = §3.O order
- **Mechanical verification commands** (Evaluator may run):
  - lib.rs re-export count: `grep -cE '^\s+[A-Z][A-Za-z]+,?$' rust/crates/sim-core/src/lib.rs` → must read `11` (within the `pub use material::{...}` block)
  - TerrainType variants: `grep -cE '^\s+(Plain|Forest|Mountain|Hill|River|Coast|Desert|Tundra|Swamp|Cave),' rust/crates/sim-core/src/material/terrain.rs` → must read `10`
  - Iron baselines present: `grep -E 'IRON_AXE_(CUT|BLUNT|DURABILITY).*=.*(11\.6|62\.96|6745\.0)' rust/crates/sim-core/src/material/derivation.rs` → must produce 3 lines
  - djb2 implementation: `grep -E 'wrapping_mul\(33\)\.wrapping_add' rust/crates/sim-core/src/material/id.rs` → must match
  - MaterialDef field count: `grep -c '^    pub ' rust/crates/sim-core/src/material/definition.rs` → must read `7`
- **Axiom #1 (변태적 디테일):** every `Explanation::source` string non-empty and contains at least one of `"DF"` / `"wiki"` / `"RimWorld"` / `"CRC"` — verified by a unit test iterating `DerivedStatKind::all_variants()`.
- **Axiom #2 (수학적 정밀):** every public lookup in `MaterialRegistry` is O(1) average; no quadratic / nested-iteration loops in this ticket's code (verified by reviewer reading registry.rs).

## 6. NOT in scope — explicit exclusion

The following are deferred to later tickets and must NOT be touched here:
- 100 RON material data files → T6~T8
- Harness 8 tests + criterion benchmarks → T9 / T10
- Localization 138 keys → T11
- Material inspector UI → V7 Week 1.4 (separate work)
- SimBridge / FFI exposure → not yet planned for this phase
- Any change to `sim-systems`, `sim-engine`, `sim-bridge`, `sim-data`, `sim-test` crates

## 7. Memory #2 Avoidance + Axiom anchor (binding process rule)

**핵심 axiom**: "Generator의 자유도 = 새 결정 권한이 아니라 명세의 충실한 옮김."

이건 axiom #1 (변태적 디테일)의 직접 적용이다. v6 archive `cognition.rs` 2342 LOC monolith가 정확히 이 axiom 위반의 결과였다 — Generator가 명세 외 결정 자유를 행사하면 결과물이 발산한다. V7 Hard Gate 5 (Solo Dev Mental Model 단순화)도 같은 정신.

따라서:
- §3.A~§3.O는 **모두 lock**. Generator는 명세된 cardinality / 이름 / 타입 / 순서 / 상수값을 그대로 옮긴다. 명세 외 추가 / 변형 / "개선" 제안은 axiom #1 위반.
- 의문이 있으면 Generator는 즉시 보고하고 구현 중단. 자유 결정 X.
- 한 step에서 단일 파일만 작성. 한 step 끝나기 전에 다음 step 시작 X.
- "all 6 steps in single shot" 시도 X.
- 한 step이 예산을 초과하면 (예: derivation.rs > 280 LOC), 즉시 scope drift 보고 + 사용자 결정 대기.
- 총 목표: ~945 LOC across 10 files. Step 2.5 종료 시점에 누적 > 800 LOC면 일시 중지 + 보고.

### 7.1 Lock Violation 절대 금지 — 4가지 합리화 패턴 차단

§3.A~§3.O 의 lock된 cardinality / 이름 / 타입 / 순서 / 상수값은 **literal로 옮겨라**. Plan, Evaluator review, Generator 자체 판단 중 어느 것이든 prompt lock과 충돌하면 **prompt가 이긴다**. Plan 또는 Evaluator가 prompt와 다른 값을 요구하면 즉시 STOP + 사용자 보고 — 자체 판단으로 따르지 마라.

다음 4가지 합리화는 **금지**되며, 등장하는 즉시 axiom #1 위반으로 STOP:

1. **"more flexible / 더 유연하게"** — `&'static str`을 `String`으로 바꾸는 것 등. lock된 타입은 lock된 이유가 있다 (zero-alloc, compile-time guarantee). "유연성"은 명세 외 결정.
2. **"reasonable for runtime / 런타임에는 이게 맞다"** — runtime 편의 명목으로 lock 변경. lock은 정확히 runtime/non-runtime 모두 고려된 결과다.
3. **"future-proof / 나중에 확장하기 좋게"** — YAGNI 위반 + axiom #1 위반. 미래 확장은 미래 ticket의 결정.
4. **"more idiomatic / 더 Rust스럽게"** — idiom 판단은 명세 단계에서 끝남. Generator는 옮긴다, 평가하지 않는다.

이 4가지 표현이 Generator 출력, Evaluator review, 또는 자체 사고 중에 등장하면 **즉시 STOP**. lock된 명세를 literal로 옮기고, 사용자에게 보고.

T2-attempt-2 실패 사례: §3.D `PropertyOutOfRange { property: &'static str, expected: &'static str }` lock을 Evaluator가 plan 기반으로 `String, String` 변경 요구 → Generator가 "runtime용으로 reasonable"이라며 수용 → axiom #1 위반 + 환경 비용 +8 LLM call 낭비. 이게 정확히 본 §7.1 차단 대상.

### 7.2 Out-of-Scope 항목 절대 금지 — §6 명시 항목 재확인

§6 NOT in scope에 명시된 항목은 **만들지 마라**. 다음은 본 ticket에서 절대 생성 / 수정 금지:

- **`crates/sim-test/`** — 별도 crate 생성 금지. 본 ticket은 sim-core 단일 crate.
- **`workspace.members` 추가** — `rust/Cargo.toml`의 `members` 배열에 새 entry 추가 금지.
- **`material/harness.rs` 또는 모든 out-of-§3.N 모듈** — §3.N module barrel은 정확히 10개. 새 module file을 material/ 아래에 만들지 마라. test는 각 module의 `#[cfg(test)] mod tests`에만 둔다 (umbrella harness module 금지).
- **새 의존성** — Cargo.toml `[workspace.dependencies]` 변경 금지. 본 ticket의 deps는 T1에서 이미 lock됨.
- **`sim-core/src/lib.rs` re-export 11개 외 추가** — §3.O lock.

Evaluator가 "harness 테스트 binary가 필요"라거나 "sim-test crate에 통합 테스트"를 요구해도 → STOP. 그건 §6 + axiom #1 위반이다. T2 ticket의 test는 sim-core 내부 `#[cfg(test)] mod tests` 단위 테스트로 한정. Harness binary / criterion benchmarks / 통합 테스트는 **T9-T10 (C4)** 에서 별도 ticket으로 다룬다.

T2-attempt-2 실패 사례: Evaluator가 "must run `cargo test -p sim-test harness_*`"를 mandatory로 demand → Generator가 sim-test crate + workspace.members 추가 + 12개 harness 함수 작성 → §6 + axiom #1 위반. 본 §7.2 차단 대상.

## 8. Reference materials (must be available to the Generator)

1. `MATERIAL_PHASE0_v0.3.md` Section 4 — struct definitions (PRIMARY BINDING SPEC)
2. `MATERIAL_PHASE0_v0.3.md` Section 3.2 — 25 derive formulas
3. `MATERIAL_PHASE0_v0.2_PARTIAL.md` Section 6 — DF + RimWorld hybrid justification
4. `WORLDSIM_V7_MASTER_DIRECTION.md` Section 6 — 6 V7 Hard Gates
5. `PHASE1_MATERIAL_INTEGRATION_COMMAND.md` Section 2 — locks lib.rs re-exports + TerrainType ordering
6. DF wiki `Material_science` — formula source citation
7. RimWorld wiki `Stuff` / `StuffProperties` — factor pattern source
8. CRC Handbook of Chemistry and Physics 102nd ed. — density / melting sanity values
9. Auto-memory `MEMORY.md` entry on V7 Hard Gates (8 gates) — for Evaluator framing

---

**Acceptance signal**: when all 10 files are written, all 6 step-level verifications pass, `cargo test --workspace` is green, clippy is clean, every cardinality+order contract in §3 holds, and §5 mechanical verification commands all return their expected outputs, this ticket is ready for Evaluator review.
