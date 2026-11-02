"""Synthetic strict-boundary tests only; these are not native runtime receipts."""
import json
import unittest
from collections.abc import Callable

from generic_gate import GateContractError
from generic_m6a_observations import (
    M6aObservation,
    paired_safety,
    paired_selected_safety,
    parse_fixed,
    parse_matched_companion,
    parse_selected_pack,
)
from generic_observations import ProbeWorld
from test_generic_gate_contract import (
    RawDocument,
    json_cases,
    mapping,
)


def sidecar(scene: str='m6a', end: int=1440, maximum: int=1440) -> RawDocument:
    row: RawDocument = {'schema_version':1,'scene':scene,'seed':0,'occurrence':'primary',
           'requested_ticks':maximum,'requested_terminal_tick':maximum,'initial_tick':0,
           'initial_population':10,'terminal_tick':end,'terminal_alive':8,'terminal_complete':True,
           'history_complete':True,'history_source':'fresh_world_append_only_chronicle',
           'knowledge_capability':{'enabled':True,'source':'resources.rules.colony.knowledge'},
           'knowledge_lost':[],'deaths':{name:0 for name in ('starvation','dehydration','exposure','combat','old_age')},'checkpoints':[]}
    if scene == 'm6a-data-only':
        row.update(schema_version=2,event_horizon={'condition':'m6a_food_chain_complete','condition_met':True,'reached_limit':end == maximum})
    return row


def fixed_report() -> RawDocument:
    return {'schema_version':1,'kind':'m6a_food_observation_v1','seed':0,'days':1,'ticks_per_day':1440,
            'clock_offset_ticks':480,'end_tick':1440,'observed_ticks':1440,'invariant_checks':2880,
            'violation_count':0,'observed_meals':2,'inputs':{'steward':True,'defence':True,'story':'on','neighbours':'on','player_orders':False},
            'final_tallies':{},'r2_spoilage':{},'common':{},'mux2':{},'fingerprint':12,
            'content_revision':'synthetic-content','observer_scope':'synthetic read-only fixture'}


def chain_report(end: int=120) -> RawDocument:
    return {'schema_version':1,'kind':'m6a_data_only_v1','passed':True,'content_revision':'synthetic-content',
            'placed':[[3,4]],'initially_growing':True,'diagnostics':[],'category_fallback':True,
            'ripe_tick':20,'harvest_job_tick':30,'harvested_pieces':4,'haul_job_tick':40,'carried_tick':50,
            'stored_tick':60,'eaten_from_store_tick':end,'intake_evidence':[{'tick':end,'agent':1,'stock_tile':[3,4],
                'pieces_before':2,'pieces_after':1,'meal':{'tick':end}}],
            'end_tick':end,'commands':[],'fixture':'synthetic source-bound native predicate'}


def fixed(row: RawDocument | None=None, report: RawDocument | None=None) -> M6aObservation:
    return parse_fixed(json.dumps(sidecar() if row is None else row),json.dumps(fixed_report() if report is None else report),ProbeWorld('m6a',0,'primary',1440))


def chain(row: RawDocument | None=None, report: RawDocument | None=None) -> M6aObservation:
    return parse_selected_pack(json.dumps(sidecar('m6a-data-only',120,8640) if row is None else row),json.dumps(chain_report() if report is None else report),8640)


def companion(end: int=121, report: RawDocument | None=None,
              row: RawDocument | None=None) -> M6aObservation:
    if report is None:
        report = chain_report(120)
        del report['passed']
        report.update(kind='m6a_data_only_companion_v1', end_tick=end,
                      workload='fixed_horizon_companion_primary_chain_oracle_required_separately',
                      requested_ticks=end, maximum_ticks=8640, chain_condition_met=True,
                      primary_oracle_satisfied=False, horizon_complete=True)
    return parse_matched_companion(
        json.dumps(sidecar('m6a-data-only-companion',end,end) if row is None else row),
        json.dumps(report),end,8640)


