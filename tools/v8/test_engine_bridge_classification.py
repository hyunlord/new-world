"""Pure source/outcome checks; no native or ordinary safety claim."""
import re
import shutil
import tempfile
import unittest
from pathlib import Path
from typing import Final, final

import generic_process
from generic_gate import GateContractError
from generic_whole_gate_workspace_classification import CONTRACTS, PINS, classify_routes
from mux2_gate_inputs import EvidenceError, Json, Node

ROOT: Final = Path(generic_process.__file__).resolve().parents[2]
SOURCE: Final = 'rust/crates/sim-test/tests/engine_bridge_ffi.rs'
TARGET: Final = 'engine_bridge_ffi'
DEPENDENCIES: Final = (SOURCE, 'rust/crates/sim-engine/src/lib.rs',
    'rust/crates/sim-systems/src/runtime/influence/building_stamp.rs',
    'rust/crates/sim-systems/src/runtime/influence/mod.rs',
    'rust/crates/sim-bridge/src/ffi/node.rs', 'rust/crates/sim-bridge/src/ffi/mod.rs')


def sections() -> list[Json]:
    return [{'package':'sim-test','target':name,'kind':kind,'source':source,'nested_passed':0,
             'cases':[{'name':case,'status':'ok','mode':None} for case in names]}
            for name,kind,source,_,names,_ in CONTRACTS]


@final
class BridgeClassificationTests(unittest.TestCase):
    def test_exact_source_cases_and_controlled_category(self) -> None:
        contract = next(row for row in CONTRACTS if row[0] == TARGET)
        names = tuple(re.findall(r'#\[test\]\s+fn (\w+)\(', (ROOT/SOURCE).read_text()))
        self.assertEqual(len(names), 22)
        self.assertEqual(contract[4], names)
        self.assertEqual(contract[3], 'CONTROLLED_EMPTY_BSS_ONLY')
        self.assertEqual(set(contract[5]), set(DEPENDENCIES))
        result = classify_routes(ROOT, dict(PINS), sections())
        selected = next(Node(row) for row in result if Node(row).field('target').value == TARGET)
        self.assertEqual(selected.field('ordinary_safety').value, 'NOT_APPLICABLE_TO_REVIEWED_MECHANISM')
        self.assertIs(selected.field('publication_authority').value, False)

    def test_missing_duplicate_unknown_and_ignored_original_cases(self) -> None:
        for change in ('missing','duplicate','unknown','ignored'):
            rows = sections()
            row = next(Node(value) for value in rows if Node(value).field('target').value == TARGET)
            cases = row.field('cases').array()
            match change:
                case 'missing': row.mapping()['cases'] = [case.value for case in cases[:-1]]
                case 'duplicate': row.mapping()['cases'] = [case.value for case in cases] + [cases[0].value]
                case 'unknown': cases[-1].mapping()['name'] = 'unreviewed'
                case 'ignored': cases[-1].mapping()['status'] = 'ignored'
            with self.subTest(change=change), self.assertRaises((GateContractError, EvidenceError)):
                _ = classify_routes(ROOT, dict(PINS), rows)

    def test_each_direct_authority_required(self) -> None:
        for path in DEPENDENCIES:
            inputs = dict(PINS)
            _ = inputs.pop(path, None)
            with self.subTest(path=path), self.assertRaises(GateContractError):
                _ = classify_routes(ROOT, inputs, sections())

    def test_engine_tick_substitution_invalidates_classification(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory).resolve()
            for name in PINS:
                target = root/name
                target.parent.mkdir(parents=True, exist_ok=True)
                _ = shutil.copyfile(ROOT/name, target)
            target = root/SOURCE
            original = target.read_text()
            call = 'BuildingStampSystem::new().tick(&mut e.world, &mut e.resources);'
            self.assertIn(call, original)
            _ = target.write_text(original.replace(call, 'e.tick();', 1))
            with self.assertRaisesRegex(GateContractError, 'source changed or unbound'):
                _ = classify_routes(root, dict(PINS), sections())


if __name__ == '__main__':
    _ = unittest.main()
