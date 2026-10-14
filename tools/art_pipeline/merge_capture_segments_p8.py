"""Merge independently captured GOD segments without changing evidence bytes.

CLI requires all four completed segment manifests and empty destination directories.
--self-test exercises synthetic evidence only; it never touches production captures.
"""
from __future__ import annotations

import argparse
from dataclasses import dataclass
import gzip
import hashlib
import json
from pathlib import Path
import shutil
import tempfile
from typing import TypeAlias, cast

Json: TypeAlias = None | bool | int | float | str | list['Json'] | dict[str, 'Json']
Record: TypeAlias = dict[str, Json]


@dataclass(frozen=True)
class Segment:
    label: str
    raw: Path
    published: Path


def digest(path: Path) -> str:
    return hashlib.sha256(path.read_bytes()).hexdigest()


def record(value: Json) -> Record:
    if not isinstance(value, dict):
        raise ValueError('expected JSON object')
    return value


def rows(value: Json) -> list[Record]:
    if not isinstance(value, list):
        raise ValueError('expected JSON array')
    return [record(item) for item in value]


def read(path: Path) -> Record:
    return record(cast(Json, json.loads(path.read_text())))


def local(directory: Path, name: Json) -> Path:
    if not isinstance(name, str) or Path(name).name != name or name in {'', '.', '..'}:
        raise ValueError(f'nonlocal evidence name: {name}')
    path = directory / name
    if not path.is_file() or path.is_symlink():
        raise ValueError(f'missing or symlink evidence: {path}')
    return path


