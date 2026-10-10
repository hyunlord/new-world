"""Installation requires frozen review evidence and safe same-body fallbacks."""
from pathlib import Path
import json
from tempfile import TemporaryDirectory
import unittest
from unittest.mock import patch
from dataclasses import replace
from v4_contracts import WorldScale, FrameCandidate, FrameSpec
from PIL import Image
import install_v4
from salvage_verdicts import candidate_fingerprint


def reviewed_fixture(root: Path) -> install_v4.InstallRequest:
    """Build a complete frozen evidence chain for one genuinely reviewed object."""
    (root/'assets/art').mkdir(parents=True)
    image=root/'raw.png';Image.new('RGBA',(3,3),(216,126,51,255)).save(image)
    (root/'assets/art/manifest.json').write_text(json.dumps({'assets':[]}))
    row={'id':'N3_01@1','slot_id':'N3_01','sheet':'N3','cell':1,'attempt':1,
         'body':'','view':'','activity':'','frame':0,'source_kind':'generated',
         'metrics':{'mean_delta_e':0.0,'palette_match_percent':100.0,'binary_alpha':True,'berry_texels':0,'fire_texels':0},
         'factor':1.0,'images':{'native':str(image),'sprite':str(image)},
         'guards':{'passed':True,'errors':[],'warnings':[]}}
    ledger=root/'ledger.json';ledger.write_text(json.dumps({'scale':{'adult_height':28,'child_height':21},'candidates':[row]}))
    verdict=root/'verdict.json';verdict.write_text(json.dumps({
        'verdicts':[{'sprite_id':'N3_01','candidate_id':'N3_01@1','verdict':'ok'}],
        'candidate_fingerprints':{'N3_01':candidate_fingerprint(row)}}))
    decision={'candidate_id':'N3_01@1','disposition':'accepted','ledger':'ledger.json',
        'ledger_sha256':install_v4.digest(ledger),'verdict':'verdict.json',
        'verdict_sha256':install_v4.digest(verdict)}
    selection=root/'selection.json';selection.write_text(json.dumps({'schema_version':1,
        'scale':{'adult_height':28,'child_height':21},'slots':{'N3_01':decision}}))
    return install_v4.InstallRequest(root,selection)


