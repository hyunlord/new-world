# POLISH blind protocol — frozen before first review

Each family/round uses two fresh agents with no inherited conversation. Reviewers
receive only opaque image/question IDs, neutral task instructions and the answer
format. No source filenames, key, candidate manifest, generation prompt, prior
answers or target thresholds enter their context. They open the actual images at
original size. They may not enlarge a 32px item or a 1× animal during that test.
Identity sprites are already displayed at 2×. Human perception is not measured.

## Tasks and scoring

- Animals: for all 20 frames at each of 1× and 2×, name the species and one primary
  action. Each reviewer/scale must name species correctly on at least 19/20;
  pose at least 14/20 at 1× and 16/20 at 2×. Standing/idle, walking/stepping,
  running/galloping, attacking/lunging, dead/carcass, grazing/eating grass are
  ordinary-language equivalents. Sitting is not standing; standing is not walking.
- Portraits: each of 34 portraits is beside four route I S-idle sprites, labelled
  A–D. Choose one option for the same person. Fixed distractors: same age and sex,
  distinct looks with different top or hair, selected by the packet's preregistered
  deterministic rule. Each reviewer must match at least 30/34; no averaging.
- Items: name one primary object, free response, for all 22 images at both 32px
  and 64px. ENV1 first-name semantics apply: secondary alternatives do not rescue
  the primary answer. Generic stone axe is not ground/polished axe; generic hide
  is neither cured hide nor wolf hide; generic grey pot is not explicitly unfired
  pot; stone fragments are not explicitly flint. Log pile sizes count as Log kind;
  both empty and filled fired pots count as ClayPot kind. Quantity is not scored.
  Exact >=17/22 at 32px and >=20/22 at 64px, separately for each reviewer.
  At 32px both reviewers must also correctly distinguish every log image from
  firewood, stone/ground axe, hide/cured hide, raw/fired pot; hide wrap and fur cloak
  must both be recognised as garments. Garment recognition is recorded separately
  from exact kind and cannot inflate exact-kind totals.

Every semantic mapping preserves the verbatim first answer and a short rationale.
Uncertainty does not invalidate an unambiguous answer. Conflicting primary names,
missing answers, duplicate IDs, unknown IDs, missing trials and missing reviewers
never produce a pass. Inspect all expected IDs, not only the intersection.

## Rounds, evidence and authority

Freeze packet images, key and hashes before dispatch. Keep the key outside the
reviewer's allowed paths. Save raw replies, agent identifiers and packet hashes
under `/Users/rexxa/work/artpolish-data/`; publish auditable summaries under this
directory. Publish earlier failures unchanged. If animal/item bars miss, allow
one regeneration round within each original try cap and repeat with two new
reviewers; no reroll of a completed review to improve scores.

Portrait and animal style results are diagnostic D1. Structure and blind results
remain separate; a board presence is not owner approval. Item style still gates.
The final report explicitly awaits the owner's choices regardless of these scores.