def merge(segments: list[Segment], raw_out: Path, published_out: Path) -> Record:
    if not segments or len({s.label for s in segments}) != len(segments):
        raise ValueError('empty or repeated segment labels')
    for output in (raw_out, published_out):
        if output.exists() and (not output.is_dir() or any(output.iterdir())):
            raise ValueError(f'output must be empty: {output}')
    destinations = {raw_out.resolve(), published_out.resolve()}
    if len(destinations) != 2 or any(p.resolve() in destinations for s in segments for p in (s.raw, s.published)):
        raise ValueError('source and output directories must differ')
    copies: dict[Path, tuple[Path, str]] = {}
    sources: list[Json] = []
    raw_rows: list[Json] = []
    published_rows: list[Json] = []
    casts: list[Json] = []
    common_sha: str | None = None
    seen_stems: set[str] = set()
    run_ids: set[str] = set()

    def schedule(source: Path, target: Path) -> None:
        if target in copies:
            raise ValueError(f'duplicate destination: {target.name}')
        copies[target] = (source, digest(source))

    for segment in segments:
        raw_manifest = local(segment.raw, 'manifest.json')
        pub_manifest = local(segment.published, 'manifest_art_god.json')
        raw = read(raw_manifest)
        pub = read(pub_manifest)
        sha = raw.get('git_sha')
        run_id = raw.get('run_id')
        if not isinstance(sha, str) or len(sha) != 40 or pub.get('git_sha') != sha:
            raise ValueError('missing or conflicting source git SHA')
        if common_sha is not None and sha != common_sha:
            raise ValueError('conflicting segment git SHA')
        common_sha = sha
        if not isinstance(run_id, str) or not run_id or run_id in run_ids or pub.get('run_id') != run_id:
            raise ValueError('missing, repeated or conflicting source run ID')
        run_ids.add(run_id)
        if raw.get('plan') != 'art_god' or pub.get('plan') != 'art_god':
            raise ValueError('expected art_god manifests')
        captures = rows(raw.get('captures'))
        published = rows(pub.get('captures'))
        if not captures or len(captures) != len(published):
            raise ValueError('empty or mismatched capture manifests')
        for capture, preview in zip(captures, published, strict=True):
            image = local(segment.raw, capture.get('file'))
            entities = local(segment.raw, capture.get('entities'))
            if image.stem in seen_stems:
                raise ValueError(f'duplicate capture stem: {image.stem}')
            seen_stems.add(image.stem)
            if preview.get('file') != capture.get('file'):
                raise ValueError('raw/published capture order differs')
            for item in (capture, preview):
                if item.get('raw_sha256') != digest(image) or item.get('raw_entities_sha256') != digest(entities):
                    raise ValueError(f'source SHA mismatch: {image.stem}')
                if item.get('run_id') != run_id or 'world_epoch' not in item:
                    raise ValueError('capture lost original run/world identity')
            zipped = local(segment.published, preview.get('entities'))
            if json.loads(gzip.decompress(zipped.read_bytes())) != json.loads(entities.read_bytes()):
                raise ValueError('published sidecar differs semantically from raw')
            local(segment.published, preview.get('published_file'))
            raw_rows.append(capture)
            published_rows.append(preview)
        for intervention in rows(raw.get('intervention_casts')):
            if intervention.get('run_id') != run_id or 'world_epoch' not in intervention:
                raise ValueError('cast lost original run/world identity')
            casts.append(intervention)
        if raw.get('intervention_casts') != pub.get('intervention_casts'):
            raise ValueError('raw/published cast history mismatch')
        inventory: Record = {}
        for directory, destination in ((segment.raw, raw_out), (segment.published, published_out)):
            for source in sorted(directory.iterdir()):
                if not source.is_file() or source.is_symlink():
                    raise ValueError(f'expected flat evidence files: {source}')
                target = destination / source.name
                if source.name == 'manifest.json' or source.name.startswith('manifest_'):
                    target = destination / 'source_runs' / segment.label / source.name
                schedule(source, target)
                inventory[str(source.resolve())] = digest(source)
        sources.append({'label': segment.label, 'run_id': run_id, 'git_sha': sha,
                        'raw_manifest': str(raw_manifest.resolve()), 'raw_manifest_sha256': digest(raw_manifest),
                        'published_manifest': str(pub_manifest.resolve()), 'published_manifest_sha256': digest(pub_manifest),
                        'source_file_sha256': inventory})
    combined: Record = {'schema_version': 3, 'plan': 'art_god', 'multi_run': True,
                        'git_sha': common_sha, 'source_runs': sources, 'intervention_casts': casts,
                        'captures': raw_rows, 'provenance': 'Independent processes; row run_id/world_epoch preserved unchanged.'}
    # Finish every validation before creating outputs. Exclusive writes never overwrite.
    for target, (source, expected) in copies.items():
        target.parent.mkdir(parents=True, exist_ok=True)
        with source.open('rb') as src, target.open('xb') as dst:
            shutil.copyfileobj(src, dst)
        if digest(target) != expected:
            raise ValueError(f'copy SHA mismatch: {target}')
    for target, captures in ((raw_out / 'manifest.json', raw_rows), (published_out / 'manifest_art_god.json', published_rows)):
        result = dict(combined)
        result['captures'] = captures
        with target.open('x') as stream:
            json.dump(result, stream, indent=2)
            stream.write('\n')
    return combined


