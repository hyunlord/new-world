# /// script
# requires-python = ">=3.11"
# dependencies = []
# ///
# Run: python3 -m pytest tools/art_pipeline/test_measurement_p8.py
"""Regression fixtures for the newly defined pass-eight boundaries."""
from ingame_qa_new_p8 import (
    PairedEvidence,
    PerformanceEvidence,
    WayFrame,
    paired_worlds,
    performance,
    way,
)
from ingame_qa_news_p8 import NewsFrame


def test_society_death_gate_cannot_borrow_other_scenario_exposure(tmp_path, monkeypatch) -> None:
    import json

    import ingame_qa_society_p8 as society
    from ingame_qa_pixels import Check
    # Given: valid death accounting, but only another scenario has an exposure marker.
    monkeypatch.setattr(society, 'check_death_frame', lambda data: [])
    monkeypatch.setattr(society, 'intervention', lambda *args, **kwargs: Check('A29', 'N/A', 'fixture'))
    seed = 20261013
    selected = {'scenario': f'SOCIETY SCENARIO seed {seed}', 'seed': seed, 'death_drawn': []}
    marker = {'reason': 4, 'in_view': True}
    for scenario, other_seed in [('NATURAL SCENARIO', 20261010),
                                 (f'SOCIETY SCENARIO seed {seed}', seed + 1)]:
        other = {'scenario': scenario, 'seed': other_seed, 'death_drawn': [marker]}
        # When: the society-scoped A26 gate receives both frames.
        frames = [(tmp_path / 'soc.png', selected), (tmp_path / 'other.png', other)]
        check = society.society_checks(frames, tmp_path, tmp_path, society_seed=seed)[0]
        # Then: neither a live frame nor a mislabeled foreign seed can supply its witness.
        assert check.status == 'FAIL'
        assert json.loads(check.detail)['frames'] == 1
        assert json.loads(check.detail)['exposure_markers_in_view'] == 0
    selected['death_drawn'] = [marker]
    positive = society.society_checks([(tmp_path / 'soc.png', selected)], tmp_path, tmp_path, society_seed=seed)[0]
    assert positive.status == 'PASS'


def test_paired_worlds_reject_equal_departure_ticks() -> None:
    # Given: pressure differs but departure is tied.
    evidence: PairedEvidence = {"P": {"pressure": 90, "threshold": 100, "departure_tick": 200},
                "G": {"pressure": 20, "threshold": 100, "departure_tick": 200},
                "same_seed_and_cast_days": True, "both_reached_day120": True}
    # When: the paired-world requirement is measured.
    check = paired_worlds(evidence)
    # Then: a pressure change alone cannot pass.
    assert check.status == "FAIL"


def test_performance_rejects_wallclock_only_samples() -> None:
    # Given: old pacing samples lack per-frame CPU measurements.
    evidence: PerformanceEvidence = {"old_pacing_ms": [1.0] * 600}
    # When: the redesigned A30 is evaluated.
    check = performance(evidence)
    # Then: pacing cannot masquerade as CPU evidence.
    assert check.status == "FAIL"


def test_way_rejects_opposite_arrow() -> None:
    # Given: a target lies east and the arrow lies west.
    evidence: list[WayFrame] = [{"frame": "soc_walk", "walking": True, "site_on_screen": False,
                 "stake_visible": False, "arrows": [[0, 50]], "screen_center": [50, 50],
                 "target_screen": [200, 50], "distance_tiles": 20,
                 "expected_distance_tiles": 20}]
    # When: geometry is measured.
    check = way(evidence)
    # Then: a marker pointing backwards fails.
    assert check.status == "FAIL"


def test_paired_worlds_accepts_only_spoil_departure() -> None:
    # Given: the same seeded worlds finish day120, only spoil departs.
    evidence: PairedEvidence = {"P": {"pressure": 90, "threshold": 100, "departure_tick": 200},
                "G": {"pressure": 20, "threshold": 100, "departure_tick": None},
                "same_seed_and_cast_days": True, "both_reached_day120": True}
    # When: the asymmetric departure is evaluated.
    check = paired_worlds(evidence)
    # Then: the explicitly permitted one-world departure passes.
    assert check.status == "PASS"


