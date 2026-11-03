"""Five reviewed source-owned mechanism/parser routes; never ordinary safety evidence."""
from pathlib import Path
from typing import Final

from generic_gate import require, text
from generic_process import file_hash
from mux2_gate_inputs import Json, Node

PINS: Final = {'rust/crates/sim-bridge/src/ffi/colony.rs': '8ac2a8e5e595bf4ec1bafe70d3121af27a77dfdf7bf855e9943882651d7017da',
 'rust/crates/sim-bridge/src/ffi/mod.rs': 'e81cd6a9f329474c579b8f3d03e6ab43069d4464d139ae4fac192c54c85ba945',
 'rust/crates/sim-bridge/src/ffi/node.rs': '8f624202216995c4cee88eaf89d189b3041965889410c24cf41e8a5575785af1',
 'rust/crates/sim-core/src/rules/colony.rs': '3fa9680569223cb974acdcf7247f0aabf8296da707752f17ac24e94f02471946',
 'rust/crates/sim-engine/src/lib.rs': '4281c820c3fa89b95588de0396bdf1233539f865a3b33856679b8520974003e8',
 'rust/crates/sim-systems/src/lib.rs': '62eabd404f524dd7f4529dcff35874bd0cfa748e34223f92a569584f90abb60b',
 'rust/crates/sim-systems/src/runtime/influence/agent_sample.rs': 'b3c473329fbf139a43c5a2a3fa12b3cf78dd0e194a75f0286aca4d3441233f7d',
 'rust/crates/sim-systems/src/runtime/influence/building_stamp.rs': '39c583e1446a2a4f94fa5d055dafece5610c4f99e0cea1fdda06f27daef05bcb',
 'rust/crates/sim-systems/src/runtime/influence/mod.rs': 'a2ad0d40d6ac87e4f5fc0d7df438f32c0dff1af063735849238909ad81e32395',
 'rust/crates/sim-systems/src/runtime/influence/update.rs': '5cba3e507ef64aa7b4c362247f46dda80cda859d7c563b32b6d467e503f4de48',
 'rust/crates/sim-systems/src/runtime/influence/visualization.rs': '9afbfca78e81ba54e7f0397aadbd5855b9905463f5b5c8b56e3f0fc4d1701184',
 'rust/crates/sim-systems/src/worldgen/mod.rs': '11525a81664d3206c72754f3772d1ba4e61cd3f9dbe2ab611be4e4124378b601',
 'rust/crates/sim-test/src/bin/worldsim_perf.rs': 'e8dc0e70806107e9e0de090bc99a206e6ae55aad54e794ea2559a95441c3c765',
 'rust/crates/sim-test/src/bin/worldsim_probe.rs': '93b27ff3ff8f948dd1e62488cbfe873b7392d8bd6b3399d93ef44e7412cfc330',
 'rust/crates/sim-test/src/lib.rs': 'd5a126519bca132d236972c8813faf27a8f58dec8c514c1e1a96c95a16ec9a49',
 'rust/crates/sim-test/src/perf.rs': '52c313e575ea5aa2e3ee71f902ca76e981d58629b0be37953c3b021fed95e7ec',
 'rust/crates/sim-test/src/probe/args.rs': '8be574955e5504c24b64ea0a645ab10cb1a87f2f6322bae44728a22fe86a00e7',
 'rust/crates/sim-test/src/scenes.rs': '7fd693a1db13dafcbce414897ea0f98f2d40e888e8a1085532b8438c13418173',
 'rust/crates/sim-test/tests/engine_bridge_postverify.rs': '5a45629313c0cd2e5248ec32d7e2e6ce2e1902e073b2e8fe06964cdae6729bff',
 'rust/crates/sim-test/tests/engine_tick_scaffold.rs': '90427b86d8c486a40c7b43c662ab3ae0224ddfcd5080e629fbffbacf24fe0cb1',
 'rust/crates/sim-test/tests/tree_snapshot.rs': '5523de18d59e8c7ea5dcfbfff52e30ce05416cdb19eee861b5c95d979e591cee'}

