# /// script
# requires-python = ">=3.11"
# dependencies = []
# ///
# Run: python3 -m unittest test_generic_whole_gate_source
"""Tiny actual Git/overlay materialization; no Cargo or gameplay execution."""
import json
import os
import sys
import unittest
from dataclasses import replace
from pathlib import Path

import test_generic_cli_prepare as fixtures
from generic_build import Overlay
from generic_cli_prepare import prepare_cli
from generic_gate import document
from mux2_gate_inputs import Node

from generic_process import file_hash
from generic_whole_gate_contract import load_session
from generic_whole_gate_source import (
    check_materialization,
    materialize,
    prepare_session,
)


class MaterializationTests(unittest.TestCase):
    def test_real_closed_selector_materializes_without_product_authority(self) -> None:
        fixture = fixtures.CliPrepareTests()
        fixture.setUp()
        self.addCleanup(fixture.doCleanups)
        selected = fixture.selected()
        overlay = fixture.fixture.make_overlay()
        home = selected.output.parent
        bindings = home/'selected-tools'
        bindings.mkdir()
        python = Path(sys.executable).resolve()
        for name, target in (('python3', python), ('cargo', selected.cargo), ('rustc', selected.rustc)):
            (bindings/name).symlink_to(target)
        row = document(overlay.manifest.read_text()).field('files').array()[0]
        role = home/'roles.json'
        _ = role.write_text(json.dumps({'schema':'whole_gate_layers.v1', 'product_sha':selected.base.source_sha,
            'harness_sha':selected.harness_sha,'files':[{'path':row.field('path').text(),'role':'O',
                'post_sha256':row.field('observer_sha256').text(),'qualification':'NOT_RUN_TEST_ONLY'}]}))
        from generic_cli_prepare import ArmSelector
        from mux2_gate_inputs import Json
        def arm(value: ArmSelector) -> dict[str, Json]:
            return {'root':str(value.root),'source_sha':value.source_sha,'cargo_home':str(value.cargo_home),
                'configs':[],'environment':[list(pair) for pair in value.environment],
                'overlay':{'patch':str(overlay.patch),'patch_sha256':overlay.patch_sha256,
                           'manifest':str(overlay.manifest),'manifest_sha256':overlay.manifest_sha256}}
        from generic_whole_gate_fixture_inputs import LEGACY_FILES, fixture_paths
        legacy, v10 = home/'legacy-fixture', home/'v10-fixture'
        v10.mkdir()
        for name in LEGACY_FILES:
            target = legacy/name
            target.parent.mkdir(parents=True, exist_ok=True)
            _ = target.write_text('synthetic provenance, not product evidence')
        for index in range(4):
            _ = (v10/f'{index}.save').write_bytes(b'WORLDSIM'+(10).to_bytes(4, 'little'))
        manifest = v10/'manifest.json'
        _ = manifest.write_text(json.dumps({item.name:file_hash(item) for item in v10.iterdir()}))
        fixture_environment = {
            'WORLDSIM_MUX_FIXTURES': str(legacy),
            'WORLDSIM_B5_LEGACY9': str(legacy/'b4/h-format9-fixture/tick4000.save'),
            'MUX2_V10_FIXTURES': str(v10), 'MUX2_V10_MANIFEST': str(manifest)}
        fixture_inputs = {str(item): file_hash(item) for item in fixture_paths(fixture_environment)}
        bash = Path('/bin/bash').resolve()
        config = home/'whole-gate.json'
        _ = config.write_text(json.dumps({'schema':'whole_gate.v1','selectors':{
            'harness_root':str(selected.harness_root),'harness_sha':selected.harness_sha,
            'published_sha':selected.published_sha,'purpose':'self_proof',
            'base':arm(selected.base),'candidate':arm(selected.candidate),
            'cargo':{'name':'cargo','path':str(selected.cargo),'sha256':selected.cargo_sha256},
            'rustc':{'name':'rustc','path':str(selected.rustc),'sha256':selected.rustc_sha256}},
            'output':str(selected.output),'layers':[str(role),str(role)],
            'python':{'path':str(python),'sha256':file_hash(python)},
            'bash':{'path':str(bash),'sha256':file_hash(bash)},
            'environment':{'PATH':str(bindings)+os.pathsep+'/usr/bin:/bin','GODOT_BIN':str(selected.cargo), **fixture_environment},
            'external_inputs':{str(selected.cargo):selected.cargo_sha256, **fixture_inputs},'acceptance':{},'observations':{'probes':[]}}))
        session_path = prepare_session(config)
        admitted = load_session(session_path,file_hash(session_path))
        admitted.field('qualification').equals('PRODUCT_UNQUALIFIED')
        for name, value in fixture_environment.items():
            admitted.field('environment').field(name).equals(value)
        from generic_whole_gate_fixture_inputs import validate_fixture_inputs
        with self.assertRaises(ValueError):
            validate_fixture_inputs({'WORLDSIM_MUX_FIXTURES': str(legacy)}, fixture_inputs)
        missing_input = dict(fixture_inputs)
        del missing_input[str(manifest)]
        with self.assertRaises(ValueError):
            validate_fixture_inputs(fixture_environment, missing_input)
        _ = (legacy/'base/source-sha256.json').write_text('changed original provenance')
        with self.assertRaises(ValueError):
            validate_fixture_inputs(fixture_environment, fixture_inputs)
        self.assertFalse((selected.base.root/'rust/target').exists())
        changed = admitted.mapping()
        changed['qualification'] = 'PRODUCT_QUALIFIED'
        _ = session_path.write_text(json.dumps(changed))
        with self.assertRaises(ValueError):
            _ = load_session(session_path,file_hash(session_path))

    def test_actual_independent_products_remain_unqualified_and_mutation_rejected(self) -> None:
        fixture = fixtures.CliPrepareTests()
        fixture.setUp()
        self.addCleanup(fixture.doCleanups)
        selection = fixture.selected()
        # Existing test helper writes only a tiny tooling overlay in its private temp Git repo.
        _ = fixture.fixture.make_overlay()
        home = selection.output.parent
        patch = home/'overlay.patch'
        manifest = home/'overlay.json'
        overlay = Overlay(patch, file_hash(patch), manifest, file_hash(manifest))
        selection = replace(selection, base=replace(selection.base, overlay=overlay),
                            candidate=replace(selection.candidate, overlay=overlay))
        prepared = prepare_cli(selection)
        role = home/'roles.json'
        row = document(manifest.read_text()).field('files').array()[0]
        _ = role.write_text(json.dumps({'schema':'whole_gate_layers.v1',
            'product_sha':selection.base.source_sha, 'harness_sha':selection.harness_sha,
            'files':[{'path':row.field('path').text(),'role':'O',
                      'post_sha256':row.field('observer_sha256').text(),
                      'qualification':'NOT_RUN_TEST_ONLY'}]}))
        result = materialize(prepared,(role,role))
        result.field('build_status').equals('NOT_RUN')
        result.field('observer_semantics').equals('REQUIRES_SEPARATE_QUALIFICATION')
        receipt = selection.output/'materialization.json'
        session = Node({'arms':result.field('arms').value,'declarations':result.field('declarations').value,
            'binding': result.field('binding').value,
            'harness':{'root':str(selection.harness_root),'source_sha':selection.harness_sha},
            'materialization':{'path':str(receipt),'sha256':file_hash(receipt)}})
        check_materialization(session)
        self.assertFalse((selection.base.root/'rust/target').exists())
        self.assertFalse((selection.candidate.root/'rust/target').exists())
        _ = (selection.base.root/'tools/v8/input.txt').write_text('late mutation')
        with self.assertRaises(ValueError):
            check_materialization(session)

    def test_role_cannot_reclassify_arbitrary_observer_as_harness(self) -> None:
        fixture = fixtures.CliPrepareTests()
        fixture.setUp()
        self.addCleanup(fixture.doCleanups)
        selection = fixture.selected()
        _ = fixture.fixture.make_overlay()
        home = selection.output.parent
        patch = home/'overlay.patch'
        manifest = home/'overlay.json'
        overlay = Overlay(patch,file_hash(patch),manifest,file_hash(manifest))
        selection = replace(selection,base=replace(selection.base,overlay=overlay),
                            candidate=replace(selection.candidate,overlay=overlay))
        row = document(manifest.read_text()).field('files').array()[0]
        role = home/'roles.json'
        _ = role.write_text(json.dumps({'schema':'whole_gate_layers.v1',
            'product_sha':selection.base.source_sha,'harness_sha':selection.harness_sha,
            'files':[{'path':row.field('path').text(),'role':'H',
                      'post_sha256':row.field('observer_sha256').text()}]}))
        with self.assertRaises(ValueError):
            _ = materialize(prepare_cli(selection),(role,role))
        self.assertEqual((selection.base.root/'tools/v8/input.txt').read_text(),'tools/v8')


if __name__ == '__main__':
    _ = unittest.main()
