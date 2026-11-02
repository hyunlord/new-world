"""Direct comparison retains safety failures independently of semantic equality."""
import json
import tempfile
import unittest
from dataclasses import replace
from pathlib import Path

from generic_fixture_registry import ExpansionContext, expand, load_registry
from generic_fixtures import compare_ordinary, fixed_observations
from generic_gate import GateContractError
from generic_process import ProcessResult
from test_generic_observations import receipt


class DirectFixtureTests(unittest.TestCase):
    def setUp(self):
        self.temp = tempfile.TemporaryDirectory()
        self.addCleanup(self.temp.cleanup)
        self.root = Path(self.temp.name).resolve()
        registry = load_registry(Path(__file__).with_name("gate_fixtures.json"))
        self.fixture = next(row for row in registry.fixtures if row.id == "core.steward_off")
        plans = expand(registry, ExpansionContext(self.root, self.root, self.root / "out",
                                                 self.root / "base-probe", self.root / "candidate-probe"))
        self.base, self.candidate = [plan for plan in plans if plan.fixture_id == self.fixture.id]
        for plan in (self.base, self.candidate):
            (plan.out / "gate_observations").mkdir(parents=True)
            (plan.out / "summary.json").write_text('{"seeds":[{"alive":8}]}')
            (plan.out / "gate_observations/colony_s0_primary.json").write_text(json.dumps(receipt()))
        # Deliberately synthetic process descriptors: these tests qualify only the
        # comparison, never child/source authentication or native world behavior.
        self.results = [ProcessResult(plan.out / "process.json", "PASS", 0, 10 + index)
                        for index, plan in enumerate((self.base, self.candidate))]

    def compare(self):
        return compare_ordinary(self.fixture, self.base, self.candidate, self.results[0], self.results[1],
                                "a" * 40, self.root / "comparison.json")

    def test_equal_fixture_with_complete_safety(self):
        result = self.compare()
        self.assertTrue(result["semantic_equal"])
        self.assertFalse(result["publication_authority"])

    def test_equal_matching_losses_fail_with_retained_result(self):
        data = receipt()
        data["knowledge_lost"] = [{"tick": 28560, "node": "clay_digging", "holder": 9}]
        for plan in (self.base, self.candidate):
            (plan.out / "gate_observations/colony_s0_primary.json").write_text(json.dumps(data))
        with self.assertRaisesRegex(GateContractError, "KnowledgeLost"):
            self.compare()
        result = json.loads((self.root / "comparison.json").read_text())
        self.assertTrue(result["semantic_equal"])
        self.assertEqual(len(result["safety_failures"]), 1)

    def test_undeclared_difference_retained(self):
        (self.candidate.out / "summary.json").write_text('{"seeds":[{"alive":7}]}')
        with self.assertRaisesRegex(GateContractError, "undeclared semantic"):
            self.compare()
        self.assertFalse(json.loads((self.root / "comparison.json").read_text())["semantic_equal"])

    def test_missing_extra_and_failed_child_observations_reject(self):
        path = self.base.out / "gate_observations/colony_s0_primary.json"
        path.rename(path.with_name("wrong.json"))
        with self.assertRaisesRegex(GateContractError, "missing or extra"):
            fixed_observations(self.fixture, self.base, self.results[0])
        failed = replace(self.results[1], status="FAIL", native_exit=7)
        with self.assertRaisesRegex(GateContractError, "did not complete"):
            fixed_observations(self.fixture, self.candidate, failed)


class GeneratedReplayBindingTests(unittest.TestCase):
    """Capture dispatch specs; no real child or native world is launched."""
    def test_external_replay_keeps_original_argv_and_pins_file(self):
        self.check_dispatch(external=True)

    def test_materialized_replay_keeps_existing_inventory_compatibility(self):
        self.check_dispatch(external=False)

    def check_dispatch(self, external: bool) -> None:
        from unittest.mock import patch

        from generic_fixture_registry import GeneratedInput, Invocation
        from generic_fixtures import ProbeProduct, run_invocation
        from generic_process import file_hash
        with tempfile.TemporaryDirectory() as tmp:
            root=Path(tmp).resolve()
            inputs=root/"inputs";inputs.mkdir()
            static=inputs/"static.json";static.write_text("{}")
            generated=(root if external else inputs)/"colony_s5_commands.json"
            generated.write_text('[{"tick":1}]')
            executable=root/"probe";executable.write_text("not executed")
            argv=(str(executable),"--commands",str(generated))
            plan=Invocation("base/replay.recorded/1","replay.recorded","base","1",(5,),("primary",),inputs,root/"out",argv,(),GeneratedInput("base/replay.generate/1",generated,file_hash(generated)),())
            product=ProbeProduct("a"*40,executable,file_hash(executable),inputs,(("static.json",file_hash(static)),))
            result=ProcessResult(plan.out/"process.json","PASS",0,4242)
            with patch("generic_fixtures.execute",return_value=result) as execute_mock:
                self.assertEqual(run_invocation(plan,product,()),result)
            spec=execute_mock.call_args.args[0]
            self.assertEqual(spec.argv,argv)
            self.assertEqual(spec.executable_sha256,file_hash(executable))
            self.assertEqual(dict(spec.environment)["WORLDSIM_GIT_SHA"],"a"*40)
            if external:
                self.assertEqual(spec.external_inputs,((generated,file_hash(generated)),))
                self.assertNotIn(generated.name,dict(spec.inputs))
            else:
                self.assertEqual(spec.external_inputs,())
                self.assertEqual(dict(spec.inputs)[generated.name],file_hash(generated))
            dependency = plan.generated_input
            assert dependency is not None
            with patch("generic_fixtures.execute") as execute_mock,self.assertRaises(GateContractError):
                run_invocation(replace(plan,generated_input=replace(dependency,sha256="0"*64)),product,())
            execute_mock.assert_not_called()
            if not external:
                with patch("generic_fixtures.execute") as execute_mock,self.assertRaisesRegex(GateContractError,"conflicting generated"):
                    run_invocation(plan,product,((generated.name,"0"*64),))
                execute_mock.assert_not_called()


if __name__ == "__main__":
    unittest.main()
