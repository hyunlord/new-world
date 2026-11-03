# /// script
# requires-python = ">=3.11"
# dependencies = []
# ///
# Run: python3 -m unittest test_generic_ui_fixed_run
"""Fixed UI lifecycle policy independent of transport and process authority."""
import json
from dataclasses import dataclass
from typing import Final, Protocol

from generic_gate import require

from generic_ui_receipts import CheckpointPlan, InstancePlan

NO_NATIVE: Final = 'geometry and synthetic marker DTOs; SubViewport/World2D are not WorldSim'
BOOT_SEED: Final = 20261010


@dataclass(frozen=True, slots=True)
class Lifecycle:
    checkpoints: tuple[CheckpointPlan, ...]
    instances: tuple[tuple[InstancePlan, ...], ...]
    classification: str
    no_native_reason: str


class CommandPlan(Protocol):
    @property
    def producer(self) -> str: ...
    @property
    def checkpoints(self) -> tuple[CheckpointPlan, ...]: ...
    @property
    def instances(self) -> tuple[tuple[InstancePlan, ...], ...]: ...


def lifecycle_plan(producer: str, bootstrap_tick: int) -> Lifecycle:
    """Source-owned fixed fixture semantics independent of process transport."""
    require(producer in {'toast', 'marker.layout', 'render.ground', 'render.people', 'render.trees'},
            '/producer', 'unsupported fixed UI fixture')
    require(type(bootstrap_tick) is int and bootstrap_tick >= 0, '/bootstrap', 'invalid horizon')
    checkpoints: list[CheckpointPlan] = [CheckpointPlan('bootstrap', 0, 1, 0)]
    instances = [InstancePlan('bootstrap', BOOT_SEED, 0, bootstrap_tick)]
    if producer == 'marker.layout':
        require(bootstrap_tick == 0, '/layout', 'synthetic fixture has no native horizon')
        return Lifecycle((), (), 'synthetic_renderer_only', NO_NATIVE)
    if producer == 'toast':
        require(bootstrap_tick == 0, '/toast', 'toast must remain at zero ticks')
    else:
        checkpoints.append(CheckpointPlan('before_colony_reset', 0, 1, bootstrap_tick))
        kinds = ('reset', 'stress_reset', 'reset') if producer == 'render.trees' else ('reset',)
        for ordinal, reason in enumerate(kinds, 2):
            stress = reason == 'stress_reset'
            if ordinal > 2:
                checkpoints.append(CheckpointPlan('before_stress_reset' if stress else 'before_colony_reset', 0, ordinal-1, 1))
            label = 'after_stress_reset' if stress else 'after_colony_reset'
            context = json.dumps({'fixture_classification': 'stress_render' if stress else 'ordinary_colony'})
            checkpoints.extend((CheckpointPlan(label, 0, ordinal, 0, context),
                                CheckpointPlan('before_explicit_tick', 0, ordinal, 0),
                                CheckpointPlan('after_explicit_tick', 0, ordinal, 1)))
            instances.append(InstancePlan(reason, 0, 0, 1))
    checkpoints.append(CheckpointPlan('terminal', 0, len(instances), instances[-1].terminal_tick, releases_node=True))
    return Lifecycle(tuple(checkpoints), (tuple(instances),), 'native_world_producer', '')
