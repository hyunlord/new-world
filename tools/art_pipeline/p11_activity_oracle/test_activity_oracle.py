"""Offline synthetic tests; these fixtures are not capture evidence."""
import copy
import unittest
from pathlib import Path
from activity_rules import Frame, body_for, candidates, daily, final_frame, load_atlas
from check_activity_sidecars import check_capture, fixed_animation_seconds
from prepare_blind_crops import inventory

ROOT=Path('/Users/rexxa/work/new-world-art-prod')


def fixture():
    agent=next(i for i in range(1,100) if body_for(i,0)=='adult_a')
    frame=Frame('craft','adult_a','sit','E','res://fixture.png',(0,0,16,20),0,False,False,False)
    meta={'p10_activity_schema':1,'entity_bits':100,'art_id':agent,'activity_kind':1,
        'activity_place_known':True,'body':'adult_a','activity':'craft','pose':'sit','facing':'E',
        'activity_frame_activity':'sit','activity_frame_view':'E','sprite_id':'craft',
        'cold_base_pose':'sit','cold_pose':'sit','moving':False,'activity_overrides':[], 'sheltered_sleep':False,
        'activity_oracle_inputs':{'schema':1,'animation_seconds':2.0,'previous_facing':'N','movement':[0,0],
            'pickup':False,'meal':False,'morning_chill_alpha':0.0,'capture_visible':True,'parent_visible_in_tree':True}}
    row={'entity_bits':100,'agent_id':agent,'kind':1,'place':[5,5],'until_tick':10,'state_tag':4,
        'renderer_present':True,'missing_metadata':[],'metadata':meta,'visible':True,'visible_in_tree':True,
        'flip_h':False,'texture_path':'res://fixture.png','region_enabled':True,'region_rect':[0,0,16,20],
        'viewport_rect':[100,100,16,20],'intersects_viewport':True,'fully_inside_viewport':True}
    data={'state_change_frame':10,'capture_frame':130,'fixed_fps':60,'fixed_delta_valid':True,'complete':True,'errors':[],'pose_oracle_checked':False,'population':1,'holder_count':1,'viewport':[1920,1080],
        'raw_agent_snapshot':{'ids':[100],'agent_ids':[agent],'states':[4],'target_kinds':[8],'life_stages':[0],'colds':[0],'xs':[5],'ys':[5]},
        'raw_activity_snapshot':{'ids':[100],'kinds':[1],'place_x':[5],'place_y':[5],'until_tick':[10]},
        'raw_building_snapshot':{'xs':[6],'ys':[5],'kinds':[2]},
        'raw_society_snapshot':{'home_agent_ids':[agent],'home_settlement_ids':[1]},
        'settlements':{'settlement_ids':[1],'formation_xs':[0],'formation_ys':[0]},'activity_holders':[row]}
    return data,(frame,)

