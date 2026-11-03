"""Shared overlay namespace and read-only identity-validation inputs."""

from pathlib import Path
from typing import Final, Protocol

HARNESS_EXACT_PATHS: Final = frozenset({'tools/play.sh'})


def harness_tool(name: str) -> bool:
    return name in HARNESS_EXACT_PATHS or (name.startswith('tools/v8/') and
        (name.endswith(('.py', '.sh')) or name == 'tools/v8/gate_f10_no_kill.json'))


# These are the reviewed full-suite and normal-boot lifecycle seams, not a
# blanket permission to overlay gameplay scripts or arbitrary integration tests.
OBSERVER_EXACT_PATHS: Final = frozenset(
    {
        "scripts/core/main_bootstrap.gd",
        "rust/crates/sim-test/tests/save_load.rs",
        "rust/crates/sim-test/tests/save_gate_observation/mod.rs",
        "rust/crates/sim-test/tests/colony_world.rs",
        "rust/crates/sim-test/tests/colony_commands.rs",
        "rust/crates/sim-test/tests/colony_jobs.rs",
        "rust/crates/sim-test/tests/colony_items.rs",
        "rust/crates/sim-test/tests/colony_steward.rs",
        "rust/crates/sim-test/tests/colony_steward_housing/construction.rs",
        "rust/crates/sim-test/tests/steward_defence.rs",
        "rust/crates/sim-test/tests/legacy_migration_current.rs",
        "rust/crates/sim-test/tests/steward_defence_migration.rs",
        "rust/crates/sim-test/tests/b5_flora.rs",
        "rust/crates/sim-test/tests/engine_appearance_ffi.rs",
        "rust/crates/sim-test/tests/engine_ascii.rs",
        "rust/crates/sim-test/tests/engine_knowledge_ffi.rs",
        "rust/crates/sim-test/tests/engine_colony_ffi.rs",
        "rust/crates/sim-test/tests/m6b_presentation_defaults.rs",
        "rust/crates/sim-test/tests/save_gate_observation/external.rs",
    }
)


class OverlayInput(Protocol):
    @property
    def patch(self) -> Path: ...
    @property
    def patch_sha256(self) -> str: ...
    @property
    def manifest(self) -> Path: ...
    @property
    def manifest_sha256(self) -> str: ...


class IdentityInput(Protocol):
    @property
    def root(self) -> Path: ...
    @property
    def source_sha(self) -> str: ...
    @property
    def overlay(self) -> OverlayInput | None: ...
