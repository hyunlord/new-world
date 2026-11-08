# 0062 — Origins gate species by climate and terrain; livestock is data-driven

## Context

The owner-approved C1 pass completes the neolithic content column in two
publications. A crop must originate in a physically harvested wild plant or
imported seeds; a tame animal must originate in a wild animal. Existing recipes,
piece construction, storage, jobs and knowledge remain the execution framework.

## Decision

Species origins describe allowed world-variety climate bands, existing terrain
classes and optional inclusive elevation bounds. Omitted origin fields admit
historical habitats. Plant and animal population paths use these predicates
without consuming stream RNG. New species populations use their own keyed draw
stream and authored population settings in the existing wildlife system.
Crop definitions resolve an optional wild source; physical seed delivery remains
mandatory and ordinary caravan goods can import seeds absent from local habitats.

Content names remain registry data. C1-a connects only supported recipe, apparel,
building, crop and storage effects. Missing tool efficiency, consumable healing,
society and record mechanics remain explicit debt. Pen construction is connected
before capture; chapter/card execution waits for C1-b's actual livestock facts.
Livestock state and its physical capture, production, breeding and slaughter
drivers are the second publication; C1-a does not claim those mechanics exist.

The authored map exceeds the previous 64-node limit. Practitioner and neighbour
knowledge masks therefore widen to 128 bits while historical node ordinals stay
unchanged. Save compatibility is verified against an actual BASE format19 file,
including the complete prior integer range, rather than inferred from a current
writer. The format version and explicit predecessor admission record this boundary.

Station houses use 4×4 footprints because the existing work tile cannot fit
beside a station inside a 3×3 wall ring. A pen's Open roof metadata does not confer
shelter. Grain storage reuses physical stockpile admission and authored food
storage lifetimes, rather than multiplying a second food clock.

## Alternatives considered

- Per-species branches and closed content enums: rejected because selected packs
  must add species and crops by data alone.
- Inventing seeds or livestock when knowledge is learned: rejected because
  knowledge cannot substitute for a physical source.
- Building tool efficiency, medical or society systems in C1-a: rejected as
  outside the single new livestock chain and the feature-first scope.
- Keeping new nodes behind an Inert-only prerequisite: rejected where the
  existing physical recipe provides a supported causal discovery path. Specific
  dependency substitutions and remaining effects are listed in C1 decisions.
- Shrinking authored station footprints despite occupied work tiles: rejected
  because previews cannot override physical placement validation.

## Consequences

Catalogue, runtime, migration, data-only extension and rendered ko/en evidence
are separate checks. Authored numbers are starting values recorded in
BALANCE_DEBT, never tuned against crash smoke. Unsupported effects remain visible
as debt. Publication requires the committed candidate's full technical gate.

## Date

2026-11-08.

## Source

Owner C1 GOAL.md, milestone14, CONTENT_BIBLE §§4.2/4.5, ADR0060/0061;
`docs/v8/c1/C1_DECISIONS.md` and `docs/v8/c1/C1_REPORT.md`.
