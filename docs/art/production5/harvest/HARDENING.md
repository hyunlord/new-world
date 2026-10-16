# Final approval boundary hardening

New preparation freezes the complete installer ledger, record fingerprints, ordered animation candidate fingerprints, and board/GIF file hashes before review. Finalization rechecks those bindings and original approved content before constructing an envelope. Reviews are bound by immutable bytes. Same-phase copies must retain exact approved pixels, source set, and parity. Key-specific errors survive normalization and installation validation.

P1 and P3 sidecars are explicitly post-review recovery evidence: installer/envelope/adjudication/review bytes match their committed checkpoints, and regenerated boards/GIF frames match the actual reviewed pixels. They do not claim an earlier timestamp. Mask-only report and review hashes similarly bind the committed P3 evidence.

The c6d88d baseline freezes actual native/sprite PNG and rendered region-atlas texels from Git blobs. Final A20 compares current content even when candidate/status/source labels stay unchanged, and checks approved masks on every current harvested adult and derived frame. Later approved c masks override only matching current RGBA and undergo separate frame/group review checks. Mask group reviews are reread, boards/GIF hashes rechecked, and physical sets retraced.

Verification: 274 Python tests passed, including ledger mutation, candidate-order mutation, board/GIF mutation, review/binding mutation, same-identity content changes, and the independently reproduced V_a2_05 rendered-mask mutation. The fresh reviewer confirmed the mutation fails and actual A20 passes: 2 installations, 74 bound frames, 61 changed villagers, 20 animation groups, zero errors. No atlas write was required for hardening or the zero-install P4 search.
