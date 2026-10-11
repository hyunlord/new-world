"""Boundary tests for the pass-specific measurement contract."""
import json
from pathlib import Path
from blind_p6 import check_blind
from capture_contract_p6 import LONG_STEMS
from measure_p6 import long_capture_set


def test_long_plan_requires_every_frame_and_rejects_duplicates():
    assert len(LONG_STEMS) == 21
    assert long_capture_set(sorted(LONG_STEMS)).status == 'PASS'
    assert long_capture_set(sorted(LONG_STEMS)[:-1]).status == 'FAIL'
    assert long_capture_set(sorted(LONG_STEMS) + [next(iter(LONG_STEMS))]).status == 'FAIL'


def test_blind_requires_each_observer_and_carry_threshold(tmp_path: Path):
    path = tmp_path / 'result.json'
    row = {'count': 24, 'accuracy_4x': .875, 'accuracy_2x': .75,
           'observer_id': 'observer_one', 'observer_labels_sha256': 'same-answers', 'absent_activities': [],
           'confusion': {'2x': {'carry': {'carry': 2, 'eat': 1}, 'eat': {'eat': 3}}}}
    second = {**row, 'observer_id': 'observer_two'}
    path.write_text(json.dumps({'observers': [row, second]}))
    assert check_blind(path).status == 'PASS'
    second['accuracy_2x'] = .69
    path.write_text(json.dumps({'observers': [row, second]}))
    assert check_blind(path).status == 'FAIL'


def test_blind_rejects_same_observer_twice(tmp_path: Path):
    path = tmp_path / 'result.json'
    path.write_text(json.dumps({'observers': [{'observer_id': 'one', 'observer_labels_sha256': 'same'}] * 2}))
    assert check_blind(path).status == 'FAIL'


def test_observed_schema_never_passes_from_renderer_metadata_alone(tmp_path: Path, monkeypatch):
    from PIL import Image
    from ingame_qa_observed_p6 import observed_checks
    from ingame_qa_types import load_capture
    monkeypatch.chdir(tmp_path)
    screenshot = tmp_path / 'l_d28_0200_F2Z.png'
    Image.new('RGB', (32, 32), '#D8E8F0').save(screenshot)
    data = {'entities': [], 'labels': [], 'zoom': [6, 6], 'tick': 39000,
            'scenario': 'NATURAL SCENARIO', 'intervention_casts': [],
            'agent_snapshot': {'agent_ids': [1], 'colds': [1], 'xs': [2], 'ys': [3],
                               'states': [3], 'target_kinds': [3], 'hungers': [0],
                               'thirsts': [0], 'sleeps': [0]},
            'time_snapshot': {'day_index': 27, 'season': 3, 'winter_severity': 1,
                              'time_of_day': 2.0},
            'settlement_snapshot': {'settlement_ids': [1], 'fire_lit': [0]},
            'intervention_state': {}, 'chill_tint_strength': .12}
    screenshot.with_suffix('.entities.json').write_text(json.dumps(data))
    checks = observed_checks([load_capture(screenshot)], tmp_path / 'god', tmp_path / 'gate')
    by_code = {check.code: check for check in checks}
    assert by_code['A21'].status == 'FAIL'
    assert by_code['A23'].status == 'FAIL'
    assert by_code['A25'].status == 'FAIL'


def test_chill_eligibility_uses_clear_interval_not_renderer_clock():
    from observation_rules_p6 import chill_window
    assert chill_window(145, 95, 100, 90) == (True, True)
    assert chill_window(188, 95, 100, 90) == (False, True)
    assert chill_window(191, 95, 100, 90) == (False, False)
    assert chill_window(145, -1, -1, 90) == (False, False)
    assert chill_window(145, 140, -1, 90) == (False, False)


def test_panel_does_not_substitute_for_effect_or_windfall_lifecycle():
    from observation_rules_p6 import intervention_visible
    assert not intervention_visible('god_rain', 'rain', 1, ['panel'])
    assert intervention_visible('god_d25_panel_log', 'rain', 1, ['panel'])
    assert intervention_visible('god_windfall_0200ms', 'windfall', .2, ['windfall'])
    assert not intervention_visible('god_windfall_0200ms', 'windfall', .2, ['panel'])
    assert intervention_visible('god_windfall_1000ms', 'windfall', 1, ['windfall_ended'])
    assert not intervention_visible('god_windfall_1000ms', 'windfall', 1, ['windfall'])
    assert not intervention_visible('god_windfall_1000ms', 'windfall', 1, [])


def test_hud_contact_board_preserves_source_pixels_and_hash(tmp_path: Path, monkeypatch):
    import hashlib
    from PIL import Image
    from review_boards_p6 import main
    monkeypatch.chdir(tmp_path)
    monkeypatch.setenv('CAPTURE_EVIDENCE', str(tmp_path / 'external'))
    raw = tmp_path / 'raw'
    raw.mkdir()
    picture = raw / 'a_d2_1200_F2.png'
    Image.new('RGB', (320, 180), '#123456').save(picture)
    picture.with_suffix('.entities.json').write_text(json.dumps({'entities': [], 'labels': [{'name': 'HudTopbar/clock', 'rect': [0, 0, 100, 20]}]}))
    (raw / 'manifest.json').write_text(json.dumps({'captures': [{'file': picture.name}]}))
    main([raw])
    index = json.loads(Path('docs/art/production6/review/hud_batches.json').read_text())
    assert index[0]['frames'][0]['sha256'] == hashlib.sha256(picture.read_bytes()).hexdigest()
    with Image.open(index[0]['board']) as board:
        assert board.width == 320
        assert board.getpixel((100, 40)) == (18, 52, 86)
    assert (tmp_path / 'external/review/identity/a_d2_1200_F2.jpg').is_file()


