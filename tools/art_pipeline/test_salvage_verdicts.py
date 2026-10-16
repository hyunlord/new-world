"""Synthetic frozen-review and stale-approval regressions; no live verdict reads."""
import hashlib
import json
from pathlib import Path
from tempfile import TemporaryDirectory
import unittest

from salvage_verdicts import validate_and_persist, propose_round2, reusable_approval, adjudicate
from PIL import Image


def digest(path):return hashlib.sha256(path.read_bytes()).hexdigest()


class VerdictsTest(unittest.TestCase):
    def fixture(self, root):
        candidate={'id':'N3_01@1','slot_id':'N3_01','body':'','images':{}}
        path=root/'candidates.json';path.write_text(json.dumps({'candidates':[candidate],'references':{},'selected':{}}))
        preview=root/'preview.json';preview.write_text(json.dumps({'candidates_sha256':digest(path),'images_sha256':{},'previews':{'N3_01':'N3_01@1'}}))
        part=root/'part.json';part.write_text(json.dumps([{'sprite_id':'N3_01','candidate_id':'N3_01@1','verdict':'ok','reason':'','observation':'clear','unfitted_acceptable':None}]))
        return path,preview,part

    def test_persist_preserves_observation_and_precedes_statistics(self):
        with TemporaryDirectory() as directory:
            root=Path(directory);ledger,preview,part=self.fixture(root)
            out=root/'authoritative.json'
            envelope=validate_and_persist([part],ledger,preview,out,expected_slots={'N3_01'})
            self.assertTrue(out.exists());self.assertNotIn('statistics',envelope)
            self.assertEqual(envelope['verdicts'][0]['observation'],'clear')
            self.assertEqual(envelope,json.loads(out.read_text()))

    def test_duplicate_missing_and_stale_mapping_rejected_without_write(self):
        with TemporaryDirectory() as directory:
            root=Path(directory);ledger,preview,part=self.fixture(root);out=root/'out.json'
            with self.assertRaises(ValueError):validate_and_persist([part,part],ledger,preview,out,expected_slots={'N3_01'})
            with self.assertRaises(ValueError):validate_and_persist([part],ledger,preview,out,expected_slots={'N3_01','N3_02'})
            values=json.loads(part.read_text());values[0]['candidate_id']='N3_01@2';part.write_text(json.dumps(values))
            with self.assertRaises(ValueError):validate_and_persist([part],ledger,preview,out,expected_slots={'N3_01'})
            self.assertFalse(out.exists())

    def test_reference_or_fingerprint_change_invalidates_old_approval(self):
        old={'references':{'adult_a':'idle@1'},'candidate_fingerprints':{'V_a1_05':'same'}}
        new={'references':{'adult_a':'idle@2'},'candidate_fingerprints':{'V_a1_05':'same'}}
        record={'sprite_id':'V_a1_05','candidate_id':'walk@1','verdict':'ok'}
        self.assertFalse(reusable_approval(record,old,new,'adult_a'))
        new['references']=dict(old['references']);self.assertTrue(reusable_approval(record,old,new,'adult_a'))
        new['candidate_fingerprints']['V_a1_05']='changed';self.assertFalse(reusable_approval(record,old,new,'adult_a'))

    def test_visual_ok_cannot_override_a_retained_technical_error(self):
        with TemporaryDirectory() as directory:
            root=Path(directory);ledger,preview,part=self.fixture(root)
            image=root/'native.png';Image.new('RGBA',(10,10),(95,84,60,255)).save(image)
            data=json.loads(ledger.read_text());data['scale']={'adult_height':28,'child_height':21}
            data['selected']={'N3_01':'N3_01@1'}
            data['candidates'][0].update(sheet='N3',cell=1,attempt=1,view='',activity='',
                images={'native':str(image)},metrics={'mean_delta_e':1.,'palette_match_percent':100.,
                'binary_alpha':True,'berry_texels':0,'fire_texels':0},
                guards={'passed':False,'errors':['invalid_grid_layout'],'warnings':[]})
            ledger.write_text(json.dumps(data))
            frozen=json.loads(preview.read_text());frozen['candidates_sha256']=digest(ledger);preview.write_text(json.dumps(frozen))
            output=root/'authoritative.json'
            validate_and_persist([part],ledger,preview,output,expected_slots={'N3_01'})
            result=adjudicate(ledger,output)
            self.assertEqual(result['accepted'],{})
            self.assertEqual(result['not_accepted']['N3_01'],['invalid_grid_layout'])

    def test_changed_region_card_invalidates_prior_approval(self):
        prior={'references':{},'candidate_fingerprints':{'N3_01':'same'},'rendered_fingerprints':{'N3_01':'old'}}
        current={'references':{},'candidate_fingerprints':{'N3_01':'same'},'rendered_fingerprints':{'N3_01':'new'}}
        self.assertFalse(reusable_approval({'sprite_id':'N3_01','verdict':'ok'},prior,current,''))

    def test_round2_switches_whole_group_and_rereviews_body_after_reference_change(self):
        records=[];groups=[]
        for attempt in (1,2):
            ids=[]
            for cell in (5,6,7,8):
                identity=f'V_a1_{cell:02d}@{attempt}';ids.append(identity)
                records.append({'id':identity,'slot_id':f'V_a1_{cell:02d}','sheet':'V_a1','cell':cell,'attempt':attempt,
                 'body':'adult_a','view':'S','activity':'walk','frame':cell-5,'guards':{'passed':True,'warnings':[]},'metrics':{'mean_delta_e':2.}})
            records.append({'id':f'V_a1_01@{attempt}','slot_id':'V_a1_01','sheet':'V_a1','cell':1,'attempt':attempt,
                'body':'adult_a','view':'S','activity':'idle','frame':0,'guards':{'passed':True,'warnings':[]},'metrics':{'mean_delta_e':2.}})
            groups.append({'body':'adult_a','sheet':'V_a1','view':'S','activity':'walk','attempt':attempt,
                'passing_original_count':4,'mean_delta_e':2.,'candidate_ids':ids,'guards':{'passed':True,'warnings':[]}})
        ledger={'candidates':records,'groups':groups,'selected':{r['slot_id']:r['id'] for r in records if r['attempt']==1},'references':{'adult_a':'V_a1_01@1'}}
        envelope={'round':1,'verdicts':[{'sprite_id':'V_a1_05','candidate_id':'V_a1_05@1','verdict':'reject','reason':'wrong_pose'}]}
        plan=propose_round2(ledger,envelope)
        for cell in (5,6,7,8):self.assertEqual(plan['ledger']['selected'][f'V_a1_{cell:02d}'],f'V_a1_{cell:02d}@2')
        self.assertIn('V_a1_01',plan['affected_slots'])
        self.assertEqual(plan['ledger']['references']['adult_a'],'V_a1_01@2')
        self.assertEqual(ledger['references']['adult_a'],'V_a1_01@1')

    def test_no_alternate_preserves_only_approved_original_same_attempt(self):
        records=[]
        for cell in (1,2):
            records.append({'id':f'V_a1_{cell:02d}@1','slot_id':f'V_a1_{cell:02d}',
                'body':'adult_a','sheet':'V_a1','view':'S','activity':'idle','attempt':1,
                'guards':{'passed':True,'warnings':[]},'metrics':{'mean_delta_e':1.}})
        ledger={'candidates':records,'groups':[],
                'selected':{r['slot_id']:r['id'] for r in records},'references':{'adult_a':'V_a1_01@1'}}
        review={'round':1,'verdicts':[
            {'sprite_id':'V_a1_01','candidate_id':'V_a1_01@1','verdict':'ok'},
            {'sprite_id':'V_a1_02','candidate_id':'V_a1_02@1','verdict':'reject'}]}
        proposal=propose_round2(ledger,review)
        self.assertEqual(proposal['ledger']['selected'],{'V_a1_01':'V_a1_01@1'})
        self.assertEqual(proposal['no_alternate_slots'],['V_a1_02'])
        self.assertEqual(ledger['selected'],{'V_a1_01':'V_a1_01@1','V_a1_02':'V_a1_02@1'})

if __name__=='__main__':unittest.main()