def test_performance_accepts_additive_one_ms_threshold() -> None:
    # Given: on/off CPU totals 1.8/1.0 ms exceed 15% but fit the 1ms limit.
    evidence: PerformanceEvidence = {"blocks": [{"layers_enabled": index % 2 == 0, "tick": 123,
        "camera": [2.0, 3.0], "samples": [{"process_ms": 1.6 if index % 2 == 0 else .8,
        "render_cpu_ms": .2, "setup_cpu_ms": .1, "gpu_ms": 0.0}] * 50} for index in range(12)],
        "alive_count": 3, "first_three_alive_tick": 123, "framing": "FV", "simulation_frozen": True,
        "vsync_disabled": True, "vsync_restored": True, "run_id": "fixture",
        "uptime": "load averages: 1.0 1.0 1.0", "processes": "", "process_priorities": [-1000, 1000]}
    # When: the user-specified OR threshold is evaluated.
    check = performance(evidence)
    # Then: the additive branch passes even though the ratio branch fails.
    assert check.status == "PASS"


def test_way_rejects_simultaneous_stake_and_arrow() -> None:
    # Given: an on-screen site incorrectly has both cues.
    evidence: list[WayFrame] = [{"frame": "soc_walk", "walking": True, "site_on_screen": True,
                 "stake_visible": True, "arrows": [[90, 50]], "screen_center": [50, 50],
                 "target_screen": [80, 50], "distance_tiles": 20,
                 "expected_distance_tiles": 20}]
    # When: mutually exclusive marker geometry is checked.
    check = way(evidence)
    # Then: duplicate cues fail.
    assert check.status == "FAIL"


def test_news_rejects_daily_summary_with_gift_included() -> None:
    from ingame_qa_news_p8 import news
    # Given: an ordinary ten-food deposit and a sixty-food gift.
    frame: NewsFrame = {"frame": "soc_feed", "previous_tick": 0, "tick": 300,
        "expected_event_ids": ["depart:1"],
        "events": [{"event_id": "depart:1", "tick": 120, "key": "HUD_FEED_FISSION_DEPARTED", "event_class": "protected", "visible": True}],
        "visible_event_ids": ["depart:1"], "overflow_event_ids": [], "more_count": 0,
        "summaries": [{"village_id": 1, "start_tick": 0, "end_tick": 300, "stored": 70, "eaten": 0, "net": 70}],
        "movements": [{"tick": 10, "village_id": 1, "delta": 10, "excluded": False},
                      {"tick": 20, "village_id": 1, "delta": 60, "excluded": True}],
        "departure_feed_frame": True, "departure_event_id": "depart:1"}
    # When: a summary incorrectly counts the gift.
    check = news([frame])
    # Then: excluded god food cannot improve the daily total.
    assert check.status == "FAIL"


def test_board_retains_source_pixels_and_hash(tmp_path) -> None:
    import hashlib

    import artboards_p8
    from PIL import Image
    # Given: an exact four-pixel test fixture, never a gameplay capture.
    source = tmp_path / 'test-only.png'
    Image.new('RGB', (2, 2), (12, 34, 56)).save(source)
    original_output = artboards_p8.OUTPUT
    artboards_p8.OUTPUT = tmp_path
    # When: the board is composed at native scale.
    try:
        record = artboards_p8.board('test-board', {'caption': 'fixture', 'paths': [str(source)]})
    finally:
        artboards_p8.OUTPUT = original_output
    # Then: source pixels and SHA match exactly.
    with Image.open(record['file']) as board_image:
        assert board_image.getpixel((0, 60)) == (12, 34, 56)
    assert record['sources'][0]['sha256'] == hashlib.sha256(source.read_bytes()).hexdigest()


def test_event_oracle_accepts_only_proven_initial_formation(tmp_path) -> None:
    import json

    from event_oracle_p8 import event_errors
    # Given: a root village founded at tick0 appears in the initial tick1 HUD.
    path = tmp_path / 'initial.png'
    payload = {'tick': 10, 'event_truth_trace': {'events': [], 'coverage_complete': True,
        'window_start_tick': 0, 'window_end_tick': 10, 'max_sample_interval_ticks': 1},
        'society_snapshot': {'settlement_ids': [1], 'parent_ids': [-1], 'founded_ticks': [0]},
        'feed_audit': {'log': [{'event_id': 'founded:1', 'key': 'HUD_FEED_SETTLEMENT_FORMED',
                             'tick': 0, 'observed_tick': 1, 'class': 'protected'}]}}
    path.with_suffix('.entities.json').write_text(json.dumps(payload))
    # When: baseline suppression is reconciled with independent raw founding data.
    errors = event_errors(path)
    # Then: the proven initial fact is allowed.
    assert errors == []


