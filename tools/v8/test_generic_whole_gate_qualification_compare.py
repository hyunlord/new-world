# /// script
# requires-python = ">=3.11"
# dependencies = []
# ///
# Run: python3 -m unittest test_generic_whole_gate_qualification_compare
"""Pure comparator fixtures, not native product qualification."""
import json
import unittest
from copy import deepcopy
from typing import final

from generic_gate import GateContractError
from generic_m6a_observations import FIXED_FIELDS
from generic_whole_gate_qualification_compare import (
    NAV_MEASUREMENT,
    NAV_TIMING,
    SYSTEM_TIMING,
    TICK_TIMING,
    compare,
    markdown_simulation,
)
from mux2_gate_inputs import EvidenceError, Json


@final
class ComparatorTests(unittest.TestCase):
    def test_markdown_timing_is_bound_to_raw_summary(self) -> None:
        summary = json.dumps({'runs':[{'seed':0,'wall_seconds':1.25,'ms_per_agent_tick':2.5}]})
        raw = b'heading\n| Seed | Wall seconds | ms / agent-tick |\n|---|---:|---:|\n| 0 | 1.250000 | 2.500000000 |\n\n## Death census\nstate'
        self.assertIn(b'| 0 | 0.000000 | 0.000000000 |', markdown_simulation(raw,summary))
        with self.assertRaises(GateContractError):
            _ = markdown_simulation(raw.replace(b'1.250000',b'1.250001'),summary)

    def test_baseline_existing_normalizer_preserves_state(self) -> None:
        a: dict[str, Json] = {'schema_version':13, 'runs':[{'seed':0,'fingerprint':77,'wall_seconds':1,'ms_per_agent_tick':2}],
            'timing':{key:{stat:1 for stat in ('median','p10','p90','min','max')} for key in ('wall_seconds','ms_per_agent_tick')}}
        b = deepcopy(a)
        runs = b['runs']; assert isinstance(runs,list)
        run = runs[0]; assert isinstance(run,dict)
        run['wall_seconds']=9
        excluded = compare('baseline',json.dumps(a),json.dumps(b))
        self.assertIn('HEAD/runs/0/wall_seconds',excluded)
        run['fingerprint']=78
        with self.assertRaises(GateContractError):
            _ = compare('baseline',json.dumps(a),json.dumps(b))

    def test_baseline_unknown_timing_named_field_not_masked(self) -> None:
        a: dict[str, Json] = {'schema_version':13,'runs':[{'seed':0,'wall_seconds':1,'ms_per_agent_tick':1}],
            'timing':{key:{stat:1 for stat in ('median','p10','p90','min','max')} for key in ('wall_seconds','ms_per_agent_tick')}}
        b=deepcopy(a);b['unknown_timing']=1
        with self.assertRaises(GateContractError):
            _ = compare('baseline',json.dumps(a),json.dumps(b))

    def test_m6a_summary_no_exclusions(self) -> None:
        a: dict[str, Json] = {key:None for key in FIXED_FIELDS}
        a.update(schema_version=1,kind='m6a_food_observation_v1',days=72,ticks_per_day=1440,end_tick=103680,
                 observed_ticks=103680,invariant_checks=207360,fingerprint=10)
        self.assertEqual(compare('m6a',json.dumps(a),json.dumps(a)),[])
        b=deepcopy(a);b['fingerprint']=11
        with self.assertRaises(GateContractError):
            _ = compare('m6a',json.dumps(a),json.dumps(b))

    def test_nav_timing_changes_pass_state_changes_reject(self) -> None:
        a: dict[str, Json] = {key:1 for key in (*NAV_TIMING,*NAV_MEASUREMENT,'agents','seed','warmup_ticks','measured_ticks','alive_end','requests','memo_hits','searches','paths_now','nav_version')}
        a.update(within_budget=True,load_before='1 1 1',load_after='1 1 1')
        b=deepcopy(a);b['movement_us']=44;b['within_budget']=False
        self.assertEqual(set(compare('nav',json.dumps(a),json.dumps(b))),{'/'+key for key in (*NAV_TIMING,*NAV_MEASUREMENT)})
        b['requests']=2
        with self.assertRaises(GateContractError):
            _ = compare('nav',json.dumps(a),json.dumps(b))

    def perf(self) -> dict[str, Json]:
        tick: dict[str, Json] = {key:1 for key in TICK_TIMING};tick['ticks']=2
        system: dict[str, Json] = {key:1 for key in SYSTEM_TIMING};system.update(name='System',calls=2,calls_incl_warmup=3)
        a: dict[str, Json] = {key:1 for key in ('seed','ticks','fingerprint','alive_end','founders','settlements_end','chronicle_entries_end','buildings_end','wall_seconds','steady_tick_ms_total','prune_ms','unaccounted_ms')}
        a.update({'pass':15,'warmup':{'ticks':1,'tick0_ms':2,'summary':deepcopy(tick)},'tick_summary':tick,
                  'systems':[system],'per_day':[{'day':1,'ticks':2,'alive_0600':3,'tick_ms':1,'systems_ms':[2]}],
                  'snapshot_timings':{'get_agent_snapshot':{'median_us':1,'p90_us':2,'rows':3}},'slowest_ticks':[]})
        return a

    def test_perf_timing_exclusion_preserves_calls_and_population(self) -> None:
        a=self.perf();b=deepcopy(a);b['wall_seconds']=9;b['slowest_ticks']=[{'tick':5,'display_day':1,'clock':'00:05','total_ms':1,'top_systems':[],'due_periodic':[]}]
        self.assertIn('/slowest_ticks',compare('perf',json.dumps(a),json.dumps(b)))
        b['alive_end']=2
        with self.assertRaises(GateContractError):
            _ = compare('perf',json.dumps(a),json.dumps(b))

    def test_perf_unknown_and_missing_fields_reject(self) -> None:
        a=self.perf();b=deepcopy(a);b['future_state']=1
        with self.assertRaises(GateContractError):
            _ = compare('perf',json.dumps(a),json.dumps(b))
        del b['wall_seconds']
        with self.assertRaises((GateContractError,EvidenceError)):
            _ = compare('perf',json.dumps(a),json.dumps(b))

    def test_perf_index_not_timing_verdict(self) -> None:
        a: dict[str, Json]={'pass':15,'runs':[{'status':'ok','fingerprint':1,'mean_ms':1,'p999_ms':1,'max_ms':1,'wall_seconds':1}]}
        b=deepcopy(a)
        runs=b['runs'];assert isinstance(runs,list)
        run=runs[0];assert isinstance(run,dict)
        run['mean_ms']=2
        self.assertTrue(compare('perf-index',json.dumps(a),json.dumps(b)))

if __name__ == '__main__':
    _ = unittest.main()
