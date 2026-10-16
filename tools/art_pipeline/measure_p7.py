# /// script
# requires-python = ">=3.10"
# dependencies = ["Pillow", "numpy"]
# ///
"""Measure live, society and intervention evidence without mixing their scopes.

Run: python3 tools/art_pipeline/measure_p7.py RAW_ART RAW_LONG GATE RAW_GOD RAW_SOCIETY [RAW_PERFORMANCE]
"""
from __future__ import annotations
from dataclasses import asdict
import json
import hashlib
from pathlib import Path
import sys
from ingame_qa_p7 import run
from ingame_qa_cold_p7 import cold
from ingame_qa_types import load_capture
from ingame_qa_pixels import Check, result
from capture_contract_p7 import LONG_STEMS, INHERITED_STEMS
from ingame_qa_society_p7 import society_checks


def long_capture_set(stems: list[str]) -> Check:
    """Require all 27 measured live-seed frames, with neither omissions nor duplicates."""
    actual = set(stems)
    return result('LONG_CAPTURE_SET',actual==LONG_STEMS and len(stems)==len(LONG_STEMS),
        f'{len(stems)}/{len(LONG_STEMS)} captures; missing={sorted(LONG_STEMS-actual)}; extra={sorted(actual-LONG_STEMS)}')


def frame_rows(directory: Path):
    """Read only captured manifest entries and their exact sidecars."""
    manifest = directory/'manifest.json'
    if not manifest.exists():
        return []
    return [(directory/row['file'],json.loads((directory/row['file']).with_suffix('.entities.json').read_text()))
            for row in json.loads(manifest.read_text())['captures']]


def main(raw: Path, long_raw: Path, gate: Path, god_raw: Path, society_raw: Path, performance_raw: Path | None = None) -> None:
    """Preserve inherited numeric thresholds and report new checks separately."""
    from ingame_qa_observed_p7 import observed_checks
    from ingame_qa_pixels_p7 import crowd
    output = Path('docs/art/production7')
    blind = output/'blind_results.json'
    checks = run(raw,output/'ingame_qa.md',Path('.'),blind if blind.exists() else None,long_raw,
        extra_long_stems=tuple(LONG_STEMS-INHERITED_STEMS))
    identity_path = output/'blind_identity_final.json'
    identity = json.loads(identity_path.read_text()) if identity_path.exists() else {}
    identity_valid = bool(identity.get('passed')) and len(identity.get('source_sha256',{})) == 2 and all(
        (raw/name).exists() and hashlib.sha256((raw/name).read_bytes()).hexdigest()==digest
        for name,digest in identity.get('source_sha256',{}).items())
    checks.append(result('A11_FINAL_PIXEL_IDENTITY',identity_valid,'Final calibration lineup matches original frozen 4x/2x grid pixels; see blind_identity_final.json'))
    if not identity_valid:
        checks = [Check('A11','FAIL','Final activity crops are not verified identical to frozen observer evidence') if check.code=='A11' else check for check in checks]
    live_rows = frame_rows(raw)+frame_rows(long_raw)
    society_rows,god_rows = frame_rows(society_raw),frame_rows(god_raw)
    live = [(path,data) for path,data in live_rows if not path.stem.startswith(('a_calib','a_lineup'))]
    contaminated = [path.stem for path,data in live if data.get('scenario')!='NATURAL SCENARIO' or data.get('seed')!=20261010
        or data.get('intervention_casts') or data.get('intervention_state',{}).get('log_kinds')]
    bad_society = [path.stem for path,data in society_rows if not path.stem.startswith('soc_') or data.get('scenario')!='SOCIETY SCENARIO seed 20261013'
        or data.get('seed')!=20261013 or data.get('intervention_casts') or data.get('intervention_state',{}).get('log_kinds')]
    bad_god = [path.stem for path,data in god_rows if not path.stem.startswith('god_') or data.get('scenario')!='INTERVENTION SCENARIO']
    checks.append(result('SCENARIO_PARTITION',not contaminated and not bad_society and not bad_god,
        json.dumps({'live':len(live),'society':len(society_rows),'intervention':len(god_rows),'contaminated_live':contaminated,'bad_society':bad_society,'bad_intervention':bad_god})))
    checks = [long_capture_set([path.stem for path,_ in frame_rows(long_raw)]) if check.code=='LONG_CAPTURE_SET' else check for check in checks]
    frames = [load_capture(path) for path,_ in live]
    checks.append(cold(frames,gate))
    checks.extend(observed_checks(frames,god_raw,gate))
    society_crowd = {path.stem:asdict(crowd(load_capture(path),Path('.'))) for path,_ in society_rows}
    checks.append(result('A18_SOCIETY',bool(society_crowd) and all(row['status']=='PASS' for row in society_crowd.values()),json.dumps({'frames':society_crowd})))
    checks.extend(society_checks(live+society_rows+god_rows,society_raw,output,performance_raw))
    integrity = output/'harvest/integrity_final.json'
    if integrity.exists():
        proof = json.loads(integrity.read_text())
        checks.append(result('A20',bool(proof.get('passed')) and bool(proof.get('head')) and proof.get('atlas_sha256') == hashlib.sha256(Path('assets/art/atlas/world_atlas.json').read_bytes()).hexdigest(),json.dumps(proof)))
    else:
        checks.append(Check('A20','N/A','Missing current-head installed-frame integrity audit'))
    for code in (f'A{i}' for i in range(1,31)):
        if not any(check.code==code for check in checks):
            checks.append(Check(code,'N/A','No measurement implementation/evidence for this inherited code'))
    records = [asdict(check) for check in checks]
    from measurement_summary_p7 import write_summary
    write_summary(records,live+society_rows+god_rows,output,gate)
    (output/'measurements.json').write_text(json.dumps(records,ensure_ascii=False,indent=2)+'\n')
    report = ['# Art Pass 7 measurements', '',
        'A1–A25: live seed; A24: live intervention. A26: all scenarios. A27–A30: society. A18_SOCIETY stays separate.',
        '', 'Canonical full numeric evidence and every failure: [measurements.json](measurements.json).', '',
        '| Check | Status | Numeric result / evidence |', '|---|---|---|']
    for row in records:
        detail = row['detail']
        try:
            structured = json.loads(detail)
            if isinstance(structured, dict):
                compact = {key:value for key,value in structured.items()
                    if isinstance(value,(str,int,float,bool)) or key in ('median_ms','ratio','samples','errors')}
                detail = json.dumps(compact,ensure_ascii=False)
        except (json.JSONDecodeError, TypeError):
            pass
        detail = detail.replace('|','/').replace('\n',' ')
        if len(detail)>480:
            detail = detail[:477]+'...'
        report.append(f"| {row['code']} | {row['status']} | {detail} |")
    (output/'measurements.md').write_text('\n'.join(report)+'\n')
    print('\n'.join(f'{row["code"]}\t{row["status"]}' for row in records))


if __name__ == '__main__':
    main(*(Path(argument) for argument in sys.argv[1:]))
