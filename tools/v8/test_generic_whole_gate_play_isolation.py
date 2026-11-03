"""Pre-native target/HOME checks and real owner FD retention under diagnostic failure."""
import os
import subprocess
import sys
import unittest
from pathlib import Path
from tempfile import TemporaryDirectory
from unittest.mock import patch

import gate_oracle_child
from generic_gate import GateContractError
from generic_play_owner import PlayOwner, PlayOwnershipUnknown
from mux2_gate_inputs import EvidenceError, Node
from test_generic_play_owner import OwnerTests

from generic_play_sources import validate_build_target
from generic_whole_gate_play import run
from generic_whole_gate_play_plan import explicit_home


class PauseReached(BaseException):
    pass


class IsolationTests(unittest.TestCase):
    def test_target_projection_is_original_worktree_and_foreign_arm_rejected(self) -> None:
        with TemporaryDirectory() as folder:
            base=Path(folder).resolve();arm=base/'admitted-arm/rust/target'
            for version in ('main','m1','v7'):
                cwd=base/'play-roots'/version/'rust';cwd.mkdir(parents=True)
                validate_build_target(Node({'CARGO_TARGET_DIR':'target'}),cwd)
                validate_build_target(Node({'CARGO_TARGET_DIR':str(cwd/'target')}),cwd)
                for target in (str(arm),'../../../../admitted-arm/rust/target',''):
                    with self.subTest(version=version,target=target),self.assertRaises(GateContractError):
                        validate_build_target(Node({'CARGO_TARGET_DIR':target}),cwd)
                (cwd/'target').symlink_to(arm,target_is_directory=True)
                with self.assertRaises(GateContractError):validate_build_target(Node({'CARGO_TARGET_DIR':'target'}),cwd)

    def test_real_tiny_child_resolves_projected_target_without_touching_admitted_bridge(self) -> None:
        with TemporaryDirectory() as folder:
            base=Path(folder).resolve();admitted=base/'arm/rust/target/debug/libsim_bridge.dylib'
            admitted.parent.mkdir(parents=True);_=admitted.write_bytes(b'admitted original')
            script="import os,pathlib;p=pathlib.Path(os.environ['CARGO_TARGET_DIR'])/'debug/libsim_bridge.dylib';p.parent.mkdir(parents=True);p.write_bytes(b'fixture build output')"
            for version in ('main','m1','v7'):
                cwd=base/'play-roots'/version/'rust';cwd.mkdir(parents=True)
                environment={'CARGO_TARGET_DIR':'target'}
                validate_build_target(Node({'CARGO_TARGET_DIR':'target'}),cwd)
                _=subprocess.run([str(Path(sys.executable).resolve()),'-c',script],cwd=cwd,env=environment,check=True,capture_output=True)
                self.assertEqual((cwd/'target/debug/libsim_bridge.dylib').read_bytes(),b'fixture build output')
                self.assertEqual(admitted.read_bytes(),b'admitted original')

    def test_home_requires_exact_explicit_session_path_not_ambient(self) -> None:
        with TemporaryDirectory() as folder:
            home=Path(folder).resolve();session=Node({'environment':{'HOME':str(home)}})
            self.assertEqual(explicit_home(session,{'HOME':str(home)}),home)
            with patch.dict(os.environ,{'HOME':str(home)}),self.assertRaises(GateContractError):
                _=explicit_home(session,{})
            for value in ('relative',str(home/'missing'),str(home.parent)):
                with self.subTest(value=value),self.assertRaises((GateContractError,EvidenceError)):
                    _=explicit_home(session,{'HOME':value})
            with self.assertRaises(EvidenceError):_=explicit_home(Node({'environment':{}}),{'HOME':str(home)})
            link=home/'alias';link.symlink_to(home,target_is_directory=True)
            with self.assertRaises(GateContractError):_=explicit_home(Node({'environment':{'HOME':str(link)}}),{'HOME':str(link)})

    def test_diagnostic_io_failure_still_reaches_pause_with_actual_owner_fd(self) -> None:
        with TemporaryDirectory() as folder:
            base=Path(folder).resolve();plan=OwnerTests().fixture(base)
            lock=base/'lane';lock.touch()
            with lock.open('r') as held,patch.object(gate_oracle_child,'LOCK',lock):
                owner=PlayOwner(plan,held.fileno())
                identity=os.fstat(owner.lane_fd)
                def pause() -> None:
                    actual=os.fstat(owner.lane_fd)
                    self.assertEqual((actual.st_dev,actual.st_ino),(identity.st_dev,identity.st_ino))
                    self.assertFalse(owner.closed)
                    raise PauseReached()
                try:
                    with patch('generic_whole_gate_play.build_plan',return_value=plan),patch('generic_whole_gate_play.PlayOwner',return_value=owner),patch.object(owner,'launch',side_effect=PlayOwnershipUnknown(owner,'fixture without children')),patch('generic_whole_gate_play.atomic_json',side_effect=OSError('injected disk full')),patch('generic_whole_gate_play.signal.pause',side_effect=pause),self.assertRaises(PauseReached):
                        _=run(base/'session','fixture','candidate',base,held.fileno())
                    _=os.fstat(owner.lane_fd)
                finally:
                    os.close(owner.lane_fd)


if __name__=='__main__':_=unittest.main()
