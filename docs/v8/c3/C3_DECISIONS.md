# C3 decisions

Status: C3-a implementation; neither publication is claimed yet.
Start BASE: `c1aeb35857d85d7409825af0a29ff76ef700cf93` (v8/main after C2-b). Authority: `goal_worldsim_c3.md` and its
planner notes; planner answers arrive as `c3-data/D-C3-<n>.md`. Evidence lives in `/Users/rexxa/work/c3-data/`.

1. Preserve the starting probe/perf binaries before edits (START.json hashes). The same BASE is used for both C3 crash
   comparisons. A genuine C2-b format-23 save written by the gated 8e4170c42 build (fullgate-b-attempt1) is kept as the
   save-compatibility fixture.
2. `origin/v8/simcheck` (five M-SIMCHECK fixes) is merged with a normal merge commit (4cb393959) and gated with C3-a.
3. Iron ore keeps the C2-reserved row (code 3, hill/mountain); camp radius 60→80 for the goal's 80-tile bar. Worldgen never
   makes swamp and origins have no river predicate, so bog iron is a new row (code 4) on plain/forest at or below river
   level (elevation 0–100; river tiles are written at 100). Values were frozen before the 0–63 census: iron within 80 tiles
   on 64/64 seeds (bog iron on 38). No tuning after observation.
4. The iron column is appended (ids 129–165). Connected: iron_smelting, forging, iron_tools, iron_weapons, paved_road,
   stone_house, town_wall, gate. Everything without a runtime hook is Inert with its pass named. Markers: iron_smelting 3,
   forging 3, iron_tools 2, paved_road 1, stone_house 1. Planner D-C3-1: paved_road is Connected with a placeable cut-stone
   floor piece (`paved_road`, drawn through the packed-earth floor style with an authored, prewarmed `cut_stone` look);
   movement is one tile per interval and floor cost only steers A*, so the speed effect is M-GROW debt.
5. Discovery conditions are AND-only. Iron smelting is discovered by practice with charcoal and known copper smelting, not
   by stocked ore, so both ore and bog worlds — and worlds with no revealed iron, where the `no_iron_nearby` card applies —
   can learn it. No new condition type.
6. Recipe OR inputs do not exist: smelt_bloom (ore) and smelt_bog_bloom (bog iron) are two recipes. Station tags keep
   metals apart (bloomery ≠ smelting). Forge recipes are tagged `forging` and run at a forge or an anvil; refining is
   forge-only. Skills are reused as in C2 (heat = pottery, hammering = woodwork, dressing stone = knap). The anvil is
   stone and log, so the first iron tool has no iron prerequisite loop.
7. The iron-era stone house is a new template `cut_stone_house` (tier 5, upgrade from mud_brick_house). The neolithic
   `stone_house` row is unchanged: load-time validation re-expands saved sites from the current template and checks the
   upgrade predecessor, so editing it would break saves that hold a stone house. Template tier range 1..=4 → 1..=5 (tier
   only sorts and labels).
8. Warehouse ×4 storage has no data hook (storage is a selector), so the template and knowledge stay Inert (M-GROW).
   The iron helmet is not apparel (one apparel slot = body garment); shield boss, hoe, sickle, saw and adze have no
   mechanical effect (M9/M-BAL debt). Only the axe has an efficiency table.
9. Town wall and gate generalise from the palisade/fence pieces by data (Connected, no defence rule: M-NATION debt);
   a tower has no completion category with an effect and stays Inert.
10. Bronze-era debts that still said "C3:" (annealing, lost wax, lime, limestone, glass, faience) are relabelled to the
    pass that will connect them, so CONTENT_DEBT does not file them under this pass.
11. Published C2-b format-23 saves load through an exact-revision adapter `legacy_bronze_v23` (revision
    `catalog-v1-fnv1a64:54c3f1d8ee2c07b9:231728`, identity prefix checked, body unchanged), tested with the genuine save
    via `C3_C2B_SAVE_PATH`.