class InstallV4Tests(unittest.TestCase):
    def test_rejects_changed_selection_evidence(self) -> None:
        with TemporaryDirectory() as directory:
            root=Path(directory)
            ledger=root/'ledger.json'; ledger.write_text('{}')
            with self.assertRaises(install_v4.InstallError):
                install_v4.verify_file(ledger, '0'*64)

    def test_policy_is_explicit(self) -> None:
        self.assertEqual(install_v4.slot_policy('S1_01'), 'retain')
        self.assertEqual(install_v4.slot_policy('S1_16'), 'retain')
        self.assertEqual(install_v4.slot_policy('S2_04'), 'retain')
        self.assertEqual(install_v4.slot_policy('S2_05'), 'suppress')
        self.assertEqual(install_v4.slot_policy('N2_04'), 'retain')
        self.assertEqual(install_v4.slot_policy('N2_05'), 'suppress')
        self.assertEqual(install_v4.slot_policy('U1_01'), 'icon')

    def test_named_icon_glyph_cannot_be_a_generic_blob(self) -> None:
        self.assertTrue(install_v4.meaningful_icon_source('U1_01','procedural:ascii:people:phase0','fallback'))
        self.assertFalse(install_v4.meaningful_icon_source('U1_01','procedural:ascii:pebble:phase0','fallback'))
        self.assertFalse(install_v4.meaningful_icon_source('U1_01','procedural:ascii:grass:phase0','fallback'))

    def test_fallback_does_not_cross_body_or_reuse_basket(self) -> None:
        image=Image.new('RGBA',(12,28),(216,126,51,255))
        source=install_v4.PlannedSprite('V_a2_01', 'adult_a','S','carry',0,
            'accepted','generated:V_a2/01@1:v4:f=1','',True,'','',image,image)
        target=install_v4.Target('V_b1_01','adult_b','S','idle',0)
        self.assertIsNone(install_v4.fallback_for(target,(source,),frozenset()))
        target=install_v4.Target('V_a1_01','adult_a','S','idle',0)
        self.assertIsNone(install_v4.fallback_for(target,(source,),frozenset()))

    def test_same_body_idle_icon_is_honest(self) -> None:
        image=Image.new('RGBA',(12,28),(216,126,51,255))
        source=install_v4.PlannedSprite('V_a1_01','adult_a','S','idle',0,
            'accepted','generated:V_a1/01@1:v4:f=1','',False,'','',image,image)
        target=install_v4.Target('V_a3_03','adult_a','E','drink',0)
        self.assertIsNone(install_v4.fallback_for(target,(source,),frozenset({'U1_11'})))

    def test_idle_icon_after_procedural_size_failure(self) -> None:
        image=Image.new('RGBA',(12,28),(216,126,51,255))
        source=install_v4.PlannedSprite('V_a1_04','adult_a','E','idle',0,
            'accepted','generated:V_a1/04@1:v4:f=1','',False,'','',image,image)
        target=install_v4.Target('V_a3_03','adult_a','E','drink',0)
        result=install_v4.fallback_for(target,(source,),frozenset({'U1_11'}),(1,1))
        self.assertIsNotNone(result)
        self.assertEqual(result.fallback_kind,'idle_icon')
        self.assertEqual(result.activity_icon,'U1_11')
        self.assertEqual(result.qa_status,'fallback')
        self.assertEqual(result.source_sprite,'V_a1_04')

    def test_frozen_plan_dry_run_and_temporary_install(self) -> None:
        with TemporaryDirectory() as directory:
            root=Path(directory);request=reviewed_fixture(root)
            before=(root/'assets/art/manifest.json').read_bytes()
            plan=install_v4.build_plan(request)
            self.assertEqual(before,(root/'assets/art/manifest.json').read_bytes())
            self.assertFalse((root/'assets/art/production').exists())
            self.assertEqual([s.slot_id for s in plan.sprites],['N3_01'])
            self.assertIn('N3_02',plan.suppressed)
            install_v4.apply_plan(plan,plan.selection_sha256)
            manifest=json.loads((root/'assets/art/manifest.json').read_text())
            entry=next(e for e in manifest['assets'] if e.get('sprite_id')=='N3_01')
            self.assertEqual(entry['source'],'generated:N3/01@1:v4:f=1')
            self.assertEqual(entry['qa_status'],'accepted')
            self.assertTrue((root/entry['native_path']).is_file())
            self.assertFalse(manifest['suppressed']['N3_02']['drawable'])

    def test_mutated_pixels_and_hold_cannot_be_installed(self) -> None:
        with TemporaryDirectory() as directory:
            root=Path(directory);request=reviewed_fixture(root)
            plan=install_v4.build_plan(request)
            Image.new('RGBA',(3,3),(99,99,99,255)).save(root/'raw.png')
            with self.assertRaises(install_v4.InstallError):
                install_v4.apply_plan(plan,plan.selection_sha256)
            self.assertFalse((root/'assets/art/production').exists())
            with self.assertRaises(install_v4.InstallError):
                install_v4.build_plan(request)
        with TemporaryDirectory() as directory:
            root=Path(directory);request=reviewed_fixture(root)
            holds=root/'holds.json';holds.write_text(json.dumps({'installation_holds':{'N3_01@1':'visible defect'}}))
            plan=install_v4.build_plan(install_v4.InstallRequest(root,request.selection,holds=holds))
            self.assertFalse(plan.sprites)
            self.assertIn('N3_01',plan.suppressed)

    def test_derived_cross_body_lineage_is_rejected(self) -> None:
        with TemporaryDirectory() as directory:
            root=Path(directory);request=reviewed_fixture(root)
            ledger=json.loads((root/'ledger.json').read_text())
            record=ledger['candidates'][0]
            record.update(source_kind='derived',source_candidate='foreign',derivation_rule='copy')
            ledger['candidates'].append({'id':'foreign','body':'adult_b','activity':'idle'})
            (root/'ledger.json').write_text(json.dumps(ledger))
            selection=json.loads(request.selection.read_text())
            decision=selection['slots']['N3_01']
            decision['disposition']='derived'
            decision['ledger_sha256']=install_v4.digest(root/'ledger.json')
            request.selection.write_text(json.dumps(selection))
            with self.assertRaisesRegex(install_v4.InstallError,'cross-body'):
                install_v4.build_plan(request)

    def test_mirror_and_south_fallback_preserve_matching_frame(self) -> None:
        sources=[]
        for frame in range(4):
            image=Image.new('RGBA',(12,28),(216,126,51,255))
            image.putpixel((frame,27),(99,40,10,255))
            sources.append(install_v4.PlannedSprite(f'V_a1_{frame+5:02d}','adult_a','S','walk',frame,
                'accepted','generated:test','',False,'','',image,image))
        for frame in range(4):
            target=install_v4.Target(f'P2V_a_E_walk_{frame+1:02d}','adult_a','E','walk',frame)
            result=install_v4.fallback_for(target,tuple(sources),frozenset())
            self.assertEqual(result.source_sprite,sources[frame].slot_id)
            mirrors=tuple(replace(source,view='W') for source in sources)
            result=install_v4.fallback_for(target,mirrors,frozenset())
            self.assertEqual(result.source_sprite,sources[frame].slot_id)

    def test_supplement_runtime_target_uses_physical_review_keys(self) -> None:
        with TemporaryDirectory() as directory:
            root=Path(directory);request=reviewed_fixture(root)
            Image.new('RGBA',(12,28),(216,126,51,255)).save(root/'raw.png')
            ledger=json.loads((root/'ledger.json').read_text());row=ledger['candidates'][0]
            row.update(id='P2_fill_01_01@1',slot_id='P2_fill_01_01',target_slot='P2V_a_W_idle_01',
                sheet='V_a1',cell=4,source_sheet='P2_fill_01',source_cell=1,body='adult_a',view='W',activity='idle',name='W_idle')
            (root/'ledger.json').write_text(json.dumps(ledger))
            (root/'verdict.json').write_text(json.dumps({'verdicts':[{'sprite_id':row['slot_id'],'candidate_id':row['id'],'verdict':'ok'}],
                'candidate_fingerprints':{row['slot_id']:candidate_fingerprint(row)}}))
            selection=json.loads(request.selection.read_text());decision=selection['slots'].pop('N3_01')
            decision.update(candidate_id=row['id'],ledger_sha256=install_v4.digest(root/'ledger.json'),
                verdict_sha256=install_v4.digest(root/'verdict.json'),semantic={key:row[key] for key in ('body','view','activity','frame','name')})
            selection['slots'][row['target_slot']]=decision;request.selection.write_text(json.dumps(selection))
            with patch.object(install_v4,'fallback_for',return_value=None):
                plan=install_v4.build_plan(request)
            self.assertEqual([sprite.slot_id for sprite in plan.sprites],[row['target_slot']])
            self.assertEqual(plan.sprites[0].candidate_id,row['id'])
            self.assertEqual(plan.sprites[0].source,'generated:P2_fill_01/01@1:v4:f=1')
            # Historical passing flags cannot rescue a currently invalid height.
            Image.new('RGBA',(12,40),(216,126,51,255)).save(root/'raw.png')
            verdict=json.loads((root/'verdict.json').read_text())
            verdict['candidate_fingerprints'][row['slot_id']]=candidate_fingerprint(row)
            (root/'verdict.json').write_text(json.dumps(verdict))
            decision['verdict_sha256']=install_v4.digest(root/'verdict.json')
            request.selection.write_text(json.dumps(selection))
            with self.assertRaisesRegex(install_v4.InstallError,'Current frame guard'):
                install_v4.build_plan(request)

    def test_physical_sheet_and_attempt_coherence_is_rechecked(self) -> None:
        image=Image.new('RGBA',(12,28),(216,126,51,255))
        candidate=FrameCandidate(FrameSpec('V_a1',1,1,'adult_a','S','idle'),image,0,100,True,0,0)
        for second in (('P2_fill_02',1,candidate),('P2_fill_01',2,candidate)):
            with self.subTest(second=second[:2]):
                with self.assertRaisesRegex(install_v4.InstallError,'Mixed physical'):
                    install_v4.validate_physical_group((('P2_fill_01',1,candidate),second),WorldScale())

    def test_assembled_cycle_and_cross_guards_reject_current_pixels(self) -> None:
        image=Image.new('RGBA',(12,28),(216,126,51,255))
        sources=tuple(install_v4.PlannedSprite(f'V_a1_{i+5:02d}','adult_a','S','walk',i,
            'fallback','derived:test','',False,'south_view','',image,image) for i in range(4))
        diagnostics=install_v4.validate_assembled(sources,(),WorldScale())
        self.assertTrue(any('leg_difference' in value for value in diagnostics))
        with self.assertRaisesRegex(install_v4.InstallError,'leg_difference'):
            install_v4.validate_assembled(tuple(replace(source,qa_status='accepted') for source in sources),(),WorldScale())
        bushes=tuple(install_v4.PlannedSprite(f'N2_{i:02d}','','','',0,
            'accepted','generated:test','',False,'','',Image.new('RGBA',(4,4),(251,42,58,255)),image) for i in range(1,5))
        with self.assertRaisesRegex(install_v4.InstallError,'berry_stage_order'):
            install_v4.validate_assembled(bushes,(),WorldScale())


if __name__ == '__main__':
    unittest.main()
