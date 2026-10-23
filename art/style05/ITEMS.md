# ART-POLISH item silhouettes — preregistered before generation

Source: `docs/v8/artpolish/POLISH_GOAL.md` Part 3. These features are fixed
before the first POLISH item image call. Canvas is 64 px; glance-reading is
tested independently at 32 px and 64 px. Numeric judge v2 remains a style gate;
neither a style pass nor a readable name constitutes owner approval.

| Kind | Distinguishing feature visible at 32 px |
|---|---|
| Log (pile) | Round cut ends facing the viewer, bark sides; large pile is a pyramid of round ends, never planks. |
| Firewood | Split wedges in a criss-cross bundle, pale split faces, no round ends. |
| StoneAxe | Grey chipped head lashed to the haft with visible binding. |
| GroundAxe | Smooth pale-green polished head with a bright highlight, slimmer haft, no binding bulge. |
| FlintBlade | Dark glossy blade with a white edge. |
| Flint nodule / pile | Black glossy lumps with white chalky crust. |
| Stone pile | Grey rounded cobbles. |
| Hide | Irregular raw hide with hair texture and darker edges. |
| CuredHide | Cream rectangle, smooth, stitched/laced holes along the edge. |
| WolfHide | Grey fur with the head and tail outline. |
| HideWrap | Folded garment with a tie cord and a one-shoulder shape. |
| FurCloak | Draped cloak with a wide fur collar, dark brown. |
| Clay | Wet red-brown slab with finger marks. |
| RawPot | Grey matte unfired pot. |
| ClayPot | Orange fired pot with a dark rim, empty or filled with water. |
| Spear | Long shaft, at least twice an axe's length, with a stone point. |
| Meat | Red cut with a bone. |

## Frozen production scope

Regenerate only `firewood`, `ground_axe`, `cured_hide`, `hide_wrap`,
`fur_cloak`, `wolf_hide`, `raw_pot`, `flint_nodule`, `flint_pile`,
`log_pile_medium`, `log_pile_large`, `flint_blade`, `clay`.
At most three POLISH attempts per identifier. Preserve an attempt for the
single permitted blind-feedback regeneration round. Preserve originals and
failed outputs in `/Users/rexxa/work/artpolish-data/items/`.

The other nine selected ENV1 images remain byte-identical: `spear`,
`clay_pot_empty`, `clay_pot_filled`, `meat`, `log_pile_small`, `stone_pile`,
`stone_axe`, `hide`, `berry_basket`. The retained spear already failed ENV1's
numeric gate; retaining its bytes does not turn that historical failure into
a pass. The same-canvas spear/axe length ratio is not newly certified by this
pass because both are outside the explicit regeneration list.

## Blind protocol

Two fresh reviewers receive opaque random IDs only, all 22 images at each
size on calm ground, without names, metadata, or answer key. Score the first
name under ENV1's exact-kind policy; alternatives do not earn credit.
At 32 px each reviewer needs at least 17/22, plus both reviewers correctly
distinguishing log/firewood, chipped/ground axe, raw/cured hide, raw/fired pot,
and recognizing hide wrap/fur cloak as garments. At 64 px each needs 20/22.
Any miss triggers one regeneration round on misread eligible kinds within
the three-attempt limit, followed by two new reviewers. Original failures
remain in the record. Hover labels and inspector names are still required
in M-ART2 regardless of these glance-reading results.

## Final repair-round preregistration (D6)

Round0 failed (32 px A9/B11, 64 px A14/B15). Under the goal's later
"one regeneration round on the misread kinds" clause, D6 extends the final
repair to three originally retained misreads: `berry_basket`,
`log_pile_small`, `stone_axe`. Their round0 bytes remain preserved. Six
unaffected ENV1 items remain byte-identical in the final cohort, including
spear. The final round covers those three plus all13 initial redesigns:
11 semantic misses and two additional numeric-only log-pile failures.

Before their first POLISH generation, the three added features are fixed:
berry basket = woven basket with a tall arched carrying handle above visible
red berries; small log pile = three parallel bark cylinders with large round
cut ends facing viewer; stone axe = broad grey chipped cutting head with
clear blade edge and thick crossed lashing on short brown haft. Other17-kind
features above remain fixed. The final-attempt selection is preregistered in
external `selection_round1.json`; it takes precedence over earlier numeric
passes so a semantic repair cannot silently revert. All attempts and failures
remain reported. No further repair round is permitted.

Retained spear's total visible length70.007px vs retained stone axe48.600px
at64px gives1.440x, already below2x; shaft length cannot exceed whole-object
length. This feature therefore fails rather than remaining uncertified.
Final axe ratios are recorded in `docs/v8/artpolish/items_length_check.json`.
