"""Synthetic policy/admission tests; these do not qualify GPU visibility."""
import json
import unittest
from pathlib import Path
from tempfile import TemporaryDirectory
from unittest.mock import patch

import generic_ui_visibility_run as visibility
from generic_fixture_run import Products
from generic_fixtures import ProbeProduct
from generic_gate import GateContractError, document
from generic_observations import Observation
from generic_process import ProcessResult, ProcessSpec, file_hash
from generic_ui_history import UIEvidence
from generic_ui_receipts import UIPlan
from mux2_gate_inputs import BRIDGE, Json, Node


def rows(arm: Path) -> tuple[Node,...]:
    return tuple(Node({'id':name,'day':day,'hour':hour,'tick':tick,'save':str(arm/'saves'/(name+'.wsave')),
                       'sha256':'a'*64}) for name,day,hour,tick in visibility.TARGETS)


def spec(arm: Path, generator: bool) -> ProcessSpec:
    return ProcessSpec(('/godot',),Path('/source'),Path('/godot'),'a'*64,Path('/source'),(),
                       arm/('generator-process' if generator else 'loader-process'))


def command_node(transcripts: tuple[visibility.Rows,...], ticks: tuple[int,...]) -> Node:
    cps: list[Json]=[]
    for transcript,tick in zip(transcripts,ticks):
        cps.append({'clock':{'tick':tick},'commands':{'total':len(transcript),'ticks':[r[0] for r in transcript],
                    'kinds':[r[1] for r in transcript],'accepted':[r[2] for r in transcript]}})
    return Node({'checkpoints':cps})