12. Chapters 8/9 and the `no_iron_nearby` card read bridge facts next to the C2 ones; the copper/bronze checks become
    tag-parameterised helpers. The card offers wait or trade; there is no prospect designation (planner D-C3-1; recorded in CONTENT_DEBT).
    A `wrought_iron` goods row (iron smelting known, no iron deposit in the world) is appended last, so it cannot shift
    existing draws while closed.
13. Art: per-row `from` pinning is not on v8/c3, so new iron items use placeholder shape × tint (planner D-C3-1).
    New walls reuse the granite/drystone look, so no new wall material look needs a step-41 prewarm.
14. The V3 capture fix (926010cbc) and V3's C2_DECISIONS corrections (e8f220341) are cherry-picked; V3 was closed
    unpublished.
15. The first C3-a pre-gate found one stale authored list (`engine_economy_ffi` recipe table) and the planner ruled on
    QUESTION-1/2 (D-C3-1); both land in a new commit on top of the first freeze, then a fresh pre-gate.
16. Pre-gate a1 (2ad516f14) failures, each fixed forward with its cause: stale authored lists
    (`engine_economy_ffi` recipe table; `spine_eras_from_data`, whose all-marker world is now in the iron era);
    the authored `bog_iron` material look lacked a category, which failed resolver parity, the data-only-extension
    resolver check and a mux2 oracle; and `spine_chain_log_house`, where the merged simcheck trajectory lets wolves kill
    four harvesters in this steward-OFF scene and the fixture restored a dead agent's priority. The fixture now skips only
    harvesters with a recorded death and prints it; an unexplained disappearance still fails. Same file, same latent
    pattern in `cord_world` left unchanged.
17. Pre-gate a2 (cfeb57c9e) failures: two knowledge tests still asserted the marker total 36 after the paved-road marker
    (now 37); and the paved road's entry in the imported `construction_styles.json` broke the art manifest hash. An
    unknown floor style already falls back to the default packed-earth style with the `cut_stone` material tint, so the
    imported file is restored unchanged. The `cut_stone` look is lighter and cooler (dL+30, da−6, db−22): with the first
    tint a built road averaged RGB 97,85,73 against 100,89,73 for the camp ground and could not be seen.

## C3-b

18. Steward rules are data on the C2-b `Bills` action: 14 (one bloom, iron ore stocked), 15 (one bloom, bog iron stocked;
    `stocked` is AND, so each ore has its own rule), 16 (one refined bar, bloom stocked), 17 (one iron axe per colonist
    chopping with a stone, copper or bronze axe). A shared `pre_iron_axe` tag on those four axes lets a single
    `replace_tag` rule cover them without two rules fighting over one bill; rule 13 orders a different bill.
19. Mind: two pure observation facts, `job_bloomery` (recipe tag `bloomery`) and `job_forging` (`forging` or `refining`),
    and two work nodes. Older policy histories restore through a `PRE_IRON` fallback, so published C2-b mind records load.
20. First-metal chronicle lines carry their causes through the existing ore-evidence column: the knowledge that gates the
    station and the recipe, the place (the template holding the station, else the station) and untracked inputs (fuel).
    Generic per metal; no iron branch in Rust. An optional per-item verb key gives "처음으로 쇠를 녹였다". Copper and bronze
    lines now read with the same cause list instead of the recipe name. FirstToolUsed names the recipe's station because
    no workshop instance is recorded.
21. `c3_chain_iron` (seed 0, the C2 chain world) follows the D-C2-1 contract: ordinary commands, nearest-target
    retargeting with recorded reasons, reissue on natural cancellation (≤6) with printed causes, a day deadline. Founder
    grants (Source::Seeded, as C2): prospecting, mining, charcoal, iron_smelting (its copper-smelting prerequisite has no
    chain in this world), mud_brick, mud_brick_house, stone_house. Forging and iron tools are discovered by practice.
    Mid-chain save reloads byte-equal; mid and full replays end byte-equal. It is disclosed apart from the tooling cap.
22. Chain captures frame each stage's subject at 2× through the camera's centring API (the camp framing queued by a load
    otherwise pulled stages back to the hearth), hide the helper and card overlays and capture each item event's Why.
    The bloomery and smelter have no art binding, light or fire, so "bloomery with fire" shows the station without fire
    (art debt).
