# Reversible entity-sidecar packing

Run only after all capture segments have finished and the GOD merge has verified their published gzip files. The utility never changes raw external captures or earlier production passes. It packs only recursive `*.entities.json.gz` files inside the supplied directory named `production8`, including baseline, verification, and segmented/combined published evidence.

```sh
python3 tools/art_pipeline/pack_evidence_p8.py pack
python3 tools/art_pipeline/pack_evidence_p8.py restore
```

`pack` creates `entities.tar.xz` and `entities.archive.json`. Every ledger entry maps the unchanged relative manifest sidecar path to an archive member and records compressed/uncompressed SHA256 and sizes. Canonical gzip files are stored as their exact decompressed JSON bytes for solid compression; if `gzip.compress(json, mtime=0)` cannot reproduce the original bytes, the original gzip is stored instead. Before any installed gzip is removed, the utility reads the completed archive, reconstructs **all** gzip files, and checks their exact bytes/SHA against the source. It also writes separately verified, read-only originals to `/Users/rexxa/work/art-production8-evidence/entity-gzip-originals`.

The default `restore` verifies the archive SHA and every member, then reconstructs sidecars in `/Users/rexxa/work/art-production8-evidence/entity-restore-verify`. To use existing tools that follow manifest sidecar paths, materialize the original relative paths in the published tree:

```sh
python3 tools/art_pipeline/pack_evidence_p8.py restore --destination docs/art/production8
```

Manifests remain unchanged: for a manifest inside `society`, its `example.entities.json.gz` maps to ledger path `society/example.entities.json.gz`. Restoration preserves original gzip bytes, JSON fields, run identities, and world epochs. Matching existing files are left alone; mismatches, symlinks, traversal, nonregular tar members, duplicate entries, or SHA failures are rejected before extraction writes. No `tar.extractall` is used.

Pack refuses existing archive, ledger, or backup destinations. If interrupted, published recovery material and/or immutable originals remain; do not delete them to force a rerun. Restoring expands the checkout beyond the committed artifact budget, so use the default external destination for audit verification. Canonical reconstruction is SHA checked against the current Python gzip implementation; a different implementation cannot silently substitute different bytes. The immutable original gzip backup remains an additional recovery path.

Verification uses only temporary fixture directories:

```sh
PYTHONPATH=tools/art_pipeline python3 -m unittest tools/art_pipeline/test_pack_evidence_p8.py
```

Tests cover canonical/noncanonical exact roundtrip, immutable originals, matching no-op restoration, overwrite conflicts, traversal, tampering, malicious tar symlink members, symlink destinations, and earlier-pass scope rejection. Real production packing is a separate authorized execution step, not part of these tests.

## Optional diagnostic GIFs and gate logs

`diagnostic-gifs-and-logs.tar.xz` preserves 13 optional cold-cycle GIFs and 219 completed gate logs as their original file bytes. `diagnostic-gifs-and-logs.archive.json` maps each original relative path/member to its SHA256 and byte count. All 232 files were read back from the archive and verified before removal; read-only original copies also remain in `/Users/rexxa/work/art-production8-evidence/auxiliary-originals-p8`. The seven required boards and root village GIFs remain directly viewable. Existing cold-cycle metadata still names the original GIF path; restore that path when playing the optional diagnostic.

```sh
python3 tools/art_pipeline/pack_evidence_p8.py restore-diagnostics --destination /Users/rexxa/work/art-production8-evidence/diagnostic-restore-verify
# To materialize original relative paths for an existing viewer instead:
python3 tools/art_pipeline/pack_evidence_p8.py restore-diagnostics --destination docs/art/production8
```

Actual external restoration verified all232 originals byte-for-byte. Later gate logs created after this archive snapshot remain direct files until explicitly archived in a later revision. Diagnostic JPEG upgrades are separately documented in `png-upgrades-diagnostics.json`; the copied PNGs are identical to their existing raw1920 sources, and the original JPEG/manifests remain read-only externally.
