# Item evidence verification

Verdict: implementation checks PASS; final numeric style and blind readability FAIL.
No game runtime or owner acceptance is claimed. Generation stopped at44calls:
37assets +7actualmodelscorecards,0toolfailures; final one repair round exhausted.

Commands from repository root (2026-10-24 KST):

```sh
PYTHONPATH=. /Users/rexxa/work/arttrial2-data/.venv/bin/python -m art.pipeline_polish.item_evidence
PYTHONPATH=. /Users/rexxa/work/arttrial2-data/.venv/bin/python -m pytest art/pipeline_polish/item_test.py -q
uvx --offline ruff check art/pipeline_polish/item_build.py art/pipeline_polish/item_evidence.py art/pipeline_polish/item_test.py
uvx --offline basedpyright --pythonpath /Users/rexxa/work/arttrial2-data/.venv/bin/python art/pipeline_polish/item_build.py art/pipeline_polish/item_evidence.py art/pipeline_polish/item_test.py
python3 /Users/rexxa/.codex/plugins/cache/sisyphuslabs/omo/4.13.0/skills/programming/scripts/python/check-no-excuse-rules.py art/pipeline_polish/item_build.py art/pipeline_polish/item_evidence.py art/pipeline_polish/item_test.py
git diff --check
```

Results: evidence command exit0;5tests passed;Ruff allchecks passed;
basedpyright0errors/0warnings/0notes;no-excuse0violations;diff check clean.
An additional scoped Ruff ALL run used120column layout, excluding copyright
boilerplate, nonsecurity deterministic RNG, CLI print, formatter comma conflict
and domain layout constants (CPY001,S311,T201,COM812,PLR2004); tests additionally
exclude S101,D103. Default repository Ruff was rerun after its import-format fix.

Regression tests cover final semantic repair selection overriding an earlier
numeric pass, refusal to overwrite dispatched packets, and independent failures
when a frozen board, answer key or source sprite changes.

`items_integrity.json` verifies44raw/toolpairs, allreference hashes, both frozen
packets and all their source hashes, retained6image identity and≤1.5MB PNG size.
Maximum newPNG1,085,460bytes. Final22sprite sources and round1boards were never
rewritten after dispatch. Initial/final design snapshots are external and match
the preregistered SHA256 stored on every asset call; versions were reconstructed
from the unchanged design prefix and checked byte-for-byte after generation.

Architectural review: item_build owns review artifact preparation; item_evidence
owns provenance verification; item_test owns boundary regression cases. Existing
JsonRecord is reused at JSON boundaries. No Any/cast/type-ignore/newdependency;
no pixel surgery or quantization. item_build is232purelines (warning band); split
measurement and presentation before adding functionality. Other files stay below200.

Final style gate11/22 (6new+5retained); all16new image-model scorecards≥4, which
does not override numeric failures. Authoritative blind results are in
`blind_items_round1.json`:32px A9/B9;64px A11/B12, mandatory pairgroups FAIL.