def test_four_carry_crops_cannot_pass_two_of_three_rule(tmp_path: Path):
    path = tmp_path / 'result.json'
    row = {'observer_id': 'one', 'observer_labels_sha256': 'frozen', 'count': 24,
           'accuracy_4x': .9, 'accuracy_2x': .8, 'absent_activities': [],
           'confusion': {'2x': {'carry': {'carry': 2, 'eat': 2}, 'eat': {'eat': 3}}}}
    path.write_text(json.dumps({'observers': [row, {**row, 'observer_id': 'two'}]}))
    assert check_blind(path).status == 'FAIL'


def test_cycle_contacts_preserve_each_shelter_and_actual_sample_times(tmp_path: Path):
    import hashlib
    from PIL import Image
    from review_boards_p6 import cycle_contacts
    original = tmp_path / 'l_d28_0200_F2Z.png'
    Image.new('RGB', (100, 100), 'black').save(original)
    original.with_suffix('.entities.json').write_text(json.dumps({'entities': [
        {'kind': 'building', 'building_kind': 1, 'visible': True, 'id': 17, 'rect': [20, 20, 10, 10]},
        {'kind': 'building', 'building_kind': 1, 'visible': False, 'id': 18, 'rect': [20, 20, 10, 10]}]}))
    samples = []
    for index in range(51):
        path = tmp_path / f'cycle_{index}.png'
        Image.new('RGB', (100, 100), (index, 20, 30)).save(path)
        samples.append({'file': path.name, 'real_time_usec': 1000000 + index * 100000})
    cycle = tmp_path / 'l_d28_0200_F2Z.cold_cycle.json'
    cycle.write_text(json.dumps({'samples': samples}))
    result = cycle_contacts(cycle, tmp_path / 'external')
    assert result['shelter_ids'] == [17]
    assert len(result['samples']) == 13
    assert [row['relative_seconds'] for row in result['samples']] == [index * .4 for index in range(13)] or all(abs(row['relative_seconds'] - index * .4) < 1e-9 for index, row in enumerate(result['samples']))
    assert result['sha256'] == hashlib.sha256(original.read_bytes()).hexdigest()
    with Image.open(result['contact']) as contact:
        assert contact.getpixel((110, 74)) == (0, 20, 30)
        assert contact.getpixel((110 + 12 * 42, 74)) == (48, 20, 30)


def test_adjacent_motion_requires_same_entities_and_short_interval(tmp_path: Path):
    from motion_evidence_p6 import adjacent_motion
    before = tmp_path / 'a_strip_00.entities.json'
    after = tmp_path / 'a_strip_01.entities.json'
    snapshot = {'agent_ids': [1], 'ids': [4294967297], 'xs': [2], 'ys': [3]}
    before.write_text(json.dumps({'tick': 10, 'agent_snapshot': snapshot}))
    after.write_text(json.dumps({'tick': 13, 'agent_snapshot': {**snapshot, 'xs': [4]}}))
    original_bytes = after.read_bytes()
    evidence = adjacent_motion(after)
    assert evidence['before']['xs'] == [2]
    assert evidence['end_tick'] - evidence['start_tick'] == 3
    assert evidence['provenance'] == 'observed_adjacent_same_animation_snapshots'
    assert after.read_bytes() == original_bytes
    after.write_text(json.dumps({'tick': 19, 'agent_snapshot': snapshot}))
    assert adjacent_motion(after) == {}
    after.write_text(json.dumps({'tick': 13, 'agent_snapshot': {**snapshot, 'ids': [8589934593]}}))
    assert adjacent_motion(after) == {}


def test_terrain_tint_does_not_hide_ground_but_panel_does(tmp_path: Path):
    from PIL import Image
    from ingame_qa_types import load_capture
    from ingame_qa_age_p6 import _terrain
    paths = [tmp_path / 'summer.png', tmp_path / 'winter.png']
    images = [(100, 100, 100), (100, 100, 104)]
    data = {'entities': [{'kind': 'night_chill_tint', 'visible': True, 'rect': [0, 0, 1920, 1080]}],
            'labels': [], 'zoom': [4, 4], 'viewport_size': [1920, 1080],
            'framebuffer_scale': 1, 'camera_position': [0, 0], 'agent_snapshot': {}}
    for path, color in zip(paths, images):
        Image.new('RGB', (1920, 1080), color).save(path)
        path.with_suffix('.entities.json').write_text(json.dumps(data))
    captures = [load_capture(path) for path in paths]
    result = _terrain(*captures, data, data)
    assert result['pass'] and result['mask_pixels'] == 620 * 1120
    data['entities'][0]['kind'] = 'panel'
    for path in paths:
        path.with_suffix('.entities.json').write_text(json.dumps(data))
    result = _terrain(*(load_capture(path) for path in paths), data, data)
    assert not result['pass'] and result['mask_pixels'] == 0
