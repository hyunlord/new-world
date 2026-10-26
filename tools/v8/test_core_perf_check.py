"""Pure negative fixtures for paired performance evidence; no benchmarks or builds.

Pytest discovers these functions; direct execution also needs only the stdlib.
"""
from __future__ import annotations

from typing import Callable
from pathlib import Path
from tempfile import TemporaryDirectory

import core_perf_check as perf


def rejects(action: Callable[[], None]) -> None:
    """Require the evidence parser to fail closed."""
    try:
        action()
    except perf.EvidenceError:
        return
    raise AssertionError('invalid evidence was accepted')


def test_three_load_samples_required_and_finite() -> None:
    assert perf.loads('7.99 9.4 10.2') == (7.99, 9.4, 10.2)
    assert perf.loads('7.99, 9.4, 10.2') == (7.99, 9.4, 10.2)
    rejects(lambda: perf.loads('7.99'))
    rejects(lambda: perf.loads('nan 2 3'))
    rejects(lambda: perf.loads('-1 2 3'))
    assert perf.loads('8 2 3')[0] >= 8


def test_full_standing_scene_matrix() -> None:
    assert len(perf.sequence(False)) == 18
    assert len(perf.sequence(True)) == 15
    assert len(set(perf.sequence(False))) == 18
    assert len(set(perf.sequence(True))) == 15
    assert perf.sequence(False)[6] == (2, 'n200')


def test_arithmetic_ratio_is_not_average_of_ratios() -> None:
    assert perf.mean_ratio([1, 1, 10], [1, 1, 1]) == 0.25
    assert abs(1 - perf.SHARE / 2 + 0.05 - perf.TICK_BAR) < 1e-15
    rejects(lambda: perf.mean_ratio([1, 1], [1, 1]))
    rejects(lambda: perf.mean_ratio([0, 1, 1], [1, 1, 1]))


def test_missing_saturation_is_pending_not_pass() -> None:
    result = perf.Document.parse(perf.saturation({'n200': [perf.Document({})]}))
    assert result.text('status') == 'PENDING'


def saturated(people: int, allocation: int) -> perf.Document:
    """Construct a bounded companion fixture, never a timing claim."""
    return perf.Document({'mind_history_memory': {
        'capacity': 16, 'people': people, 'saturated_people': people,
        'measured_after_saturation': True, 'method': 'synthetic test fixture',
        'allocator_live_bytes': allocation, 'allocator_delta_bytes': 100,
        'before': {'allocator_live_bytes': allocation - 100},
        'after': {'allocator_live_bytes': allocation, 'min_records': 16,
                  'max_records': 16, 'records': people * 16}}})


def test_saturation_slopes_are_separate_companion_not_standing_c3() -> None:
    result = perf.Document.parse(perf.saturation({
        'n200': [saturated(200, 1_000_000)] * 3,
        'n2000': [saturated(2000, 10_000_000)] * 3,
        'n200_side64': [saturated(200, 500_000)] * 3}))
    assert result.text('status') == 'MEASURED'
    assert result.number('bytes_per_agent') == 5000
    assert result.text('comparison_to_4096_bytes_per_agent') == 'MISS'
    assert result.values.get('standing_c3_instrument_unchanged') is True


def test_unsaturated_companion_rejected() -> None:
    row = saturated(1, 1000)
    data = dict(row.child('mind_history_memory').values)
    data['saturated_people'] = 0
    rejects(lambda: perf.saturation({'n200': [perf.Document({'mind_history_memory': data})]}))


def test_missing_or_duplicate_measured_system_rejected() -> None:
    rejects(lambda: perf.system_us(perf.Document({'systems': [], 'measured_ticks': 2880}), 'ColonyMindSystem'))
    row = {'name': 'AgentDecisionSystem', 'total_ms': 2880}
    rejects(lambda: perf.system_us(perf.Document({'systems': [row, row], 'measured_ticks': 2880}), 'AgentDecisionSystem'))
    assert perf.system_us(perf.Document({'systems': [row], 'measured_ticks': 2880}), 'AgentDecisionSystem') == 1000


def test_external_receipt_output_is_exclusive_and_byte_identical() -> None:
    with TemporaryDirectory() as directory:
        ordinary = Path(directory) / 'ordinary.json'
        fresh = Path(directory) / 'fresh.json'
        receipt = {'status': 'PASS', 'files': []}
        perf.write(ordinary, receipt)
        perf.write(fresh, receipt, exclusive=True)
        assert fresh.read_bytes() == ordinary.read_bytes()
        original = fresh.read_bytes()
        try:
            perf.write(fresh, {'status': 'FAIL'}, exclusive=True)
        except FileExistsError:
            pass
        else:
            raise AssertionError('exclusive receipt overwrote an existing file')
        assert fresh.read_bytes() == original


def test_read_only_check_rejects_session_local_output_before_reading_inputs() -> None:
    with TemporaryDirectory() as directory:
        session = Path(directory)
        rejects(lambda: perf.check(session, session / 'receipt.json'))
        assert not (session / 'receipt.json').exists()


def schedule_fixture() -> list[perf.Document]:
    rows = []
    for build_only in (False, True):
        for repeat, scene in perf.sequence(build_only):
            for role in (('base', 'new') if repeat % 2 else ('new', 'base')):
                start = len(rows) * 2
                rows.append(perf.Document({'build_only': build_only, 'repeat': repeat,
                    'scene': scene, 'role': role, 'attempt': 1, 'started': start,
                    'ended': start + 1, 'status': 'VALID', 'pair_accepted': True}))
    return rows


