"""UI-only succession: preserve published CORE evidence, require fresh performance.

The historical P3d chain is inherited from a pinned accepted gate, not executed
in foreign worktrees. Outcomes/why/story and raw performance are recomputed by
core_gate_checks; seed-0 shadow fixtures run fresh. Historical failures persist.
"""
from __future__ import annotations

import os
from pathlib import Path
import re
import tempfile
from typing import Final

import core_p7_publication as inventory
import core_perf_check as perf
import core_shadow_freeze as freeze
import core_shadow_successor as prior

BASE: Final = '3814828c06d1a7a211b13b54368e4e0a488cc0e3'
RUNTIME: Final = '826a7ecd9e38ca3fc1fff253baa0933aee593022'
ACCEPTED_HEAD: Final = 'cb162f3c638ab9569dd7127e5dee7d38c5d3927f'
PINS: Final = {
    'gate': '9f8c07d6ef1f4a5e8367d7d473cbf5a117730788398e57a44ea8d439c14001a5',
    'terminal': '9c0b464b0971110b287347373d23c8ba8845e1c2994e2a935763b1a9df3b4b0b',
    'evaluation': '6ade751f165fafe9c8bd490629c9a098a7c9b8348ea99c9eac64b719c2029806',
    'shadow': '5a3838b9283d0b78d11f00fb696e464590e7908827575852257e53d0d53cdf78',
}
ALLOWED: Final = ('scripts/', 'scenes/', 'shaders/', 'assets/', 'localization/',
                  'tools/v8/', 'docs/', '.claude/skills/')
MUTABLE_EXISTING_TOOLS: Final = frozenset({
    'tools/v8/core_gate_checks.sh', 'tools/v8/gate.sh',
    'tools/v8/mux21_fps.gd', 'tools/v8/mux21_layout_test.gd',
    'tools/v8/mux21_play_capture.gd', 'tools/v8/mux21_profile.gd',
    'tools/v8/mux21_toast_test.gd', 'tools/v8/test_core_mind_ui.gd',
})
LIVE_SCOPES: Final = ('rust', 'scripts', 'scenes', 'shaders', 'assets',
                      'localization', 'tools/v8', 'project.godot', 'sim_bridge.gdextension')
need = inventory.need


def reference(path: Path) -> freeze.Document:
    """Create a canonical external artifact reference."""
    path = freeze.canonical(path)
    return {'path': str(path), 'sha256': freeze.digest(path), 'size': path.stat().st_size}


def read_ref(root: Path, value: freeze.JSON) -> Path:
    """Verify byte identity and size, never trust a status field alone."""
    return freeze.read_reference(root, value)


def committed_tree(root: Path, revision: str) -> dict[str, str]:
    """Bind Git blob identity and mode, including symlink text without following it."""
    result: dict[str, str] = {}
    for record in inventory.git(root, 'ls-tree', '-rz', revision).split(b'\0'):
        if not record:
            continue
        header, name = record.split(b'\t', 1)
        mode, kind, object_id = header.split()
        need(kind == b'blob' and mode in (b'100644', b'100755', b'120000'),
             'unsupported committed tree entry')
        path = os.fsdecode(name)
        need(path not in result, 'duplicate committed tree entry')
        result[path] = mode.decode('ascii') + ':' + object_id.decode('ascii')
    return result


def exact_delta(before: dict[str, str], after: dict[str, str]) -> freeze.Document:
    """Inventory the complete committed change; reject simulation/config edits."""
    result: freeze.Document = {}
    for path in sorted(set(before) | set(after)):
        if before.get(path) == after.get(path):
            continue
        need(not (path.startswith('tools/v8/') and path in before)
             or path in MUTABLE_EXISTING_TOOLS, 'frozen historical tool changed: ' + path)
        need(path.startswith(ALLOWED) or path in ('CHANGELOG.md', 'README.md'),
             'UI successor changed forbidden path: ' + path)
        result[path] = {'before_entry': before.get(path), 'after_entry': after.get(path)}
    return result


