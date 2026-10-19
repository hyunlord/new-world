"""Negative evidence boundaries: no absent data may produce a P7 visual pass."""
from pathlib import Path
from capture_contract_p7 import LONG_STEMS
from observation_rules_p7 import intervention_visible


def test_long_contract_keeps_all_inherited_frames_and_adds_exposure():
    assert len(LONG_STEMS) == 27
    assert {'l_d57_0301_DE', 'l_d85_0401_DE'} <= LONG_STEMS


def test_dust_and_drop_have_independent_end_boundaries():
    assert intervention_visible('god_windfall', 'windfall', .59, ['windfall_dust', 'windfall_drop'])
    assert intervention_visible('god_windfall', 'windfall', .6, ['windfall_dust_ended', 'windfall_drop'])
    assert not intervention_visible('god_windfall', 'windfall', .6, ['windfall_dust', 'windfall_drop'])
    assert intervention_visible('god_windfall', 'windfall', .8, ['windfall_dust_ended', 'windfall_drop_ended'])


def test_missing_society_evidence_does_not_pass(tmp_path: Path):
    from ingame_qa_society_p7 import society_checks
    checks = society_checks([], tmp_path, tmp_path)
    assert {check.code for check in checks} == {'A26','A27','A28','A29','A30'}
    assert all(check.status != 'PASS' for check in checks)


def test_duplicated_feed_death_fails():
    from ingame_qa_deaths_p7 import check_death_frame
    data = {'tick': 101, 'tallies': {'deaths_starvation':0,'deaths_dehydration':0,'deaths_combat':0,'deaths_old_age':0,'deaths_exposure':1},
        'hud_death_total':1, 'labels':[{'name':'HudTopbar/deaths','text':'사망 1'}],
        'death_truth_trace':{'source':'independent FFI recent_deaths census','max_sample_interval_ticks':8,'window_start_tick':0,'window_end_tick':101,
            'deaths':[{'death_id':'100:2:3:4:0','tick':100,'x':2,'y':3,'reason':4}]},
        'feed_log':[{'death_id':'100:2:3:4:0','key':'HUD_FEED_DEATH_EXPOSURE','tick':100}]*2,
        'death_drawn':[{'tick':100,'x':2,'y':3,'reason':4,'color':[.70,.84,.97]}]}
    assert any('feed' in error for error in check_death_frame(data))


def test_offscreen_flag_does_not_prove_visible_founding_site():
    from ingame_qa_founding_p7 import check_group
    group={'trail_visible':True,'marker_visible':True,'marker_in_view':False,'site_in_view':False,
        'dash_px':2,'gap_px':3,'trail_alpha':.6,'trail_color':'E8D8A8','bundles':[]}
    assert any('view' in error for error in check_group(group))


def test_nonfinite_frame_samples_cannot_pass_performance(tmp_path: Path):
    import json
    from ingame_qa_performance_p7 import performance
    for count in (1,3):
        row = {'samples_ms':[float('nan')]*300, 'alive_count':count, 'run_id':'same',
            'seed':20261013, 'zoom':1, 'simulation_frozen':True,
            'load_status':{'uptime':'load recorded','processes':'','uptime_exit_code':0,'pgrep_exit_code':1}}
        (tmp_path/f'frame_times_soc_{count}.json').write_text(json.dumps(row))
    (tmp_path/'audit_closure.json').write_text(json.dumps({'items':[
        {'id':f'U{i:02}', 'status':'justified','reason':'fixture','evidence':'fixture'} for i in range(1,40)]}))
    check = performance(tmp_path,tmp_path)
    assert check.status == 'FAIL'
    assert 'finite and positive' in check.detail


def test_preview_reduction_preserves_duration_and_selected_pixels(tmp_path: Path):
    from PIL import Image
    from compact_previews_p7 import compact, frames
    path = tmp_path/'cycle.gif'
    originals = [Image.new('RGB',(64,64),(index*40,10,20)) for index in range(5)]
    originals[0].save(path,save_all=True,append_images=originals[1:],duration=[100,120,140,160,180],loop=0,optimize=False)
    result = compact(path)
    actual,durations = frames(path)
    assert result['duration_ms'] == 700
    assert durations == [220,300,180]
    assert [image.getpixel((0,0)) for image in actual] == [(0,10,20),(80,10,20),(160,10,20)]


