# /// script
# requires-python = ">=3.10"
# dependencies = ["Pillow", "numpy"]
# ///
"""Synthetic integration contracts; no production review files are read."""
import json
from pathlib import Path
import tempfile
import unittest
from PIL import Image
from merge_fill_selection import FillInput, merge_selection
from salvage_verdicts import sha

class MergeTest(unittest.TestCase):
    def test_mapping_group_replacement_hold_and_provenance(self):
        with tempfile.TemporaryDirectory() as directory:
            root=Path(directory)
            image=root/'native.png';Image.new('RGBA',(12,28),(100,80,60,255)).save(image)
            def record(identity, slot, sheet, cell, target=None, kind='generated'):
                return dict(id=identity,slot_id=slot,target_slot=target or slot,sheet=sheet,cell=cell,
                    attempt=1,body='adult_a',view='S',activity='talk',frame=cell-1,name='S_talk',
                    source_kind=kind,images={'native':str(image)},guards={'errors':[]},
                    metrics=dict(mean_delta_e=0,palette_match_percent=100,binary_alpha=True,berry_texels=0,fire_texels=0))
            old=[record('old1','V_a3_09','V_a3',9),record('old2','V_a3_10','V_a3',10)]
            new=[record('new1','P2_fill_01_01','P2_fill_01',1,'V_a3_09'),
                 record('new2','P2_fill_01_02','P2_fill_01',2,'P2V_a_S_talk_03','derived')]
            def write(name,data):
                path=root/name;path.write_text(json.dumps(data));return path
            oldledger=write('old.json',{'candidates':old});oldverdict=write('oldverdict.json',{})
            entries={r['slot_id']:dict(candidate_id=r['id'],disposition='accepted',ledger=str(oldledger),
                ledger_sha256=sha(oldledger),verdict=str(oldverdict),verdict_sha256=sha(oldverdict)) for r in old}
            baseline=write('baseline.json',dict(scale={'adult_height':28,'child_height':21},references={},slots=entries,accepted={}))
            ledger=write('fill.json',dict(scale={'adult_height':28,'child_height':21},candidates=new,selected={r['slot_id']:r['id'] for r in new}))
            from salvage_verdicts import candidate_fingerprint
            verdict=write('verdict.json',dict(candidates_sha256=sha(ledger),candidate_fingerprints={r['slot_id']:candidate_fingerprint(r) for r in new},
                verdicts=[dict(sprite_id=r['slot_id'],candidate_id=r['id'],verdict='ok') for r in new]))
            adjudication=write('adjudication.json',dict(authoritative_sha256=sha(verdict),accepted={r['slot_id']:r['id'] for r in new}))
            source=FillInput(ledger,adjudication,verdict)
            original=baseline.read_bytes()
            result=merge_selection(root,baseline,[source],{})
            self.assertEqual(result['slots']['V_a3_09']['candidate_id'],'old1')
            result=merge_selection(root,baseline,[source],{'old1':'hold'})
            self.assertEqual(result['slots']['V_a3_09']['candidate_id'],'new1')
            self.assertEqual(result['slots']['V_a3_10']['disposition'],'unfilled')
            extra=result['slots']['P2V_a_S_talk_03']
            self.assertEqual(extra['disposition'],'derived')
            self.assertEqual(extra['ledger_sha256'],sha(ledger))
            self.assertEqual(extra['verdict_sha256'],sha(verdict))
            self.assertEqual(extra['semantic']['activity'],'talk')
            self.assertEqual(baseline.read_bytes(),original)
            # A different physical sheet cannot silently join the same activity.
            new[1]['sheet']='P2_fill_02'
            ledger=write('fill.json',dict(scale={'adult_height':28,'child_height':21},candidates=new,selected={r['slot_id']:r['id'] for r in new}))
            verdict=write('verdict.json',dict(candidates_sha256=sha(ledger),candidate_fingerprints={r['slot_id']:candidate_fingerprint(r) for r in new},
                verdicts=[dict(sprite_id=r['slot_id'],candidate_id=r['id'],verdict='ok') for r in new]))
            adjudication=write('adjudication.json',dict(authoritative_sha256=sha(verdict),accepted={r['slot_id']:r['id'] for r in new}))
            mixed=merge_selection(root,baseline,[source],{'old1':'hold'})
            self.assertEqual(set(mixed['accepted'].values()),{'new1'})
            # Current frame guards win over the saved adjudication.
            new[0]['metrics']['mean_delta_e']=100
            ledger=write('fill.json',dict(scale={'adult_height':28,'child_height':21},candidates=new,selected={r['slot_id']:r['id'] for r in new}))
            verdict=write('verdict.json',dict(candidates_sha256=sha(ledger),candidate_fingerprints={r['slot_id']:candidate_fingerprint(r) for r in new},
                verdicts=[dict(sprite_id=r['slot_id'],candidate_id=r['id'],verdict='ok') for r in new]))
            adjudication=write('adjudication.json',dict(authoritative_sha256=sha(verdict),accepted={r['slot_id']:r['id'] for r in new}))
            guarded=merge_selection(root,baseline,[source],{'old1':'hold'})
            self.assertNotIn('new1',guarded['accepted'].values())
            self.assertEqual(set(guarded['accepted'].values()),{'new2'})
            adjudication.write_text(json.dumps({'authoritative_sha256':'wrong','accepted':{}}))
            with self.assertRaises(ValueError):merge_selection(root,baseline,[source],{})

if __name__=='__main__':unittest.main()
