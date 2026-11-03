"""Private tiny Git identity fixtures; never executes native builds."""

import json
import tempfile
import unittest
from pathlib import Path
from typing import final

import generic_identity_overlay as identity_module
from generic_build import BuildSpec, Overlay
from generic_identity_overlay import IdentityContext
from generic_process import file_hash
from generic_sources import freeze_binding, freeze_source, git
from mux2_gate_inputs import Node


def write_identity(harness: Path, product: Path, output: Path) -> tuple[Overlay, Path]:
    """Make an explicit private fixture manifest; caller still must authenticate it."""
    hsha = git(harness, "rev-parse", "HEAD").decode().strip()
    psha = git(product, "rev-parse", "HEAD").decode().strip()
    h, p = Node(freeze_source(harness, hsha)), Node(freeze_source(product, psha))
    selected = output / "selected-source.json"
    _ = selected.write_text(
        json.dumps(
            {
                "harness_sha": hsha,
                "product_sha": psha,
                "qualification": "PRODUCT_UNQUALIFIED",
                "files": identity_module.selected_roles(harness, h),
            }
        )
    )
    patch = output / "overlay.patch"
    _ = patch.write_bytes(b"")
    manifest = output / "overlay.json"
    _ = manifest.write_text(
        json.dumps(
            {
                "schema_version": 2,
                "kind": "whole_gate_identity.v1",
                "harness_root": str(harness),
                "harness_sha": hsha,
                "product_base_sha": psha,
                "observer_patch_sha256": file_hash(patch),
                "allowed_paths": [],
                "files": [],
                "identity_namespace": identity_module.identity_namespace(h, p),
                "selected_source": {
                    "path": str(selected),
                    "sha256": file_hash(selected),
                    "mode": selected.lstat().st_mode,
                },
            }
        )
    )
    role = output / "roles.json"
    _ = role.write_text(
        json.dumps({"schema": "whole_gate_layers.v1", "product_sha": psha, "harness_sha": hsha, "files": []})
    )
    return Overlay(patch, file_hash(patch), manifest, file_hash(manifest)), role


@final
class IdentityFixture:
    def __init__(self, owner: unittest.TestCase) -> None:
        temporary = tempfile.TemporaryDirectory()
        owner.addCleanup(temporary.cleanup)
        self.home = Path(temporary.name).resolve()
        self.product = self.home / "product"
        self.product.mkdir()
        _ = git(self.product, "init", "-q")
        for name in identity_module.EXPLICIT:
            path = self.product / name
            path.parent.mkdir(parents=True, exist_ok=True)
            _ = path.write_text("{}\n" if name.endswith(".json") else "# harmless fixture\n")
        observer = self.product / "tools/v8/observer.gd"
        _ = observer.write_text("# observer fixture\n")
        matrix = {
            "schema": "whole_gate_observer_matrix.v1",
            "observer_postimages": {"tools/v8/observer.gd": ["100644", file_hash(observer)]},
        }
        _ = (self.product / "tools/v8/whole_gate_observer_matrix.json").write_text(json.dumps(matrix))
        self.gameplay = "rust/crates/sim-core/src/rules/colony.rs"
        path = self.product / self.gameplay
        path.parent.mkdir(parents=True)
        _ = path.write_text("const HUT_LOGS: u8 = 12;\n")
        _ = (self.product / "tools/v8/bench_script.json").write_text("[]\n")
        self.hsha = self.commit()
        self.harness = self.home / "harness"
        _ = git(self.product, "clone", "-q", str(self.product), str(self.harness))
        _ = path.write_text("const HUT_LOGS: u8 = 13;\n")
        self.psha = self.commit()
        self.output = self.home / "overlay"
        self.output.mkdir()

    def commit(self) -> str:
        _ = git(self.product, "add", ".")
        _ = git(
            self.product,
            "-c",
            "user.name=Fixture",
            "-c",
            "user.email=fixture@example.invalid",
            "commit",
            "-qm",
            "fixture",
        )
        return git(self.product, "rev-parse", "HEAD").decode().strip()

    def spec(self, overlay: Overlay) -> BuildSpec:
        return BuildSpec(
            self.product,
            self.psha,
            self.home / "unused-cargo",
            "0" * 64,
            self.home / "unused-rustc",
            "0" * 64,
            self.home / "unused-cache",
            self.home / "unused-build",
            (),
            (),
            overlay=overlay,
        )

    def identity(self) -> tuple[BuildSpec, IdentityContext]:
        from generic_identity_overlay import IdentityContext

        _ = git(self.harness, "fetch", "-q", str(self.product), self.psha)
        _ = git(self.harness, "update-ref", "refs/remotes/origin/v8/main", self.hsha)
        overlay, _ = write_identity(self.harness, self.product, self.output)
        selected = self.output / "selected-source.json"
        role = self.output / "roles.json"
        context = IdentityContext(
            json.dumps(freeze_source(self.harness, self.hsha)),
            json.dumps(freeze_source(self.product, self.psha)),
            json.dumps(freeze_binding(self.harness, self.hsha, self.hsha, "defect_proof", candidate_sha=self.psha)),
            role,
            file_hash(role),
            role.lstat().st_mode,
            selected,
            file_hash(selected),
            selected.lstat().st_mode,
        )
        return self.spec(overlay), context
