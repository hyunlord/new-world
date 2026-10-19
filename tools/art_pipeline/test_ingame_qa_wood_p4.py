"""Missing Pass 5 evidence cannot become a vacuous wood pass."""
import json
from pathlib import Path
from tempfile import TemporaryDirectory
import unittest
from ingame_qa_wood_p4 import wood
from test_ingame_qa import capture

class WoodTests(unittest.TestCase):
    def test_missing_columns_and_carry_frame_fail(self):
        with TemporaryDirectory() as folder:
            root = Path(folder)
            (root/'assets/art/atlas').mkdir(parents=True)
            (root/'assets/art/atlas/world_atlas.json').write_text('{"sprites":{}}')
            (root/'frame.entities.json').write_text('{"entities":[]}')
            check = wood([capture(root/'frame.png',())],root)
            self.assertEqual(check.status,'FAIL')
            detail = json.loads(check.detail)
            self.assertIn('frame: missing settlement columns', detail['errors'])
            self.assertIn('missing l_d10_1500_F2 carry observation',detail['errors'])

class CompleteWoodEvidenceTests(unittest.TestCase):
    def setUp(self):
        self.temp = TemporaryDirectory(); self.root = Path(self.temp.name)
        (self.root/'assets/art/atlas').mkdir(parents=True)
        self.atlas = {'sprites':{'walk':{'activity':'walk','body':'adult_a','drawable':True},'basket':{'activity':'carry','body':'adult_a','contains_basket':True,'drawable':True}}}
        self.data = {'tick':10,'camera_position':[464,44],'viewport_size':[1920,1080],'zoom':[4,4],
            'settlement_snapshot':{'settlement_ids':[1],'formation_xs':[0],'formation_ys':[0],'wood_stocks':[1],'fire_lit':[0]},
            'resource_snapshot':{'xs':[0],'ys':[0],'kinds':[3],'amounts':[1],'maxes':[1]},
            'agent_snapshot':{'agent_ids':[7],'carried_woods':[1],'xs':[1],'ys':[0],'states':[1]},
            'agent_motion_evidence':{'start_tick':8,'end_tick':10,'before':{'agent_ids':[7],'xs':[0],'ys':[0]}},
            'labels':[{'name':'/HudTopbar/wood','text':'Firewood 1'}],
            'entities':[{'kind':'building','id':1,'building_kind':0,'tile_x':0,'tile_y':0,'visible':True,'sprite_id':'S1_01','settlement_id':1,'woodpile_stage':1},
                {'kind':'camp_prop','id':101,'woodpile_stage':1,'shown':True,'sprite_id':'S2_15'},
                {'kind':'deadwood','id':1,'tile_x':0,'tile_y':0,'shown':True,'deadwood_stage':3,'sprite_id':'N1_15'},
                {'kind':'agent','id':7,'visible':True,'frame_id':'walk','body':'adult_a','activity':'carry_wood','fallback_kind':'wood_icon'},
                {'kind':'activity_icon','id':7,'shown':True,'visible':True,'sprite_id':'S2_15'}]}
    def tearDown(self):
        self.temp.cleanup()
    def measure(self):
        (self.root/'assets/art/atlas/world_atlas.json').write_text(json.dumps(self.atlas))
        (self.root/'l_d10_1500_F2.entities.json').write_text(json.dumps(self.data))
        return wood([capture(self.root/'l_d10_1500_F2.png',())],self.root)
    def test_valid_observed_wood_fallback_passes(self):
        self.assertEqual(self.measure().status,'PASS')
    def test_food_basket_cannot_pass_as_wood_fallback(self):
        self.data['entities'][3]['frame_id']='basket'
        self.assertEqual(self.measure().status,'FAIL')
    def test_missing_wood_icon_fails(self):
        self.data['entities'].pop()
        self.assertEqual(self.measure().status,'FAIL')
    def test_missing_visible_stockpile_label_fails(self):
        self.data['entities'].append({'kind':'building','id':2,'building_kind':2,'tile_x':1,'tile_y':1,'visible':True})
        self.assertEqual(self.measure().status,'FAIL')
    def test_missing_carrier_column_does_not_become_na(self):
        del self.data['agent_snapshot']['carried_woods']
        self.assertEqual(self.measure().status,'FAIL')

    def test_unassigned_stockpile_does_not_require_label(self):
        self.data['entities'].extend([
            {'kind':'building','id':2,'building_kind':2,'tile_x':1,'tile_y':1,'visible':True},
            {'kind':'building','id':3,'building_kind':2,'tile_x':5,'tile_y':5,'visible':True},
            {'kind':'stockpile_label','id':(1<<32)|1,'wood_stock':1,'text':'Firewood 1','shown':True,'visible':True,'rect':[0,0,80,20]}])
        self.assertEqual(self.measure().status,'PASS')

    def test_assigned_offscreen_label_is_not_required_to_be_visible(self):
        self.data['entities'].extend([
            {'kind':'building','id':2,'building_kind':2,'tile_x':1,'tile_y':1,'visible':True},
            {'kind':'stockpile_label','id':(1<<32)|1,'wood_stock':1,'text':'Firewood 1','shown':True,'visible':False,'rect':[0,-30,80,20]}])
        self.assertEqual(self.measure().status,'PASS')

    def test_previously_displaced_stationary_carrier_is_diagnostic_only(self):
        self.data['agent_snapshot']['states']=[0]
        self.data['entities'][3]['activity']='idle'
        self.data['entities'][3]['fallback_kind']=''
        self.data['entities'].pop()
        check = self.measure()
        self.assertEqual(check.status,'PASS')
        detail = json.loads(check.detail)
        self.assertEqual(detail['counts']['displaced_wood_carriers'],1)
        self.assertEqual(detail['counts']['carrying'],0)
        self.assertEqual(detail['carry_status'],'N/A')

    def test_currently_walking_displaced_carrier_with_idle_without_icon_fails(self):
        self.data['entities'][3]['activity']='idle'
        self.data['entities'][3]['fallback_kind']=''
        self.data['entities'].pop()
        self.assertEqual(self.measure().status,'FAIL')

    def test_missing_state_column_fails_instead_of_na(self):
        del self.data['agent_snapshot']['states']
        self.assertEqual(self.measure().status,'FAIL')

    def test_split_label_target_and_tint(self):
        self.data['settlement_snapshot']['wood_targets'] = [273]
        self.data['entities'].extend([
            {'kind':'building','id':2,'building_kind':2,'tile_x':1,'tile_y':1,'visible':True},
            {'kind':'stockpile_label','id':(1<<32)|1,'text':'Food 5','shown':True,'visible':True,'rect':[0,0,40,20]},
            {'kind':'stockpile_wood_label','id':(1<<32)|1,'wood_stock':1,'wood_target':273,'text':'Firewood 1/273','modulate':[0.96,0.62,0.32,1.0],'shown':True,'visible':True,'rect':[48,0,90,20]}])
        self.assertEqual(self.measure().status,'PASS')
        self.data['entities'][-1]['text'] = 'Firewood 1'
        self.assertEqual(self.measure().status,'FAIL')
        self.data['entities'][-1]['text'] = 'Firewood 1/273'
        self.data['entities'][-1]['modulate'] = [1.0,1.0,1.0,1.0]
        self.assertEqual(self.measure().status,'FAIL')
