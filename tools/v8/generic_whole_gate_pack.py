"""Source-selected step36 mechanism prefix under its actual aggregate wait."""
from pathlib import Path
from typing import Final

from generic_gate import boundary, require
from generic_integration_accounting import CASES, PARENT, account36
from generic_integration_prefix_run import GODOT_ENV_SHA, SCRIPT36_SHA, TOPOLOGY36_SHA
from generic_process import file_hash
from generic_whole_gate_aggregate import LIMIT, aggregate_output
from generic_workspace_cases import Listing, digest
from mux2_gate_inputs import Json, Node

from generic_whole_gate_renderer import original_anchors

SOURCE: Final = 'rust/crates/sim-test/tests/data_only_ext.rs'
SOURCE_SHA: Final = 'b5d4fce2d88ca47992eb773c60569b1c1da4b702131f176c25f3690e020449ed'


@boundary
def collect(session: Node, stage: Node) -> dict[str, Json]:
    stage.field('step').equals(36)
    stdout, pid = aggregate_output(session, stage)
    arm = stage.field('arm').text()
    product = session.field('arms').field(arm)
    root = Path(product.field('root').text())
    for name, sha in ((SOURCE, SOURCE_SHA), ('tools/v8/data_only_ext.sh', SCRIPT36_SHA),
                      ('tools/v8/godot_bin.sh', GODOT_ENV_SHA)):
        product.field('source_inputs').field(name).equals(sha)
        require(file_hash(root / name) == sha, name, 'unreviewed pack mechanism source')
    source = (root / SOURCE).read_text()
    start = source.index('#[test]\nfn selected_pack_runs_')
    end = source.index('\n#[test]\nfn selected_pack_child', start)
    require(digest(source[start:end].encode()) == TOPOLOGY36_SHA, '/topology', 'changed nine-child synchronous topology')
    require('WORLDSIM_DATA_ONLY_CASE' not in stage.field('environment').mapping(), '/environment', 'injected child selector')
    log = Path(session.field('output').text()) / arm / 'data_only_ext'
    anchors = original_anchors(stage)
    anchors.subtree(log)
    raw = anchors.bytes(log / 'simulation.log')
    marker = b'\nrunning 2 tests\n'
    require(raw.count(marker) == 1, '/prefix', 'missing or repeated original pack invocation')
    prefix, native = raw.split(marker, 1)
    require(prefix.count(b'Running tests/data_only_ext.rs (') == 1, '/prefix', 'original selected native target missing')
    # These are source-derived case names, not purported independent --list
    # executions. account36 checks the exact nine-child/parent outcome envelope.
    selected = Listing(('selected_pack_child', PARENT), (), SOURCE_SHA)
    accounting = account36(marker + native, selected, Listing((), (), SOURCE_SHA))
    require('DATA_ONLY_EXT PASS: seven-family simulation and selected-scene UI; rendered capture is separate' in stdout,
            '/oracle', 'original compound oracle missing')
    anchors.subtree(log)
    _ = aggregate_output(session, stage)
    return {'status': 'PASS', 'qualification': LIMIT, 'aggregate_pid': pid,
            'source_case_inventory': list(selected.names), 'nested_case_order': list(CASES),
            'native_log_sha256': accounting.raw_sha256,
            'nested_ranges': [[a, b, sha] for a, b, sha in accounting.nested_ranges],
            'classification': 'controlled_partial_jobs_rewritten_clock_pressure_catalogue_boundaries',
            'ordinary_safety': 'UI_continuous_selected_pack_world_joined_separately',
            'independent_child_wait': False, 'independent_listing': False}


@boundary
def evaluate_family(session: Node, base: Node, candidate: Node) -> dict[str, Json]:
    base.field('arm').equals('base')
    candidate.field('arm').equals('candidate')
    return {'status': 'PASS', 'base': collect(session, base), 'candidate': collect(session, candidate)}
