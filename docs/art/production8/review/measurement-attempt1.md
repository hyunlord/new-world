# Retained measurement attempt 1 diagnostic

The first invocation of `python3 tools/art_pipeline/measure_p8.py docs/art/production8/qa_config.json` exited1 before emitting the formal A1-A33 verdict. Tool session90342 supplied this traceback (transcribed from the exact tool result, not an original log file):

```
File tools/art_pipeline/measure_p8.py, line82, in main
    checks.append(cold(live, Path(config['gate'])))
File tools/art_pipeline/ingame_qa_cold_p7.py, line20, in cold
    codes = {row[0]: int(row[1]) for line in results_path.read_text().splitlines()
ValueError: invalid literal for int() with base 10: 'exit_code'
```

Cause: the preparation fixture TSV had a header; production gate.sh writes headerless test-name/exit-code/elapsed rows. The original58PASS rows and partial inherited reports were frozen externally before converting this fixture to the real gate contract. No exit code, threshold, earlier-pass implementation, or visual verdict was changed. The subsequent run uses the same captures and predicates.