def test_event_oracle_rejects_later_unobserved_formation(tmp_path) -> None:
    import json

    from event_oracle_p8 import event_errors
    # Given: a new formation after baseline has no independent event record.
    path = tmp_path / 'later.png'
    payload = {'tick': 10, 'event_truth_trace': {'events': [], 'coverage_complete': True,
        'window_start_tick': 0, 'window_end_tick': 10, 'max_sample_interval_ticks': 1},
        'society_snapshot': {'settlement_ids': [1], 'parent_ids': [-1], 'founded_ticks': [8]},
        'feed_audit': {'log': [{'event_id': 'founded:1', 'key': 'HUD_FEED_SETTLEMENT_FORMED',
                             'tick': 8, 'observed_tick': 8, 'class': 'protected'}]}}
    path.with_suffix('.entities.json').write_text(json.dumps(payload))
    # When: source completeness is checked.
    errors = event_errors(path)
    # Then: postbaseline omissions remain failures.
    assert errors == ['founded:1: feed event absent from independent census']


def test_way_adapter_uses_logical_positions_and_raw_distance(tmp_path) -> None:
    import json

    from way_adapter_p8 import way_frames
    # Given: logical geometry points east-northeast, framebuffer decoys point away.
    path = tmp_path / 'soc_FD.png'
    payload = {'society_snapshot': {'founder_fission_ids': [1], 'founder_leader': [1],
        'founder_agent_ids': [5], 'founder_site_xs': [13], 'founder_site_ys': [14]},
        'agent_snapshot': {'agent_ids': [5], 'xs': [10], 'ys': [10]},
        'framebuffer_scale': 2,
        'society_presentation': {'founding_groups': [{'fission_id': 1,
            'target_screen_position': [200, 100], 'edge_screen_position': [95, 65],
            'edge_framebuffer_position': [0, 0], 'site_screen': [-400, -400],
            'screen_center': [50, 50], 'viewport_size': [100, 100],
            'site_on_screen': False, 'stake_visible': False, 'edge_visible': True,
            'displayed_distance_tiles': 5, 'distance_tiles': 999}]}}
    path.with_suffix('.entities.json').write_text(json.dumps(payload))
    # When: the adapter reads independent raw tile positions and logical geometry.
    rows = way_frames([path])
    # Then: framebuffer decoys and claimed distance cannot contaminate measurement.
    assert rows[0]['expected_distance_tiles'] == 5
    assert way(rows).status == 'PASS'


def test_way_adapter_rejects_missing_group_marker(tmp_path) -> None:
    import json

    from way_adapter_p8 import way_frames
    # Given: raw founders are walking but no presentation group was drawn.
    path = tmp_path / 'soc_FD.png'
    payload = {'society_snapshot': {'founder_fission_ids': [1]},
        'agent_snapshot': {'agent_ids': [5], 'xs': [10], 'ys': [10]},
        'society_presentation': {'founding_groups': []}}
    path.with_suffix('.entities.json').write_text(json.dumps(payload))
    # When: every raw walking group is checked.
    rows = way_frames([path])
    # Then: absent geometry cannot vacuously pass.
    assert way(rows).status == 'FAIL'


def test_inherited_founding_rejects_offscreen_site_despite_arrow() -> None:
    from ingame_qa_founding_p8 import check_group
    # Given: the actual site is offscreen and one observed edge cue points to it.
    group = {'trail_visible': True, 'marker_visible': True, 'marker_in_view': True,
        'site_in_view': False, 'site_on_screen': False, 'stake_visible': False,
        'edge_visible': True, 'dash_px': 2, 'gap_px': 3, 'trail_alpha': .6,
        'trail_color': 'E8D8A8', 'bundles': []}
    # When: unchanged A28 requires the actual site to be on screen.
    errors = check_group(group)
    # Then: A28 still fails; separate A32 is allowed to accept the arrow.
    assert errors == ['site marker is outside view']


