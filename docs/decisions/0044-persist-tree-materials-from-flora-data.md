# ADR 0044 — Persist tree materials from versioned flora data

- Status: accepted for implementation, verification pending
- Date: 2026-10-25
- Scope: M-UX Stage B5, D44; intentional behavior change

## Context

All historical trees physically yielded oak while the renderer independently hashed oak/pine/birch. Species must be data-backed physical identity shared by felling and presentation. Existing saves and frozen B4 comparisons must remain interpretable.

## Decision

Use existing Wood MaterialId as species, a version1 flora registry in the installed content catalog, and a distinct keyed worldgen draw. Preserve exact placement and log-yield draws. Author Forest elevation0–159 weights oak600/pine250/birch150 and160–255 weights200/650/150. These are game content choices, not measured ecological distributions. Canonical name ordering and checked integer cumulative selection define exact behavior.

Persist a per-world default material and sparse standing-tree overrides in the separately serialized ContentState, format10. Missing entries use the saved scalar, never a changing catalog default. Felling resolves material before removing the tree and removes its override. Keep the existing Tree and SimResources wire layouts.

Freeze format9's old content/body and accept only the measured bundled revision; retain explicit6/7/8 adapters. Historical trees migrate to historical oak in the codec, without new random selection. Merge only missing validated embedded Wood stats into legacy material tables, preserving every saved entry. New format10 validates exact selected revision, tree membership, physical Wood, saved stats and canonical sparse representation.

Godot receives authoritative aligned species alongside tree coordinates. Standing trees, removed-tree stumps and quarter-scale canopy share that identity. Remove the independent species lottery, retaining sprite size/variant/jitter and seasonal behavior. Existing material/art fallback remains; no new images are required.

Flora extensions append membership and explicitly replace complete distributions through a required Option field; validate base first and forbid default replacement. Test restricted fixture member addition using already embedded physical pine separately from the real host's full-base distribution replacement. This does not claim arbitrary material-file support.

## Consequences

Different log materials intentionally alter stacking, construction material, crafting and downstream economy. Old saves may display oak where the old visual-only hash displayed pine/birch. Neither is behavior/visual preservation. Original baseline identity, all nine OFF recaptures with explained differences, actual old-save migration, ordinary pine chop/build/save, bridge agreement and actual rendered standing/stump/canopy/wall evidence are required. Frozen B4 executables and bars remain untouched.

## Alternatives considered

Rejected: a new content enum, Tree wire edit, or renderer-only species metadata, because these duplicate identity or complicate old saves without delivering physical consistency. Rejected: resolving implicit old trees from today's catalog default, because data updates would reinterpret stored worlds.


## Date

2026-10-25.

## Source

`docs/v8/mux/MUX_GOAL.md` B5; `docs/v8/mux/MUX_DECISIONS.md` D44; verified format9 writer `bdbd4fb2ea9c5b446648778ea7b0b2da14635049`.
