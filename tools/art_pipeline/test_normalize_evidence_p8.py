"""Source-mixing and failed-cast regressions for paired QA normalization."""
import json
from pathlib import Path

import pytest
from normalize_evidence_p8 import (
    Document,
    Json,
    array,
    load,
    mapping,
    normalize,
    number,
    rows,
)


def at(value: Json, *keys: str | int) -> Json:
    """Narrow each JSON union before accessing nested fixture evidence."""
    for key in keys:
        if isinstance(key, int):
            value = array(value)[key]
        else:
            value = mapping(value)[key]
    return value


def fixture(tmp_path: Path) -> tuple[Path, Path, Path]:
    def write(name: str, data: Document) -> Path:
        path = tmp_path / name
        path.write_text(json.dumps(data))
        return path
    casts: list[Json] = []
    captures: list[Json] = []
    for label, kind, epoch in [('P', 'spoil_food', 1), ('G', 'gift_food', 2)]:
        for day in [30, 31, 32]:
            casts.append({'world_label': label, 'world_epoch': epoch, 'seed': 20261013,
                'capture_options': {'paired_world': label, 'case_id': 'a33', 'role': 'cast'},
                'tick': (day - 1) * 1440 + 360 - 480, 'kind': kind, 'amount': 60,
                'settlement_id': 7, 'reply': {'ok': True, 'applied': 60},
                'precast_original': {'found': True, 'settlement_id': 7, 'food': 120,
                    'members': 35, 'adult_need_per_day': 2.4, 'scarcity_days': 4,
                    'scarcity_line': 336, 'below_scarcity_line': True,
                    'pressure': 700, 'threshold': 1200,
                    'need_provenance': {'valid': True, 'source_files': [{'path': 'fixture.rs', 'sha256': 'a' * 64}]}}})
        for day, role in [(31, 'morning'), (32, 'morning'), (33, 'morning'), (120, 'final')]:
            tick = (day - 1) * 1440 + (720 if role == 'final' else 360) - 480
            options: Document = {'paired_world': label, 'case_id': 'a33', 'role': role}
            name = f'{label}_{day}.entities.json'
            side: Document = {'world_label': label, 'world_epoch': epoch, 'seed': 20261013,
                'tick': tick, 'capture_options': options,
                'original_settlement_observation': {'found': True, 'settlement_id': 7,
                    'pressure': 900 if label == 'P' else 300, 'threshold': 1200},
                'society_snapshot': {'fission_parents': [42, 7],
                    'fission_depart_ticks': [1, 46000 if label == 'P' else 50000]}}
            write(name, side)
            captures.append({key: side[key] for key in ['world_label', 'world_epoch', 'seed', 'tick', 'capture_options']} | {'entities': name})
    manifest = write('manifest.json', {'run_id': 'capture-run', 'git_sha': 'b' * 40,
        'captures': captures, 'intervention_casts': casts})
    perf = write('perf.json', {'valid': True, 'errors': [], 'seed': 20261013, 'tick': 126994,
        'run_id': 'cpu-run', 'git_sha': 'b' * 40, 'load_before': {'uptime': ['load 1'], 'processes': ['Godot']},
        'load_after': {'uptime': ['load 2'], 'processes': ['Godot']}})
    calendar = write('calendar.json', {'society_seed': 20261013, 'clock': {'offset': 480, 'ticks_per_day': 1440},
        'selection': [{'seed': 20261013, 'first_three_alive_tick': 126994,
            'cast_days': [{'day': day} for day in [30, 31, 32]]}]})
    return manifest, perf, calendar


def test_normalizes_actual_original_departure_and_sources(tmp_path: Path) -> None:
    result = normalize(*fixture(tmp_path), [])
    assert at(result, 'paired_worlds', 'P', 'departure_tick') == 46000
    assert at(result, 'paired_worlds', 'same_seed_and_cast_days') is True
    assert at(result, 'paired_worlds', 'both_reached_day120') is True
    assert at(result, 'performance', 'first_three_alive_tick') == 126994
    assert at(result, 'performance', 'uptime') == 'load 1'
    assert at(result, 'normalization', 'valid') is True


