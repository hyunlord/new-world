# /// script
# requires-python = ">=3.11"
# dependencies = ["pillow", "numpy", "scipy", "scikit-image"]
# ///
# How to run: python -m art.pipeline5.prod2_generations prepare LOOK gap REASON
"""Reserve every new image call before dispatch; finish immutable bounded evidence."""
from __future__ import annotations

import json
import re
import shutil
import sys
from dataclasses import asdict, dataclass
from pathlib import Path
from typing import Final

from art.pipeline5.attempt_integrity import digest
from art.pipeline5.prod1_process import evaluate
from art.pipeline5.prod1_records import JsonValue, ProductionRequest
from art.pipeline5.prod1_storage import production_lock, publish

DATA: Final = Path('/Users/rexxa/work/artprod2-data')
REPO: Final = Path(__file__).resolve().parents[2]


@dataclass(frozen=True, slots=True)
class Locations:
    data: Path = DATA
    repo: Path = REPO

    @property
    def ledger(self) -> Path:
        return self.repo / 'docs/v8/artprod2/PROD2_GENERATION_LOG.json'

    @property
    def output(self) -> Path:
        return self.repo / 'art/character/route_i'


DEFAULT_LOCATIONS: Final = Locations()


def write_json(path: Path, value: dict[str, JsonValue]) -> None:
    """Atomically replace an evidence document after serializing it completely."""
    path.parent.mkdir(parents=True, exist_ok=True)
    temporary = path.with_suffix('.tmp')
    _ = temporary.write_text(json.dumps(value, indent=2) + '\n')
    _ = temporary.replace(path)


def refresh(locations: Locations) -> None:
    """Count pending and failed intents equally; never infer budget from successes."""
    records = [json.loads(path.read_text()) for path in sorted((locations.data / 'generation').glob('*/try*/intent.json'))]
    write_json(locations.ledger, {'reserved_calls': len(records), 'maximum_calls': 40,
                                'maximum_per_look': 2, 'attempts': records})


def corrected_prompt(original: str, reason: str) -> str:
    """Remove PROD1 sleep sizing instructions while retaining look/style cues."""
    start = original.find('Create one')
    original = original[start:] if start >= 0 else original
    sentences = re.split(r'(?<=[.!?])\s+', original)
    obsolete = re.compile(r'same scale|identical body scale|SAME body scale|sleep width|sleep length|sleeping figure total|critical dimension|ARMS EXTENDED|This reaches|Fingertips-to-toes|stretch.*(?:HEAD|silhouette)|extra silhouette length', re.IGNORECASE)
    retained = ' '.join(sentence for sentence in sentences if not obsolete.search(sentence))
    return retained + ('\nPROD2 correction: retain a natural straight lying body, head left, no exaggerated '
        'arm stretching. Sleep has its own uniform scale: opaque bbox long axis becomes 56px adult '
        'or 40px child. Adult sleep length52-60 height<=36; child length37-43 height<=28. '
        'Idle/walk/work/carry retain the common S-idle scale. Both facings must show the same '
        'identity, apparel and hair. I-1 supplies ink/watercolor style only. Targeted correction: ' + reason + '\n')


def review_target(scores: Path, look: str) -> bool:
    """Only the failed first round with two complete reviewers authorizes retries."""
    data = json.loads(scores.read_text())
    expected = {(f'p1_{number:02d}', zoom) for number in range(1, 13) for zoom in (1, 2)}
    reviewers = data.get('reviewers', [])
    complete = len(reviewers) == 2 and all(
        len(reviewer.get('cases', [])) == 24
        and {(case.get('id'), case.get('zoom')) for case in reviewer['cases']} == expected
        and len(reviewer.get('summaries', [])) == 2
        and {summary.get('zoom') for summary in reviewer['summaries']} == {1, 2}
        for reviewer in reviewers)
    return (data.get('schema') == 2 and data.get('round') == 0
            and data.get('all_numeric_bars_met') is False and complete
            and any(target.get('look') == look for target in data.get('misread_looks', [])))