def self_test() -> None:
    with tempfile.TemporaryDirectory() as temporary:
        base = Path(temporary)
        segments: list[Segment] = []
        for label in ('live', 'society', 'p', 'g'):
            raw, pub = base / label / 'raw', base / label / 'pub'
            raw.mkdir(parents=True)
            pub.mkdir()
            image, entity = raw / f'{label}.png', raw / f'{label}.entities.json'
            image.write_bytes(b'unchanged raw image ' + label.encode())
            entity.write_text('{"run_id":"' + label + '","world_epoch":1}')
            (pub / f'{label}.jpg').write_bytes(b'unchanged JPEG')
            (pub / f'{entity.name}.gz').write_bytes(gzip.compress(entity.read_bytes(), mtime=0))
            capture: Record = {'file': image.name, 'entities': entity.name, 'published_file': f'{label}.jpg', 'run_id': label, 'world_epoch': 1, 'raw_sha256': digest(image), 'raw_entities_sha256': digest(entity)}
            manifest: Record = {'git_sha': 'a' * 40, 'run_id': label, 'plan': 'art_god', 'captures': [capture], 'intervention_casts': [{'run_id': label, 'world_epoch': 1}]}
            (raw / 'manifest.json').write_text(json.dumps(manifest))
            published = json.loads(json.dumps(manifest))
            published['captures'][0]['entities'] += '.gz'
            (pub / 'manifest_art_god.json').write_text(json.dumps(published))
            segments.append(Segment(label, raw, pub))
        result = merge(segments, base / 'out', base / 'published')
        assert result['multi_run'] is True and len(rows(result['captures'])) == 4
        assert (base / 'out/live.png').read_bytes() == (segments[0].raw / 'live.png').read_bytes()
        assert (base / 'published/live.entities.json.gz').read_bytes() == (segments[0].published / 'live.entities.json.gz').read_bytes()
        def rejected(items: list[Segment], expected: str) -> None:
            try:
                merge(items, base / 'bad', base / 'badpub')
            except ValueError as error:
                assert expected in str(error), str(error)
            else:
                raise AssertionError('invalid merge accepted')
            assert not (base / 'bad').exists()
        rejected(segments + [segments[0]], 'repeated segment')
        last_raw = segments[3].raw / 'manifest.json'
        last_pub = segments[3].published / 'manifest_art_god.json'
        saved_raw, saved_pub = last_raw.read_bytes(), last_pub.read_bytes()
        duplicate = read(last_raw)
        duplicate_preview = read(last_pub)
        rows(duplicate['captures'])[0]['file'] = 'live.png'
        rows(duplicate_preview['captures'])[0]['file'] = 'live.png'
        (segments[3].raw / 'live.png').write_bytes(b'duplicate stem')
        last_raw.write_text(json.dumps(duplicate))
        last_pub.write_text(json.dumps(duplicate_preview))
        rejected(segments, 'duplicate capture stem')
        last_raw.write_bytes(saved_raw)
        last_pub.write_bytes(saved_pub)
        (segments[3].raw / 'live.png').unlink()
        for path in (last_raw, last_pub):
            changed = read(path)
            changed['git_sha'] = 'b' * 40
            path.write_text(json.dumps(changed))
        rejected(segments, 'conflicting segment git SHA')
        last_raw.write_bytes(saved_raw)
        last_pub.write_bytes(saved_pub)
        try:
            merge(segments, base / 'out', base / 'published')
        except ValueError as error:
            assert 'output must be empty' in str(error)
        else:
            raise AssertionError('existing output overwritten')
        manifest = read(segments[3].raw / 'manifest.json')
        manifest['git_sha'] = 'b' * 40
        (segments[3].raw / 'manifest.json').write_text(json.dumps(manifest))
        rejected(segments, 'conflicting source git SHA')
        manifest['git_sha'] = 'a' * 40
        (segments[3].raw / 'manifest.json').write_text(json.dumps(manifest))
        (segments[0].raw / 'live.png').write_bytes(b'tampered')
        rejected(segments, 'source SHA mismatch')
        print('PASS merge_capture_segments_p8: bytes, identities, duplicate, conflicting SHA, tamper')


def main() -> None:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--self-test', action='store_true')
    parser.add_argument('--evidence-root', type=Path, default=Path('/Users/rexxa/work/art-production8-evidence'))
    parser.add_argument('--docs-root', type=Path, default=Path('docs/art/production8'))
    args = parser.parse_args()
    if args.self_test:
        self_test()
        return
    segments = [Segment(label, args.evidence_root / f'final-god-{label}/shots/art_god', args.docs_root / f'god-{label}') for label in ('live', 'society', 'p', 'g')]
    result = merge(segments, args.evidence_root / 'final/shots/art_god', args.docs_root / 'god')
    print(f"Merged {len(rows(result['captures']))} captures from four source runs")


if __name__ == '__main__':
    main()