def accepted_gate(text: str, terminal: freeze.Document) -> None:
    """Require original complete45 gate and terminal success, not a partial log."""
    need(text.startswith('head ' + ACCEPTED_HEAD + '\n'), 'historical gate HEAD differs')
    steps = re.findall(r'^\s*(\d+)\s+\w+\s+exit\s+(\d+)\s+', text, re.MULTILINE)
    need(len(steps) == 45 and {int(n) for n, _ in steps} == set(range(1, 46)),
         'historical gate incomplete/duplicate')
    need(all(code == '0' for _, code in steps), 'historical gate failed')
    need(text.rstrip().endswith('status 0'), 'historical gate not terminal success')
    need(terminal.get('status') == 'EXITED' and type(terminal.get('actual_exit_code')) is int
         and terminal['actual_exit_code'] == 0 and terminal.get('wrapper_exit_code') == 0,
         'historical launcher did not finish successfully')


def same_historical_result(original: freeze.Document, observed: freeze.Document, code: int) -> None:
    """Keep exact old failures; inheritance cannot recategorize them as fresh PASS."""
    need(original == observed, 'recomputed historical raw receipt changed')
    failures = original.get('failures')
    need(type(failures) is list, 'missing historical failures')
    expected = 1 if failures else 0
    need(code == expected and original.get('status') == ('FAIL' if failures else 'PASS'),
         'historical status/exit mismatch')


def check_references(root: Path, value: freeze.JSON) -> None:
    """Pin every nested reference in a frozen receipt without executing its code."""
    match value:
        case dict() as mapping:
            if set(mapping) == {'path', 'sha256', 'size'}:
                read_ref(root, mapping)
            else:
                for item in mapping.values():
                    check_references(root, item)
        case list() as sequence:
            for item in sequence:
                check_references(root, item)
        case str() | int() | float() | bool() | None:
            return


def load_config(root: Path) -> freeze.Document:
    """Bind final source/config independently through explicit environment pins."""
    path = freeze.external(root, Path(os.environ['CORE_UI_CONFIG']))
    freeze.pin(path, os.environ['CORE_UI_CONFIG_SHA256'])
    data = freeze.load(path)
    need(set(data) == {'schema_version', 'kind', 'head', 'base', 'rust_sha256',
                      'lock_sha256', 'delta', 'history', 'fresh_session'}, 'UI config fields differ')
    freeze.version(data)
    need(data['kind'] == 'published_ui_successor' and data['base'] == BASE, 'UI base differs')
    return data


def source(root: Path, data: freeze.Document) -> str:
    """Prove committed/live runtime and all UI/tool inputs match declared HEAD."""
    head = inventory.git(root, 'rev-parse', 'HEAD').decode().strip()
    need(data['head'] == head, 'UI final HEAD changed; regenerate sealed config')
    inventory.git(root, 'merge-base', '--is-ancestor', BASE, head)
    rust = inventory.committed(root, BASE, 'rust')
    need(inventory.committed(root, RUNTIME, 'rust') == rust
         == inventory.committed(root, head, 'rust'), 'UI successor changed committed Rust')
    need(perf.source_hash() == data['rust_sha256'] == prior.source_digest(rust), 'live Rust differs')
    need(freeze.digest(root / 'rust/Cargo.lock') == data['lock_sha256'] == inventory.LOCK, 'lock differs')
    before = committed_tree(root, BASE)
    after = committed_tree(root, head)
    need(exact_delta(before, after) == data['delta'], 'undeclared committed delta')
    verify_live(root, head)
    return head


def verify_live(root: Path, head: str) -> None:
    """Reject dirty or missing committed runtime/configuration inputs."""
    for scope in LIVE_SCOPES:
        names = inventory.git(root, 'ls-files', '-z', '--cached', '--others', '--exclude-standard', '--', scope)
        actual: dict[str, str] = {}
        for name in filter(None, names.split(b'\0')):
            relative = os.fsdecode(name)
            path = root / relative
            need(path.is_file() and not path.is_symlink(), 'missing/symlink live input')
            actual[relative] = freeze.digest(path)
        need(actual == inventory.committed(root, head, scope), 'live input differs: ' + scope)


