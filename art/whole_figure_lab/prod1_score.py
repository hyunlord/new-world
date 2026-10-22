# /// script
# requires-python = ">=3.11"
# dependencies = []
# ///
"""Score two blind reviewers without guessing unregistered language equivalents."""
from __future__ import annotations

import hashlib
import json
import sys
from dataclasses import dataclass
from pathlib import Path
from typing import Final

FIELDS: Final = ('activity', 'item', 'top', 'hair', 'sex', 'age')
HISTORICAL_SOURCE: Final = '/Users/rexxa/work/arttrial5-data/production/reviews/round1_scores.json'
HISTORICAL_SHA256: Final = '6e13a094bf67792ef28760b196bd8112ecbfd0758ef175812177961acee78a7c'
MAPPING: Final = {'activity': {'idle standing': 'idle',
              'standing idle': 'idle',
              'walking': 'walk',
              'overhead chopping': 'work',
              'chopping or striking overhead': 'work',
              'walking while carrying': 'carry',
              'sleeping curled on ground': 'sleep',
              'sleeping curled on the ground': 'sleep',
              'overhead tool swing': 'work',
              'overhead tool swing; chopping or striking': 'work',
              'carrying a load while stepping': 'carry',
              'carrying while walking': 'carry',
              'sleeping curled on side': 'sleep',
              'sleeping or lying curled on side': 'sleep'},
 'item': {'none': 'none',
          'none visible': 'none',
          'stone axe': 'stone_axe',
          'axe': 'stone_axe',
          'bundle of logs': 'log_bundle',
          'bundle of logs on shoulders': 'log_bundle',
          'bundle of brown logs across shoulders': 'log_bundle'},
 'sex': {'male': 'M', 'masculine': 'M', 'female': 'F', 'feminine': 'F'},
 'age': {'adult': 'adult', 'child': 'child'},
 'hair': {'short dark brown': 'short',
          'dark topknot': 'bun',
          'dark top bun': 'bun',
          'dark bun': 'bun',
          'long dark hair tied back': 'long_tied',
          'dark ponytail': 'long_tied',
          'short dark brown hair': 'short',
          'short brown': 'short',
          'dark brown high bun': 'bun',
          'dark brown topknot or bun': 'bun',
          'dark brown bun': 'bun',
          'long dark brown hair tied behind head': 'long_tied',
          'dark brown low ponytail': 'long_tied',
          'dark brown tied back': 'long_tied'},
 'top': {'one-shoulder hide wrap': 'hide_wrap',
         'brown one-shoulder hide wrap': 'hide_wrap',
         'brown one-shoulder wrap': 'hide_wrap',
         'light hide wrap': 'hide_wrap',
         'light brown wrap': 'hide_wrap',
         'pale-edged brown hide wrap': 'hide_wrap',
         'brown cloak over hide tunic': 'fur_cloak',
         'dark brown cloak and hide tunic': 'fur_cloak',
         'dark brown cloak and tunic': 'fur_cloak',
         'dark brown cloak over a pale-edged hide tunic': 'fur_cloak',
         'dark brown cloak over hide tunic': 'fur_cloak'}}
CANONICAL: Final = {
    'activity': ('idle', 'walk', 'work', 'carry', 'sleep'),
    'item': ('none', 'stone_axe', 'log_bundle'),
    'top': ('loincloth', 'hide_wrap', 'fur_cloak'),
    'hair': ('short', 'long_tied', 'bun'),
    'sex': ('M', 'F'), 'age': ('adult', 'child'),
}