CONTRACTS: Final = (('worldsim_perf',
  'bin',
  'rust/crates/sim-test/src/bin/worldsim_perf.rs',
  'PARSER_ONLY_NO_WORLD',
  ('tests::cli_rejects_unknown_subcommands',),
  ('rust/crates/sim-test/src/bin/worldsim_perf.rs', 'rust/crates/sim-test/src/perf.rs')),
 ('worldsim_probe',
  'bin',
  'rust/crates/sim-test/src/bin/worldsim_probe.rs',
  'PARSER_ONLY_NO_WORLD',
  ('tests::cli_rejects_unknown_flags',),
  ('rust/crates/sim-test/src/bin/worldsim_probe.rs', 'rust/crates/sim-test/src/probe/args.rs')),
 ('engine_tick_scaffold',
  'test',
  'rust/crates/sim-test/tests/engine_tick_scaffold.rs',
  'CONTROLLED_EMPTY_PHASE2',
  ('harness_scaffold_tick_count_zero_on_fresh_engine',
   'harness_scaffold_process_equivalent_drives_current_tick_to_n',
   'harness_scaffold_resources_current_tick_propagated_by_tick'),
  ('rust/crates/sim-test/tests/engine_tick_scaffold.rs',
   'rust/crates/sim-engine/src/lib.rs',
   'rust/crates/sim-systems/src/lib.rs',
   'rust/crates/sim-systems/src/runtime/influence/mod.rs',
   'rust/crates/sim-systems/src/runtime/influence/agent_sample.rs',
   'rust/crates/sim-systems/src/runtime/influence/building_stamp.rs',
   'rust/crates/sim-systems/src/runtime/influence/update.rs',
   'rust/crates/sim-systems/src/runtime/influence/visualization.rs')),
 ('engine_bridge_postverify',
  'test',
  'rust/crates/sim-test/tests/engine_bridge_postverify.rs',
  'CONTROLLED_EMPTY_PHASE2',
  ('harness_ffi_bridge_baseline_queue_empty_on_fresh_engine',
   'harness_ffi_bridge_baseline_dirty_regions_warmth_zero_on_fresh_engine',
   'harness_ffi_bridge_pre_tick_queue_len_equals_3_after_3_ffi_enqueue_calls',
   'harness_ffi_bridge_pre_tick_dirty_regions_zero_after_enqueue_before_tick',
   'harness_ffi_bridge_ffi_enqueue_path_drains_queue_via_full_pipeline',
   'harness_ffi_bridge_ffi_dirty_regions_warmth_count_3_after_ffi_enqueue_path',
   'harness_ffi_bridge_ffi_dirty_regions_exact_bounds_match_enqueue_coordinates',
   'harness_ffi_bridge_ffi_dirty_regions_non_warmth_channel_spot_check_beauty',
   'harness_ffi_bridge_oob_ffi_returns_false',
   'harness_ffi_bridge_pre_tick_queue_len_equals_2_after_2_inbounds_1_oob',
   'harness_ffi_bridge_oob_clamp_at_enqueue_time_dirty_count_equals_2',
   'harness_ffi_bridge_idempotent_retick_pending_all_zero_after_5_empty_queue_ticks',
   'harness_ffi_bridge_idempotent_retick_dirty_regions_stable_at_3_after_5_idle_ticks'),
  ('rust/crates/sim-test/tests/engine_bridge_postverify.rs',
   'rust/crates/sim-engine/src/lib.rs',
   'rust/crates/sim-systems/src/lib.rs',
   'rust/crates/sim-systems/src/runtime/influence/mod.rs',
   'rust/crates/sim-systems/src/runtime/influence/agent_sample.rs',
   'rust/crates/sim-systems/src/runtime/influence/building_stamp.rs',
   'rust/crates/sim-systems/src/runtime/influence/update.rs',
   'rust/crates/sim-systems/src/runtime/influence/visualization.rs',
   'rust/crates/sim-bridge/src/ffi/node.rs',
   'rust/crates/sim-bridge/src/ffi/mod.rs')),
 ('tree_snapshot',
  'test',
  'rust/crates/sim-test/tests/tree_snapshot.rs',
  'CONTROLLED_TREE_IDENTITY',
  ('tree_snapshot_uses_saved_identity_and_stable_coordinate_order',
   'missing_saved_tree_identity_is_not_disguised_as_a_species',
   'an_override_is_not_replaced_by_the_persisted_default'),
  ('rust/crates/sim-test/tests/tree_snapshot.rs',
   'rust/crates/sim-test/src/lib.rs',
   'rust/crates/sim-test/src/scenes.rs',
   'rust/crates/sim-bridge/src/ffi/colony.rs',
   'rust/crates/sim-systems/src/worldgen/mod.rs',
   'rust/crates/sim-core/src/rules/colony.rs',
   'rust/crates/sim-engine/src/lib.rs',
   'rust/crates/sim-systems/src/lib.rs')))

def classify_routes(root: Path, inputs: dict[str, str], sections: list[Json]) -> list[Json]:
    """Consume already authenticated original aggregate sections, with exact case sets."""
    classified: list[Json] = []
    for name, kind, source, category, names, dependencies in CONTRACTS:
        matches = [Node(row) for row in sections if Node(row).field('package').value == 'sim-test'
                   and Node(row).field('kind').value == kind
                   and Node(row).field('target').value == name]
        require(len(matches) == 1, '/classification/target', 'missing or duplicate reviewed target')
        row = matches[0]
        row.field('source').equals(source)
        cases = row.field('cases').array()
        observed = [text(case.field('name')) for case in cases]
        require(len(observed) == len(names) and set(observed) == set(names),
                '/classification/cases', 'original reviewed case set differs')
        for case in cases:
            case.field('status').equals('ok')
            case.field('mode').equals(None)
        row.field('nested_passed').equals(0)
        bindings: dict[str, Json] = {}
        for path in dependencies:
            expected = PINS[path]
            require(inputs.get(path) == expected and file_hash(root/path) == expected,
                    path, 'reviewed classification source changed or unbound')
            bindings[path] = expected
        classified.append({'package': 'sim-test', 'target': name, 'kind': kind,
                           'source': source, 'classification': category,
                           'cases': row.field('cases').value, 'source_pins': bindings,
                           'ordinary_safety': 'NOT_APPLICABLE_TO_REVIEWED_MECHANISM',
                           'authority': 'SOURCE_QUALIFIED_ORIGINAL_AGGREGATE_ONLY',
                           'publication_authority': False})
    return classified


def classified_target(row: Node, classifications: list[Json]) -> bool:
    """Match only successful explicit classifications, never a filename pattern."""
    return any(all(row.field(key).value == Node(item).field(key).value
                   for key in ('package', 'target', 'kind')) for item in classifications)
