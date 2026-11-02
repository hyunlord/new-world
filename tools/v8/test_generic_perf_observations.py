"""Synthetic performance safety boundaries; no native benchmark claim."""
import json
import unittest
from dataclasses import replace

from generic_gate import GateContractError
from generic_perf_observations import PerfObservation, Plan, paired_safety, parse_native
from test_generic_gate_contract import RawDocument, json_cases
from test_generic_observations import receipt


def fixture(build: bool = False) -> tuple[Plan, RawDocument, RawDocument]:
    ticks = 0 if build else 28800
    workload: RawDocument = {'label':'n10','side':64,'repeat':1} if build else {'mode':'natural','scene':'colony','side':64,'feed':'none'}
    plan = Plan('build_only' if build else 'perf',ticks,0,10 if build else None,'/evidence/report.json',42,json.dumps(workload))
    native=receipt()
    for key in ('scene','seed','occurrence','checkpoints'):
        del native[key]
    native.update(requested_ticks=ticks,requested_terminal_tick=ticks,terminal_tick=ticks,
                  clock_offset_ticks=480,ticks_per_day=1440,terminal_alive=10 if build else 8)
    report: RawDocument = dict(workload)
    if build:
        report.update(agents=10,alive=10,build_only=True)
    else:
        report.update(agents_requested=None,alive_end=8,seed=0,founders=10,ticks=ticks,
                      ticks_per_day=1440,clock_offset_ticks=480)
    return plan,native,report


def parse(plan: Plan, native: RawDocument, report: RawDocument) -> PerfObservation:
    raw=json.dumps(report)
    envelope={'schema_version':1,'producer_pid':42,'raw_report_path':'/evidence/report.json',
              'raw_report_bytes':raw,'observation':native}
    return parse_native(json.dumps(envelope),raw,plan)


class PerfSafetyTests(unittest.TestCase):
    def test_complete_world_and_actual_zero_tick_construction(self) -> None:
        for build in (False,True):
            plan,native,report=fixture(build)
            observation=parse(plan,native,report)
            self.assertEqual(paired_safety(observation,observation)['candidate_alive'],10 if build else 8)

    def test_matching_candidate_loss_still_fails(self) -> None:
        plan,native,report=fixture()
        native['knowledge_lost']=[{'tick':100,'node':'clay_digging','holder':9}]
        observation=parse(plan,native,report)
        with self.assertRaisesRegex(GateContractError,'KnowledgeLost'):
            _ = paired_safety(observation,observation)

    def test_partial_foreign_or_wrong_workload_is_rejected(self) -> None:
        for key,value in json_cases([('terminal_complete',False),('history_complete',False),
                                    ('terminal_tick',28799),('initial_tick',1),('terminal_alive',True)]):
            plan,native,report=fixture();native[key]=value
            with self.subTest(key=key),self.assertRaises(GateContractError):
                _ = parse(plan,native,report)
        plan,native,report=fixture()
        for altered in (replace(plan,producer_pid=43),replace(plan,raw_path='/foreign'),
                        replace(plan,ticks=0),replace(plan,seed=1),replace(plan,agents=11),
                        replace(plan,workload_json='{}')):
            with self.assertRaises(GateContractError): _ = parse(altered,native,report)

    def test_construction_and_disabled_knowledge_require_native_consistency(self) -> None:
        plan,native,report=fixture(True)
        native['initial_population']=11
        with self.assertRaises(GateContractError): _ = parse(plan,native,report)
        plan,native,report=fixture()
        native['knowledge_capability']={'enabled':False,'source':'resources.rules.colony.knowledge'}
        with self.assertRaises(GateContractError): _ = parse(plan,native,report)
        native['knowledge_lost']=None
        self.assertFalse(parse(plan,native,report).world.knowledge_enabled)

    def test_bench_and_navigation_bind_warmup_and_interventions(self) -> None:
        for kind in ('bench','nav'):
            plan,native,_report=fixture()
            workload: RawDocument = {'label':'n10','side':64,'script':False,'wolves':0} if kind == 'bench' else {'script':'u_script.json'}
            plan=replace(plan,kind=kind,agents=10,workload_json=json.dumps(workload))
            report: RawDocument = dict(workload)
            report.update(agents=10,alive_end=8,seed=0,warmup_ticks=100,measured_ticks=28700)
            intervention: RawDocument = {'gifts':1,'gift_requested':10,'gift_applied':10}
            if kind == 'bench':
                report.update(founders=10,alive_after_warmup=10,gifts=1)
                intervention.update(wolves_requested=0,wolves_before=0,wolves_after=0)
            native.update(warmup_alive=10,interventions=intervention)
            observation=parse(plan,native,report)
            self.assertEqual(paired_safety(observation,observation)['candidate_alive'],8)
            report['measured_ticks']=28699
            with self.assertRaises(GateContractError):
                _ = parse(plan,native,report)

    def test_raw_bytes_and_pid_binding(self) -> None:
        plan,native,report=fixture();raw=json.dumps(report)
        envelope={'schema_version':1,'producer_pid':42,'raw_report_path':plan.raw_path,
                  'raw_report_bytes':raw,'observation':native}
        with self.assertRaises(GateContractError):
            _ = parse_native(json.dumps(envelope),raw+' ',plan)


if __name__ == '__main__':
    _ = unittest.main()