# Exact observed historical decisions; age scope preserves adult-only bare-chest equivalence.
HISTORICAL_ALIASES: Final = {'adult': {'activity': {'carrying a load while stepping': 'carry', 'carrying while walking': 'carry', 'chopping or striking overhead': 'work', 'idle standing': 'idle', 'overhead chopping': 'work', 'overhead tool swing': 'work', 'overhead tool swing; chopping or striking': 'work', 'sleeping curled on ground': 'sleep', 'sleeping curled on side': 'sleep', 'sleeping curled on the ground': 'sleep', 'sleeping or lying curled on side': 'sleep', 'standing idle': 'idle', 'walking': 'walk', 'walking while carrying': 'carry'}, 'age': {'adult': 'adult'}, 'hair': {'dark brown bun': 'bun', 'dark brown high bun': 'bun', 'dark brown tied back': 'long_tied', 'dark brown topknot or bun': 'bun', 'dark bun': 'bun', 'dark top bun': 'bun', 'dark topknot': 'bun', 'short dark brown': 'short', 'short dark brown hair': 'short'}, 'item': {'bundle of brown logs across shoulders': 'log_bundle', 'bundle of logs': 'log_bundle', 'bundle of logs on shoulders': 'log_bundle', 'none': 'none', 'none visible': 'none', 'stone axe': 'stone_axe'}, 'sex': {'feminine': 'F', 'male': 'M', 'masculine': 'M'}, 'top': {'bare chest': 'loincloth', 'bare chest, tan waist wrap': 'loincloth', 'bare chest, tan waist wrap with pale edge': 'loincloth', 'bare chest; pale edged brown loincloth': 'loincloth', 'bare upper body': 'loincloth', 'bare upper body with brown waist wrap': 'loincloth', 'bare upper body with tan waist cloth': 'loincloth', 'bare upper body, pale waist cloth': 'loincloth', 'brown cloak over hide tunic': 'fur_cloak', 'brown hide wrap with pale diagonal shoulder strap; much of chest bare': 'hide_wrap', 'brown one-shoulder hide wrap': 'hide_wrap', 'brown one-shoulder wrap': 'hide_wrap', 'brown wrap with pale diagonal strap and pale hem': 'hide_wrap', 'dark brown cloak and hide tunic': 'fur_cloak', 'dark brown cloak and tunic': 'fur_cloak', 'dark brown cloak over a pale-edged hide tunic': 'fur_cloak', 'dark brown cloak over hide tunic': 'fur_cloak', 'dark brown cloak with pale collar over brown tunic': 'fur_cloak', 'dark brown cloak with pale diagonal collar and trim': 'fur_cloak', 'dark brown draped cloak with pale trim': 'fur_cloak', 'dark brown sleeveless tunic or wrap, pale edging and dark cloak': 'fur_cloak', 'dark brown tunic or wrap with broad pale shoulder trim and dark cloak': 'fur_cloak', 'one-shoulder hide wrap': 'hide_wrap', 'pale tan diagonal chest wrap': 'hide_wrap', 'tan one-shoulder hide wrap with pale edging': 'hide_wrap', 'tan one-shoulder wrap with much chest exposed': 'hide_wrap', 'tan one-shoulder wrap with pale edging': 'hide_wrap'}}, 'child': {'activity': {'carrying a load while stepping': 'carry', 'carrying while walking': 'carry', 'chopping or striking overhead': 'work', 'overhead chopping': 'work', 'overhead tool swing': 'work', 'overhead tool swing; chopping or striking': 'work', 'sleeping curled on ground': 'sleep', 'sleeping curled on side': 'sleep', 'sleeping curled on the ground': 'sleep', 'sleeping or lying curled on side': 'sleep', 'walking': 'walk', 'walking while carrying': 'carry'}, 'age': {'child': 'child'}, 'hair': {'dark brown high bun': 'bun', 'dark brown low ponytail': 'long_tied', 'dark brown topknot or bun': 'bun', 'dark bun': 'bun', 'dark ponytail': 'long_tied', 'dark topknot': 'bun', 'long dark brown hair tied behind head': 'long_tied', 'long dark hair tied back': 'long_tied', 'short brown': 'short', 'short dark brown': 'short', 'short dark brown hair': 'short'}, 'item': {'axe': 'stone_axe', 'bundle of brown logs across shoulders': 'log_bundle', 'bundle of logs': 'log_bundle', 'bundle of logs on shoulders': 'log_bundle', 'none': 'none', 'none visible': 'none', 'stone axe': 'stone_axe'}, 'sex': {'feminine': 'F', 'masculine': 'M'}, 'top': {'brown one-shoulder wrap': 'hide_wrap', 'brown torso wrap with pale crossed edges': 'hide_wrap', 'brown wrap with pale diagonal shoulder band and pale hem': 'hide_wrap', 'light brown wrap': 'hide_wrap', 'light hide wrap': 'hide_wrap', 'mostly bare chest with narrow pale diagonal strap and brown waist wrap': 'hide_wrap', 'one-shoulder hide wrap': 'hide_wrap', 'pale tan diagonal wrap': 'hide_wrap', 'pale-edged brown hide wrap': 'hide_wrap', 'tan and brown wrap across torso': 'hide_wrap', 'tan diagonal wrap with pale edging': 'hide_wrap', 'tan one-shoulder wrap': 'hide_wrap', 'tan one-shoulder wrap with pale diagonal border': 'hide_wrap'}}}
ALIAS_PROVENANCE: Final = {'sources': {'/Users/rexxa/work/arttrial5-data/production/reviews/round0_scores.json': '55a57a78caf5472c79c6f9185e579be15957885185af0ea24ba2614b12bbf153', '/Users/rexxa/work/arttrial5-data/production/reviews/round1_scores.json': '6e13a094bf67792ef28760b196bd8112ecbfd0758ef175812177961acee78a7c'}, 'accepted_age_field_phrase_count': 106, 'conflicting_pairs': [], 'unknown_age_field_phrases': [['adult', 'age', 'unknown'], ['adult', 'hair', 'dark brown'], ['adult', 'hair', 'dark brown hair swept behind head; length unclear'], ['adult', 'hair', 'dark brown long hair falling behind shoulder'], ['adult', 'hair', 'dark brown shoulder-length'], ['adult', 'hair', 'dark brown tied back or shoulder-length'], ['adult', 'hair', 'dark brown tied-up hair'], ['adult', 'hair', 'dark swept-back'], ['adult', 'hair', 'long dark brown'], ['adult', 'hair', 'long dark brown hair swept behind shoulder'], ['adult', 'item', 'short handled gray stone axe or hammer'], ['adult', 'item', 'stone axe or hammer'], ['adult', 'sex', 'unknown'], ['adult', 'top', 'dark brown cloak or heavy shoulder wrap'], ['adult', 'top', 'unknown'], ['child', 'age', 'unknown'], ['child', 'hair', 'dark brown hair; style obscured by logs'], ['child', 'hair', 'dark brown, length unknown under load'], ['child', 'hair', 'dark brown, partly hidden'], ['child', 'hair', 'long dark brown'], ['child', 'hair', 'unknown'], ['child', 'item', 'short handled gray stone axe or hammer'], ['child', 'item', 'stone axe or hammer'], ['child', 'sex', 'unknown'], ['child', 'top', 'bare-looking chest with tan waist cloth'], ['child', 'top', 'brown cloak or blanket with pale border'], ['child', 'top', 'brown garment or blanket'], ['child', 'top', 'brown garment with a pale shoulder band; precise cut unclear'], ['child', 'top', 'brown hide tunic'], ['child', 'top', 'brown sleeveless tunic or wrap; upper garment partly obscured'], ['child', 'top', 'brown tunic'], ['child', 'top', 'brown tunic or hide wrap'], ['child', 'top', 'brown tunic or wrap with pale neckline and hem'], ['child', 'top', 'brown tunic with pale edging'], ['child', 'top', 'brown tunic with pale trim'], ['child', 'top', 'pale tan narrow chest wrap and skirt'], ['child', 'top', 'small brown wrap with a pale band across chest and pale hem'], ['child', 'top', 'small tan chest wrap'], ['child', 'top', 'tan chest band and short skirt with pale edging'], ['child', 'top', 'tan hide wrap or covering'], ['child', 'top', 'unknown']]}


