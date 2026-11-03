"""One reviewed rules-value target; no world or ordinary safety inference."""
from pathlib import Path
from typing import Final

from generic_gate import require, text
from generic_process import file_hash
from mux2_gate_inputs import Json, Node

SOURCE: Final = 'rust/crates/sim-core/tests/baseline_matches_v7_final.rs'
CASES: Final = ('baseline_matches_v7_final', 'baseline_differs_between_worlds_only_where_drawn')
PINS: Final = {'rust/crates/sim-core/tests/baseline_matches_v7_final.rs': 'fbb5df41e4540191a9ff82aee25c8f064cf298de5e3cc240162e4e278415a4c6', 'rust/crates/sim-core/src/lib.rs': '248f9587aa8576fda1b769a0609f1b921a8b5f13244292d78a1b23f7e45bfe2c', 'rust/crates/sim-core/src/rules/mod.rs': '356d87da9a76b43eed2d594b5f4930e5ca3130060ca68b2387d1c0bb2e0336da', 'rust/crates/sim-core/src/rules/baseline.rs': 'd35d5f273ea51cc60c19dba9894cc62eb447120c6bd15b098035416f60c4a6ef', 'rust/crates/sim-core/src/rules/calendar.rs': '354390aa6214d7a74eab27db58793dbcc2353ea8995c9153523f29a043f05073', 'rust/crates/sim-core/src/rules/routine.rs': '0a1091b9d599002487cefa1621860e2d42334c8472a9ea1978292d4a3468f8ad', 'rust/crates/sim-core/src/variety.rs': '3802e85a572c224be712990ea5eedaa4fe536a0ce55027e29f5aa1edade56803', 'rust/crates/sim-core/src/rng.rs': 'e5d029ea1e5738822f4a1b66307e67d8bc7e20dba97876c5209ff1fd1855fed8', 'rust/crates/sim-core/tests/data/v7_final_rules_seed0.txt': 'd68d83544ee6a9c4b30e383c3509880ed2507b9dbb4043136debd75427029db4', 'rust/crates/sim-core/tests/data/v7_final_rules_seed1.txt': '0e0545f846addf9f941d212fe773e4c6d5220eddeec2a06bbf72128720e2e00a', 'rust/crates/sim-core/tests/data/v7_final_rules_seed7.txt': 'd6f63c5c34a72d46451c1e5a850a039bdfb2b72fbd1c43e89baa766da3676f68', 'rust/crates/sim-core/tests/data/v7_final_rules_seed42.txt': '14897a53e0d7672ed33ad3025696884fe959ac0f2e2398d0a9c8b0f91c59c901'}


def classify_core_pure(root: Path, inputs: dict[str, str], sections: list[Json]) -> list[Json]:
    """Require the entire original target census plus defining source and fixture pins."""
    matches = [Node(row) for row in sections
               if Node(row).field('package').value == 'sim-core'
               and Node(row).field('target').value == 'baseline_matches_v7_final'
               and Node(row).field('kind').value == 'test']
    require(len(matches) == 1, '/core-pure/target', 'missing or duplicate reviewed target')
    row = matches[0]
    row.field('source').equals(SOURCE)
    cases = row.field('cases').array()
    names = [text(case.field('name')) for case in cases]
    require(len(names) == len(CASES) and set(names) == set(CASES),
            '/core-pure/cases', 'original reviewed case set differs')
    for case in cases:
        case.field('status').equals('ok')
        case.field('mode').equals(None)
    row.field('nested_passed').equals(0)
    bindings: dict[str, Json] = {}
    for name, digest in PINS.items():
        require(inputs.get(name) == digest and file_hash(root / name) == digest,
                name, 'reviewed classification source changed or unbound')
        bindings[name] = digest
    return [{'package': 'sim-core', 'target': 'baseline_matches_v7_final', 'kind': 'test',
             'source': SOURCE, 'classification': 'PURE_RULE_VALUES_NO_WORLD',
             'cases': row.field('cases').value, 'source_pins': bindings,
             'ordinary_safety': 'NOT_APPLICABLE_NO_WORLD',
             'authority': 'SOURCE_QUALIFIED_ORIGINAL_AGGREGATE_ONLY',
             'publication_authority': False}]
