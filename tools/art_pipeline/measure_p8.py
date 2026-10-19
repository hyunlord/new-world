# /// script
# requires-python = ">=3.11"
# dependencies = ["Pillow", "numpy"]
# ///
# Run: python3 tools/art_pipeline/measure_p8.py QA_CONFIG_JSON
"""Merge unchanged inherited criteria with the newly measured P8 requirements."""
from __future__ import annotations

import gzip
import hashlib
import json
import sys
from dataclasses import asdict
from pathlib import Path
from typing import TypedDict

from ingame_qa_cold_p7 import cold
from ingame_qa_new_p8 import paired_worlds, performance, way
from ingame_qa_news_p8 import news
from ingame_qa_p7 import run
from ingame_qa_pixels import Check, result
from ingame_qa_pixels_p7 import crowd, edge_contrast
from ingame_qa_types import load_capture


class Config(TypedDict):
    raw_art: str
    raw_long: str
    raw_society: str
    raw_god: str
    morning_stems: list[str]
    warm_control_stem: str | None
    gate: str
    expected_long_stems: list[str]
    winter_noon_stem: str
    society_seed: int
    evidence: str
    departure_feed_stem: str


def read_evidence(path: Path):
    raw = path.read_bytes()
    return json.loads(gzip.decompress(raw) if path.suffix == '.gz' else raw)


def brief_detail(detail: str) -> str:
    try:
        value = json.loads(detail)
    except json.JSONDecodeError:
        return detail[:600]
    if isinstance(value, dict):
        fields = [f'{key}={item}' for key, item in value.items()
                  if isinstance(item, (str, int, float, bool)) or item is None]
        fields.extend(f'{key}: {len(item)} entries' for key, item in value.items()
                      if isinstance(item, (list, dict)))
        return '; '.join(fields)[:600]
    if isinstance(value, list):
        return f'{len(value)} detailed records; see exact evidence'
    return str(value)[:600]


def write_measurements(checks: list[Check], output: Path) -> None:
    encoded = (json.dumps([asdict(check) for check in checks], ensure_ascii=False, indent=2) + '\n').encode('utf-8')
    (output / 'measurements.json.gz').write_bytes(gzip.compress(encoded, mtime=0))
    lines = ['# Art Pass 8 measurements', '', 'N/A is not PASS. Natural A1–A25; society A26–A31 and separate A18. Intervention evidence remains explicitly separate.', '',
             'Full exact details: [measurements.json.gz](measurements.json.gz). Input audit: [qa_evidence.json.gz](qa_evidence.json.gz), lossless gzip JSON. [Reading compressed evidence](EVIDENCE_FORMAT.md).', '',
             '| Check | Status | Summary |', '|---|---|---|']
    lines.extend(f'| {check.code} | {check.status} | {brief_detail(check.detail).replace(chr(10), " ").replace("|", "/")} |' for check in checks)
    (output / 'measurements.md').write_text('\n'.join(lines) + '\n')


def compress_inherited_reports(output: Path) -> None:
    for name in ('ingame_qa.json', 'ingame_qa.md'):
        path = output / name
        original = path.read_bytes()
        compressed = path.with_suffix(path.suffix + '.gz')
        compressed.write_bytes(gzip.compress(original, mtime=0))
        if gzip.decompress(compressed.read_bytes()) != original:
            raise ValueError('Compressed inherited report differs: ' + name)
        path.unlink()