def test_panel_confirmation_does_not_override_negative_world_observation(tmp_path: Path, monkeypatch):
    import hashlib
    import json
    from PIL import Image
    from ingame_qa_observed_p7 import observed_checks
    monkeypatch.chdir(tmp_path)
    review = tmp_path/'docs/art/production7'
    review.mkdir(parents=True)
    raw = tmp_path/'god';raw.mkdir()
    image = raw/'god_gift.png';Image.new('RGB',(1,1)).save(image)
    image.with_suffix('.entities.json').write_text(json.dumps({'seed':20261010,'scenario':'INTERVENTION SCENARIO',
        'entities':[],'zoom':[1,1],'intervention_casts':[{'kind':'gift_wood','reply':{'ok':True}}]}))
    (raw/'manifest.json').write_text(json.dumps({'captures':[{'file':image.name}]}))
    (review/'visual_observations.json').write_text(json.dumps({'frames':{'god_gift':{
        'sha256':hashlib.sha256(image.read_bytes()).hexdigest(),'intervention_effects':['gift_wood'],
        'panel_action_readable':True,'world_effect_visibly_identified':False}}}))
    (raw/'results.tsv').write_text('test_god_hand\t0\ntest_interventions_real_p6\t0\n')
    check = next(row for row in observed_checks([],raw,raw) if row.code=='A24')
    assert check.status=='FAIL'
    assert 'only panel confirmation' in check.detail


def test_founder_suppression_requires_independent_raw_reason():
    from ingame_qa_founding_p7 import check_group
    group={'trail_visible':True,'marker_visible':True,'marker_in_view':True,'site_in_view':True,
        'dash_px':2,'gap_px':3,'trail_alpha':.6,'trail_color':'E8D8A8',
        'bundles':[{'agent_id':101,'agent_in_view':True,'visible':False,'suppressed':'not_founder'}]}
    raw={101:{'state':1,'target':0,'carried_food':0,'carried_wood':0}}
    entities={101:{'visible':True,'activity':'walk'}}
    assert any('unsupported' in error for error in check_group(group,raw,entities))
    for reason in ('hidden_agent','real_load','eat','asleep'):
        group['bundles'][0]['suppressed']=reason
        assert any('unsupported' in error for error in check_group(group,raw,entities))
    group['bundles'][0]['suppressed']='real_load';raw[101]['carried_wood']=1
    assert check_group(group,raw,entities)==[]
    raw[101].update(state=3,target=1,carried_wood=0);entities[101]['activity']='eat'
    group['bundles'][0]['suppressed']='eat'
    assert check_group(group,raw,entities)==[]


def test_tampered_observer_copy_invalidates_blind_result(tmp_path: Path):
    import hashlib,json,time
    from blind_p7 import grade,check_blind
    original=tmp_path/'source.png';original.write_bytes(b'original frozen pixels')
    shown=tmp_path/'observer.png';shown.write_bytes(original.read_bytes())
    key={'created_ns':time.time_ns()-1000000000,'A27':{str(i):{'count':2,'parents':{'2':1}} for i in range(1,5)},
         'A28':{'1':'moving away to settle','2':'moving away to settle'},
         'sources':{'A27_1':{'source':str(original),'image':str(shown),'sha256':hashlib.sha256(original.read_bytes()).hexdigest()}}}
    key_path=tmp_path/'key.json';key_path.write_text(json.dumps(key))
    key_path.with_suffix('.sha256').write_text(hashlib.sha256(key_path.read_bytes()).hexdigest())
    labels=[]
    for name in ('one','two'):
        path=tmp_path/f'{name}.json';path.write_text(json.dumps({'observer_id':name,'A27':key['A27'],'A28':key['A28']}));labels.append(path)
    output=tmp_path/'results.json';grade(key_path,labels,output)
    assert check_blind(output,'A27').status=='PASS'
    shown.write_bytes(b'different pixels shown to observer')
    assert check_blind(output,'A27').status=='FAIL'
