"""Selection regressions: failures remain evidence and groups never mix attempts."""
import unittest
from pathlib import Path
from tempfile import TemporaryDirectory
from PIL import Image
from salvage_v4 import select_candidates


def candidate(cell, attempt, *, activity='walk', body='adult_a', sheet='V_a1', failed=False):
    slot=f'{sheet}_{cell:02d}'
    return {'id':f'{slot}@{attempt}','slot_id':slot,'sheet':sheet,'cell':cell,'attempt':attempt,
            'body':body,'view':'S' if body else '', 'activity':activity,'frame':cell-5,'name':activity,
            'source_kind':'generated','raw_path':'fixture','raw_box':[0,0,1,1], 'images':{},
            'factor':1.,'metrics':{'mean_delta_e':2.,'palette_match_percent':100.,'binary_alpha':True,
            'berry_texels':0,'fire_texels':20},'guards':{'errors':['geometry'] if failed else [],'warnings':[],
            'passed':not failed},'review_status':'pending','conversion_error':None}


class SalvageTest(unittest.TestCase):
    def _images(self, records, root):
        for record in records:
            image=Image.new('RGBA',(18,28),(95,84,60,255))
            if record['cell']%2==0:
                for x in range(5):image.putpixel((x,26),(174,150,111,255))
            path=root/f"{record['id']}.png";image.save(path)
            record['images']={'native':str(path),'sprite':str(path),'raw':str(path),'unfitted':str(path)}

    def test_group_stays_in_one_attempt_and_preserves_failure(self):
        with TemporaryDirectory() as directory:
            root=Path(directory)
            records=[candidate(i,a,failed=a==2 and i==8) for a in (1,2) for i in range(5,9)]
            self._images(records,root)
            result=select_candidates(records,root,[(95,84,60),(174,150,111)])
            self.assertEqual({value.split('@')[1] for value in result['selected'].values()},{'1'})
            self.assertEqual(sum(r['source_kind']=='generated' for r in records),8)
            self.assertFalse(next(r for r in records if r['id']=='V_a1_08@2')['guards']['passed'])

    def test_missing_cycle_frame_derived_from_same_attempt_and_rechecked(self):
        with TemporaryDirectory() as directory:
            root=Path(directory);records=[candidate(i,1,failed=i==8) for i in range(5,9)]
            self._images(records,root)
            records[1]['grid']={'period_xy':[4.,4.]}
            records[1]['factor_source']='standing_group'
            records[1]['guards']['warnings']=['grid_suspect']
            records[-1]['grid']={'period_xy':[8.,8.]}
            records[-1]['factor_source']='failed_target'
            result=select_candidates(records,root,[(95,84,60),(174,150,111)])
            selected=result['selected']['V_a1_08']
            derived=next(r for r in records if r['id']==selected)
            self.assertEqual(derived['source_kind'],'derived')
            self.assertEqual(derived['source_candidate'],'V_a1_06@1')
            self.assertTrue(derived['group_guards']['passed'])
            self.assertEqual(derived['grid'],records[1]['grid'])
            self.assertEqual(derived['factor_source'],'standing_group')
            self.assertIn('grid_suspect',derived['guards']['warnings'])
            self.assertIn('derived_fidelity_inherited',derived['guards']['warnings'])

    def test_failed_cycle_is_not_selected(self):
        with TemporaryDirectory() as directory:
            root=Path(directory);records=[candidate(i,1) for i in range(5,9)]
            self._images(records,root)
            for record in records:Image.new('RGBA',(18,28),(95,84,60,255)).save(record['images']['native'])
            result=select_candidates(records,root,[(95,84,60),(174,150,111)])
            self.assertEqual(result['selected'],{})
            self.assertIn('leg_difference',result['groups'][0]['guards']['errors'])

    def test_cross_slot_berry_failure_excludes_bushes_and_baskets(self):
        with TemporaryDirectory() as directory:
            root=Path(directory)
            records=[candidate(i,1,body='',sheet='N2',activity='') for i in range(1,5)]
            records.append(candidate(11,1,body='',sheet='S2',activity=''))
            self._images(records,root)
            for record,count in zip(records,[1,3,2,0,0]):
                record['metrics']['berry_texels']=count
            result=select_candidates(records,root,[(95,84,60),(174,150,111)])
            self.assertIn('berry_stage_order',result['initial_cross_guards']['errors'])
            self.assertTrue(result['cross_guards']['passed'])
            self.assertEqual(result['selected'],{})
            self.assertTrue(all(r['review_status']=='ineligible' for r in records))

    def test_cross_guards_recover_alternative_bush_and_basket(self):
        with TemporaryDirectory() as directory:
            root=Path(directory)
            records=[candidate(i,1,body='',sheet='N2',activity='') for i in range(1,5)]
            records.extend([candidate(1,2,body='',sheet='N2',activity=''),
                            candidate(11,1,body='',sheet='S2',activity=''),
                            candidate(11,2,body='',sheet='S2',activity='')])
            self._images(records,root)
            for record,count in zip(records,[1,3,2,0,5,0,0]):record['metrics']['berry_texels']=count
            Image.new('RGBA',(10,10),(95,84,60,255)).save(records[-1]['images']['native'])
            result=select_candidates(records,root,[(95,84,60),(174,150,111)])
            self.assertEqual(result['selected']['N2_01'],'N2_01@2')
            self.assertEqual(result['selected']['S2_11'],'S2_11@2')
            self.assertEqual(result['initial_object_selection']['N2_01'],'N2_01@1')
            self.assertTrue(result['cross_guards']['passed'])

if __name__=='__main__':unittest.main()
