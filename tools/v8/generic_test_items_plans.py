"""Eleven source-selected colony_items worlds with actual resolved command order."""
from generic_gate import document, integer, require
from generic_test_endpoint_types import (
    ExpectedOccurrence,
    Snapshot,
    bound_bytes,
    fields,
)
from generic_test_history_graph import HistoryEvidence, extends
from generic_test_jobs_evidence import JobEvidence, coordinates
from generic_test_plans import TESTS as ALL_TESTS
from generic_test_plans import Planned
from mux2_gate_inputs import Json, Node

TESTS = ALL_TESTS['colony_items']
INPUTS: dict[str, str] = {'rust/crates/sim-test/tests/colony_items.rs': '76608c79b7c3e30f6dcd36d935e34d895e3836eaf69bee209f5599eaf5acff61', 'rust/crates/sim-test/tests/colony_support/mod.rs': 'b5fb68e830fee08ff7e04694a23919ec62cfdcdab3431a69aed58899db616525', 'rust/crates/sim-test/tests/save_gate_observation/mod.rs': 'ab7c324c94846f231128040f7ccf34fbd2c8a9339a927a65962c1f2c17ce2cdf', 'rust/crates/sim-test/src/lib.rs': 'd5a126519bca132d236972c8813faf27a8f58dec8c514c1e1a96c95a16ec9a49', 'rust/crates/sim-test/src/probe/colony.rs': '97d9bc0216240cfb91f39e52871d4227919ba7d379cfd9c3ee8f9eb8f9111cfd', 'rust/crates/sim-core/src/colony/command.rs': 'e34ac1d2a0715d971f957b9760609475e15f0f09bb2a5ba2b16741b41b2de05b'}


def submitted(before: Snapshot, after: Snapshot, terminal: Snapshot, expected: list[Json]) -> None:
    require(before.tick == after.tick == 360, '/orders', 'original order tick differs')
    extends(before, after, '/orders')
    require(not before.commands and len(after.commands) == len(expected), '/orders', 'missing or extra submitted commands')
    require(terminal.commands == after.commands, '/orders', 'command history changed after submission')
    for actual, payload in zip(after.commands, expected, strict=True):
        require(actual.tick == 360 and actual.accepted, '/orders', 'original order rejected or retimed')
        document(actual.command_json).equals(payload)


def items_plan(graph: HistoryEvidence) -> tuple[tuple[Planned, ...], dict[str, Json]]:
    for name, digest in INPUTS.items():
        require(dict(graph.inputs).get(name) == digest, name, 'unreviewed colony_items source')
        _ = bound_bytes(graph.input_root / name, digest)
    plans: list[Planned] = []
    setups: dict[str, Json] = {}
    for test, seeds, end in ((TESTS[0], (0, 3), 0), (TESTS[1], (1,), 4680), (TESTS[2], tuple(range(8)), 14400)):
        for seed in seeds:
            item = JobEvidence(graph, test, 'primary', seed)
            if test == TESTS[0]:
                initial, terminal = item.chain(())
                require(initial == terminal and not terminal.commands, item.folder, 'tick-zero placement case changed')
            elif test == TESTS[1]:
                initial, terminal = item.chain(('selected_quarry', 'before_orders', 'after_orders', 'quarry_completion'))
                chosen, detail = item.checkpoint('selected_quarry')
                fields(detail, 'boulder node_left stockpile')
                require(chosen == initial, item.folder, 'quarry selection no longer precedes360 ticks')
                x, y = coordinates(detail.field('boulder'))
                stock = [integer(value) for value in detail.field('stockpile').array()]
                require(len(stock) == 4, item.folder, 'stockpile rectangle required')
                left = integer(detail.field('node_left'))
                require(8 <= left <= 20, item.folder, 'original boulder amount differs')
                before, empty = item.checkpoint('before_orders')
                empty.equals({})
                after, empty = item.checkpoint('after_orders')
                empty.equals({})
                expected: list[Json] = [
                    {'tick': 360, 'cmd': 'add_stockpile', 'x0': stock[0], 'y0': stock[1], 'x1': stock[2], 'y1': stock[3]},
                    {'tick': 360, 'cmd': 'designate_quarry', 'x0': x, 'y0': y, 'x1': x, 'y1': y}]
                submitted(before, after, terminal, expected)
                require(after.commands[1].count == 1, item.folder, 'original quarry count must be one')
                finished, result = item.checkpoint('quarry_completion')
                fields(result, 'first_exhaustion_loop_index quarry_batches nodes_exhausted stone_created stocked')
                require(finished == terminal and 0 <= integer(result.field('first_exhaustion_loop_index')) < 4320,
                        item.folder, 'exhaustion occurrence missing or substituted as endpoint')
                result.field('quarry_batches').equals((left + 3) // 4)
                result.field('nodes_exhausted').equals(1)
                result.field('stone_created').equals(left)
                result.field('stocked').equals(left)
                setups[f'{test}/{seed}'] = detail.value
            else:
                _, terminal = item.chain(('resolved_orders', 'after_orders'))
                before, detail = item.checkpoint('resolved_orders')
                fields(detail, 'commands')
                after, empty = item.checkpoint('after_orders')
                empty.equals({})
                expected = [row.value for row in detail.field('commands').array()]
                require(bool(expected), item.folder, 'original resolved U-script is missing')
                names = [Node(row).field('cmd').text() for row in expected]
                require(names[0] == 'designate_chop_tiles', item.folder, 'U-script first command changed')
                remaining = names[1:]
                for optional in ('add_stockpile', 'place_blueprint'):
                    if remaining and remaining[0] == optional:
                        remaining = remaining[1:]
                require(len(remaining) <= 3 and all(name == 'designate_quarry' for name in remaining),
                        item.folder, 'source U-script and nearest-node order changed')
                submitted(before, after, terminal, expected)
                setups[f'{test}/{seed}'] = detail.value
            require(terminal.tick == end, item.folder, 'fixed original horizon changed')
            plans.append(Planned(ExpectedOccurrence(test, test, seed, 'primary', 0, end, True), 'original_endpoint'))
    return tuple(plans), setups
