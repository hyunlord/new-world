"""Private tiny Git identity fixtures; never executes native builds."""

import json
import unittest
from dataclasses import replace
from pathlib import Path
from typing import final

import generic_identity_overlay as identity_module
import test_generic_cli_prepare as fixtures
from generic_cli_prepare import PreparedCli, prepare_cli
from generic_gate import document
from generic_identity_test_fixture import write_identity
from generic_process import file_hash
from generic_sources import git
from generic_whole_gate_source import check_materialization, materialize
from mux2_gate_inputs import Node


@final
class MaterializationIdentityTests(unittest.TestCase):
    def prepared(self) -> tuple[PreparedCli, tuple[Path, Path], Path]:
        fixture = fixtures.CliPrepareTests()
        fixture.setUp()
        self.addCleanup(fixture.doCleanups)
        selection = fixture.selected()
        h = selection.harness_root
        for name in identity_module.EXPLICIT:
            path = h / name
            path.parent.mkdir(parents=True, exist_ok=True)
            _ = path.write_text("{}\n" if name.endswith(".json") else "# fixture\n")
        observer = h / "tools/v8/observer.gd"
        _ = observer.write_text("# observer\n")
        _ = (h / "tools/v8/whole_gate_observer_matrix.json").write_text(
            json.dumps(
                {
                    "schema": "whole_gate_observer_matrix.v1",
                    "observer_postimages": {"tools/v8/observer.gd": ["100644", file_hash(observer)]},
                }
            )
        )

        def commit(root: Path) -> str:
            _ = git(root, "add", ".")
            _ = git(
                root, "-c", "user.name=Fixture", "-c", "user.email=fixture@example.invalid", "commit", "-qm", "fixture"
            )
            return git(root, "rev-parse", "HEAD").decode().strip()

        original_game = h / "rust/crates/sim-core/src/rules/colony.rs"
        original_game.parent.mkdir(parents=True, exist_ok=True)
        _ = original_game.write_text("const HUT_LOGS:u8=12;\n")
        hsha = commit(h)
        p = selection.candidate.root
        _ = git(p, "fetch", "-q", str(h), hsha)
        _ = git(p, "checkout", "--detach", hsha)
        game = p / "rust/crates/sim-core/src/rules/colony.rs"
        game.parent.mkdir(parents=True, exist_ok=True)
        _ = game.write_text("const HUT_LOGS:u8=13;\n")
        psha = commit(p)
        _ = git(h, "fetch", "-q", str(p), psha)
        base_overlay = fixture.fixture.make_overlay()
        base_roles = fixture.fixture.home / "base-roles.json"
        row = document(base_overlay.manifest.read_text()).field("files").array()[0].mapping()
        _ = base_roles.write_text(
            json.dumps(
                {
                    "schema": "whole_gate_layers.v1",
                    "product_sha": selection.published_sha,
                    "harness_sha": hsha,
                    "files": [
                        {
                            "path": row["path"],
                            "role": "O",
                            "post_sha256": row["observer_sha256"],
                            "qualification": "NOT_RUN_TEST_ONLY",
                        }
                    ],
                }
            )
        )
        output = fixture.fixture.home / "identity-overlay"
        output.mkdir()
        candidate_overlay, candidate_roles = write_identity(h, p, output)
        overlays = [base_overlay, candidate_overlay]
        selected = replace(
            selection,
            harness_sha=hsha,
            purpose="defect_proof",
            base=replace(selection.base, overlay=overlays[0]),
            candidate=replace(selection.candidate, source_sha=psha, overlay=overlays[1]),
        )
        return prepare_cli(selected), (base_roles, candidate_roles), game

    def test_base_nonempty_and_candidate_identity_keep_separate_sources(self) -> None:
        prepared, roles, game = self.prepared()
        assert prepared.arms[0].debug.overlay is not None
        assert prepared.arms[1].debug.overlay is not None
        self.assertTrue(prepared.arms[0].debug.overlay.patch.read_bytes())
        self.assertEqual(prepared.arms[1].debug.overlay.patch.read_bytes(), b"")
        receipt = materialize(prepared, roles)
        receipt.field("build_status").equals("NOT_RUN")
        receipt.field("binding").field("purpose").equals("defect_proof")
        self.assertEqual(game.read_text(), "const HUT_LOGS:u8=13;\n")
        path = prepared.selectors.output / "materialization.json"
        session = Node(
            {
                "materialization": {"path": str(path), "sha256": file_hash(path)},
                "harness": {"root": str(prepared.selectors.harness_root), "source_sha": prepared.selectors.harness_sha},
                "arms": receipt.field("arms").value,
                "binding": receipt.field("binding").value,
                "declarations": receipt.field("declarations").value,
            }
        )
        check_materialization(session)
        selected = Path(receipt.field("arms").field("candidate").field("identity_selected_source").field("path").text())
        _ = selected.write_text("{}")
        with self.assertRaisesRegex(ValueError, "selected H/O role source changed"):
            check_materialization(session)

    def test_changed_source_between_admission_and_materialization_rejects(self) -> None:
        prepared, roles, game = self.prepared()
        _ = game.write_text("const HUT_LOGS:u8=14;\n")
        with self.assertRaises(ValueError):
            _ = materialize(prepared, roles)
