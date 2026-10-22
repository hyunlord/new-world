# /// script
# requires-python = ">=3.11"
# dependencies = ["pillow", "numpy", "scipy", "scikit-image"]
# ///
# How to run: python -m art.pipeline5.sheet_slice --raw SHEET --look LOOK --age adult --facing S --attempt 1 --prompt PROMPT --judge JSON --output-root OUTPUT --diagnostics-root EXTERNAL
"""Keep every attempt diagnostic; deliver only complete passing five-pose sheets."""
from __future__ import annotations

import getopt
import json
import re
import sys
from dataclasses import asdict, dataclass
from pathlib import Path

from PIL import Image

from art.pipeline5.atomic_delivery import publish
from art.pipeline5.attempt_integrity import digest, verify_judge
from art.pipeline5.sheet_geometry import normalize_panel, split_panels
from art.pipeline5.whole_checks import ALPHA_THRESHOLD, FRAMES, check_whole


@dataclass(frozen=True, slots=True)
class SheetRequest:
    raw: Path
    look: str
    age: str
    facing: str
    attempt: int
    prompt: Path
    judge: Path
    output_root: Path
    diagnostics_root: Path

    def __post_init__(self) -> None:
        if (self.age not in ('adult', 'child') or self.facing not in ('S', 'E')
                or self.attempt not in (1, 2) or re.fullmatch(r'[A-Za-z0-9_]+', self.look) is None):
            message = 'Expected safe look slug, adult|child, S|E and attempt1|2'
            raise ValueError(message)


def process(request: SheetRequest) -> bool:
    """Never publish a partial sheet or overwrite retained attempt evidence."""
    diagnostic = request.diagnostics_root / request.look / request.facing / f'try{request.attempt}'
    diagnostic.mkdir(parents=True, exist_ok=False)
    judge = json.loads(request.judge.read_text())
    summary = {'look': request.look, 'age': request.age, 'facing': request.facing, 'attempt': request.attempt,
               'raw': str(request.raw), 'raw_sha256': digest(request.raw),
               'prompt_sha256': digest(request.prompt), 'judge': judge,
               'alpha_threshold': ALPHA_THRESHOLD, 'passed': False, 'frames': []}
    try:
        binding = verify_judge(request.judge, request.raw)
        summary['judge_binding'] = asdict(binding)
        with Image.open(request.raw) as source:
            if source.mode != 'RGBA' or source.size != (1536, 1024):
                message = f'raw_sheet: expected1536x1024 RGBA, got{source.size} {source.mode}'
                raise ValueError(message)
            panels = split_panels(source)
        idle_height = panels.idle_height
        scale = (40 if request.age == 'child' else 56)/idle_height
        summary.update({'scale': scale, 'idle_source_height': idle_height,
                        'split_columns': panels.split_columns,
                        'original_components': panels.original_components,
                        'major_components': panels.original_major_components,
                        'dust_area_limit': panels.dust_area_limit,
                        'attached_fragments': [asdict(fragment) for fragment in panels.attached_fragments],
                        'normalization_version': 4, 'panel_detection_version': 3, 'ownership': asdict(panels.ownership)})
        accepted = binding.numeric_pass
        frame_records = []
        for frame, panel, box in zip(FRAMES, panels.images, panels.boxes, strict=True):
            try:
                normalized = normalize_panel(panel, scale, frame == 'sleep')
                image = normalized.image
            except ValueError as error:
                panel.save(diagnostic / f'{frame}_rejected_source.png')
                frame_records.append({'frame': frame, 'passed': False, 'error': str(error)})
                accepted = False
                continue
            result = check_whole(image, request.age, frame)
            accepted = accepted and result.passed
            path = diagnostic / f'{frame}.png'
            image.save(path)
            metadata = {'template_version': 3, 'image': path.name, 'look': request.look,
                        'age': request.age, 'facing': request.facing, 'frame': frame,
                        'attempt': request.attempt, 'source_box': box, 'scale': scale,
                        'prompt_sha256': summary['prompt_sha256'], 'raw_sha256': summary['raw_sha256'],
                        'image_sha256': digest(path), 'judge': judge, 'checks': asdict(result),
                        'normalization': asdict(normalized.transform),
                        'diagnostics': str(diagnostic), 'split_columns': panels.split_columns,
                        'normalization_version': 4, 'panel_detection_version': 3, 'ownership': asdict(panels.ownership),
                        'attached_fragments': [asdict(fragment) for fragment in panels.attached_fragments]}
            path.with_suffix('.json').write_text(json.dumps(metadata, indent=2)+'\n')
            frame_records.append({'frame': frame, **asdict(result)})
        summary['frames'] = frame_records
        summary['passed'] = accepted
        if accepted:
            _ = (diagnostic/'sheet_checks.json').write_text(json.dumps(summary, indent=2)+'\n')
            publish(diagnostic, request.output_root/request.look/request.facing)
    except (ValueError, OSError) as error:
        summary['passed'] = False
        summary['error'] = str(error)
    (diagnostic / 'sheet_checks.json').write_text(json.dumps(summary, indent=2)+'\n')
    print(json.dumps({'passed': summary['passed'], 'diagnostics': str(diagnostic)}))
    return summary['passed'] is True


def main() -> int:
    """Parse named paths and constrained identities at the CLI boundary."""
    names = ('raw', 'look', 'age', 'facing', 'attempt', 'prompt', 'judge', 'output-root', 'diagnostics-root')
    try:
        options, rest = getopt.getopt(sys.argv[1:], '', [name+'=' for name in names])
        values = dict(options)
        if rest or set(values) != {'--'+name for name in names}:
            message = 'Required options: ' + ' '.join('--'+name for name in names)
            raise ValueError(message)
        request = SheetRequest(Path(values['--raw']), values['--look'], values['--age'], values['--facing'],
                               int(values['--attempt']), Path(values['--prompt']), Path(values['--judge']),
                               Path(values['--output-root']), Path(values['--diagnostics-root']))
        return 0 if process(request) else 1
    except (ValueError, OSError, getopt.GetoptError) as error:
        print(json.dumps({'passed': False, 'error': str(error)}))
        return 2


if __name__ == '__main__':
    raise SystemExit(main())
