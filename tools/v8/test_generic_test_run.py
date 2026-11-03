"""Offline tiny Python children, never native simulation or full-gate evidence."""
import json
import subprocess
import sys
import tempfile
import unittest
from dataclasses import asdict, replace
from pathlib import Path
from unittest.mock import patch

import generic_build
import generic_test_run
from generic_bench import AuditError, Product
from generic_build import BuildSpec, build_product
from generic_gate import document
from generic_process import file_hash
from generic_sources import inventory
from generic_test_history import FORMAT_SOURCE, identity
from generic_test_invocation import invocations
from generic_test_run import (
    JOBS_INPUTS,
    JOBS_RULES,
    TESTS,
    fixed_plan,
    run_pair,
    save_plan,
    verify_test_product,
)
from jobs_fixture import child_emission, source_path
from mux2_gate_inputs import Json, Node
from test_generic_test_history import CHILD, cases, graph

ROOT = Path(generic_build.__file__).resolve().parents[2]


HUT_CHILD = """
folder=parent/'hut_sleeps_four_s2_gifted_hut'
def checkpoint(label,tick,detail):
 (folder/('checkpoint_'+label+'.json')).write_text(json.dumps({'state':state(tick),'detail':detail}))
checkpoint('before_log_gift',0,{'intervention':'direct_ledger_create_and_ground_stack_insert','item':'log','count':12,'tile':[5,5],'ground_logs':0,'log_ledger':[0,0,0,0,0]})
checkpoint('after_log_gift',0,{'ground_logs':12,'log_ledger':[12,12,0,0,0],'requested_count':12,'tile':[5,5]})
checkpoint('accepted_direct_blueprint',0,{'intervention':'direct_apply_command','accepted_assertion_completed':True,'command':{'tick':0,'cmd':'place_blueprint','kind':'hut','x':10,'y':10}})
checkpoint('hut_completion',COMPLETION_TICK,{'actual_completed_ticks':COMPLETION_TICK,'limit_ticks':4320,'centre':[11,11],'owned_hut_assertion_completed':True})
"""


class InvocationTests(unittest.TestCase):
    def test_named_plans_preserve_original_flags_and_distinct_steps(self) -> None:
        plan = invocations
        self.assertEqual([r.target for r in plan('gate17')],
                         ['colony_commands','colony_jobs','colony_world'])
        self.assertEqual([(r.step,r.profile,r.package,r.kind,r.filter,r.ignored,r.argv) for r in plan('gate24')],
                         [(24,'release','sim-test','integration_test',None,'exclude',('--test-threads=8',))])
        self.assertEqual({r.profile for r in plan('gate04')},{'debug'})
        self.assertEqual({r.profile for r in plan('gate05')},{'release'})
        self.assertEqual({r.argv for r in plan('development')},{('--test-threads=1','--nocapture')})
        self.assertEqual(len(plan('gate04')),4)
        with self.assertRaises(ValueError):
            _ = plan('unknown')


class PlansTests(unittest.TestCase):
    def test_fixed_twins_and_original_full_outcomes(self) -> None:
        worlds, commands = fixed_plan('colony_world'), fixed_plan('colony_commands')
        self.assertEqual((len(worlds),len(commands)), (20,3))
        self.assertEqual([(p.occurrence.seed,p.occurrence.branch) for p in worlds[-4:]],
                         [(0,'first'),(0,'second'),(4,'first'),(4,'second')])
        self.assertEqual([p.occurrence.end_tick for p in worlds], [14400]*16+[2880]*4)
        self.assertEqual([p.occurrence.branch for p in commands], ['original','recorded_replay','idle_control'])
        self.assertEqual([len(TESTS[name]) for name in ('save_load','colony_world','colony_commands')], [7,6,11])

    def test_all46_source_bound_dynamic_intervals(self) -> None:
        with graph() as data:
            planned = save_plan(data.binding())
            self.assertEqual(tuple(p.occurrence for p in planned), data.expected)
            self.assertEqual(len(planned),46)
            self.assertEqual(sum(p.occurrence.branch == 'loaded_no_ticks' for p in planned),1)
            loaded = next(p for p in planned if p.occurrence.branch == 'loaded_2')
            self.assertEqual((loaded.occurrence.start_tick,loaded.occurrence.end_tick),(28000,37000))

    def test_event_outside_source_limit_rejected(self) -> None:
        with graph() as data:
            path = data.folder('save_load_roundtrip_through_deaths_and_births_s0_original/save_chain.json')
            row = document(path.read_text()).mapping()
            Node(row['state']).mapping()['tick'] = 100001
            _ = path.write_text(json.dumps(row))
            with self.assertRaises(ValueError):
                _ = save_plan(data.binding())


