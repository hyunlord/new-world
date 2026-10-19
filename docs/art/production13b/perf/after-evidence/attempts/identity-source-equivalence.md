# Identity-to-measurement provenance qualification

Full source manifests differ:655 paths, all untracked.183 `.gd.uid` files have checkout-local generated IDs;471 `.import` files differ only in UID. All471 referenced compiled imported-resource pairs are byte-identical; all remaining import settings and Git-tracked runtime sources are identical.

The remaining difference is ignored `rust/Cargo.lock`: the measurement worktree retains its earlier resolved dependency versions. It was NOT used to build the measured bridge. The measured release/debug library is a verified byte-for-byte copy of the root release build at f5c8951e, SHA25630ed97544d8fab39146c612c0cf87cd4b59245f427bfba5896fe188e9c6ad042, which is the same library used in merged identity. Both lockfiles and their hashes are preserved here. No source or build changes were made during measurement.

Do not claim the two complete manifests are equal. Claim only exact tracked runtime source, imported resource bytes and loaded bridge equivalence, plus stable measurement-local full snapshots. Detailed655-path evidence is identity-source-manifest-differences.json. No additional identity/performance acceptance is inferred from this audit alone.
