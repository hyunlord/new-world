# /// script
# requires-python = ">=3.10"
# dependencies = ["Pillow", "numpy"]
# ///
# How to run: python3 tools/art_pipeline/finalize_masks_p5.py
"""Join immutable review rounds, retaining the exact approved pixel version."""
from collections import Counter
import hashlib
from pathlib import Path
from mask_inputs_p5 import OUTPUT, load
from remask_candidates_p5 import write
from salvage_verdicts import candidate_fingerprint


def main() -> None:
    """Freeze final candidate/review bindings and report each independent scope."""
    initial=load(OUTPUT/'candidates.json')['candidates']
    revised=load(OUTPUT/'round2/candidates.json')['candidates']
    one={r['candidate_id']:r for r in load(OUTPUT/'round1/final_reviews.json')['verdicts']}
    parts=sorted((OUTPUT/'round2').glob('reviews_*.json'))
    two={r['candidate_id']:r for path in parts for r in load(path)['verdicts']}
    expected={r['id']+':r2' for r in initial if one[r['id']]['reason']=='hair_mask_wrong'}
    assert set(two)==expected
    write(OUTPUT/'round2/final_reviews.json',{'verdicts':list(two.values()),
          'review_sources_sha256':{path.name:hashlib.sha256(path.read_bytes()).hexdigest() for path in parts}})
    revised_lookup={r['round1_candidate_id']:r for r in revised}
    records=[]; verdicts=[]
    for original in initial:
        verdict=one[original['id']]
        record=original
        if verdict['reason']=='hair_mask_wrong':
            record=revised_lookup[original['id']]
            verdict=two[record['id']]
        source_index=OUTPUT/('round2/review_index.json' if record is not original else 'review_index.json')
        assert load(source_index)['candidate_fingerprints'][record['id']]==candidate_fingerprint(record)
        records.append(record);verdicts.append(verdict)
    write(OUTPUT/'final_candidates.json',{'schema_version':1,'candidates':records})
    write(OUTPUT/'final_reviews.json',{'verdicts':verdicts})
    write(OUTPUT/'final_review_index.json',{'candidate_fingerprints':{r['id']:candidate_fingerprint(r) for r in records},
          'source_indices':['docs/art/production5/masks/review_index.json','docs/art/production5/masks/round2/review_index.json'],
          'source_verdicts':['docs/art/production5/masks/round1/final_reviews.json','docs/art/production5/masks/round2/final_reviews.json']})
    byid={r['id']:r for r in records}
    counts=Counter((byid[r['candidate_id']]['mask_scope'],r['verdict']) for r in verdicts)
    write(OUTPUT/'final_summary.json',{'counts':{f'{scope}/{verdict}':count for (scope,verdict),count in sorted(counts.items())},
          'initial_approved_retained':sum(v['verdict']=='ok' for v in one.values()),
          'round2_approved':sum(v['verdict']=='ok' for v in two.values()),'review_rounds':2,
          'raw_context_limitation':'Round1 recovery cards lacked raw at top level;76 original raw/unfitted contexts supplied later without changing earlier verdicts.'})
    print(dict(counts))


if __name__=='__main__':
    main()