def history(root: Path, data: freeze.Document) -> tuple[freeze.Document, freeze.Document]:
    """Revalidate accepted historical pins and preserve the frozen chain metadata."""
    refs = data['history']
    need(type(refs) is dict and set(refs) == set(PINS), 'history references differ')
    paths = {name: read_ref(root, refs[name]) for name in PINS}
    for name, expected in PINS.items():
        freeze.pin(paths[name], expected)
    accepted_gate(paths['gate'].read_text(), freeze.load(paths['terminal']))
    evaluation, shadow = freeze.load(paths['evaluation']), freeze.load(paths['shadow'])
    need(evaluation['final_head'] == ACCEPTED_HEAD, 'historical evaluation HEAD differs')
    for record in (evaluation, shadow):
        check_references(root, record)
    need(evaluation['p3d_successor'] == shadow['p3d_successor'], 'historical chains differ')
    chain = freeze.load(read_ref(root, shadow['p3d_successor']))
    need(chain['candidate_commit'] == RUNTIME, 'historical runtime differs')
    check_references(root, chain)
    return evaluation, shadow


def check(root: Path, config: freeze.Document, *, evaluation: bool) -> freeze.Document:
    """Adapter for existing step44/45; legacy evidence content cannot be changed."""
    data = load_config(root)
    source(root, data)
    old_eval, old_shadow = history(root, data)
    expected = dict(old_eval if evaluation else old_shadow)
    if evaluation:
        expected['final_head'] = data['head']
    need(config == expected, 'UI config modifies inherited evidence')
    build = freeze.load(read_ref(root, freeze.load(read_ref(root, config['p3d_successor']))['build_receipt']))
    print('CORE_UI inherited published P3d chain; original foreign-worktree verifier NOT_RERUN; '
          'frozen references checked, fresh fixtures/raw outcomes follow', flush=True)
    return {'source_commit': RUNTIME, 'rust_source_sha256': data['rust_sha256'],
            'binary_sha256': build['binary_sha256'], 'policy_sha256': build['policy_sha256']}


def performance(root: Path, observed: freeze.Document, code: int, gate_dir: Path) -> list[str]:
    """Authenticate inherited failure record and require new no-waiver comparison."""
    data = load_config(root)
    source(root, data)
    old_eval, _ = history(root, data)
    original = freeze.load(read_ref(root, old_eval['perf_receipt']))
    same_historical_result(original, observed, code)
    from core_ui_successor_perf import check as check_fresh
    name = data['fresh_session']
    need(type(name) is str, 'fresh session path required')
    session = freeze.external(root, Path(name))
    need(session == gate_dir / 'bench/perf-session', 'fresh performance is not this gate session')
    gate_text = (gate_dir / 'gate.txt').read_text()
    need(gate_text.startswith('head ' + str(data['head']) + '\n')
         and re.search(r'^\s*25\s+bench\s+exit\s+0\s+', gate_text, re.MULTILINE) is not None,
         'fresh performance gate HEAD/step25 differs')
    with tempfile.TemporaryDirectory(prefix='ui-perf-recheck-') as directory:
        output = Path(directory).resolve() / 'receipt.json'
        need(check_fresh(session, output) == 0, 'fresh UI standing/C3 comparison failed')
        need(freeze.digest(output) == freeze.digest(session / 'receipt.json'),
             'fresh performance receipt changed on recomputation')
    failures = original['failures']
    need(type(failures) is list and all(type(item) is str for item in failures), 'bad failures')
    print('CORE_UI historical raw failures retained: ' + ', '.join(failures), flush=True)
    print('CORE_UI fresh published-BASE comparison PASS; no P7 exception', flush=True)
    return failures