def test_windfall_ring_survives_drop_until_revised_boundary() -> None:
    from observation_rules_p8 import intervention_visible
    # Given: Part4 holds dust1s and fades through1.3s, independently of the0.8s drop.
    observations = ['windfall_dust', 'windfall_drop_ended']
    # When: a fading-ring frame is evaluated.
    accepted = intervention_visible('god_windfall', 'windfall', 1.15, observations)
    # Then: the new ring is expected while the pile drop has ended.
    assert accepted


def test_windfall_ring_must_end_at_one_point_three_seconds() -> None:
    from observation_rules_p8 import intervention_visible
    # Given: an observer still sees dust at the exact terminal boundary.
    observations = ['windfall_dust', 'windfall_drop_ended']
    # When: the terminal frame is checked.
    accepted = intervention_visible('god_windfall', 'windfall', 1.3, observations)
    # Then: lingering dust fails the explicit duration.
    assert not accepted


def test_changed_final_lineup_invalidates_prior_activity_pass(tmp_path) -> None:
    import hashlib
    import json

    from blind_identity_p8 import final_identity, guard_activity
    from ingame_qa_pixels import Check
    # Given: a proof binds earlier lineup bytes and observer grading passed.
    sources = {}
    for name in ('a_lineup_z4.png', 'a_lineup_z2.png'):
        (tmp_path / name).write_bytes(b'original calibration pixels')
        sources[name] = hashlib.sha256((tmp_path / name).read_bytes()).hexdigest()
    proof = tmp_path / 'proof.json'
    proof.write_text(json.dumps({'passed': True, 'source_sha256': sources}))
    (tmp_path / 'a_lineup_z2.png').write_bytes(b'changed calibration pixels')
    # When: final source identity guards the inherited grading result.
    checks = guard_activity([Check('A11', 'PASS', 'prior observer grading')], final_identity(tmp_path, proof))
    # Then: both final identity and A11 fail without rewriting observer answers.
    assert [check.status for check in checks] == ['FAIL', 'FAIL']


def test_performance_rejects_normalization_provenance_errors() -> None:
    # Given: otherwise valid controlled CPU samples carry a rejected source-tick audit.
    evidence: PerformanceEvidence = {'blocks': [{'layers_enabled': index % 2 == 0, 'tick': 123,
        'camera': [2.0, 3.0], 'samples': [{'process_ms': .8,
        'render_cpu_ms': .2, 'setup_cpu_ms': .1, 'gpu_ms': 0.0}] * 50} for index in range(12)],
        'alive_count': 3, 'first_three_alive_tick': 123, 'framing': 'FV', 'simulation_frozen': True,
        'vsync_disabled': True, 'vsync_restored': True, 'run_id': 'fixture',
        'uptime': 'load averages: 1.0 1.0 1.0', 'processes': '', 'process_priorities': [-1000, 1000],
        'normalization_errors': ['raw tick differs from measured first-three-alive tick']}
    # When: normalization provenance participates in A30.
    check = performance(evidence)
    # Then: favorable CPU numbers cannot override invalid provenance.
    assert check.status == 'FAIL'


def test_performance_rejects_explicit_raw_invalidity() -> None:
    # Given: valid-sized numbers come from an explicitly invalid raw measurement.
    evidence: PerformanceEvidence = {'blocks': [{'layers_enabled': index % 2 == 0, 'tick': 123,
        'camera': [2.0, 3.0], 'samples': [{'process_ms': .8,
        'render_cpu_ms': .2, 'setup_cpu_ms': .1, 'gpu_ms': 0.0}] * 50} for index in range(12)],
        'alive_count': 3, 'first_three_alive_tick': 123, 'framing': 'FV', 'simulation_frozen': True,
        'vsync_disabled': True, 'vsync_restored': True, 'run_id': 'fixture',
        'uptime': 'load averages: 1.0 1.0 1.0', 'processes': '', 'process_priorities': [-1000, 1000],
        'valid': False, 'errors': ['render timer was not enabled']}
    # When: raw validity is evaluated.
    check = performance(evidence)
    # Then: the source rejection remains a failure.
    assert check.status == 'FAIL'
