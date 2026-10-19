"""Reduce only published GIF temporal samples; preserve decoded pixels and duration.

Run after final publication: python3 tools/art_pipeline/compact_previews_p7.py RAW_LONG
Full raw cycles and all measurement inputs stay untouched.
"""
from __future__ import annotations
import gzip
import hashlib
import json
from pathlib import Path
import sys
from PIL import Image, ImageChops, ImageSequence


def frames(path: Path) -> tuple[list[Image.Image], list[int]]:
    """Decode complete composited GIF frames and their actual encoded durations."""
    with Image.open(path) as source:
        images, durations = [], []
        for frame in ImageSequence.Iterator(source):
            images.append(frame.convert('RGB'))
            durations.append(int(frame.info.get('duration', 0)))
    return images, durations


def compact(path: Path) -> dict:
    """Keep every second frame, accumulating each omitted frame's duration."""
    original, durations = frames(path)
    selected = original[::2]
    paired = [sum(durations[index:index+2]) for index in range(0,len(durations),2)]
    if not selected or any(duration <= 0 for duration in paired):
        raise ValueError(f'Missing positive frame durations: {path}')
    temporary = path.with_suffix('.compact.tmp.gif')
    try:
        selected[0].save(temporary,save_all=True,append_images=selected[1:],duration=paired,
                         loop=0,disposal=1,optimize=True)
        actual, actual_durations = frames(temporary)
        if sum(actual_durations) != sum(durations) or len(actual) != len(selected):
            raise ValueError(f'GIF duration/frame count changed unexpectedly: {path}')
        for before,after in zip(selected,actual,strict=True):
            if before.size != after.size or ImageChops.difference(before,after).getbbox() is not None:
                raise ValueError(f'GIF encoding changed selected pixels: {path}')
        record = {'original_frames':len(original),'preview_frames':len(actual),
            'duration_ms':sum(durations),'preview_fps':len(actual)*1000/sum(durations),
            'size':list(actual[0].size),'original_bytes':path.stat().st_size,
            'preview_bytes':temporary.stat().st_size,'source_sha256':hashlib.sha256(path.read_bytes()).hexdigest(),
            'preview_sha256':hashlib.sha256(temporary.read_bytes()).hexdigest(),
            'verification':'Each selected decoded RGB frame identical; total encoded duration identical.'}
        if temporary.stat().st_size >= path.stat().st_size:
            raise ValueError(f'Temporal reduction did not reduce size: {path}')
        temporary.replace(path)
        return record
    finally:
        temporary.unlink(missing_ok=True)


def main(raw: Path) -> None:
    published = Path('docs/art/production7/shots')
    records = []
    for path in sorted(published.glob('*.cold_cycle.gif')):
        sidecar = path.with_suffix('.json.gz')
        data = json.loads(gzip.decompress(sidecar.read_bytes()))
        if data.get('published_visual_temporal_reduction'):
            continue
        original = raw/path.with_suffix('.json').name
        if not original.exists():
            raise FileNotFoundError(original)
        record = compact(path)
        record['full_raw_external_path'] = str(original.resolve())
        record['published_file'] = str(path)
        record['note'] = 'Derived preview only: every second temporal sample; no spatial or selected-pixel changes. Full raw cycle remains measurement source.'
        data['published_visual_temporal_reduction'] = record
        sidecar.write_bytes(gzip.compress((json.dumps(data,ensure_ascii=False)+'\n').encode(),mtime=0))
        records.append(record)
    index = Path('docs/art/production7/compact_previews.json')
    prior = json.loads(index.read_text()) if index.exists() else []
    index.write_text(json.dumps(prior+records,indent=2)+'\n')
    print(json.dumps({'cycles':len(records),'bytes_saved':sum(row['original_bytes']-row['preview_bytes'] for row in records)}))


if __name__ == '__main__':
    main(Path(sys.argv[1]))
