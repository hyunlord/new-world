"""Reject incomplete or relaxed UI-successor performance evidence."""
import copy
from pathlib import Path
import sys
import unittest
sys.path.insert(0, str(Path(__file__).resolve().parent))
import core_ui_successor_perf as ui
from core_perf_check import Document, EvidenceError

class UIAdmissionTests(unittest.TestCase):
    def test_no_other_process_accepts_empty(self):
        self.assertEqual(ui.competing_processes('123 /bin/zsh -lc something\n'), [])

    def test_detects_godot_and_cargo_without_matching_command_arguments(self):
        self.assertEqual(len(ui.competing_processes('123 /Applications/Godot 4.6.app/Contents/MacOS/Godot\n456 /opt/homebrew/bin/cargo\n789 /bin/zsh -lc cargo build\n')), 2)

    def test_standing_has_no_side64_exception(self):
        old = {'scenes': [{'label': name, 'tick_mean_us': 100., 'tick_p99_us': 200.} for name in ('n200','n1000','n2000','n200_script','n200_side64','n1000_wolves')], 'alloc_per_agent':100., 'ceilings':{name:'PASS' for name in ('n1000_mean_le_5ms','n2000_mean_le_10ms','bytes_per_agent_le_4096','bytes_per_tile_le_96','rss_per_agent_le_5120','rss_per_tile_le_128')}}
        new = copy.deepcopy(old)
        new['scenes'][4]['tick_mean_us'] = 111.
        self.assertEqual(ui.failures(Document.parse(old), Document.parse(new)), ['n200_side64/tick_mean_us>1.1'])

    def test_missing_ceiling_rejected(self):
        row = {'scenes': [], 'alloc_per_agent':100., 'ceilings':{}}
        with self.assertRaises(EvidenceError):
            ui.failures(Document.parse(row), Document.parse(row))


class UIRawTests(unittest.TestCase):
    def fixture(self, out):
        out = out.resolve()
        import json
        import core_perf_check as p
        config = {'base_head': ui.BASE, 'candidate_head': 'a' * 40, 'identical_runtime_binary': True}
        p.write(out / 'config.json', config)
        records, admissions = [], []
        for role in ('base', 'new'):
            (out / 'accepted' / role).mkdir(parents=True)
        (out / 'attempts').mkdir()
        for build in (False, True):
            for repeat, scene in p.sequence(build):
                for role in (('base','new') if repeat % 2 else ('new','base')):
                    label=f'{"rss" if build else "timed"}-{scene}-r{repeat}-a1-{role}'
                    raw=out / 'attempts' / (label+'.json')
                    people=2000 if scene=='n2000' else 1000 if scene in ('n1000','n1000_wolves') else 200
                    side=64 if scene=='n200_side64' else 192
                    allocation=people*100+side*side*10
                    data={'label':scene,'agents':people,'side':side,'build_only':build,'repeat':repeat,
                          'script':scene=='n200_script','wolves':4 if scene=='n1000_wolves' else 0,
                          'seed':15100,'warmup_ticks':1440,'measured_ticks':2880,'build_profile':'release',
                          'tick_mean_us':100.,'tick_p99_us':200.,'load_before':'1 2 3',
                          'alloc_bytes':{'after_build':allocation},'rss_kib':{'after_build':allocation/1024}}
                    p.write(raw,data)
                    before=out/'attempts'/(label+'-before.txt');after=out/'attempts'/(label+'-after.txt')
                    before.write_text('123 /bin/zsh\n');after.write_text('123 /bin/zsh\n')
                    started=len(records)*2+1
                    monitor=out/'attempts'/(label+'-monitor.json')
                    p.write(monitor, {'pid':123,'started':started,'ended':started+1,'deadline':10000.,'deadline_exceeded':False,'exit_code':0,'samples':[{'utc':started,'snapshot':''},{'utc':started+1,'snapshot':''}]})
                    records.append({'process_monitor':p.frozen_file(monitor),'role':role,'scene':scene,'repeat':repeat,'attempt':1,'build_only':build,
                        'started':started,'ended':started+1,'external_load':[1.,2.,3.],'internal_load':[1.,2.,3.],
                        'exit_code':0,'raw':str(raw),'sha256':p.sha(raw),'status':'VALID','pair_accepted':True,
                        'processes_before':p.frozen_file(before),'processes_after':p.frozen_file(after)})
                    admissions.append({'label':label,'utc':started,'loads_1_5_15':[1.,2.,3.],'status':'ADMITTED'})
                    (out/'accepted'/role/f'{"rss" if build else "timed"}-{scene}-r{repeat}.json').write_bytes(raw.read_bytes())
        p.write(out/'attempts.json',records)
        (out/'admission.jsonl').write_text(''.join(json.dumps(row)+'\n' for row in admissions))
        for role in ('base','new'):
            p.write(out/f'{role}-summary.json',{'commit':config['base_head' if role=='base' else 'candidate_head'],
                'scenes':[{'label':name,'repeats':3,'tick_mean_us':100.,'tick_p99_us':200.} for name in p.SCENES],
                'memory_ceiling_source':'alloc','alloc_per_agent':100.,'alloc_per_tile':10.,
                'rss_outer':{'repeats':5,'per_agent_median':100.,'per_tile_median':10.},
                'ceilings':{name:'PASS' for name in p.CEILINGS}})

    def test_complete_matrix_rechecks_byte_identically(self):
        from tempfile import TemporaryDirectory
        from unittest.mock import patch
        with TemporaryDirectory() as directory:
            out=Path(directory)/'session';out.mkdir();self.fixture(out)
            with patch.object(ui,'verify_config'):
                self.assertEqual(ui.check(out),0)
                fresh=Path(directory)/'fresh.json'
                self.assertEqual(ui.check(out,fresh),0)
                self.assertEqual(fresh.read_bytes(),(out/'receipt.json').read_bytes())

    def test_tampered_raw_and_missing_pair_rejected(self):
        from tempfile import TemporaryDirectory
        from unittest.mock import patch
        import json
        for mode in ('raw','missing'):
            with self.subTest(mode=mode), TemporaryDirectory() as directory:
                out=Path(directory);self.fixture(out)
                if mode=='raw':
                    raw=out/'attempts/timed-n200-r1-a1-base.json';raw.write_text('{}')
                else:
                    path=out/'attempts.json';rows=json.loads(path.read_text());path.write_text(json.dumps(rows[:-1]))
                with patch.object(ui,'verify_config'), self.assertRaises(EvidenceError):
                    ui.check(out)

    def test_quiet_claim_with_godot_process_rejected(self):
        from tempfile import TemporaryDirectory
        from unittest.mock import patch
        import json
        import core_perf_check as p
        with TemporaryDirectory() as directory:
            out=Path(directory);self.fixture(out)
            process=out/'attempts/timed-n200-r1-a1-base-before.txt';process.write_text('123 /Applications/Godot.app/Contents/MacOS/Godot\n')
            path=out/'attempts.json';rows=json.loads(path.read_text());rows[0]['processes_before']=p.frozen_file(process);path.write_text(json.dumps(rows))
            with patch.object(ui,'verify_config'), self.assertRaises(EvidenceError):
                ui.check(out)

