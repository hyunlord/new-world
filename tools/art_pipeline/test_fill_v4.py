"""Fill catalogs preserve budgets, same-body factors and the six-call denominator."""
import json
from pathlib import Path
from tempfile import TemporaryDirectory
import unittest
from PIL import Image
from fill_catalog import load_catalog, prompt_for_fill
from fill_v4 import contract_spec, fill_factors, checkpoint
from salvage_review import load_review
from salvage_verdicts import validate_and_persist
import hashlib
from v4_contracts import FrameCandidate, guard_frame, target_size


class FillTest(unittest.TestCase):
    def test_object_alias_controls_budget_and_fire_guard(self):
        sheet={'id':'P2_any','body':''};cell={'cell':16,'view':'','activity':'','budget_alias':{'sheet':'S1','cell':2}}
        spec=contract_spec(sheet,cell,1,(30,30))
        self.assertEqual(target_size(spec),(24,26))
        self.assertIn('fire_ramp',guard_frame(FrameCandidate(spec,Image.new('RGBA',(20,20),(90,60,30,255)),1.,100.,True,0,0)).errors)

    def test_nonstanding_inherits_recorded_reference_factor(self):
        sheet={'id':'P2_rest','body':'child'}
        cells=[{'cell':1,'view':'E','activity':'drink','budget_alias':None}]
        specs=[contract_spec(sheet,cells[0],1,(20,18))]
        factors=fill_factors(specs,{'id':'reference','factor':.63,'body':'child'})
        self.assertEqual(factors[0].factor,.63)
        self.assertEqual(factors[0].source,'final_reference_factor:reference')
        with self.assertRaises(ValueError):fill_factors(specs,{'id':'wrong_body','factor':.63,'body':'adult_a'})

    def test_first_six_catalog_cells_and_missing_child_activities(self):
        root=Path(__file__).resolve().parents[2]
        catalog=load_catalog(root/'docs/art/production2/fill_sheets.json')
        self.assertEqual(sum(len(s['cells']) for s in catalog['sheets'][:6]),96)
        self.assertEqual(len({c['target_slot'] for s in catalog['sheets'][:6] for c in s['cells']}),96)
        children=[c for s in catalog['sheets'][:6] if s['body']=='child' for c in s['cells']]
        self.assertTrue({'carry','gather','build','drink'}.issubset({c['activity'] for c in children}))
        for sheet in catalog['sheets']:
            text=prompt_for_fill(sheet)
            import re
            self.assertEqual(set(re.findall(r'#[0-9A-Fa-f]{6}',text)),{'#FF00FF'})
            self.assertNotRegex(text,r'(?i)texel|logical.pixel|\d[–-]\d+ image pixels|\d[- ]pixel|native.*budget')
            self.assertEqual(text.count('Row '),4)

    def test_embedded_catalog_review_and_verdict_coverage(self):
        with TemporaryDirectory() as directory:
            root=Path(directory)
            sheet={'id':'Fill_custom','body':'child','cells':[{'cell':i,'target_slot':f'child_new_{i}',
                'name':'drink','view':'E','activity':'drink','frame':i-1} for i in range(1,17)]}
            records=[{'id':f'Fill_custom_{i:02d}@1','slot_id':f'Fill_custom_{i:02d}','body':'child',
                'target_slot':f'child_new_{i}','images':{}} for i in range(1,17)]
            data={'schema_version':1,'candidates':records,'selected':{},'references':{},
                  'review_catalog':{'schema_version':1,'sheets':[sheet]}}
            path=root/'ledger.json';path.write_text(json.dumps(data))
            loaded=load_review(path);self.assertEqual(list(loaded.catalog),['Fill_custom'])
            self.assertEqual(loaded.candidates[0].target_slot,'child_new_1')
            preview=root/'preview.json';preview.write_text(json.dumps({'candidates_sha256':hashlib.sha256(path.read_bytes()).hexdigest(),
                'images_sha256':{},'previews':{r['slot_id']:r['id'] for r in records}}))
            part=root/'part.json';part.write_text(json.dumps([{'sprite_id':r['slot_id'],'candidate_id':r['id'],
                'verdict':'ok','reason':'','observation':'synthetic','unfitted_acceptable':None} for r in records]))
            envelope=validate_and_persist([part],path,preview,root/'verdicts.json')
            self.assertEqual(len(envelope['verdicts']),16)

    def test_checkpoint_denominator_and_duplicate_exclusion(self):
        record={'id':'candidate','slot_id':'physical','target_slot':'new','source_kind':'generated'}
        ledgers=[{'attempted_cells':16,'candidates':[record]} for _ in range(6)]
        reports=[{'accepted':{'physical':'candidate'}} for _ in range(6)]
        result=checkpoint(ledgers,reports,{'slots':{}})
        self.assertEqual(result['attempted_cells'],96)
        self.assertEqual(result['newly_accepted_count'],1)
        self.assertTrue(result['stop_generation'])
        self.assertEqual(checkpoint(ledgers,reports,{'slots':{'new':{'disposition':'accepted'}}})['newly_accepted_count'],0)
        held={'slots':{'new':{'disposition':'accepted','candidate_id':'old@1'}}}
        self.assertEqual(checkpoint(ledgers,reports,held,{'old@1'})['newly_accepted_count'],1)

if __name__=='__main__':unittest.main()