@pytest.mark.parametrize('mutation', ['world', 'day', 'missing_target', 'cpu_tick', 'cast_amount'])
def test_invalid_source_cannot_set_success_contract(tmp_path: Path, mutation: str) -> None:
    manifest, perf, calendar = fixture(tmp_path)
    path = tmp_path / 'P_120.entities.json' if mutation in ['world', 'day'] else (perf if mutation == 'cpu_tick' else manifest)
    data = load(path)
    if mutation == 'world': data['world_epoch'] = 2
    if mutation == 'day': data['tick'] = number(data['tick']) - 1440
    if mutation == 'missing_target': rows(data['intervention_casts'])[0]['reply'] = {'ok': False, 'applied': 0}
    if mutation == 'cpu_tick': data['tick'] = number(data['tick']) + 1
    if mutation == 'cast_amount': rows(data['intervention_casts'])[0]['amount'] = 40
    path.write_text(json.dumps(data))
    result = normalize(manifest, perf, calendar, [])
    assert at(result, 'normalization', 'valid') is False
    assert at(result, 'normalization', 'errors')
    if mutation != 'cpu_tick':
        assert not (at(result, 'paired_worlds', 'same_seed_and_cast_days') and at(result, 'paired_worlds', 'both_reached_day120'))


def test_separate_processes_may_share_epoch(tmp_path: Path) -> None:
    manifest, perf, calendar = fixture(tmp_path)
    data = load(manifest)
    for row in rows(data['intervention_casts']) + rows(data['captures']):
        row['run_id'] = 'run-' + str(row['world_label'])
        row['world_epoch'] = 1
        if 'entities' in row:
            path = tmp_path / str(row['entities'])
            side = load(path)
            side.update(run_id=row['run_id'], world_epoch=1, git_sha='b' * 40)
            path.write_text(json.dumps(side))
    manifest.write_text(json.dumps(data))
    assert at(normalize(manifest, perf, calendar, []), 'normalization', 'valid') is True


def test_mixed_code_sha_rejected(tmp_path: Path) -> None:
    manifest, perf, calendar = fixture(tmp_path)
    path = tmp_path / 'G_120.entities.json'
    data = load(path)
    data['git_sha'] = 'c' * 40
    path.write_text(json.dumps(data))
    assert at(normalize(manifest, perf, calendar, []), 'paired_worlds', 'same_seed_and_cast_days') is False


def test_pacing_sources_retained_read_only(tmp_path: Path) -> None:
    manifest, perf, calendar = fixture(tmp_path)
    path = tmp_path / 'old.json'
    path.write_text(json.dumps({'median_ms': 15.8, 'samples_ms': [15.7, 15.9], 'run_id': 'old'}))
    before = path.read_bytes()
    result = normalize(manifest, perf, calendar, [path])
    assert at(result, 'performance', 'old_pacing_ms') == [15.8]
    assert at(result, 'performance', 'old_pacing_sources', 0, 'raw', 'samples_ms') == [15.7, 15.9]
    assert path.read_bytes() == before


def test_output_preserves_blind_fields_and_rejects_outside_docs(tmp_path: Path) -> None:
    import tempfile

    from normalize_evidence_p8 import EvidenceError, write_evidence
    root = Path(__file__).resolve().parents[2] / 'docs/art/production8'
    with tempfile.TemporaryDirectory(dir=root, prefix='.normalize-test-') as directory:
        path = Path(directory) / 'qa_evidence.json'
        path.write_text(json.dumps({'blind_way': {'observer': 'original'}, 'blind_crowds': [1, 2]}))
        write_evidence(path, {'normalization': {'valid': True}})
        updated = load(path)
        assert updated['blind_way'] == {'observer': 'original'}
        assert updated['blind_crowds'] == [1, 2]
    with pytest.raises(EvidenceError):
        write_evidence(tmp_path / 'outside.json', {})


def test_source_cannot_be_reused_as_output() -> None:
    import tempfile

    from normalize_evidence_p8 import EvidenceError, write_evidence
    root = Path(__file__).resolve().parents[2] / 'docs/art/production8'
    with tempfile.TemporaryDirectory(dir=root, prefix='.normalize-test-') as directory:
        path = Path(directory) / 'raw.json'
        path.write_text('{"raw":true}')
        with pytest.raises(EvidenceError):
            write_evidence(path, {'normalization': {'sources': [{'path': str(path)}]}})
        assert path.read_text() == '{"raw":true}'