class UIConfigTests(unittest.TestCase):
    def test_legacy_base_cannot_enter_new_comparison(self):
        with self.assertRaises(EvidenceError):
            ui.verify_config(Document.parse({'kind':'ui_successor_fresh_performance_v1',
                'base_head':'9855bd082ad88a4c49296410f206f148d98c4df9'}))

    def test_pass_string_cannot_replace_raw_session(self):
        from tempfile import TemporaryDirectory
        with TemporaryDirectory() as directory:
            path=Path(directory)/'receipt.json';path.write_text('{"status":"PASS"}')
            with self.assertRaises(FileNotFoundError):
                ui.check(Path(directory))

    def test_admission_stops_on_deadline_without_launch(self):
        from tempfile import TemporaryDirectory
        from unittest.mock import patch
        from core_p7_admission import AdmissionError
        with TemporaryDirectory() as directory, patch.object(ui,'process_snapshot',return_value=''):
            with self.assertRaises(AdmissionError):
                ui.admission(Path(directory),'expired',0)


class UIMonitorTests(unittest.TestCase):
    def test_owned_child_deadline_is_enforced_and_recorded(self):
        from tempfile import TemporaryDirectory
        from unittest.mock import patch
        import os
        import time
        import core_ui_successor_perf_validate as validator
        with TemporaryDirectory() as directory, patch.object(ui,'process_snapshot',return_value=''):
            out=Path(directory)
            result=validator.run_monitored([sys.executable,'-c','import time; time.sleep(60)'],os.environ,
                out/'child.log',out/'monitor.json',time.time()+0.1)
            self.assertNotEqual(result.returncode,0)
            self.assertFalse(result.clean)
            self.assertLess(result.ended-result.started,2)
            self.assertTrue((out/'monitor.json').is_file())

    def test_midrun_competitor_cannot_be_hidden_by_clean_endpoints(self):
        import core_ui_successor_perf_validate as validator
        monitor=Document.parse({'pid':123,'started':1.,'ended':3.,'deadline':5.,'deadline_exceeded':False,
            'samples':[{'utc':1.,'snapshot':''},{'utc':2.,'snapshot':'456 /opt/homebrew/bin/cargo\n'},{'utc':3.,'snapshot':''}]})
        self.assertFalse(validator.monitor_clean(monitor))

    def test_sparse_monitor_does_not_prove_exclusive_run(self):
        import core_ui_successor_perf_validate as validator
        monitor=Document.parse({'pid':123,'started':1.,'ended':10.,'deadline':20.,'deadline_exceeded':False,
            'samples':[{'utc':1.,'snapshot':''},{'utc':10.,'snapshot':''}]})
        self.assertFalse(validator.monitor_clean(monitor))

if __name__ == '__main__':
    unittest.main()
