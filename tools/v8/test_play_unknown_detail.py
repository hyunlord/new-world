"""Private FD and source-only owner construction; no child or native launch."""
import os
import tempfile
import unittest
from pathlib import Path
from typing import override
from unittest.mock import patch

import gate_oracle_child
import test_generic_play_owner as owner_fixtures
from generic_gate import GateContractError, document
from generic_play_owner import PlayOwner, PlayOwnershipUnknown


class UnknownDetailTests(unittest.TestCase):
    def test_bounded_cause_preserves_unknown_and_actual_fd(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            root=Path(directory).resolve();plan=owner_fixtures.OwnerTests().fixture(root)
            lock=root/'private.lock';lock.touch()
            with lock.open('r') as held,patch.object(gate_oracle_child,'LOCK',lock):
                owner=PlayOwner(plan,held.fileno());plan.output.mkdir()
                error=GateContractError('CONTRACT_REJECTED','/coverage','missing original children '+('x'*3000))
                try:
                    with self.assertRaises(PlayOwnershipUnknown):owner.retain_failure(error)
                    row=document((plan.output/'unknown.json').read_text())
                    row.field('settlement').equals('UNKNOWN');row.field('lane_retained').equals(True)
                    self.assertEqual(row.field('cause_detail').text(),str(error)[:2048])
                    self.assertEqual(os.fstat(owner.lane_fd).st_ino,lock.stat().st_ino)
                    self.assertFalse(owner.closed)
                    self.assertIsNone(owner.shell)
                finally:os.close(owner.lane_fd)

    def test_unprintable_cause_does_not_escape_ownership(self) -> None:
        class Unprintable(RuntimeError):
            @override
            def __str__(self) -> str:
                raise TypeError('cause unavailable')
        with tempfile.TemporaryDirectory() as directory:
            root=Path(directory).resolve();plan=owner_fixtures.OwnerTests().fixture(root)
            lock=root/'private.lock';lock.touch()
            with lock.open('r') as held,patch.object(gate_oracle_child,'LOCK',lock):
                owner=PlayOwner(plan,held.fileno());plan.output.mkdir()
                try:
                    with self.assertRaises(PlayOwnershipUnknown):owner.retain_failure(Unprintable())
                    row=document((plan.output/'unknown.json').read_text())
                    row.field('settlement').equals('UNKNOWN')
                    row.field('cause_detail').equals('cause text unavailable: TypeError')
                    self.assertIsInstance(owner.diagnostic_error,TypeError)
                    self.assertFalse(owner.closed)
                finally:os.close(owner.lane_fd)


if __name__=='__main__':
    _ = unittest.main()
