"""Exact original worldgen configuration values; no world generation is executed."""
from pathlib import Path
from typing import Final

from generic_gate import require, text
from generic_process import file_hash
from mux2_gate_inputs import Json, Node

SOURCE: Final = 'rust/crates/sim-systems/tests/baseline_worldgen_matches_v7_final.rs'
CASES: Final = ('baseline_worldgen_matches_v7_final', 'stress_world_is_the_baseline_world_resized')
PINS: Final = {'rust/crates/sim-core/src/lib.rs': '248f9587aa8576fda1b769a0609f1b921a8b5f13244292d78a1b23f7e45bfe2c',
 'rust/crates/sim-core/src/rng.rs': 'e5d029ea1e5738822f4a1b66307e67d8bc7e20dba97876c5209ff1fd1855fed8',
 'rust/crates/sim-core/src/variety.rs': '3802e85a572c224be712990ea5eedaa4fe536a0ce55027e29f5aa1edade56803',
 'rust/crates/sim-core/tests/data/v7_final_worldgen_seed0.txt': 'b65ef7f462f77a36b58f9cf0fd4f801863ccd34f473da16b246e45684ba6f5db',
 'rust/crates/sim-core/tests/data/v7_final_worldgen_seed1.txt': 'c585bf7fb0ed6e916c0541df1d154395bc991bae1017f296a51a3684a1012bd7',
 'rust/crates/sim-core/tests/data/v7_final_worldgen_seed42.txt': 'cc78c2d34c6c119d1ecdd60812193ce072171730afb7692ecbba189a2ad0e569',
 'rust/crates/sim-core/tests/data/v7_final_worldgen_seed7.txt': '32f15df8178dae23c218bbf4eb6c6f1dc59383c07264df39d45abb21942fa829',
 'rust/crates/sim-systems/src/lib.rs': '62eabd404f524dd7f4529dcff35874bd0cfa748e34223f92a569584f90abb60b',
 'rust/crates/sim-systems/src/worldgen/mod.rs': '11525a81664d3206c72754f3772d1ba4e61cd3f9dbe2ab611be4e4124378b601',
 'rust/crates/sim-systems/tests/baseline_worldgen_matches_v7_final.rs': '288f87cd26563fc7bdb5163f361cf030b974952d31af0cbedf73da156d6458de'}


def classify_config_pure(root: Path, inputs: dict[str, str], sections: list[Json]) -> list[Json]:
    """Require the entire original target census plus defining source and fixture pins."""
    matches = [Node(row) for row in sections
               if Node(row).field('package').value == 'sim-systems'
               and Node(row).field('target').value == 'baseline_worldgen_matches_v7_final'
               and Node(row).field('kind').value == 'test']
    require(len(matches) == 1, '/config-pure/target', 'missing or duplicate reviewed target')
    row = matches[0]
    row.field('source').equals(SOURCE)
    cases = row.field('cases').array()
    names = [text(case.field('name')) for case in cases]
    require(len(names) == len(CASES) and set(names) == set(CASES),
            '/config-pure/cases', 'original reviewed case set differs')
    for case in cases:
        case.field('status').equals('ok')
        case.field('mode').equals(None)
    row.field('nested_passed').equals(0)
    bindings: dict[str, Json] = {}
    for name, digest in PINS.items():
        require(inputs.get(name) == digest and file_hash(root / name) == digest,
                name, 'reviewed classification source changed or unbound')
        bindings[name] = digest
    return [{'package': 'sim-systems', 'target': 'baseline_worldgen_matches_v7_final', 'kind': 'test',
             'source': SOURCE, 'classification': 'PURE_WORLDGEN_CONFIG_NO_WORLD',
             'cases': row.field('cases').value, 'source_pins': bindings,
             'ordinary_safety': 'NOT_APPLICABLE_NO_WORLD',
             'authority': 'SOURCE_QUALIFIED_ORIGINAL_AGGREGATE_ONLY',
             'publication_authority': False}]