class OracleTests(unittest.TestCase):
    def test_exact_repeated_add_boundary(self):
        data,_=fixture()
        seconds=fixed_animation_seconds(data,2.0)
        self.assertEqual(seconds,1.9999999999999978)
        self.assertEqual(candidates(5,'child','E',seconds,True)[0],('run','E'))
        self.assertEqual(candidates(5,'child','E',seconds+1/60,True)[0],('hop','S'))
        self.assertEqual(candidates(5,'child','E',2.0,True)[0],('hop','S'))
    def test_schedule_and_serialized_time_fail_closed(self):
        data,_=fixture()
        for key,value in [('capture_frame',129),('state_change_frame',11),('fixed_fps',30),('fixed_delta_valid',False)]:
            altered=dict(data);altered[key]=value
            with self.assertRaises(ValueError): fixed_animation_seconds(altered,2.0)
        with self.assertRaises(ValueError): fixed_animation_seconds(data,119/60)
    def test_every_kind_body_view_chain_real_atlas(self):
        atlas=load_atlas(ROOT)
        for kind in range(7):
            for body in ('adult_a','adult_b','child'):
                for face in 'EWNS':
                    for seconds in (0.0,0.5):
                        with self.subTest(kind=kind,body=body,face=face,seconds=seconds):
                            pose,actual_face=daily(kind,body,face,seconds,True,atlas)
                            expected=['crouch','sit','build','sit','idle','idle','sit'][kind]
                            if kind==0 and body=='child': expected='sit'
                            if kind==5 and body=='child': expected='run' if seconds==0.5 and face in 'EW' else 'hop'
                            self.assertEqual(pose,expected)
                            self.assertTrue(any((f.body,f.pose,f.view)==(body,pose,'E' if actual_face=='W' else actual_face) for f in atlas))
    def test_play_half_second_boundary(self):
        self.assertEqual(candidates(5,'child','W',0.499,True)[0],('hop','S'))
        self.assertEqual(candidates(5,'child','W',0.5,True)[0],('run','W'))
        self.assertEqual(candidates(5,'child','N',0.5,True)[0],('hop','S'))
    def test_valid_fixture_is_not_h7(self):
        data,atlas=fixture(); report=check_capture(data,atlas)
        self.assertTrue(report['pose_check_complete']); self.assertFalse(report['h7_pass'])
    def test_missing_input_and_wrong_body_fail(self):
        for key in ('activity_oracle_inputs','body','sprite_id','activity_overrides'):
            data,atlas=fixture(); del data['activity_holders'][0]['metadata'][key]
            self.assertFalse(check_capture(data,atlas)['pose_check_complete'])
    def test_hidden_and_offscreen_are_checked(self):
        for hidden in (False,True):
            data,atlas=fixture(); row=data['activity_holders'][0]
            row['viewport_rect']=[3000,100,16,20]; row['intersects_viewport']=row['fully_inside_viewport']=False
            if hidden:
                row['metadata']['activity_oracle_inputs']['capture_visible']=False
                row['visible']=row['visible_in_tree']=False
            self.assertTrue(check_capture(data,atlas)['pose_check_complete'])
            row['metadata']['pose']='idle'
            self.assertFalse(check_capture(data,atlas)['pose_check_complete'])
    def test_duplicate_or_missing_holder_fails(self):
        data,atlas=fixture(); data['activity_holders']*=2
        self.assertFalse(check_capture(data,atlas)['pose_check_complete'])
        data['activity_holders']=[]
        self.assertFalse(check_capture(data,atlas)['pose_check_complete'])
    def test_unlisted_cold_fails_then_explicit_override_passes(self):
        data,atlas=fixture(); data['raw_agent_snapshot']['colds']=[1]
        cold=Frame('cold','adult_a','crossed','S','res://fixture.png',(0,0,16,20),0,False,False,False)
        atlas+=(cold,)
        self.assertFalse(check_capture(data,atlas)['pose_check_complete'])
        meta=data['activity_holders'][0]['metadata']
        meta.update(pose='crossed',cold_pose='crossed',activity_frame_activity='crossed',activity_frame_view='S',sprite_id='cold',activity_overrides=['cold'])
        self.assertTrue(check_capture(data,atlas)['pose_check_complete'])
    def test_pickup_then_meal_suppresses_cold(self):
        data,atlas=fixture(); meta=data['activity_holders'][0]['metadata']; data['raw_agent_snapshot']['colds']=[1]
        meta['activity_oracle_inputs'].update(pickup=True,meal=True)
        meta.update(activity='gather_wood',pose='eat',facing='N',cold_base_pose='eat',cold_pose='eat',activity_frame_activity='eat',activity_frame_view='S',sprite_id='meal',activity_overrides=['pickup','meal'])
        meal=Frame('meal','adult_a','eat','S','res://fixture.png',(0,0,16,20),0,False,False,False)
        self.assertTrue(check_capture(data,atlas+(meal,))['pose_check_complete'])
        meta['activity_overrides'].append('cold')
        self.assertFalse(check_capture(data,atlas+(meal,))['pose_check_complete'])
    def test_sheltered_sleep_holder_hidden(self):
        data,atlas=fixture(); raw=data['raw_agent_snapshot']; raw['states']=[3]; raw['target_kinds']=[3]
        data['raw_building_snapshot']={'xs':[5],'ys':[5],'kinds':[1]}; row=data['activity_holders'][0]
        row.update(state_tag=3,visible=False,visible_in_tree=False)
        row['metadata'].update(activity='asleep',pose='asleep',facing='N',cold_base_pose='asleep',cold_pose='asleep',activity_frame_activity='asleep',activity_frame_view='S',sprite_id='sleep',sheltered_sleep=True)
        sleep=Frame('sleep','adult_a','asleep','S','res://fixture.png',(0,0,16,20),0,False,False,False)
        self.assertTrue(check_capture(data,atlas+(sleep,))['pose_check_complete'])
    def test_chill_is_listed_independently(self):
        data,atlas=fixture(); meta=data['activity_holders'][0]['metadata']
        meta['activity_oracle_inputs']['morning_chill_alpha']=0.5
        cold=Frame('chill','adult_a','crossed','S','res://fixture.png',(0,0,16,20),0,False,False,False)
        meta.update(pose='crossed',cold_pose='crossed',activity_frame_activity='crossed',activity_frame_view='S',sprite_id='chill',activity_overrides=['morning_chill'])
        self.assertTrue(check_capture(data,atlas+(cold,))['pose_check_complete'])
        meta['activity_overrides']=['cold']
        self.assertFalse(check_capture(data,atlas+(cold,))['pose_check_complete'])
    def test_false_body_and_atlas_region_fail(self):
        for key,value in [('body','child'),('activity_frame_view','N'),('sprite_id','invented')]:
            data,atlas=fixture(); data['activity_holders'][0]['metadata'][key]=value
            self.assertFalse(check_capture(data,atlas)['pose_check_complete'])
        data,atlas=fixture(); data['activity_holders'][0]['region_rect']=[0,0,15,20]
        self.assertFalse(check_capture(data,atlas)['pose_check_complete'])
    def test_raw_place_changes_expected_facing(self):
        data,atlas=fixture(); data['raw_building_snapshot']['xs']=[4]
        self.assertFalse(check_capture(data,atlas)['pose_check_complete'])
    def test_unsupported_holder_state_fails_closed(self):
        data,atlas=fixture(); data['raw_agent_snapshot']['states']=[2]; data['raw_agent_snapshot']['target_kinds']=[5]
        data['activity_holders'][0]['state_tag']=2
        report=check_capture(data,atlas)
        self.assertFalse(report['pose_check_complete'])
        self.assertIn('unsupported non-activity state', report['errors'][0]['error'])
    def test_crop_missing_kind_reports_no_fake(self):
        data,_=fixture(); report=inventory([('capture.json',data)])
        self.assertFalse(report['complete']); self.assertEqual(len(report['crops']),1); self.assertEqual(len(report['missing_kinds']),7)
    def test_crop_quota_and_nearest_recipe(self):
        data,_=fixture(); row=data['activity_holders'][0]; rows=[]
        for kind in range(7):
            for index in range(2):
                entry=copy.deepcopy(row); entry.update(kind=kind,entity_bits=kind*2+index,agent_id=kind*2+index); rows.append(entry)
        data['activity_holders']=rows
        report=inventory([('capture.json',data)])
        self.assertTrue(report['complete']); self.assertEqual(len(report['crops']),14)
        self.assertTrue(all(r['upscale']==4 and r['resampling']=='nearest' for r in report['crops']))
        self.assertEqual(len({r['blind_id'] for r in report['crops']}),14)

if __name__=='__main__': unittest.main()
