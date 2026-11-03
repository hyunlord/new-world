"""Real temporary Git H-only launcher overlays; never builds or launches play."""
import json
import unittest
from dataclasses import replace
from pathlib import Path

import test_generic_cli_prepare as fixtures
from generic_build import Overlay
from generic_cli_prepare import PreparedCli, prepare_cli
from generic_gate import GateContractError
from generic_play_harness import check_launcher
from generic_process import file_hash
from generic_sources import git
from generic_whole_gate_source import check_materialization, materialize
from mux2_gate_inputs import Json, Node


class LauncherMaterializationTests(unittest.TestCase):
    def prepared(self, role: str = 'H') -> tuple[PreparedCli, Path]:
        fixture = fixtures.CliPrepareTests();fixture.setUp();self.addCleanup(fixture.doCleanups)
        selection = fixture.selected();base=selection.base.root;harness=selection.harness_root
        launcher=base/'tools/play.sh';_ = launcher.write_text('#!/bin/bash\n# original fixture\n');launcher.chmod(0o755)
        _ = git(base,'add','tools/play.sh')
        _ = git(base,'-c','user.name=Fixture','-c','user.email=f@example.invalid','commit','-qm','original launcher')
        original=git(base,'rev-parse','HEAD').decode().strip()
        for root in (harness,selection.candidate.root):
            _ = git(root,'fetch','-q',str(base),original);_ = git(root,'checkout','--detach',original)
        _ = git(harness,'update-ref','refs/remotes/origin/v8/main',original)
        before=file_hash(launcher)
        selected=harness/'tools/play.sh';_ = selected.write_text('#!/bin/bash\n# H protocol fixture\n')
        _ = git(harness,'add','tools/play.sh')
        _ = git(harness,'-c','user.name=Fixture','-c','user.email=f@example.invalid','commit','-qm','H launcher')
        hsha=git(harness,'rev-parse','HEAD').decode().strip();after=file_hash(selected)
        patch=base.parent/'play.patch';_ = patch.write_bytes(git(harness,'diff','--binary','--full-index',original,hsha,'--','tools/play.sh'))
        manifest=base.parent/'play-overlay.json'
        _ = manifest.write_text(json.dumps({'schema_version':1,'product_base_sha':original,
            'observer_patch_sha256':file_hash(patch),'allowed_paths':['tools/play.sh'],
            'files':[{'path':'tools/play.sh','operation':'modify','base0_sha256':before,'observer_sha256':after}]}))
        overlay=Overlay(patch,file_hash(patch),manifest,file_hash(manifest))
        roles=base.parent/'play-roles.json'
        _ = roles.write_text(json.dumps({'schema':'whole_gate_layers.v1','product_sha':original,'harness_sha':hsha,
            'files':[{'path':'tools/play.sh','role':role,'post_sha256':after,'qualification':'NOT_RUN'}]}))
        selection=replace(selection,harness_sha=hsha,published_sha=original,
            base=replace(selection.base,source_sha=original,overlay=overlay),
            candidate=replace(selection.candidate,source_sha=original,overlay=overlay))
        return prepare_cli(selection),roles

    def test_exact_h_launcher_materializes_and_rechecks_without_build(self) -> None:
        prepared,roles=self.prepared()
        receipt=materialize(prepared,(roles,roles));harness=prepared.selectors.harness_root
        path=prepared.selectors.output/'materialization.json'
        inputs: dict[str,Json]={'tools/play.sh':file_hash(harness/'tools/play.sh')}
        session=Node({'arms':receipt.field('arms').value,'declarations':receipt.field('declarations').value,
            'binding':receipt.field('binding').value,'harness':{'root':str(harness),
            'source_sha':prepared.selectors.harness_sha,'inputs':inputs},
            'materialization':{'path':str(path),'sha256':file_hash(path)}})
        check_materialization(session)
        for arm in ('base','candidate'):
            check_launcher(session,arm)
            product=Path(session.field('arms').field(arm).field('root').text())
            self.assertFalse((product/'rust/target').exists())
        _ = (prepared.selectors.base.root/'tools/play.sh').write_text('late mutation')
        with self.assertRaises(ValueError):check_materialization(session)
        with self.assertRaises(GateContractError):check_launcher(session,'base')

    def test_exact_h_launcher_cannot_be_declared_observer(self) -> None:
        prepared,roles=self.prepared('O')
        with self.assertRaisesRegex(GateContractError,'cannot be classified as O'):
            _ = materialize(prepared,(roles,roles))


if __name__ == '__main__':
    _ = unittest.main()