@dataclass(frozen=True, slots=True)
class Row:
    """One strictly identified case, with classification separated from notes."""
    number: int
    zoom: int
    answers: dict[str, str]
    look: str = ''


def mapped(field: str, raw: str, age: str = '') -> str:
    """Only exact registered phrases or canonical labels count as known."""
    if raw in CANONICAL[field]:
        return raw
    return MAPPING[field].get(raw, HISTORICAL_ALIASES.get(age, {}).get(field, {}).get(raw, 'unknown'))


def read_rows(path: Path, *, key: bool = False) -> tuple[Row, ...]:
    """Require every one of twelve identities at both scales exactly once."""
    data = json.loads(path.read_text())
    if not isinstance(data, list):
        raise TypeError(f'Expected a JSON array of 24 rows: {path}')
    result = []
    seen = set()
    for raw in data:
        if not isinstance(raw, dict):
            raise TypeError(f'Expected a row object: {path}')
        if key:
            number = raw.get('number')
        else:
            identifier = raw.get('id')
            if identifier not in {f'p1_{n:02d}' for n in range(1, 13)}:
                raise ValueError(f'Invalid specimen id in {path}: {identifier}')
            number = int(identifier[-2:])
        zoom = raw.get('zoom')
        if type(number) is not int or number not in range(1, 13) or type(zoom) is not int or zoom not in (1, 2):
            raise ValueError(f'Invalid specimen number/zoom: {path}')
        if (number, zoom) in seen:
            raise ValueError(f'Duplicate specimen {number}/{zoom}: {path}')
        seen.add((number, zoom))
        answers = {}
        for field in FIELDS:
            if field not in raw:
                raise ValueError(f'Missing field {field}: {path}')
            value = raw[field]
            if not isinstance(value, str):
                raise TypeError(f'Non-text field {field}: {path}')
            if key and value not in CANONICAL[field]:
                raise ValueError(f'Noncanonical answer key field {field}: {path}')
            answers[field] = value
        look = f"{answers['age']}_{answers['sex']}_{answers['top']}_{answers['hair']}" if key else ''
        result.append(Row(number, zoom, answers, look))
    if seen != {(n, z) for n in range(1, 13) for z in (1, 2)}:
        raise ValueError(f'Missing specimen rows; need 24 unique rows: {path}')
    if key:
        for n in range(1, 13):
            paired = [r for r in result if r.number == n]
            if paired[0].answers != paired[1].answers:
                raise ValueError(f'Answer key differs between scales for specimen {n}')
    return tuple(result)