def main(config_path: Path) -> None:
    """Write exclusively under production8 and keep missing observations explicit."""
    config: Config = json.loads(config_path.read_text())
    output = Path('docs/art/production8')
    raw, long_raw, society_raw = (Path(config[key]) for key in ('raw_art', 'raw_long', 'raw_society'))
    evidence = read_evidence(Path(config['evidence']))
    blind = output / 'blind_results.json'
    checks = run(raw, output / 'ingame_qa.md', Path('.'), blind if blind.exists() else None, long_raw)
    compress_inherited_reports(output)
    from blind_identity_p8 import final_identity, guard_activity
    checks = guard_activity(checks, final_identity(raw, output / 'blind_identity_final.json'))
    long_manifest = json.loads((long_raw / 'manifest.json').read_text())
    stems = [Path(row['file']).stem for row in long_manifest['captures']]
    expected = set(config['expected_long_stems'])
    long_check = result('LONG_CAPTURE_SET', set(stems) == expected and len(stems) == len(expected),
                        json.dumps({'expected': sorted(expected), 'actual': stems}))
    winter = long_raw / (config['winter_noon_stem'] + '.png')
    edge_rows = [edge_contrast(load_capture(path), Path('.')) for path in (raw / 'a_d2_1200_F2.png', winter)]
    edge = result('A5', all(row.status == 'PASS' for row in edge_rows), json.dumps([asdict(row) for row in edge_rows]))
    replacements = {'LONG_CAPTURE_SET': long_check, 'A5': edge}
    checks = [replacements.get(check.code, check) for check in checks]
    live = []
    contamination = []
    for directory in (raw, long_raw):
        manifest = json.loads((directory / 'manifest.json').read_text())
        for row in manifest['captures']:
            path = directory / row['file']
            data = json.loads(path.with_suffix('.entities.json').read_text())
            if path.stem.startswith(('a_calib', 'a_lineup')):
                continue
            if data.get('scenario') != 'NATURAL SCENARIO' or data.get('seed') != 20261010 or data.get('intervention_casts') or data.get('intervention_state', {}).get('log_kinds'):
                contamination.append(path.stem)
            live.append(load_capture(path))
    society = []
    society_errors = []
    for row in json.loads((society_raw / 'manifest.json').read_text())['captures']:
        path = society_raw / row['file']
        data = json.loads(path.with_suffix('.entities.json').read_text())
        if not path.stem.startswith('soc_') or data.get('scenario') != f"SOCIETY SCENARIO seed {config['society_seed']}" or data.get('seed') != config['society_seed'] or data.get('intervention_casts'):
            society_errors.append(path.stem)
        society.append(load_capture(path))
    checks.append(result('SCENARIO_PARTITION', not contamination and not society_errors,
                         json.dumps({'natural_errors': contamination, 'society_errors': society_errors})))
    checks.append(cold(live, Path(config['gate'])))
    crowds = [crowd(frame, Path('.')) for frame in society]
    checks.append(result('A18_SOCIETY', bool(crowds) and all(row.status == 'PASS' for row in crowds), json.dumps([asdict(row) for row in crowds])))
    from harvest_integrity_p8 import audit_all
    from ingame_qa_observed_p8 import observed_checks
    from ingame_qa_society_p8 import society_checks
    god_raw = Path(config['raw_god'])
    observed = observed_checks(live, god_raw, Path(config['gate']),
                              morning_stems=set(config['morning_stems']), warm_control_stem=config['warm_control_stem'])
    checks.extend(observed)
    all_rows = []
    for directory in (raw, long_raw, society_raw, god_raw):
        for row in json.loads((directory / 'manifest.json').read_text())['captures']:
            path = directory / row['file']
            all_rows.append((path, json.loads(path.with_suffix('.entities.json').read_text())))
    checks.extend(society_checks(all_rows, society_raw, output, society_seed=config['society_seed']))
    try:
        integrity = audit_all()
        checks.append(result('A20', integrity['passed'], json.dumps(integrity)))
    except (OSError, KeyError) as error:
        checks.append(Check('A20', 'FAIL', 'Current installed-pixel provenance audit failed: ' + str(error)))
    from feed_adapter_p8 import feed_frames
    from way_adapter_p8 import way_frames
    society_paths = [frame.path for frame in society]
    normalized_news = feed_frames(society_paths, config['departure_feed_stem'])
    normalized_way = way_frames(society_paths)
    checks.extend([performance(evidence['performance']) if evidence.get('performance') else Check('A30', 'N/A', 'Missing paired CPU evidence'),
                   news(normalized_news),
                   way(normalized_way),
                   paired_worlds(evidence.get('paired_worlds', {}))])
    geometry = next(check for check in checks if check.code == 'A32_GEOMETRY')
    blind_way = evidence.get('blind_way', [])
    blind_valid = bool(blind_way) and all(len(row['answers']) == 2 and all(answer == row['expected'] for answer in row['answers'])
        and Path(row['source']).is_file() and hashlib.sha256(Path(row['source']).read_bytes()).hexdigest() == row['sha256'] for row in blind_way)
    expected_blind = {row['frame'] for row in normalized_way if row['walking'] and '_FD' in row['frame']}
    actual_blind = {Path(row['source']).stem for row in blind_way}
    blind_valid = blind_valid and bool(expected_blind) and expected_blind == actual_blind
    checks.append(result('A32', geometry.status == 'PASS' and blind_valid, json.dumps({'geometry': asdict(geometry), 'blind': blind_way})))
    crowd_blind = evidence.get('blind_crowds', [])
    crowd_bound = len(crowd_blind) == 12 and all(len(row['answers']) == 2 and set(row['answers']).issubset({'yes', 'roughly', 'no'})
        and Path(row['source']).is_file() and hashlib.sha256(Path(row['source']).read_bytes()).hexdigest() == row['sha256'] for row in crowd_blind)
    checks.append(Check('A18b', 'OBSERVED' if crowd_bound else 'N/A', json.dumps({'frames': len(crowd_blind),
        'answers': crowd_blind, 'source_binding_valid': crowd_bound, 'threshold': 'Companion observation only; never replaces A18'})))
    for code in (f'A{index}' for index in range(1, 34)):
        if not any(check.code == code for check in checks):
            checks.append(Check(code, 'N/A', 'Current-frame independent measurement not supplied; inherited threshold remains unchanged'))
    write_measurements(checks, output)
    print('\n'.join(f'{check.code}\t{check.status}' for check in checks))


if __name__ == '__main__':
    main(Path(sys.argv[1]))
