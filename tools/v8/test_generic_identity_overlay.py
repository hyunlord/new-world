"""Identity census and admission rejection regressions."""

import json
import unittest
from dataclasses import replace
from typing import final

from generic_build import Overlay, overlay_files
from generic_gate import document
from generic_identity_test_fixture import IdentityFixture
from generic_process import file_hash
from generic_sources import freeze_source, git
from mux2_gate_inputs import Node


@final
class IdentityTests(unittest.TestCase):
    def test_compose_and_authenticated_identity_preserve_mutant(self) -> None:
        fixture = IdentityFixture(self)
        from generic_build import derived_snapshot

        spec, context = fixture.identity()
        assert spec.overlay is not None
        index = git(fixture.product, "ls-files", "--stage", "-z")
        original = freeze_source(fixture.product, fixture.psha)
        self.assertEqual(spec.overlay.patch.read_bytes(), b"")
        self.assertEqual(overlay_files(spec, context), {})
        _ = derived_snapshot(fixture.product, original, {}, index)
        self.assertEqual((fixture.product / fixture.gameplay).read_text(), "const HUT_LOGS: u8 = 13;\n")

    def test_identity_without_context_rejects(self) -> None:
        fixture = IdentityFixture(self)
        spec, _ = fixture.identity()
        with self.assertRaisesRegex(ValueError, "identity context required"):
            _ = overlay_files(spec)

    def test_selected_role_mutation_rejected(self) -> None:
        fixture = IdentityFixture(self)
        spec, context = fixture.identity()
        assert spec.overlay is not None
        _ = context.selected_path.write_text("{}")
        with self.assertRaisesRegex(ValueError, "external input changed"):
            _ = overlay_files(spec, context)

    def test_patch_must_be_zero_bytes_even_with_updated_hash(self) -> None:
        fixture = IdentityFixture(self)
        spec, context = fixture.identity()
        assert spec.overlay is not None
        _ = spec.overlay.patch.write_bytes(b"not a patch")
        manifest = document(spec.overlay.manifest.read_text()).mapping()
        manifest["observer_patch_sha256"] = file_hash(spec.overlay.patch)
        _ = spec.overlay.manifest.write_text(json.dumps(manifest))
        overlay = Overlay(
            spec.overlay.patch, file_hash(spec.overlay.patch), spec.overlay.manifest, file_hash(spec.overlay.manifest)
        )
        with self.assertRaisesRegex(ValueError, "exactly zero bytes"):
            _ = overlay_files(replace(spec, overlay=overlay), context)

    def test_empty_schema_one_stays_rejected(self) -> None:
        fixture = IdentityFixture(self)
        patch = fixture.output / "empty.patch"
        _ = patch.write_bytes(b"")
        manifest = fixture.output / "overlay.json"
        _ = manifest.write_text(
            json.dumps(
                {
                    "schema_version": 1,
                    "product_base_sha": fixture.psha,
                    "observer_patch_sha256": file_hash(patch),
                    "allowed_paths": [],
                    "files": [],
                }
            )
        )
        with self.assertRaisesRegex(ValueError, "empty or duplicate paths"):
            _ = overlay_files(fixture.spec(Overlay(patch, file_hash(patch), manifest, file_hash(manifest))))

    def test_identity_validator_green(self) -> None:
        fixture = IdentityFixture(self)
        from generic_identity_overlay import (
            identity_namespace,
            selected_roles,
            validate_identity,
        )

        harness = Node(freeze_source(fixture.harness, fixture.hsha))
        product = Node(freeze_source(fixture.product, fixture.psha))
        census = identity_namespace(harness, product)
        roles = selected_roles(fixture.harness, harness)
        manifest = Node(
            {
                "schema_version": 2,
                "kind": "whole_gate_identity.v1",
                "harness_root": str(fixture.harness),
                "harness_sha": fixture.hsha,
                "product_base_sha": fixture.psha,
                "allowed_paths": [],
                "files": [],
                "identity_namespace": census,
            }
        )
        selected = Node(
            {
                "harness_sha": fixture.hsha,
                "product_sha": fixture.psha,
                "qualification": "PRODUCT_UNQUALIFIED",
                "files": roles,
            }
        )
        validate_identity(fixture.harness, harness, product, manifest, selected)
        self.assertEqual((fixture.product / fixture.gameplay).read_text(), "const HUT_LOGS: u8 = 13;\n")

    def test_missing_changed_extra_mode_or_symlink_compatible_source_rejected(self) -> None:
        fixture = IdentityFixture(self)
        from generic_identity_overlay import identity_namespace

        h = Node(freeze_source(fixture.harness, fixture.hsha))
        original = freeze_source(fixture.product, fixture.psha)
        for defect in ("missing", "changed", "extra", "mode", "symlink"):
            value = document(json.dumps(original)).mapping()
            rows = Node(value["tracked_manifest"]).mapping()
            fields = Node(rows["tools/v8/observer.gd"]).value
            assert isinstance(fields, list)
            name = "tools/v8/observer.gd"
            if defect == "missing":
                del rows[name]
            elif defect == "changed":
                fields[2] = "0" * 64
            elif defect == "extra":
                rows["tools/v8/extra.py"] = rows[name]
            elif defect == "mode":
                fields[0] = "100755"
            else:
                fields[0] = "120000"
            with self.subTest(defect=defect), self.assertRaises(ValueError):
                _ = identity_namespace(h, Node(value))

    def test_census_and_role_omissions_duplicates_and_wrong_identity_rejected(self) -> None:
        fixture = IdentityFixture(self)
        from generic_identity_overlay import validate_identity

        spec, context = fixture.identity()
        assert spec.overlay is not None
        h, p = document(context.harness_json), document(context.product_json)
        original = document(spec.overlay.manifest.read_text()).mapping()
        selected = document(context.selected_path.read_text()).mapping()
        for defect in (
            "missing_census",
            "duplicate_census",
            "missing_role",
            "extra_role",
            "duplicate_role",
            "H_SHA",
            "P_SHA",
            "H_root",
        ):
            m, r = document(json.dumps(original)).mapping(), document(json.dumps(selected)).mapping()
            census = Node(m["identity_namespace"]).value
            roles = Node(r["files"]).value
            assert isinstance(census, list) and isinstance(roles, list)
            if defect == "missing_census":
                _ = census.pop()
            elif defect == "duplicate_census":
                census.append(census[0])
            elif defect == "missing_role":
                _ = roles.pop()
            elif defect in ("extra_role", "duplicate_role"):
                roles.append(dict(Node(roles[0]).mapping()))
                if defect == "extra_role":
                    Node(roles[-1]).mapping()["path"] = "tools/v8/extra.py"
            elif defect == "H_SHA":
                m["harness_sha"] = "0" * 40
            elif defect == "P_SHA":
                m["product_base_sha"] = "0" * 40
            else:
                m["harness_root"] = str(fixture.product)
            with self.subTest(defect=defect), self.assertRaises(ValueError):
                validate_identity(fixture.harness, h, p, Node(m), Node(r))

    def test_selected_mode_and_late_overlay_hash_changes_reject(self) -> None:
        fixture = IdentityFixture(self)
        spec, context = fixture.identity()
        assert spec.overlay is not None
        context.selected_path.chmod(context.selected_mode ^ 0o100)
        with self.assertRaisesRegex(ValueError, "external input changed"):
            _ = overlay_files(spec, context)
        context.selected_path.chmod(context.selected_mode)
        overlay = replace(spec.overlay, patch_sha256="f" * 64)
        with self.assertRaisesRegex(ValueError, "artifact hash mismatch"):
            _ = overlay_files(replace(spec, overlay=overlay), context)

    def test_existing_nonempty_overlay_stays_strict_without_build(self) -> None:

        from test_generic_build import BuildContractTests

        fixture = BuildContractTests()
        fixture.setUp()
        self.addCleanup(fixture.doCleanups)
        overlay = fixture.make_overlay()
        with self.assertRaisesRegex(ValueError, "artifact hash mismatch"):
            _ = overlay_files(replace(fixture.spec, overlay=replace(overlay, patch_sha256="0" * 64)))
        changed = overlay_files(replace(fixture.spec, overlay=overlay))
        self.assertEqual(set(changed), {"tools/v8/input.txt"})
        self.assertEqual((fixture.root / "tools/v8/input.txt").read_text(), "observer-derived")
