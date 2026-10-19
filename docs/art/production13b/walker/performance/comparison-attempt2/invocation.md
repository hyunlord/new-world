# Calibration invocation correction

Comparison attempt1 used the inherited validator default calibration (production13), which rejected both ledgers with calibration provenance mismatch. That failed report is preserved.

Comparison attempt2 reran the same completed measurements without source edits or recapture, setting `P13_REFERENCE=/Users/rexxa/work/new-world-art-prod/docs/art/production13b/perf/calibration.json`. This canonical file was read and SHA-verified as `e240ebf3592bd25d70f673350dc0f990295b8a413cfbcc0d2e819153033520b4`, matching both capture validity ledgers. No thresholds or inherited validator code changed. Exit0; both walker performance gates pass. Identity remains a separate requirement.