class RunnerTests(unittest.TestCase):
    def exercise(self, defect: str = '', invocation: str = 'development') -> Node:
        with tempfile.TemporaryDirectory() as temporary:
            home = Path(temporary).resolve()
            products: dict[str,dict[str,Product]] = {}
            for arm in ('base','candidate'):
                root = home/arm
                (root/'rust').mkdir(parents=True)
                format_source = root/FORMAT_SOURCE
                format_source.parent.mkdir(parents=True)
                _ = format_source.write_text('pub const FORMAT_VERSION: u32 = 15;\n')
                for name in JOBS_INPUTS:
                    path = root/name
                    path.parent.mkdir(parents=True,exist_ok=True)
                    _ = path.write_bytes(source_path(name).read_bytes())
                if defect == 'hut_beds' and arm == 'candidate':
                    path = root/JOBS_RULES
                    _ = path.write_text(path.read_text().replace('hut_beds: 4,','hut_beds: 5,',1))
                binaries: dict[str,Path] = {}
                for target in TESTS:
                    rows: list[dict[str,Json]] = cases() if target == 'save_load' else (
                        [{'test':'hut_sleeps_four','seed':2,'branch':'gifted_hut','start':0,'end':1640,
                          'stop':'actual_hut_completion_plus_1440'}] if target == 'colony_jobs' else [
                        {'test':p.occurrence.test,'seed':p.occurrence.seed,'branch':p.occurrence.branch,
                         'start':p.occurrence.start_tick,'end':p.occurrence.end_tick,'stop':p.stop}
                        for p in fixed_plan(target)])
                    if target == 'save_load':
                        # The adapter derives stops from its source plan; fixtures use independently
                        # generated save graph endpoints from the existing parser regression suite.
                        with graph(rows) as data:
                            stops = {identity(p.occurrence):p.stop for p in save_plan(data.binding())}
                        for row in rows:
                            row['stop'] = stops[f"{row['test']}_s{row['seed']}_{row['branch']}"]
                    if defect == 'hut_horizon' and arm == 'candidate' and target == 'colony_jobs':
                        rows[0]['end'] = 1641
                    if defect == 'horizon' and arm == 'candidate' and target == 'colony_world':
                        rows[0]['end'] = 14399
                    if defect == 'missing_world' and arm == 'candidate' and target == 'colony_commands':
                        _ = rows.pop()
                    _ = (root/'rust'/f'{target}.json').write_text(json.dumps(rows))
                    package = root/'rust/crates/sim-test'
                    package.mkdir(parents=True,exist_ok=True)
                    _ = (package/f'{target}.json').write_text(json.dumps(rows))
                    child = CHILD.replace("'plan.json'",repr(target+'.json')).replace(
                        "'synthetic planned boundary'", "r['stop']")
                    child = child[:child.index("for test in sorted({r['test'] for r in rows})")]
                    if target == 'colony_jobs':
                        child += child_emission()
                        child += HUT_CHILD.replace('COMPLETION_TICK', '201' if defect == 'hut_horizon' and arm == 'candidate' else '200')
                        if defect == 'hut_missing' and arm == 'candidate':
                            child += "(folder/'checkpoint_before_log_gift.json').unlink()\n"
                        if defect == 'hut_owned' and arm == 'candidate':
                            child += "p=folder/'checkpoint_hut_completion.json';r=json.loads(p.read_text());r['detail']['owned_hut_assertion_completed']=False;p.write_text(json.dumps(r))\n"
                        if defect == 'hut_limit' and arm == 'candidate':
                            child += "p=folder/'checkpoint_hut_completion.json';r=json.loads(p.read_text());r['detail']['actual_completed_ticks']=4320;p.write_text(json.dumps(r))\n"
                        if defect == 'hut_gift' and arm == 'candidate':
                            child += "p=folder/'checkpoint_after_log_gift.json';r=json.loads(p.read_text());r['detail']['log_ledger'][0]=11;p.write_text(json.dumps(r))\n"
                        if defect == 'hut_setup' and arm == 'candidate':
                            child += "for n in ('before_log_gift','after_log_gift'):\n p=folder/('checkpoint_'+n+'.json');r=json.loads(p.read_text());r['detail']['tile']=[4,5];p.write_text(json.dumps(r))\n"
                    names = TESTS[target][:-1] if defect == 'missing_test' and arm == 'candidate' else TESTS[target]
                    child += '\nfor test in '+repr(names)+": print('test '+test+' ... ok')\n"
                    if defect != 'missing_summary':
                        child += "print('test result: ok. "+str(len(names))+" passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.01s')\n"
                    if defect == 'extra_world' and arm == 'candidate':
                        child += "(root/'save_load_pid999999').mkdir()\n"
                    if defect == 'exit' and arm == 'candidate':
                        child += 'sys.exit(7)\n'
                    if defect == 'prior_rewrite' and arm == 'candidate' and target == 'save_load':
                        victim = home/'out/base-save_load/stdout.log'
                        child += 'pathlib.Path('+repr(str(victim))+ ").write_text('rewritten')\n"
                    binary = root/target
                    _ = binary.write_text('#!'+sys.executable+'\n'+child)
                    binary.chmod(0o755)
                    binaries[target] = binary
                inputs = tuple(inventory(root,['rust',*TESTS]).items())
                products[arm] = {target:Product(root,binary,'a'*40,file_hash(binary),home/'unused','b'*64,inputs)
                                 for target,binary in binaries.items()}
            with patch('generic_test_run.prepare'), patch('generic_test_run.verify_test_product',
                 side_effect=AuditError('postflight failed') if defect == 'postflight' else None), \
                 patch('generic_test_run.lane'):
                selected = {row.target for row in invocations(invocation)}
                result = run_pair({k:v for k,v in products['base'].items() if k in selected},
                                  {k:v for k,v in products['candidate'].items() if k in selected},
                                  home/'out',invocation=invocation)
                if defect == '' and invocation != 'development':
                    for arm in ('base','candidate'):
                        for target in selected:
                            path = home/f'out/{arm}-{target}-invocation.json'
                            binding = generic_test_run.authenticate_invocation(path,products[arm][target],invocation)
                            self.assertEqual(binding.process.argv[1:],('--test-threads=8',))
                            self.assertEqual(binding.process.cwd,products[arm][target].root/'rust/crates/sim-test')
                            original = path.read_bytes()
                            altered_cwd = document(original.decode()).mapping()
                            altered_cwd['cwd'] = str(products[arm][target].root/'rust')
                            _ = path.write_text(json.dumps(altered_cwd))
                            with self.assertRaises(ValueError):
                                _ = generic_test_run.authenticate_invocation(path,products[arm][target],invocation)
                            _ = path.write_bytes(original)
                            relabeled = document(original.decode()).mapping()
                            other = 'gate24' if target == 'save_load' and invocation != 'gate24' else 'gate05'
                            if invocation == other:
                                other = 'gate04'
                            relabeled['plan'] = other
                            relabeled['identity'] = document(json.dumps(asdict(next(row for row in invocations(other) if row.target == target)))).value
                            _ = path.write_text(json.dumps(relabeled))
                            with self.assertRaises(ValueError):
                                _ = generic_test_run.authenticate_invocation(path,products[arm][target],other)
                            altered = document(original.decode()).mapping()
                            Node(altered['identity']).mapping()['threads'] = 1
                            _ = path.write_text(json.dumps(altered))
                            with self.assertRaises(ValueError):
                                _ = generic_test_run.authenticate_invocation(path,products[arm][target],invocation)
            return document(result.read_text())

    def test_gate_invocations_and_reauthentication(self) -> None:
        for plan,counts in (('gate17',[3,10,20]),('gate24',[46]),('gate04',[46,20,3,10]),('gate05',[46,20,3,10])):
            with self.subTest(plan=plan):
                result = self.exercise(invocation=plan)
                result.field('status').equals('COMPONENT_PASS')
                result.field('workspace_enumeration').equals('INCOMPLETE')
                self.assertEqual([len(row.field('occurrences').array()) for row in result.field('families').array()
                                  if 'occurrences' in row.mapping()],counts)
        self.exercise('missing_summary','gate24').field('status').equals('FAIL')

    def test_full_original_binaries_preserve69_and_observe_ten_job_worlds(self) -> None:
        result = self.exercise()
        result.field('status').equals('COMPONENT_PASS')
        result.field('coverage').equals('save_load46 + colony_world20 + colony_commands3 + colony_jobs10 planned ordinary occurrences per arm')
        result.field('invocation_coverage').equals({'save_load':46,'colony_world':20,'colony_commands':3,'colony_jobs':10})
        result.field('cargo_working_directory_equivalence').equals('NOT_ORIGINAL_CARGO_QUALIFICATION')
        result.field('broader_gate17_37').equals('INCOMPLETE')
        result.field('publication_authorized').equals(False)
        self.assertEqual([len(row.field('occurrences').array()) for row in result.field('families').array() if 'occurrences' in row.mapping()], [46,20,3,10])

    def test_hut_mismatched_horizon_or_setup_remains_unresolved(self) -> None:
        for defect in ('hut_horizon','hut_setup'):
            with self.subTest(defect=defect):
                self.exercise(defect).field('status').equals('UNRESOLVED')

    def test_hut_declared_setup_and_native_condition_must_authenticate(self) -> None:
        for defect in ('hut_gift','hut_missing','hut_owned','hut_limit','hut_beds'):
            with self.subTest(defect=defect):
                self.exercise(defect).field('status').equals('FAIL')

    def test_native_failure_omissions_horizon_and_mutation_retain_fail(self) -> None:
        for defect in ('exit','missing_world','missing_test','horizon','prior_rewrite','extra_world','postflight'):
            with self.subTest(defect=defect):
                result = self.exercise(defect)
                result.field('status').equals('FAIL')
                self.assertTrue(result.field('errors').array())


