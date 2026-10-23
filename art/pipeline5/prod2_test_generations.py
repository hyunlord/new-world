# /// script
# requires-python = ">=3.11"
# dependencies = ["pillow", "numpy", "scipy", "scikit-image", "pytest"]
# ///
# How to run: python -m pytest art/pipeline5/prod2_test_generations.py
"""Pending calls spend budget and accepted looks require scored review retries."""
import json
from pathlib import Path

import pytest

from art.pipeline5.prod1_records import JsonValue
from art.pipeline5.prod2_generations import Locations, corrected_prompt, prepare

LOOK = 'adult_M_hide_wrap_short'


def setup_locations(root: Path) -> Locations:
    locations = Locations(root / 'data', root / 'repo')
    locations.data.mkdir()
    reference = locations.repo / 'art/character/candidates/I-1.png'
    reference.parent.mkdir(parents=True)
    reference.write_bytes(b'reference bytes')
    prompt = locations.data / 'original.txt'
    prompt.write_text('Create one sheet. Same scale for all ten poses. Child sleep width48-60px. Keep warm watercolor.')
    (locations.data / 'FROZEN_SOURCES.json').write_text(json.dumps([{'look': LOOK, 'attempt': 1, 'prompt': str(prompt)}]))
    (locations.data / 'PART0_RESULTS_FULL.json').write_text('{"determinism_passed":true}')
    return locations


def test_pending_intents_exhaust_per_look_budget(tmp_path: Path) -> None:
    # Given two image calls reserved but not yet returned.
    locations = setup_locations(tmp_path)
    prepare(LOOK, 'gap', 'clean alpha', locations)
    prepare(LOOK, 'gap', 'clean alpha', locations)
    # When a third call is requested, then it is refused and both pending calls remain counted.
    with pytest.raises(ValueError, match='generation_limit'):
        prepare(LOOK, 'gap', 'clean alpha', locations)
    assert json.loads(locations.ledger.read_text())['reserved_calls'] == 2


def test_total_budget_counts_failed_and_pending_calls(tmp_path: Path) -> None:
    # Given forty reserved historical calls.
    locations = setup_locations(tmp_path)
    for number in range(40):
        path = locations.data / 'generation' / str(number) / 'try1/intent.json'
        path.parent.mkdir(parents=True)
        path.write_text(json.dumps({'look': str(number), 'status': 'reserved'}))
    # When a new look is requested, then the global budget refuses it.
    with pytest.raises(ValueError, match='generation_limit'):
        prepare(LOOK, 'gap', 'clean alpha', locations)


def test_part0_is_required(tmp_path: Path) -> None:
    # Given incomplete Part0 evidence.
    locations = setup_locations(tmp_path)
    (locations.data / 'PART0_RESULTS_FULL.json').unlink()
    # When a generation is requested, then no budget is reserved.
    with pytest.raises(ValueError, match='Part0'):
        prepare(LOOK, 'gap', 'clean alpha', locations)
    assert not list((locations.data / 'generation').glob('*/try*/intent.json'))


def test_accepted_look_requires_scored_single_review(tmp_path: Path) -> None:
    # Given an accepted look and recorded reviewer target.
    locations = setup_locations(tmp_path)
    accepted = locations.output / LOOK / 'MANIFEST.json'
    accepted.parent.mkdir(parents=True)
    raw = locations.data / 'accepted.png'
    raw.write_bytes(b'accepted raw')
    accepted.write_text(json.dumps({'raw': str(raw)}))
    # When gap or unscored review is requested, then each is refused.
    with pytest.raises(ValueError, match='gap generation'):
        prepare(LOOK, 'gap', 'clean alpha', locations)
    with pytest.raises(ValueError, match='review needs'):
        prepare(LOOK, 'review', 'hair misread', locations)
    (locations.data / 'scores.json').write_text(json.dumps(review_scores()))
    intent = prepare(LOOK, 'review', 'hair misread', locations)
    assert len(json.loads(intent.read_text())['references']) == 2
    with pytest.raises(ValueError, match='review needs'):
        prepare(LOOK, 'review', 'hair misread again', locations)


def test_prompt_removes_obsolete_sleep_scale() -> None:
    # Given a PROD1 common-scale and adult-band child instruction.
    original = 'Create one sheet. Same scale for all ten poses. Sleep width48–60px for children. Sleeping arms must stretch BEYOND HEAD leftward adding 25 percent extra silhouette length. Keep warm watercolor.'
    # When converted to PROD2, then conflicting rules disappear and style survives.
    result = corrected_prompt(original, 'clean alpha')
    assert 'Same scale for all ten' not in result
    assert 'width48' not in result
    assert 'extra silhouette length' not in result
    assert 'Keep warm watercolor' in result
    assert 'child length37-43' in result


def test_finish_preserves_raw_and_publishes_only_complete_pass(tmp_path: Path, monkeypatch: pytest.MonkeyPatch) -> None:
    # Given a budgeted call with a ten-panel synthetic source.
    from art.pipeline5.prod1_test_process import fixture_request, passing_judge
    from art.pipeline5.prod2_generations import finish
    locations = setup_locations(tmp_path)
    source_root = tmp_path / 'source'
    source_root.mkdir()
    source = fixture_request(source_root)
    prepare(LOOK, 'gap', 'clean alpha', locations)
    monkeypatch.setattr('art.pipeline5.prod1_process.judge_panels', passing_judge)
    # When the tool original is finished through the real evaluate/publication pipeline.
    accepted = finish(LOOK, 1, source.raw, 'tool-output', locations=locations)
    # Then source bytes, accepted frame set and call count remain bound together.
    assert accepted
    intent = json.loads((locations.data / 'generation' / LOOK / 'try1/intent.json').read_text())
    assert intent['status'] == 'accepted'
    assert Path(intent['raw']).read_bytes() == source.raw.read_bytes()
    assert len(list((locations.output / LOOK).glob('*/*.png'))) == 10
    assert json.loads(locations.ledger.read_text())['reserved_calls'] == 1
    with pytest.raises(ValueError, match='already finished'):
        finish(LOOK, 1, source.raw, 'tool-output', locations=locations)


def review_scores() -> dict[str, JsonValue]:
    """Return a complete scorer-shaped failed first round."""
    return {'schema': 2, 'round': 0, 'all_numeric_bars_met': False,
            'misread_looks': [{'look': LOOK}],
            'reviewers': [{'cases': [{'id': f'p1_{number:02d}', 'zoom': zoom}
                                   for number in range(1, 13) for zoom in (1, 2)],
                           'summaries': [{'zoom': 1}, {'zoom': 2}]} for _ in range(2)]}


@pytest.mark.parametrize('change', ['round1', 'passing', 'missing_rows', 'wrong_target'])
def test_review_wrong_round_or_target_cannot_authorize(tmp_path: Path, change: str) -> None:
    # Given a score report that does not authorize this target.
    from art.pipeline5.prod2_generations import review_target
    report = review_scores()
    if change == 'round1':
        report['round'] = 1
    elif change == 'passing':
        report['all_numeric_bars_met'] = True
    elif change == 'missing_rows':
        reviewers = report['reviewers']
        assert isinstance(reviewers, list)
        reviewer = reviewers[0]
        assert isinstance(reviewer, dict)
        cases = reviewer['cases']
        assert isinstance(cases, list)
        _ = cases.pop()
    else:
        report['misread_looks'] = [{'look': 'child_M_hide_wrap_short'}]
    path = tmp_path / 'scores.json'
    path.write_text(json.dumps(report))
    # When review authorization is checked, then invalid evidence fails closed.
    assert not review_target(path, LOOK)