class PlanTests(unittest.TestCase):
    def test_loader_context_matches_native_json_number_representation(self) -> None:
        arm=Path('/external/base')
        entries=rows(arm)
        entries[0].mapping()['clock']={'tick':240,'winter_severity':-1,'time_of_day':12.0}
        with patch.object(visibility,'file_hash',return_value='b'*64):
            value=visibility.plan(spec(arm,False),entries,False)
        actual_row={'id':'day1_noon','day':1.0,'hour':12.0,'tick':240.0,
                    'save':'/external/base/saves/day1_noon.wsave','sha256':'a'*64,
                    'clock':{'tick':240.0,'winter_severity':-1.0,'time_of_day':12.0}}
        expected={'save_path':actual_row['save'],'save_sha256':'a'*64,
                  'manifest_path':'/external/base/saves/manifest.json','manifest_sha256':'b'*64,
                  'manifest_row':actual_row}
        def canonical(raw: str) -> str:
            return json.dumps(json.loads(raw),sort_keys=True,separators=(',',':'))
        self.assertEqual(canonical(value.checkpoints[1].context_json),canonical(json.dumps(expected)))
        self.assertEqual(canonical(value.checkpoints[4].context_json),canonical(json.dumps(actual_row)))
        self.assertEqual(canonical(value.checkpoints[2].context_json),
                         canonical(json.dumps({'zoom':0.5,'saved_tick':240.0,'scene_id':'day1_noon'})))

    def test_loader_projection_keeps_boolean_and_rejects_lossy_numbers(self) -> None:
        self.assertIs(visibility.loader_context(True),True)
        for number in (1 << 53,float('inf'),float('nan')):
            with self.assertRaises(GateContractError):
                _=visibility.loader_context(number)
        original: dict[str,Json]={'tick':240,'save':'/own/save','sha256':'a'*64}
        expected=json.dumps(visibility.loader_context(original),sort_keys=True)
        for key,value in (('tick',241),('tick',True),('save','/foreign/save'),('sha256','b'*64)):
            changed: dict[str,Json]=dict(original)
            changed[key]=value
            self.assertNotEqual(json.dumps(visibility.loader_context(changed),sort_keys=True),expected)

    def test_generator_retains_two_engines_and_all_prefix_links(self) -> None:
        arm=Path('/external/base')
        value=visibility.plan(spec(arm,True),rows(arm),True)
        self.assertEqual(len(value.checkpoints),11)
        self.assertEqual([(v.reason,v.initial_tick,v.terminal_tick) for v in value.instances[0]],
                         [('bootstrap',0,0),('reset',0,28200)])
        self.assertEqual([c.tick for c in value.checkpoints],[0,0,240,240,840,840,27600,27600,28200,28200,28200])
        self.assertTrue(value.checkpoints[-1].releases_node)

    def test_loader_retains_four_bootstraps_and_four_loaded_worlds(self) -> None:
        arm=Path('/external/base')
        with patch.object(visibility,'file_hash',return_value='b'*64):
            value=visibility.plan(spec(arm,False),rows(arm),False)
        self.assertEqual(len(value.checkpoints),20)
        self.assertEqual(len(value.instances),4)
        for slot,(_,_,_,tick) in enumerate(visibility.TARGETS):
            self.assertEqual([c.tick for c in value.checkpoints[slot*5:slot*5+5]],[0,tick,tick,tick,tick])
            self.assertTrue(value.checkpoints[slot*5+4].releases_node)
            self.assertEqual(value.instances[slot][1].reason,'load')

    def test_incomplete_scene_plan_rejected(self) -> None:
        with self.assertRaises(GateContractError):
            _=visibility.plan(spec(Path('/arm'),True),(),True)

    def test_generator_commands_only_factory_explicit_on_then_native_steward(self) -> None:
        boot=((0,'SetSteward',1),)
        prefix=boot*2+((3,'Steward',0),(4,'Steward',1))
        actual=visibility.commands(command_node((boot,boot,prefix,prefix),(0,0,240,240)),True)
        self.assertEqual(actual[-1],prefix)
        for bad in (boot,boot+((0,'SetSteward',0),),prefix+((5,'Build',1),)):
            with self.assertRaises(GateContractError):
                _=visibility.commands(command_node((boot,boot,bad),(0,0,240)),True)

    def test_loaded_commands_must_match_own_saved_prefix(self) -> None:
        boot=((0,'SetSteward',1),);prefix=boot*2+((3,'Steward',1),)
        node=command_node((boot,prefix,prefix,prefix,prefix),(0,240,240,240,240))
        self.assertEqual(len(visibility.commands(node,False,(prefix,)*4)),5)
        with self.assertRaises(GateContractError):
            _=visibility.commands(node,False,(boot,)*4)

    def test_rejected_and_truncated_command_columns_fail(self) -> None:
        node=Node({'checkpoints':[{'clock':{'tick':0},'commands':{'total':1,'ticks':[0],'kinds':['SetSteward'],'accepted':[]}}]})
        with self.assertRaises(GateContractError):_=visibility.commands(node,True)

    def test_original_flags_positional_generator_and_loader_external_bindings(self) -> None:
        product=ProbeProduct('a'*40,Path('/godot'),'b'*64,Path('/source'),())
        external=((Path('/arm/saves/manifest.json'),'c'*64),)
        with patch.object(visibility,'check_product'):
            g=visibility.process_spec(product,Path('/arm'),True)
            l=visibility.process_spec(product,Path('/arm'),False,external)
        self.assertEqual(g.argv[-2:],('--','/arm/saves'))
        self.assertNotIn('--out',g.argv)
        self.assertIn('--headless',g.argv);self.assertNotIn('--headless',l.argv)
        self.assertEqual(l.argv[-5:],('--','--manifest','/arm/saves/manifest.json','--out','/arm/run'))
        self.assertEqual((g.timeout_seconds,l.timeout_seconds),(1800,1200))
        self.assertEqual(l.external_inputs,external)
        self.assertEqual(dict(l.environment)['HOME'],str(Path.home()))
        self.assertEqual(dict(l.environment)['WORLDSIM_TEST_SETTINGS_DIR'],'/arm/loader-process/settings')

    def test_source_inventory_cannot_skip_pinned_producer(self) -> None:
        with self.assertRaisesRegex(GateContractError,'inventory'):
            visibility.check_product(ProbeProduct('a'*40,Path('/godot'),'b'*64,Path('/source'),()))

    def test_manifest_rejects_wrong_calendar_and_cross_arm_save(self) -> None:
        with TemporaryDirectory() as directory:
            arm=Path(directory).resolve();(arm/'saves').mkdir()
            entries: list[Json]=[]
            for name,day,hour,tick in visibility.TARGETS:
                save=arm/'saves'/(name+'.wsave');_=save.write_bytes(name.encode())
                entries.append({'id':name,'day':day,'hour':hour,'tick':tick,'save':str(save),'sha256':file_hash(save)})
            product=ProbeProduct('a'*40,Path('/godot'),'b'*64,Path('/source'),((BRIDGE,'c'*64),(visibility.GENERATOR,'d'*64)))
            data: dict[str,Json]={'schema':1,'seed':0,'steward':'on','bridge_sha256':'c'*64,'script_sha256':'d'*64,'scenes':entries}
            path=arm/'saves/manifest.json';_=path.write_text(json.dumps(data))
            self.assertEqual(len(visibility.manifest(arm,product)),4)
            first=Node(entries[0]).mapping()
            first['tick']=241;_=path.write_text(json.dumps(data))
            with self.assertRaises(GateContractError):_=visibility.manifest(arm,product)
            first['tick']=240;first['save']='/another/arm/day1_noon.wsave';_=path.write_text(json.dumps(data))
            with self.assertRaises(GateContractError):_=visibility.manifest(arm,product)

    def test_generator_done_does_not_override_runtime_error(self) -> None:
        with TemporaryDirectory() as directory:
            arm=Path(directory).resolve();folder=arm/'generator-process';folder.mkdir()
            _=(folder/'stdout.log').write_text('MUX2_VISIBILITY_SAVES_DONE scenes=4\n')
            _=(folder/'stderr.log').write_text('ERROR: bad\n')
            with self.assertRaises(GateContractError):visibility.domain(arm,Path('/source'),True)


