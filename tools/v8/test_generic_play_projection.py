# /// script
# requires-python = ">=3.11"
# dependencies = []
# ///
# Run: python3 -m unittest test_generic_play_projection
"""Tiny real projected child; only the source-owner grant is explicitly mocked."""
import json
import os
import tempfile
import unittest
from pathlib import Path
from unittest.mock import patch

from generic_gate import GateContractError, document
from generic_process import atomic_json, file_hash
from mux2_gate_inputs import Json

import gate_play_child as child
import gate_play_protocol as protocol
from generic_play_projection import PlayProjection


class ProjectionFixture:
    root: Path
    requested: Path
    exported: Path
    projection: PlayProjection
    selection: child.Selection
    argv: tuple[str,...]
    watched: Path

    def __init__(self, case: unittest.TestCase, mutate: bool = False) -> None:
        temp=tempfile.TemporaryDirectory();case.addCleanup(temp.cleanup)
        self.root=Path(temp.name).resolve()
        self.requested=self.root/'original';self.requested.mkdir()
        self.exported=self.root/'export';self.exported.mkdir()
        join=self.root/'build-join.json';_=join.write_text('explicit synthetic source grant fixture')
        self.watched=self.exported/'watched';_=self.watched.write_text('original')
        godot=self.root/'tiny-godot';_=godot.write_text('#!/bin/sh\npwd\n'+('printf drift > watched\n' if mutate else ''))
        godot.chmod(0o700)
        self.projection=PlayProjection('m1',self.requested,self.exported,'892edca1d31b26e535a3e9b62b37d62b99bb1130',
             join,file_hash(join),'0'*64,((self.watched,file_hash(self.watched)),))
        manifest=self.root/'projection.json';atomic_json(manifest,self.projection.declaration())
        self.argv=(str(godot),'--headless','--path','.', '--quit-after','300')
        role: dict[str,Json]={'argv':list(self.argv),'cwd':str(self.requested),'executable':str(godot),
              'executable_sha256':file_hash(godot),'inputs':{},'artifacts':[],'output':str(self.root/'child'),
              'timeout_seconds':600,'merged':True,'projection_manifest':str(manifest)}
        cases: dict[str,Json]={}
        for index,label in enumerate(child.CASES):
            if index in (1,2):cases[label]=None;continue
            plan=self.root/f'case{index}.json';_=plan.write_text(json.dumps({'schema_version':1,'roles':{'headless':role}}))
            cases[label]={'path':str(plan),'sha256':file_hash(plan)}
        suite=self.root/'suite.json';atomic_json(suite,{'schema_version':1,'cases':cases})
        self.selection=child.select_case(suite,file_hash(suite),'--headless-check m1','headless')

    def run(self) -> int:
        before=Path.cwd()
        try:
            os.chdir(self.requested)
            with patch.object(protocol,'enabled',return_value=True),patch.object(protocol,'child_request') as request,patch.object(protocol,'child_returned'):
                result=child.run(self.selection.plan,self.selection.plan_sha256,'headless',self.argv,self.selection)
                request.assert_called_once_with('headless',self.argv,self.selection.plan,self.selection.plan_sha256,self.projection.declaration() if result==0 else request.call_args.args[-1])
                return result
        finally:os.chdir(before)


class PlayProjectionTests(unittest.TestCase):
    def test_actual_child_uses_export_and_preserves_requested_identity(self) -> None:
        fixture=ProjectionFixture(self)
        self.assertEqual(fixture.run(),0)
        receipt=document((fixture.root/'child/receipt.json').read_text())
        receipt.field('cwd').equals(str(fixture.exported));receipt.field('requested_cwd').equals(str(fixture.requested))
        receipt.field('argv').equals(list(fixture.argv));receipt.field('requested_argv').equals(list(fixture.argv))
        receipt.field('native_exit').equals(0)
        self.assertEqual((fixture.root/'child/stdout.log').read_text().strip(),str(fixture.exported))
        self.assertEqual(list(fixture.requested.iterdir()),[])

    def test_actual_child_mutation_is_retained_failure(self) -> None:
        fixture=ProjectionFixture(self,True)
        self.assertEqual(fixture.run(),125)
        receipt=document((fixture.root/'child/receipt.json').read_text())
        receipt.field('native_exit').equals(0);receipt.field('launcher_exit').equals(125)
        self.assertTrue(receipt.field('errors').array())

    def test_projection_rejects_build_or_changed_workload(self) -> None:
        fixture=ProjectionFixture(self)
        with self.assertRaises(GateContractError):_=fixture.projection.command('build',fixture.argv,fixture.requested)
        with self.assertRaises(GateContractError):_=fixture.projection.command('headless',(*fixture.argv[:-1],'301'),fixture.requested)
