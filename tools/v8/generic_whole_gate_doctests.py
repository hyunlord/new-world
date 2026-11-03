"""Source-reviewed zero-runnable-example groups; never infer world safety from counts."""
import hashlib
import json
from pathlib import Path
from typing import Final

from generic_gate import require, text
from generic_process import file_hash
from mux2_gate_inputs import Json, Node

EXPECTED: Final = {'sim-bridge': '78b3818ae6a643af936d3d5e1c3515846358c8bff676f194e99a453bc7012d48', 'sim-core': 'cab3e7ff8861d2753b7b1746776b0908780825c2713e1e304d3acb07820258e7', 'sim-engine': 'ff0e2d53967311c463405e8f2ac3bb410e1f15d3494a12482af3618b0f02a5c2', 'sim-systems': 'aa04ee3c93326380cc81cfcd7e5a5cfc02966b3af6e24f7eb3df213d10285b42', 'sim-test': 'd30dcf483673bc675c00fe2790165c7765b0b00437a71cb368a49e1edf60fad4'}


def source_closure(root: Path, package: str) -> dict[str, str]:
    require(package in EXPECTED, '/doctest', 'unreviewed package')
    base = root / 'rust/crates' / package
    paths = [*(base / 'src').rglob('*.rs'), base / 'Cargo.toml',
             *base.glob('build.rs'), root / 'rust/Cargo.toml', root / 'rust/Cargo.lock']
    return {path.relative_to(root).as_posix(): file_hash(path) for path in sorted(paths)}


def classify_doctests(root: Path, inputs: dict[str, str], sections: list[Json]) -> list[Json]:
    """Consume already authenticated original aggregate sections, not a new listing."""
    docs = [Node(row) for row in sections if Node(row).field('kind').text() == 'doctest']
    packages = [text(row.field('package')) for row in docs]
    require(len(packages) == len(set(packages)) and set(packages) == set(EXPECTED),
            '/doctest', 'missing, duplicate or unknown original group')
    result: list[Json] = []
    for row in docs:
        package = text(row.field('package'))
        closure = source_closure(root, package)
        for name, digest in closure.items():
            require(inputs.get(name) == digest, name, 'unbound doctest source')
        digest = hashlib.sha256(json.dumps(closure, sort_keys=True, separators=(',', ':')).encode()).hexdigest()
        unchanged = digest == EXPECTED[package]
        empty = not row.field('cases').array()
        result.append({'package': package, 'kind': 'doctest',
                       'status': 'PASS' if unchanged and empty else 'INCOMPLETE',
                       'classification': 'SOURCE_REVIEWED_NO_RUNNABLE_SNIPPETS' if unchanged and empty else 'SOURCE_SNIPPET_REVIEW_REQUIRED',
                       'source_closure_sha256': digest, 'source_files': dict(closure),
                       'actual_cases': row.field('cases').value,
                       'reason': 'reviewed prose/text diagrams and actual empty case set' if unchanged and empty
                       else 'source closure changed or actual named snippets require individual classification',
                       'ordinary_worlds': 0 if unchanged and empty else None,
                       'independent_listing': False, 'native_child_wait': False,
                       'publication_authority': False})
    return result
