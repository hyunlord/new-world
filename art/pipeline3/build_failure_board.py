# /// script
# requires-python = ">=3.11"
# dependencies = ["pillow", "numpy"]
# ///
# How to run: python3 -m art.pipeline3.build_failure_board
"""Index every generated attempt and render nonselected evidence without taste scores."""
import json
from dataclasses import dataclass
from pathlib import Path
from textwrap import wrap
from typing import Final

from PIL import Image, ImageDraw

from .check_layer import Finding, check_layer
from .failure_marks import marked_preview
from .layer_schema import load_layer
from .set_checks import combinations

RAW: Final = Path('/Users/rexxa/work/arttrial3-data/route_a')
ACCEPTED: Final = Path('art/character/route_a')
BOARD: Final = Path('art/character/boards/failures_a.png')
INDEX: Final = Path('docs/v8/arttrial3/FAILURE_INDEX.md')


@dataclass(frozen=True, slots=True)
class Attempt:
    """A source attempt with its selected check evidence, never inferred taste."""
    folder: Path
    selected: bool
    findings: tuple[Finding, ...]
    evidence: str
    sidecar: Path | None


def read_findings(path: Path) -> tuple[Finding, ...]:
    """Flatten recorded failed result records, retaining exact rule and detail."""
    raw = json.loads(path.read_text())
    records = raw if isinstance(raw, list) else [raw]
    found: list[Finding] = []
    for record in records:
        if 'layer' in record:
            records.extend([record['layer'], *record.get('fixed_upper', [])])
        if record.get('passed') is False:
            found.extend(Finding(str(f['rule']), str(f['detail'])) for f in record.get('findings', []))
    return tuple(dict.fromkeys(found))


def attempts() -> list[Attempt]:
    """Selected raw identities override historical processing failures."""
    selected: set[Path] = set()
    for path in ACCEPTED.glob('*.json'):
        data = json.loads(path.read_text())
        if data.get('body_type') == 'adult' and 'raw' in data:
            selected.add(Path(data['raw']).resolve())
    result: list[Attempt] = []
    for folder in sorted(RAW.glob('*/try*')):
        raw = folder / 'raw.png'
        if not raw.exists():
            continue
        sidecars: list[Path] = []
        for candidate in folder.glob('*.json'):
            data = json.loads(candidate.read_text())
            if isinstance(data, dict) and 'schema_version' in data and 'mask' in data:
                sidecars.append(candidate)
        sidecar = next((p for p in sidecars if p.name == 'candidate.json'), next(iter(sorted(sidecars)), None))
        checks = [folder / name for name in ('accepted_check.json', 'alpha_cleanup_check.json', 'checks_final.json', 'check.json', 'checks.json')]
        evidence = next((p for p in checks if p.exists()), None)
        findings: tuple[Finding, ...] = ()
        source = 'NO_CHECK_RECORD'
        if evidence is not None:
            findings = read_findings(evidence)
            source = str(evidence)
        elif sidecar is not None:
            findings = check_layer(load_layer(sidecar)).findings
            source = f'Current checker recomputation: {sidecar} (historical result absent)'
        cleanup = folder / 'alpha_cleanup.json'
        if cleanup.exists():
            sidecar = cleanup
        if not findings and sidecar is not None and raw.resolve() not in selected:
            layer = load_layer(sidecar)
            if layer.kind == 'top':
                peers = []
                for path in ACCEPTED.glob('*.json'):
                    data = json.loads(path.read_text())
                    if data.get('body_type') == 'adult' and data.get('kind') != 'top':
                        peers.append(load_layer(path))
                results = combinations([*peers, layer])
                findings = tuple(dict.fromkeys(f for r in results for f in r.findings))
                source += '; current combination recomputation against accepted adult peers'
        if folder.parent.name == 'top_hide_wrap_N' and folder.name == 'try3':
            findings = ()
            source += '; UNUSED: third call was already in flight; only unextracted dressed-image check ran, no final extraction'
        result.append(Attempt(folder, raw.resolve() in selected, findings, source, sidecar))
    return result


def render(records: list[Attempt]) -> None:
    """Place every nonselected attempt on a diagnostic board with exact rules."""
    rejected = [r for r in records if not r.selected]
    rows = max(1, (len(rejected) + 3) // 4)
    board = Image.new('RGB', (1024, rows * 350 + 36), '#eee9e0')
    draw = ImageDraw.Draw(board)
    draw.text((10, 10), 'A: REJECTED / UNUSED ATTEMPTS - STRUCTURAL DIAGNOSTICS ONLY', fill='black')
    for index, record in enumerate(rejected):
        x, y = index % 4 * 256, index // 4 * 350 + 36
        status = 'REJECTED' if record.findings else 'UNUSED / NOT VERIFIED'
        color = '#a50000' if record.findings else '#555555'
        preview = None
        if record.sidecar is not None:
            preview = marked_preview(load_layer(record.sidecar), list(record.findings))
        if preview is None:
            with Image.open(record.folder / 'raw.png') as source:
                preview = source.convert('RGBA')
        factor = min(220 / preview.width, 240 / preview.height)
        preview = preview.resize((round(preview.width * factor), round(preview.height * factor)), Image.Resampling.NEAREST)
        board.paste(preview, (x + (256 - preview.width) // 2, y + 33), preview)
        draw.rectangle((x + 3, y + 31, x + 252, y + 274), outline=color, width=2)
        draw.text((x + 7, y + 3), f'{record.folder.parent.name}/{record.folder.name}', fill='black')
        draw.text((x + 7, y + 16), status, fill=color)
        rules = ', '.join(dict.fromkeys(f.rule for f in record.findings)) or 'No failed final rule recorded'
        for line, text in enumerate(wrap(rules, 35)):
            draw.text((x + 7, y + 280 + line * 13), text, fill=color)
    BOARD.parent.mkdir(parents=True, exist_ok=True)
    board.save(BOARD, optimize=True)
    if BOARD.stat().st_size > 1_500_000:
        message = f'Board exceeds 1.5MB: {BOARD.stat().st_size}'
        raise RuntimeError(message)


def main() -> None:
    """Write a complete ledger and board; original attempts remain untouched."""
    records = attempts()
    render(records)
    selected = sum(r.selected for r in records)
    failed = sum(bool(r.findings) and not r.selected for r in records)
    lines = ['# Route A generated-attempt failure index', '',
             f'{len(records)} raw attempts; {selected} selected source attempts; {failed} rejected; '
             f'{len(records)-selected-failed} nonselected without failed final rules.', '',
             'Board: `art/character/boards/failures_a.png`. All nonselected sources appear, including unused passing candidates. '
             'Selected raw sources may have earlier processing failures; their current accepted source identity controls selection. '
             'Checker failures are structural evidence, not taste judgements. Missing historical check files are explicitly labeled when recomputed. '
             'No source images were changed. Red geometry marks locate recorded failures where a sidecar exists.', '',
             '| Attempt | Disposition | Rules/details | Evidence |', '|---|---|---|---|']
    for r in records:
        status = 'SELECTED' if r.selected else 'REJECTED' if r.findings else 'UNUSED / NOT VERIFIED'
        detail = '; '.join(f'{f.rule}: {f.detail}' for f in r.findings) or 'No failed rule in selected record'
        lines.append(f'| {r.folder.parent.name}/{r.folder.name} | {status} | {detail} | {r.evidence} |')
    INDEX.write_text('\n'.join(lines) + '\n')
    print(f'{len(records)} attempts; {failed} rejected; board {BOARD.stat().st_size} bytes')


if __name__ == '__main__':
    main()
