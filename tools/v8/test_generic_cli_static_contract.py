"""Source-contract integration using actual tiny issued source proofs."""
import tempfile
import unittest
from dataclasses import replace
from pathlib import Path
from unittest.mock import patch

import generic_full_gate as gate
import generic_static_run as static
import test_generic_static_run as fixtures
from generic_gate import GateContractError
from mux2_gate_inputs import Node


class ContractIntegrationTests(unittest.TestCase):
    def test_issued_source_receipt_preserves_complete_authentication(self) -> None:
        fixture=fixtures.Base0SourceTests();self.addCleanup(fixture.doCleanups)
        with tempfile.TemporaryDirectory() as directory:
            request,contract=fixture.source_fixture(Path(directory).resolve(),derived=True)
            result=static.run_docs_successor(request)
            self.assertIs(request.product.source_contract,contract)
            self.assertEqual(gate.authenticate_static(result.component.receipt,14,request.product,request.environment),'PASS')
            # Source-only acceptance cannot override the original process/native oracle.
            with patch.object(gate,'static_process_terminal',side_effect=GateContractError('INVALID_EVIDENCE','/process','retained native check')) as process:
                with self.assertRaisesRegex(GateContractError,'retained native check'):
                    _=gate.authenticate_static(result.component.receipt,14,request.product,request.environment)
                self.assertEqual(process.call_count,1)
            with self.assertRaises((GateContractError,ValueError)):
                _=gate.authenticate_static(result.component.receipt,14,replace(request.product,source_contract=None),request.environment)
            with self.assertRaises((GateContractError,ValueError)):
                _=gate.authenticate_static(result.component.receipt,14,replace(request.product,source_contract=replace(contract)),request.environment)

    def test_final_source_recheck_follows_complete_authentication(self) -> None:
        fixture=fixtures.Base0SourceTests();self.addCleanup(fixture.doCleanups)
        with tempfile.TemporaryDirectory() as directory:
            request,_=fixture.source_fixture(Path(directory).resolve())
            result=static.run_docs_successor(request)
            actual=gate.static_process_terminal
            reached: list[bool]=[]
            def change(path: Path,node: Node,product: static.StaticProduct,environment: static.ExecutionEnvironment,step: int) -> None:
                actual(path,node,product,environment,step)
                reached.append(True)
                _=(product.root/'tools/v8/gate.sh').write_text('late source change')
            with patch.object(gate,'static_process_terminal',side_effect=change),self.assertRaises((GateContractError,ValueError)):
                _=gate.authenticate_static(result.component.receipt,14,request.product,request.environment)
            self.assertEqual(reached,[True])


if __name__=='__main__':_=unittest.main()
