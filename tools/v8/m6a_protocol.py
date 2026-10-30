"""Explicit, immutable evaluation protocols; retired evidence stays legacy."""
from dataclasses import dataclass
from typing import Final

from m6a_evidence import EvidenceError, Node

R1_DEADLINE: Final = 1793377131  # Owner R1: 2026-10-30T16:18:51Z.
# R2 brief 2026-10-30: hard stop 14 h after the R2 start check (21:38 KST) = 2026-10-31T02:38:00Z.
R2_DEADLINE: Final = 1793414280
# Repair rounds share the fresh-seed, BASE-column and deadline-bound rules.
REPAIRS: Final = ('repair_r1', 'repair_r2')


@dataclass(frozen=True, slots=True)
class Protocol:
    name: str
    first_seed: int
    rounds: int
    deadline: int

    @property
    def heldout(self) -> list[int]:
        return list(range(self.first_seed, self.first_seed + 32))

    @property
    def freeze_kind(self) -> str:
        return {'repair_r1': 'm6a_freeze_r1_v1', 'repair_r2': 'm6a_freeze_r2_v1'}.get(self.name, 'm6a_freeze_v1')


def protocol(name: str = 'legacy') -> Protocol:
    match name:
        case 'legacy':
            return Protocol(name, 1296, 4, 1793335103)
        case 'repair_r1':
            return Protocol(name, 1328, 2, R1_DEADLINE)
        case 'repair_r2':
            # R2 brief: calibration rounds are unlimited (every round logged); fresh held-out 1360-1391.
            return Protocol(name, 1360, 1_000_000, R2_DEADLINE)
        case _:
            raise EvidenceError('unsupported M6a evaluation protocol: ' + name)


def recorded(node: Node) -> Protocol:
    fields = dict(node.fields())
    return protocol(fields['evaluation_protocol'].text() if 'evaluation_protocol' in fields else 'legacy')


def check_freeze(node: Node, spec: Protocol) -> None:
    if (node.get('kind').text() != spec.freeze_kind or recorded(node).name != spec.name
            or node.get('days').integer() != 72
            or [seed.integer() for seed in node.get('heldout_seeds').rows()] != spec.heldout):
        raise EvidenceError('freeze evaluation protocol differs')