def resequence(rows: list[perf.Document]) -> list[perf.Document]:
    return [perf.Document({**row.values, 'started': index * 2, 'ended': index * 2 + 1})
            for index, row in enumerate(rows)]


def test_grouped_but_nonadjacent_pairs_rejected() -> None:
    rows = schedule_fixture()
    perf.validate_schedule(rows)
    rows[:4] = [rows[0], rows[2], rows[1], rows[3]]
    rejects(lambda: perf.validate_schedule(resequence(rows)))


def test_retry_order_and_whole_pair_admission() -> None:
    rows = schedule_fixture()
    failed = [perf.Document({**row.values, 'status': 'INVALID', 'pair_accepted': False}) for row in rows[:2]]
    retry = [perf.Document({**row.values, 'attempt': 2}) for row in rows[:2]]
    valid = resequence(failed + retry + rows[2:])
    perf.validate_schedule(valid)
    skipped = [perf.Document({**row.values, 'attempt': 3}) for row in retry]
    rejects(lambda: perf.validate_schedule(resequence(failed + skipped + rows[2:])))
    rejects(lambda: perf.validate_schedule(resequence(rows[:2] + retry + rows[2:])))
    half = [perf.Document({**rows[0].values, 'pair_accepted': False}), *rows[1:]]
    rejects(lambda: perf.validate_schedule(half))
    rejects(lambda: perf.validate_schedule(rows[:-1]))


def test_real_dimensions_cannot_be_relabelled() -> None:
    correct = {'label': 'n1000_wolves', 'agents': 1000, 'side': 192, 'script': False,
               'wolves': 4, 'seed': 15100, 'warmup_ticks': 1440, 'measured_ticks': 2880,
               'build_profile': 'release'}
    perf.validate_dimensions(perf.Document(correct), 'n1000_wolves', False, 1)
    for field, wrong in [('agents', 200), ('side', 64), ('script', True), ('wolves', 0), ('seed', 1)]:
        corrupt = perf.Document({**correct, field: wrong})
        rejects(lambda: perf.validate_dimensions(corrupt, 'n1000_wolves', False, 1))
    rss = {'label': 'n200_side64', 'agents': 200, 'side': 64, 'build_only': True, 'repeat': 1}
    perf.validate_dimensions(perf.Document(rss), 'n200_side64', True, 1)
    rejects(lambda: perf.validate_dimensions(perf.Document({**rss, 'side': 192}), 'n200_side64', True, 1))


def test_wrong_valid_base_commit_rejected_before_proof_can_authorize_it() -> None:
    current = perf.git('rev-parse', 'HEAD')
    assert current != perf.EXPECTED_BASE
    rejects(lambda: perf.validate_base(current, current, perf.git('rev-parse', current + ':rust')))


def test_report_descendant_requires_ancestry_and_identical_source() -> None:
    current = perf.git('rev-parse', 'HEAD')
    perf.validate_candidate(perf.EXPECTED_BASE, current, 'same', 'same')
    rejects(lambda: perf.validate_candidate(current, perf.EXPECTED_BASE, 'same', 'same'))
    rejects(lambda: perf.validate_candidate(perf.EXPECTED_BASE, current, 'before', 'after'))


def test_frozen_untracked_lock_requires_unchanged_external_manifest() -> None:
    with TemporaryDirectory() as directory:
        lock = Path(directory) / 'Cargo.lock'
        manifest = Path(directory) / 'SHA256SUMS'
        lock.write_text('version = 4\n')
        manifest.write_text(f'{perf.sha(lock)}  {lock}\n')
        retained_pin = perf.sha(manifest)
        perf.validate_frozen_lock(lock, manifest, retained_pin)
        lock.write_text('version = 3\n')
        rejects(lambda: perf.validate_frozen_lock(lock, manifest, retained_pin))
        manifest.write_text(f'{perf.sha(lock)}  {lock}\n')
        rejects(lambda: perf.validate_frozen_lock(lock, manifest, retained_pin))
        manifest.unlink()
        rejects(lambda: perf.validate_frozen_lock(lock, manifest, retained_pin))
        manifest.write_text('')
        rejects(lambda: perf.validate_frozen_lock(lock, manifest, perf.sha(manifest)))
        manifest.write_text(f'{perf.sha(lock)}  {lock}\n')
        retained_pin = perf.sha(manifest)
        lock.unlink()
        rejects(lambda: perf.validate_frozen_lock(lock, manifest, retained_pin))


if __name__ == '__main__':
    test_three_load_samples_required_and_finite()
    test_full_standing_scene_matrix()
    test_arithmetic_ratio_is_not_average_of_ratios()
    test_missing_saturation_is_pending_not_pass()
    test_saturation_slopes_are_separate_companion_not_standing_c3()
    test_unsaturated_companion_rejected()
    test_missing_or_duplicate_measured_system_rejected()
    test_external_receipt_output_is_exclusive_and_byte_identical()
    test_read_only_check_rejects_session_local_output_before_reading_inputs()
    test_grouped_but_nonadjacent_pairs_rejected()
    test_retry_order_and_whole_pair_admission()
    test_real_dimensions_cannot_be_relabelled()
    test_wrong_valid_base_commit_rejected_before_proof_can_authorize_it()
    test_report_descendant_requires_ancestry_and_identical_source()
    test_frozen_untracked_lock_requires_unchanged_external_manifest()
    print('15 evidence fixtures passed; no measurements performed')