def digest(path: Path) -> str:
    return hashlib.sha256(path.read_bytes()).hexdigest()


def score(key_path: Path, reviewer_paths: tuple[Path, Path], output: Path, round_number: int = 0) -> None:
    """Write exact raw evidence and all per-field decisions; never authorize a third round."""
    if round_number not in (0, 1):
        raise ValueError('Only initial round 0 and one targeted round 1 are allowed')
    if reviewer_paths[0].resolve() == reviewer_paths[1].resolve():
        raise ValueError('Two distinct reviewer response files required')
    truth = {(r.number, r.zoom): r for r in read_rows(key_path, key=True)}
    reviews = [read_rows(p) for p in reviewer_paths]
    summaries = []
    targets: dict[str, set[str]] = {}
    for reviewer, (path, rows) in enumerate(zip(reviewer_paths, reviews, strict=True), 1):
        cases = []
        for row in rows:
            expected = truth[(row.number, row.zoom)]
            fields = {}
            for field in FIELDS:
                raw = row.answers[field]
                value = mapped(field, raw, expected.answers['age'])
                correct = value != 'unknown' and value == expected.answers[field]
                fields[field] = {'raw': raw, 'mapped': value, 'truth': expected.answers[field], 'correct': correct,
                    'mapping_status': 'REGISTERED' if value != 'unknown' else 'UNKNOWN_PENDING_EXPLICIT_EQUIVALENCE_REVIEW'}
                if not correct:
                    targets.setdefault(expected.look, set()).add(field)
            cases.append({'id': f'p1_{row.number:02d}', 'zoom': row.zoom, 'look': expected.look, 'fields': fields})
        scales = []
        for zoom in (1, 2):
            counts = {field: sum(c['fields'][field]['correct'] for c in cases if c['zoom'] == zoom) for field in FIELDS}
            bars = {'activity': (counts['activity'], 12, 8 if zoom == 1 else 10),
                    'item_top_hair': (counts['item'] + counts['top'] + counts['hair'], 36, 30),
                    'sex_age': (counts['sex'] + counts['age'], 24, 20)}
            scales.append({'zoom': zoom, 'field_correct': counts,
                'bars': {name: {'correct': values[0], 'denominator': values[1], 'bar': values[2],
                               'passed': values[0] >= values[2]} for name, values in bars.items()},
                'all_numeric_bars_met': all(values[0] >= values[2] for values in bars.values())})
        summaries.append({'reviewer': reviewer, 'raw_path': str(path.resolve()), 'raw_sha256': digest(path),
                          'verbatim_copy': f'reviewer_{reviewer}.verbatim.json', 'summaries': scales, 'cases': cases})
    output.mkdir(parents=True, exist_ok=True)
    for number, path in enumerate(reviewer_paths, 1):
        (output / f'reviewer_{number}.verbatim.json').write_bytes(path.read_bytes())
    result = {'schema': 1, 'round': round_number, 'answer_key': str(key_path.resolve()),
        'answer_key_sha256': digest(key_path), 'historical_mapping_source': HISTORICAL_SOURCE,
        'historical_mapping_sha256': HISTORICAL_SHA256, 'mapping_dictionary': MAPPING,
        'canonical_aliases': CANONICAL, 'historical_aliases': HISTORICAL_ALIASES,
        'alias_provenance': ALIAS_PROVENANCE, 'reviewers': summaries,
        'all_numeric_bars_met': all(s['all_numeric_bars_met'] for r in summaries for s in r['summaries']),
        'eligibility': 'Numeric scores only; freshness, blind access, and accepted-source provenance require independent audit',
        'notes_policy': 'Notes never alter classifications; unregistered phrases are unknown and wrong',
        'targeted_retry_available': round_number == 0 and bool(targets),
        'misread_looks': [{'look': look, 'facings': ['S', 'E'], 'fields': sorted(fields)} for look, fields in sorted(targets.items())],
        'retry_rule': 'One targeted round only; generation try ceiling must be independently checked; no third review round'}
    (output / 'scores.json').write_text(json.dumps(result, indent=2) + '\n')


if __name__ == '__main__':
    if len(sys.argv) not in (5, 6):
        raise SystemExit('Usage: python3 -m art.whole_figure_lab.prod1_score KEY REVIEW_A REVIEW_B OUTPUT [ROUND_0_OR_1]')
    score(Path(sys.argv[1]), (Path(sys.argv[2]), Path(sys.argv[3])), Path(sys.argv[4]), int(sys.argv[5]) if len(sys.argv) == 6 else 0)