class BuildConsumerTests(unittest.TestCase):
    def test_actual_build_chain_and_dirty_source_rejection(self) -> None:
        for profile in ('release','debug'):
            with self.subTest(profile=profile):
                self.build_chain(profile)

    def build_chain(self, profile: str) -> None:
        with tempfile.TemporaryDirectory() as temporary:
            home = Path(temporary).resolve()
            root = home/'source'
            root.mkdir()
            _ = subprocess.run(['git','init','-q',str(root)],check=True)
            for folder in ('rust/crates','tools/v8','scripts','scenes','shaders','assets','localization'):
                (root/folder).mkdir(parents=True,exist_ok=True)
                _ = (root/folder/'input').write_text('synthetic')
            for name in ('rust/Cargo.toml','rust/Cargo.lock','project.godot','sim_bridge.gdextension','tools/play.sh'):
                _ = (root/name).write_text('synthetic')
            _ = (root/'rust/rust-toolchain.toml').write_text('[toolchain]\nchannel="1.93"\n')
            _ = (root/'.gitignore').write_text('target/\n')
            for name in (FORMAT_SOURCE,'rust/crates/sim-test/tests/save_load.rs'):
                path = root/name
                path.parent.mkdir(parents=True,exist_ok=True)
                _ = path.write_bytes(source_path(name).read_bytes())
            _ = subprocess.run(['git','-C',str(root),'add','.'],check=True)
            _ = subprocess.run(['git','-C',str(root),'-c','user.name=Test','-c','user.email=test@example.invalid',
                                'commit','-qm','synthetic fixture'],check=True)
            sha = subprocess.check_output(['git','-C',str(root),'rev-parse','HEAD'],text=True).strip()
            compiler = home/'compiler'
            _ = compiler.write_text('#!'+sys.executable+'''\nimport json,os,pathlib,sys
if '--version' in sys.argv:
 print('synthetic compiler\\nrelease: 1.93.0\\nhost: synthetic');sys.exit(0)
p=pathlib.Path(os.environ['CARGO_TARGET_DIR'])/('release' if '--release' in sys.argv else 'debug')/'deps/save_load-synthetic'
p.parent.mkdir(parents=True);p.write_text('not a native executable');p.chmod(0o755)
print(json.dumps({'reason':'compiler-artifact','target':{'name':'save_load','kind':['test']},'profile':{'test':True},'fresh':False,'executable':str(p),'filenames':[str(p)]}))
print(json.dumps({'reason':'build-finished','success':True}))
''')
            compiler.chmod(0o755)
            cargo_home = home/'cargo-home'
            cargo_home.mkdir()
            spec = BuildSpec(root,sha,compiler,file_hash(compiler),compiler,file_hash(compiler),cargo_home,
                             home/'build',('sim-test',),(),profile='debug' if profile == 'debug' else 'release',test_targets=('save_load',))
            built = build_product(spec)
            self.assertEqual(built.status,'PASS')
            receipt = spec.output/'artifact-save_load.json'
            build = document(built.receipt.read_text())
            binary = built.artifacts[0][1]
            inputs = tuple((name,Node(value).text()) for name,value in build.field('effective_inputs_before').mapping().items())
            product = Product(root,binary,sha,file_hash(binary),receipt,file_hash(receipt),inputs)
            verify_test_product(product,'save_load',profile)
            with self.assertRaises(ValueError):
                verify_test_product(product,'save_load','release' if profile == 'debug' else 'debug')
            _ = (root/'rust/Cargo.toml').write_text('dirty')
            with self.assertRaises(ValueError):
                verify_test_product(product,'save_load',profile)
            with self.assertRaises(ValueError):
                verify_test_product(replace(product,input_files=()),'save_load')


if __name__ == '__main__':
    _ = unittest.main()