class PairTests(unittest.TestCase):
    def run_fixture(self, target: str) -> tuple[str,str]:
        with TemporaryDirectory() as directory:
            root=Path(directory).resolve();products: list[ProbeProduct]=[]
            for name in ('base-source','candidate-source'):
                source=root/name;source.mkdir();_= (source/'input').write_bytes(b'input');_=(source/'godot').write_bytes(b'godot')
                products.append(ProbeProduct('a'*40,source/'godot',file_hash(source/'godot'),source,(('input',file_hash(source/'input')),)))
            def check(product: ProbeProduct) -> None:
                if file_hash(product.input_root/'input')!=dict(product.input_files)['input']:
                    raise GateContractError('INPUT','/input','product changed')
            def run(_product: ProbeProduct, arm: Path, frozen: dict[Path,dict[str,str]]) -> visibility.ArmEvidence:
                arm.mkdir();folder=arm/'generator-process';folder.mkdir();_=(folder/'process.json').write_bytes(b'original')
                visibility.freeze(folder,frozen)
                if arm.name=='candidate':
                    if target=='artifact':_=(arm.parent/'base/generator-process/process.json').write_bytes(b'changed')
                    if target=='extra':_=(arm.parent/'base/generator-process/extra').write_bytes(b'extra')
                    if target=='input':_=(products[0].input_root/'input').write_bytes(b'changed')
                    if target=='unicode':raise UnicodeDecodeError('utf-8',b'\xff',0,1,'invalid')
                process=ProcessResult(folder/'process.json','PASS',0,123)
                p=UIPlan('synthetic','synthetic',spec(arm,True),'res://synthetic',visibility.HELPER,(),())
                e=UIEvidence(folder/'process.json',file_hash(folder/'process.json'),p,process,file_hash(process.receipt))
                alive=7 if target=='safety' and arm.name=='candidate' else 10
                state=Observation(0,28200,10,alive,True,(),'[]')
                return visibility.ArmEvidence((state,)*10,(),e,e,())
            with patch.object(visibility,'check_product',side_effect=check),patch.object(visibility,'run_arm',side_effect=run):
                result=visibility.run_pair(Products(*products),root/'out')
            return result.status,result.receipt.read_text()

    def test_cross_arm_mutations_cannot_pass(self) -> None:
        for target in ('artifact','extra','input','unicode','safety'):
            with self.subTest(target=target):
                status,receipt=self.run_fixture(target)
                self.assertEqual(status,'FAIL');self.assertTrue(document(receipt).field('errors').array())

    def test_unchanged_synthetic_policy_chain_is_not_native_qualification(self) -> None:
        status,_=self.run_fixture('unchanged');self.assertEqual(status,'PASS')

    def test_overlapping_product_roots_rejected_before_launch(self) -> None:
        with TemporaryDirectory() as directory:
            root=Path(directory).resolve();product=ProbeProduct('a'*40,root/'godot','b'*64,root,())
            with self.assertRaisesRegex(GateContractError,'independent'):
                _=visibility.run_pair(Products(product,product),root.parent/(root.name+'-out'))


if __name__=='__main__':
    _=unittest.main()