def prepare(look: str, phase: str, reason: str, locations: Locations = DEFAULT_LOCATIONS) -> Path:
    """Reserve budget durably before the parent makes any external image call."""
    if phase not in ('gap', 'review') or not reason.strip():
        message = 'phase must be gap/review with a nonempty correction reason'
        raise ValueError(message)
    generation = locations.data / 'generation'
    with production_lock(generation):
        intents = [json.loads(path.read_text()) for path in generation.glob('*/try*/intent.json')]
        own = [entry for entry in intents if entry['look'] == look]
        attempt = len(own) + 1
        directory = generation / look / f'try{attempt}'
        request = ProductionRequest(directory / 'raw.png', look, look.split('_')[0], attempt,
                                    directory / 'prompt.txt', locations.output, locations.data / 'new-production', reason)
        if len(intents) >= 40 or len(own) >= 2:
            message = 'generation_limit: max40 total and max2 per look, including pending/failed calls'
            raise ValueError(message)
        part0 = locations.data / 'PART0_RESULTS_FULL.json'
        if not part0.is_file() or not json.loads(part0.read_text()).get('determinism_passed'):
            message = 'Part0 must complete with deterministic reslicing before generation'
            raise ValueError(message)
        accepted = locations.output / look / 'MANIFEST.json'
        if phase == 'gap' and accepted.exists():
            message = 'gap generation cannot target an accepted look'
            raise ValueError(message)
        if phase == 'review':
            scores = [*locations.data.rglob('scores.json'), *(locations.repo / 'docs/v8/artprod2').rglob('scores.json')]
            if (not accepted.exists() or any(entry['phase'] == 'review' for entry in own)
                    or not any(review_target(path, look) for path in scores)):
                message = 'review needs an accepted look named in scores and permits one review attempt per look'
                raise ValueError(message)
        frozen = json.loads((locations.data / 'FROZEN_SOURCES.json').read_text())
        originals = [entry for entry in frozen if entry['look'] == look]
        if not originals:
            message = 'look has no preserved PROD1 prompt'
            raise ValueError(message)
        original = max(originals, key=lambda entry: entry['attempt'])
        prompt = corrected_prompt(Path(original['prompt']).read_text(), reason)
        reference = locations.repo / 'art/character/candidates/I-1.png'
        references = [reference]
        if phase == 'review':
            references.insert(0, Path(json.loads(accepted.read_text())['raw']))
        reference_records: list[JsonValue] = [{'path': str(path), 'sha256': digest(path)} for path in references]
        directory.mkdir(parents=True, exist_ok=False)
        _ = request.prompt_path.write_text(prompt)
        write_json(directory / 'intent.json', {'look': look, 'attempt': attempt, 'phase': phase,
                   'reason': reason, 'status': 'reserved', 'prompt': prompt,
                   'prompt_path': str(request.prompt_path), 'prompt_sha256': digest(request.prompt_path),
                   'references': reference_records})
        refresh(locations)
        return directory / 'intent.json'


def finish(look: str, attempt: int, generated: Path, output_hint: str,
           metadata: Path | None = None, locations: Locations = DEFAULT_LOCATIONS) -> bool:
    """Copy the returned original, evaluate all ten panels, publish complete passes."""
    generation = locations.data / 'generation'
    with production_lock(generation):
        directory = generation / look / f'try{attempt}'
        intent_path = directory / 'intent.json'
        intent = json.loads(intent_path.read_text())
        request = ProductionRequest(directory / 'raw.png', look, look.split('_')[0], attempt,
                                    directory / 'prompt.txt', locations.output, locations.data / 'new-production', intent['reason'])
        if intent['status'] != 'reserved' or request.raw.exists():
            message = 'attempt already finished or raw already preserved; refuse overwrite'
            raise ValueError(message)
        if digest(request.prompt_path) != intent['prompt_sha256']:
            message = 'reserved prompt changed'
            raise ValueError(message)
        for reference in intent['references']:
            if digest(Path(reference['path'])) != reference['sha256']:
                message = 'reserved reference changed'
                raise ValueError(message)
        generated_hash = digest(generated)
        _ = shutil.copyfile(generated, request.raw)
        if digest(request.raw) != generated_hash or digest(generated) != generated_hash:
            message = 'generated source changed during preservation'
            raise ValueError(message)
        intent.update({'raw': str(request.raw), 'raw_sha256': digest(request.raw), 'output_hint': output_hint})
        if metadata is not None:
            _ = shutil.copyfile(metadata, directory / 'tool_metadata.json')
            intent['tool_metadata_sha256'] = digest(directory / 'tool_metadata.json')
        intent['status'] = 'evaluating'
        write_json(intent_path, intent)
        refresh(locations)
        request.diagnostic.mkdir(parents=True, exist_ok=False)
        result = evaluate(request)
        write_json(request.diagnostic / 'sheet_checks.json', asdict(result))
        if result.passed:
            publish(request, result)
        intent.update({'status': 'accepted' if result.passed else 'rejected',
                       'passing_panels': result.passing_panels, 'diagnostics': result.diagnostics,
                       'error': result.error, 'failures': [asdict(frame) for frame in result.frames if not frame.passed]})
        write_json(intent_path, intent)
        refresh(locations)
        return result.passed


def main() -> int:
    """Expose prepare/finish without making external generation calls."""
    args = sys.argv[1:]
    if len(args) == 4 and args[0] == 'prepare':
        path = prepare(args[1], args[2], args[3])
        print(path.read_text())
        return 0
    if len(args) in (5, 6) and args[0] == 'finish':
        accepted = finish(args[1], int(args[2]), Path(args[3]), args[4], Path(args[5]) if len(args) == 6 else None)
        print(json.dumps({'accepted': accepted}))
        return 0 if accepted else 1
    print('prepare LOOK gap|review REASON | finish LOOK TRY GENERATED_PATH OUTPUT_HINT [METADATA_JSON]')
    return 2


if __name__ == '__main__':
    raise SystemExit(main())
