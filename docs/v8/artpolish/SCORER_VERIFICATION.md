# Blind score implementation verification

`art/pipeline_polish/blind_score.py` applies the unchanged bars separately to one
reviewer's complete packet. Missing/duplicate trials reject input; item asset IDs
must equal the entire22-image inventory. Verbatim answer interpretation remains
an explicit separately documented semantic judgement, not this code's claim.

Verification run before any blind review:

- `python -m pytest art/pipeline_polish/test_blind_score.py -q`: 7 passed.
- `uvx --offline ruff check` on both scorer files: passed.
- `uvx --offline basedpyright ... --pythonpath
  /Users/rexxa/work/arttrial2-data/.venv/bin/python`: zero errors/warnings.
- Three isolated in-memory mutations each produced assertion failures (exit1):
  animal2× pose bar16→15, portrait bar30→29, omission of mandatory item-pair gate.
  Production files were not mutated. Logs:
  `/Users/rexxa/work/artpolish-data/verification/scorer_mutation_*.log`.

The first test run also recorded the absent scorer module before implementation;
this import failure alone is not behavioral proof. The mutation results demonstrate
that the tests reject weakened thresholds and totals that conceal pair failures.
