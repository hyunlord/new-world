# ENV1 scene readability protocol

This supplements the preregistered protocol in `ENV1_PLAN.md` before the first
scene reviewer receives an image. The original thresholds remain unchanged.

## Packet and task

Each condition has two independent fresh reviewers. Each receives four opaque
image IDs in a separately shuffled order: both renderers at both required zooms.
Only the images and neutral questions enter the packet. The key, manifests,
asset names, other responses and previous verdicts remain outside it.

For every image, the reviewer records:

1. The number of people visible in the map area, excluding HUD icons.
2. A point inside the storage building or its wall construction site, or `absent`.
3. The number of visible people carrying something.
4. Whether wolves are present and a point at each wolf, or `absent`.
5. Day or night.
6. What each of the three people marked A, B and C is doing.

Marks refer to the same simulation IDs across the two renderers where the
declared overlay changes their positions or actions. A mark labels an actor,
not its answer. Geometry-only labels on the baseline are recorded separately
from its untouched source screenshot.

The image timer begins immediately before its image-view tool call. The answer
timestamp follows recording that image's answers. A reviewer must attempt all
six tasks within 60 seconds per image and preserve any late answer as late.
These are model/tool elapsed times, including image loading and recording;
they are not human reaction times. The score table shows both visual correctness
and the separate timing result, without discarding late or failed observations.

## Frozen scoring

People count passes within ten percent of the key's visible count. Carriers
require an exact count. Storage requires a point in one registered visible
footprint. The after image preserves the native unfinished storage site as
well as the declared finished storehouse; either is a valid answer to the
neutral question. Wolves require correct presence and a point at every visible wolf;
correct absence passes when the source contains none. Day/night must be exact;
winter noon is day. Each activity receives a separate binary score, also shown
as a sum out of three.

Activity keys use the displayed posture. Native internal routines such as
crafting, repairs or tending fire are retained separately when the current
renderer shows the ordinary standing frame without an action signal. They do
not turn a correct visual answer of standing into an error. Activity answers
are evaluated by their ordinary meaning, not a narrow string
whitelist: standing/waiting/idle matches an awake idle figure; carrying/hauling
matches carry; chopping/using an axe on wood matches chop; crafting/working at
a bench matches craft; sleeping/asleep matches sleep. Merely standing near an
object does not imply interacting with it. Multiple conflicting alternatives
are ambiguous and do not pass unless the reviewer clearly selects one primary
answer. Uncertain but unambiguous answers remain scoreable, with confidence
retained. Every mapping preserves the exact answer and a short rationale.

The per-cell comparison is the sum of both reviewers for each task, zoom and
condition. Every required cell remains in the table, including mismatches and
non-comparable cells. After must be at least before in the raw displayed-scene
score for **every** cell; the
three individual activity cells are also retained. Both reviewers must count
the half-zoom LOD people within ten percent. Aggregate averages cannot hide a
failed cell. One repair and a full repeat with new reviewer pairs are permitted
as registered in `ENV1_PLAN.md`; the first round remains published.

## Registered comparability limits

The real before images use the exact three native saves and camera projection.
The after images also contain the required declared M3–M5 overlay. Therefore
this comparison tests reading two displayed scenes; it cannot isolate an art
style effect from layout, actor-action and object-content changes.

The native saves contain a Stockpile zone inside an unfinished wall perimeter,
not a completed RoomRole Storehouse. Its source-backed visible footprint is a
valid storage-area target. The after image has the explicitly declared finished
storehouse. The mismatch remains in each affected comparison record.

Native saves have no wolves; the after overlay has two. Correct native absence
is scored, but absence versus locating two animals is an unequal task. Likewise
the overlay changes seven existing actors' positions, tops and activities.
The native HUD reports total population, which can differ from the number in
the map viewport. Keys retain both values and use the visible count.

Unknown or visually unverifiable truth is `NOT_VERIFIED`, never a guessed PASS.
Per-task comparability is fixed in the key before review. Storage, carrier,
wolf and activity comparisons are `NOT_COMPARABLE` as a controlled style test;
people have a declared-position mismatch; clock class is comparable. No cell
is removed from the raw score denominator. Overall controlled non-inferiority
is `NOT_VERIFIED` even if raw displayed-scene scores satisfy every numerical
bar. Owner approval remains pending.
