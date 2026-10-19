"""Produce one concise authoritative row per A1-A30, retaining failed diagnostics."""
from __future__ import annotations
import json
from pathlib import Path


def decoded(detail: str) -> dict:
    try:
        value = json.loads(detail)
        return value if isinstance(value,dict) else {}
    except json.JSONDecodeError:
        return {}


def crowd_summary(record: dict) -> dict:
    frames = decoded(record['detail']).get('frames',{})
    rows = []
    for stem,row in frames.items():
        detail = decoded(row['detail'])
        visible,hidden = detail.get('visible',0),detail.get('half_occluded',0)
        rows.append({'frame':stem,'status':row['status'],'half_occluded':hidden,'visible':visible,
                     'fraction':hidden/visible if visible else 0,'allowed':detail.get('limit')})
    return {'frames':len(rows),'failed_frames':sum(row['status']!='PASS' for row in rows),
            'worst':sorted(rows,key=lambda row:row['fraction'],reverse=True)[:5]}


def write_summary(records: list[dict], frames: list[tuple[Path,dict]], output: Path, gate: Path) -> None:
    """Keep full failures in measurements.json; expose exact numeric acceptance here."""
    by_code = {row['code']:row for row in records}
    review_path = output/'visual_observations.json'
    reviews = json.loads(review_path.read_text()).get('frames',{}) if review_path.exists() else {}
    rows = []
    for number in range(1,31):
        code = f'A{number}'
        record = by_code[code]
        data = decoded(record['detail'])
        metrics = {}
        detail = record['detail']
        if code=='A1':
            metrics=data
            detail=f"Gameplay {data.get('gameplay_only',{}).get('count')} frames, failures {len(data.get('gameplay_only',{}).get('failures',[]))}; all-frame diagnostic {data.get('old_all_frames',{}).get('status')}: {data.get('old_all_frames',{}).get('failures')}"
        elif code=='A5':
            metrics={stem:decoded(row['detail']) for stem,row in data.items()}
            metrics={stem:{key:value for key,value in row.items() if key!='actors'} for stem,row in metrics.items()}
            detail=json.dumps(metrics,ensure_ascii=False)
        elif code=='A11':
            metrics={'calibration_lineup':True,'observers':[{key:value for key,value in row.items() if key.startswith('accuracy') or key in ('observer_id','count','carry_correct','carry_total','absent_activities')} for row in data.get('observers',[])],
                'final_pixel_identity':by_code.get('A11_FINAL_PIXEL_IDENTITY',{}).get('status')}
            for measured,source in zip(metrics['observers'],data.get('observers',[]),strict=True):
                measured['carry_correct']={zoom:source.get('confusion',{}).get(zoom,{}).get('carry',{}).get('carry',0) for zoom in ('4x','2x')}
            detail=json.dumps(metrics,ensure_ascii=False)+'; natural activity recognition not established'
        elif code=='A16':
            metrics={key:row.get('status') for key,row in data.items() if isinstance(row,dict)}
            detail=json.dumps(metrics)
        elif code=='A17':
            metrics={key:data.get(key) for key in ('counts','real_carrier_ids','real_bodies','errors') if key in data}
            detail=json.dumps(metrics,ensure_ascii=False)
        elif code=='A18':
            metrics={'live':crowd_summary(record),'society':crowd_summary(by_code['A18_SOCIETY'])}
            detail='; '.join(f"{scope}: {value['failed_frames']}/{value['frames']} frames FAIL; worst {value['worst'][0] if value['worst'] else 'none'}" for scope,value in metrics.items())
        elif code=='A19':
            metrics={key:data.get(key) for key in ('tests','captured_visible_cold','cycles_checked','target_labels_checked','errors')}
            detail=json.dumps(metrics,ensure_ascii=False)
        elif code=='A20':
            metrics={key:data.get(key) for key in ('head','final_bound_frames','changed_villager_frames','animation_groups','installations','errors')}
            detail=json.dumps(metrics,ensure_ascii=False)
        elif code in ('A21','A22','A23','A24','A25'):
            metrics={key:data.get(key) for key in ('frames_checked','errors')}
            if code=='A21':
                metrics['available_raw_cycles']=sum(path.with_suffix('.cold_cycle.json').exists() for path,sidecar in frames if sidecar.get('scenario')=='NATURAL SCENARIO')
                metrics['observed_full_cycles']=sum(bool(reviews.get(path.stem,{}).get('doorway_cycle_visible')) for path,sidecar in frames if sidecar.get('scenario')=='NATURAL SCENARIO' and path.with_suffix('.cold_cycle.json').exists())
            if code=='A22':
                metrics['observed_chill_ids']={stem:reviews.get(stem,{}).get('chill_agent_ids',[]) for stem in ('l_d29_0630_F2','l_d29_0715_F2','l_d27_0715_F2')}
            detail=json.dumps(metrics,ensure_ascii=False)
        elif code=='A26':
            metrics=data.copy()
            seen={}
            for path,sidecar in frames:
                scenario=f"{sidecar.get('scenario')} / {sidecar.get('seed')}"
                events=seen.setdefault(scenario,{})
                for event in sidecar.get('death_truth_trace',{}).get('deaths',[]):events[event['death_id']]=event['reason']
            metrics['unique_census_events_by_scenario']={key:{'deaths':len(events),'by_reason':[sum(reason==index for reason in events.values()) for index in range(5)]} for key,events in seen.items()}
            detail=json.dumps(metrics,ensure_ascii=False)
        elif code in ('A27','A28'):
            metrics={key:value for key,value in data.items() if key!='blind'}
            blind=data.get('blind',{})
            metrics['blind_status']=blind.get('status')
            metrics['blind_observers']=[{'observer_id':row['observer_id'],'correct':row['correct'][code],'total':row['total'][code]} for row in decoded(blind.get('detail','')).get('observers',[])]
            detail=json.dumps(metrics,ensure_ascii=False)
        elif code=='A29':
            metrics=data;detail=json.dumps(metrics,ensure_ascii=False)
        elif code=='A30':
            metrics={key:data.get(key) for key in ('median_ms','ratio','threshold','samples','simulation_frozen','load_1m','load_condition','scope','git_sha','expected_git_sha','audit_items','errors') if key in data}
            detail=json.dumps(metrics,ensure_ascii=False) if metrics else record['detail']
        elif data:
            metrics={key:value for key,value in data.items() if isinstance(value,(int,float,bool,str)) or key=='errors'}
            detail=json.dumps(metrics,ensure_ascii=False) if metrics else detail
        rows.append({'code':code,'status':record['status'],'detail':detail,'metrics':metrics})
    assert len(rows)==30 and len({row['code'] for row in rows})==30
    report={'gate_source':str(gate),'provisional_gate':gate.name!='final','canonical':'measurements.json',
        'scope':'One acceptance row per A1-A30; calibration-only and failed subsidiary diagnostics remain explicit.', 'rows':rows}
    (output/'measurement_summary.json').write_text(json.dumps(report,ensure_ascii=False,indent=2)+'\n')
