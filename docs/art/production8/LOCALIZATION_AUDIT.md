# Part 7 localization final audit

Status: **PASS** for the current files. Re-executed at Part7; all original source SHA256 remain identical. The full final gate remains a separate requirement. No localization/product files were changed by this audit.

- GOAL section11 required keys:8; en/ko rows:16; exact-value/occurrence/registry/format checks:80, failures:0.
- Duplicate keys:0 in both Fluent sources and both compiled JSONs and key_registry.json (JSON duplicates checked while parsing, before dictionary collapse).
- Compiled locale keys:5221 each; registry5226 total/5221 active. All5226 registry IDs round-trip to their keys. The5 retained inactive keys are the existing preserve-key-IDs behavior.
- `%+d` is preserved exactly in both daily-summary strings. Placeholder sequences match GOAL for all8keys. This audit checks strings; it does not itself claim a new runtime formatting execution.
- Numeric evidence, exact audited values, per-file SHA256 and audited HEAD: `localization_audit.json`.

| Key | Registry ID | en / ko |
|---|---:|---|
| HUD_FEED_DAY_SUMMARY_FMT | 5218 | exact / exact |
| HUD_FEED_MORE_FMT | 5220 | exact / exact |
| FOUNDING_SITE_DISTANCE_FMT | 5221 | exact / exact |
| VILLAGE_RESTLESS_CALM | 5222 | exact / exact |
| VILLAGE_RESTLESS_UNEASY | 5225 | exact / exact |
| VILLAGE_RESTLESS_READY | 5223 | exact / exact |
| VILLAGE_RESTLESS_TITLE | 5224 | exact / exact |
| HUD_FEED_GOD_CAST_FMT | 5219 | exact / exact |

## Reproduce the audit without rewriting compiled outputs

Run from the repository root. This validates current sources, compiled files, global registry alignment and duplicates; it reads the frozen GOAL table directly rather than repeating expected translations in code.

```bash
python3 - <<'PY'
from pathlib import Path
from collections import Counter
import json, re
p = Path('.')
g = (p/'docs/art/production8/GOAL.md').read_text().split('# 11. PART 7')[1].split('# 12. PART 8')[0]
expected = [tuple(x.strip() for x in line.strip('|').split('|')) for line in g.splitlines() if line.startswith('| ') and not line.startswith('| key')]
assert len(expected) == 8

def no_duplicates(items):
    assert len(items) == len(dict(items)), 'duplicate JSON key'
    return dict(items)

registry = json.loads((p/'localization/key_registry.json').read_text(), object_pairs_hook=no_duplicates)
assert len(registry['keys']) == len(registry['key_to_id']) == registry['key_count']
assert all(registry['key_to_id'][key] == i for i, key in enumerate(registry['keys']))
checks = 0
for lang, column in [('en', 1), ('ko', 2)]:
    entries = re.findall(r'^([A-Za-z][A-Za-z0-9_-]*)\s*=\s*(.*)$', (p/f'localization/fluent/{lang}/messages.ftl').read_text(), re.M)
    counts = Counter(key for key, _ in entries)
    assert all(n == 1 for n in counts.values()), 'duplicate Fluent key'
    fluent = dict(entries)
    compiled = json.loads((p/f'localization/compiled/{lang}.json').read_text(), object_pairs_hook=no_duplicates)
    for row in expected:
        key, value = row[0], row[column]
        assert fluent.get(key) == value
        assert compiled['strings'].get(key) == value
        assert counts[key] == 1
        assert registry['keys'][registry['key_to_id'][key]] == key
        assert re.findall(r'%[+]?\w', fluent[key]) == re.findall(r'%[+]?\w', value)
        checks += 5
print(f'PASS keys=8 locale_rows=16 checks={checks} duplicates=0 registry={registry["key_count"]}')
PY
```

Observed output: `PASS keys=8 locale_rows=16 checks=80 duplicates=0 registry=5226`.
