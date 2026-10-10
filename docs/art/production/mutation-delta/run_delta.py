"""Committed-target inspector geometry mutation audit; never mutate production."""
from pathlib import Path
import argparse
import fcntl
import hashlib
import importlib.util
import json
import sys
import time

parser = argparse.ArgumentParser(description=__doc__)
parser.add_argument('--head', required=True)
parser.add_argument('--output', type=Path, required=True)
args = parser.parse_args()
root = Path('/Users/rexxa/work/art-production-audit')
spec = importlib.util.spec_from_file_location('audit', root / 'tools/art_pipeline/mutation_audit.py')
audit = importlib.util.module_from_spec(spec)
sys.modules['audit'] = audit
spec.loader.exec_module(audit)
assert audit.git(root, 'rev-parse', 'HEAD') == args.head
output = args.output.resolve()
output.mkdir(parents=True, exist_ok=True)
source = 'scripts/ui/panels/agent_inspector_panel.gd'
test = 'rust/crates/sim-test/tests/harness_fix_inspector_overlap.rs'
recipes = [
 ('harness_inspector_top_offset_const_exists_and_value', 'INSPECTOR_TOP_OFFSET := 124.0', 'INSPECTOR_TOP_OFFSET := 125.0', 'top offset changes from approved geometry'),
 ('harness_inspector_ready_applies_offset_to_panel_rect', 'position = Vector2(1548, INSPECTOR_TOP_OFFSET)', 'position = Vector2(1548, 0)', 'panel no longer binds top offset'),
 ('harness_inspector_ready_applies_offset_to_panel_rect', 'size = Vector2(PANEL_WIDTH, 620)', 'size = Vector2(360, 620)', 'panel no longer binds width constant'),
 ('harness_inspector_no_overlap_invariant_cross_file', 'position = Vector2(1548, INSPECTOR_TOP_OFFSET)', 'position = Vector2(12, 836)', 'valid inspector rectangle overlaps actual bottom-left news'),
 ('harness_inspector_no_overlap_invariant_cross_file', 'INSPECTOR_TOP_OFFSET := 124.0', 'INSPECTOR_TOP_OFFSET := 80.0', 'valid inspector rectangle overlaps actual topbar'),
 ('harness_inspector_anchors_and_width_untouched_regression', 'PANEL_WIDTH := 360.0', 'PANEL_WIDTH := 361.0', 'width changes from approved geometry'),
 ('harness_inspector_anchors_and_width_untouched_regression', 'position = Vector2(1548, INSPECTOR_TOP_OFFSET)', 'position = Vector2(1549, INSPECTOR_TOP_OFFSET)', 'right edge differs from actual topbar'),
]
entries = []
for function, before, after, reason in recipes:
 text = (root / source).read_text()
 assert text.count(before) == 1
 operation = 'substitute:' + after
 ident = hashlib.sha256('\0'.join([test, function, source, before, operation]).encode()).hexdigest()[:20]
 entries.append(audit.Candidate(ident, test, function, before, source, 'eligible', reason,
                audit.functions((root / test).read_text())[function][0],
                hashlib.sha256(text.encode()).hexdigest(), 1, operation))
original_mutated = audit.mutated
def semantic_mutated(before, token, operation):
 if operation.startswith('substitute:'):
  assert before.count(token.encode()) == 1
  return before.replace(token.encode(), operation.removeprefix('substitute:').encode())
 return original_mutated(before, token, operation)
audit.mutated = semantic_mutated
with (output / 'audit.lock').open('w') as lock:
 fcntl.flock(lock, fcntl.LOCK_EX | fcntl.LOCK_NB)
 audit.recover(root, output)
 start = time.time()
 audit.run(root, entries, output)
 rows = [json.loads(line) for line in (output / 'results.jsonl').read_text().splitlines()]
 assert len(rows) == 7 and all(row['outcome'] == 'killed' for row in rows)
 assert audit.git(root, 'status', '--porcelain', '--untracked-files=no') == ''
 (output / 'summary.json').write_text(json.dumps(dict(target_head=args.head,
  previous_audit_head='0189eead423c0496bda04d93b2e2da57a3efea27',
  previous_killed=467, delta_killed=7, delta_survived=0, delta_invalid=0,
  previous_ledger_rows=336, delta_ledger_rows=7, elapsed_seconds=time.time()-start,
  worktree_clean=True), indent=2)+'\n')
 print('PASS: seven semantic geometry mutants killed')