class M6aBoundaryTests(unittest.TestCase):
    def test_matched_companion_keeps_both_original_chain_oracles(self) -> None:
        later = chain(sidecar('m6a-data-only',121,8640),chain_report(121))
        self.assertEqual(paired_selected_safety(chain(),later,companion())['candidate_alive'],8)
        with self.assertRaises(GateContractError):
            paired_selected_safety(chain(),later,companion(122))
        failed_row=sidecar('m6a-data-only',8640,8640)
        mapping(failed_row['event_horizon'])['condition_met']=False
        failed_report=chain_report(8640)
        failed_report.update(passed=False,harvested_pieces=0)
        with self.assertRaises(GateContractError):
            paired_selected_safety(chain(failed_row,failed_report),later,companion())

    def test_companion_does_not_waive_candidate_loss_or_survival(self) -> None:
        for key,value in json_cases([('terminal_alive',7),('knowledge_lost',[
                {'tick':100,'node':'doorway','holder':None}])]):
            row=sidecar('m6a-data-only',121,8640); row[key]=value
            with self.assertRaises(GateContractError):
                paired_selected_safety(chain(),chain(row,chain_report(121)),companion())

    def test_companion_cannot_impersonate_primary(self) -> None:
        with self.assertRaises(GateContractError):
            paired_selected_safety(companion(),chain(),companion())
        with self.assertRaises(GateContractError):
            companion(row=sidecar('m6a-data-only',121,8640))
        with self.assertRaises(GateContractError):
            companion(report=chain_report(121))

    def test_fixed_report_actual_observation_binding(self) -> None:
        observation = fixed()
        self.assertEqual(observation.world.terminal_tick,1440)
        self.assertTrue(observation.native_passed)
        self.assertEqual(paired_safety(observation,fixed())['candidate_alive'],8)

    def test_fixed_wrong_report_endpoint_census_or_workload_rejected(self) -> None:
        for key,value in json_cases([('days',2),('ticks_per_day',720),('end_tick',1439),('observed_ticks',100),
                          ('invariant_checks',1440),('seed',1),('kind','m6a_data_only_v1'),('inputs',{})]):
            report = fixed_report();report[key] = value
            with self.subTest(key=key),self.assertRaises(GateContractError): fixed(report=report)
        row = sidecar();row['initial_population'] = True
        with self.assertRaises(GateContractError): fixed(row=row)

    def test_native_fixed_violation_is_retained_but_not_admitted(self) -> None:
        report = fixed_report();report['violation_count'] = 1
        failed = fixed(report=report)
        self.assertFalse(failed.native_passed)
        with self.assertRaises(GateContractError): paired_safety(fixed(),failed)

    def test_event_actual_endpoint_is_not_requested_maximum(self) -> None:
        observation = chain()
        self.assertEqual(observation.requested_maximum,8640)
        self.assertEqual(observation.world.terminal_tick,120)
        self.assertTrue(observation.condition_met)
        self.assertFalse(observation.reached_limit)
        _ = paired_safety(observation,chain())

    def test_different_early_completions_need_real_matched_base_companion(self) -> None:
        later = chain(sidecar('m6a-data-only',121,8640),chain_report(121))
        with self.assertRaisesRegex(GateContractError,'matched-horizon BASE companion'):
            _ = paired_safety(chain(),later)

    def test_native_chain_predicate_not_standalone_boolean(self) -> None:
        for key,value in json_cases([('category_fallback',False),('initially_growing',False),('harvested_pieces',0),
                          ('stored_tick',None),('ripe_tick',121),('passed',False),('end_tick',119),
                          ('intake_evidence',[])]):
            report = chain_report();report[key] = value
            with self.subTest(key=key),self.assertRaises(GateContractError): chain(report=report)

    def test_limit_failure_remains_diagnostic_not_pass(self) -> None:
        row = sidecar('m6a-data-only',8640,8640)
        mapping(row['event_horizon'])['condition_met'] = False
        report = chain_report(8640)
        report.update(passed=False,harvested_pieces=0)
        failed = chain(row,report)
        self.assertTrue(failed.reached_limit)
        self.assertFalse(failed.native_passed)
        with self.assertRaises(GateContractError): paired_safety(failed,failed)

    def test_incomplete_inconsistent_or_unrecognised_event_fields_fail(self) -> None:
        mutations: list[Callable[[RawDocument], None]] = [lambda x:x.update(terminal_complete=False),lambda x:x.update(history_complete=False),
            lambda x:x.update(requested_ticks=120),lambda x:x.update(requested_terminal_tick=120),
            lambda x:x.update(terminal_tick=8641),lambda x:x.update(initial_tick=1),
            lambda x:mapping(x['event_horizon']).update(reached_limit=True),lambda x:mapping(x['event_horizon']).update(condition_met=False),
            lambda x:mapping(x['event_horizon']).update(condition='other'),lambda x:x.update(unrecognised=True),
            lambda x:mapping(x['event_horizon']).update(extra=True),lambda x:x.update(schema_version=1)]
        for mutation in mutations:
            row = sidecar('m6a-data-only',120,8640);mutation(row)
            with self.subTest(mutation=mutation),self.assertRaises(GateContractError): chain(row=row)

    def test_unknown_missing_and_duplicate_report_fields_rejected(self) -> None:
        for builder,parser in [(fixed_report,fixed),(chain_report,chain)]:
            for key in builder():
                report=builder();del report[key]
                with self.subTest(key=key),self.assertRaises(GateContractError): parser(report=report)
            report=builder();report['extra']=1
            with self.assertRaises(GateContractError): parser(report=report)
        with self.assertRaises(GateContractError):
            _ = parse_selected_pack(json.dumps(sidecar('m6a-data-only',120,8640)), '{"passed":true,"passed":true}',8640)

    def test_full_native_knowledge_loss_and_survival_still_gate_event_pairs(self) -> None:
        mutations: list[Callable[[RawDocument], None]] = [lambda x:x.update(terminal_alive=7), lambda x:x.update(knowledge_lost=[{'tick':100,'node':'doorway','holder':None}])]
        for mutate in mutations:
            row=sidecar('m6a-data-only',120,8640);mutate(row)
            with self.assertRaises(GateContractError): paired_safety(chain(),chain(row=row))

    def test_pair_kind_seed_content_and_fixed_horizon_not_interchangeable(self) -> None:
        with self.assertRaises(GateContractError): paired_safety(fixed(),chain())
        report=chain_report();report['content_revision']='other'
        with self.assertRaises(GateContractError): paired_safety(chain(),chain(report=report))


if __name__ == '__main__':
    _ = unittest.main()
