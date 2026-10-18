# Predicting a God's Hand: A Benchmark for Intervention Effects in a Simulated Agent Society, with Paired Counterfactual Branches, a Sequentially Confounded Split and a Measured Headroom

[AUTHORS]

Submitted to [VENUE] (datasets and benchmarks track). Draft generated from `docs/worldmodel/paper/paper_src.md` by
`tools/worldmodel/wm5/figures.py`; every quoted result is mapped to its source file in `paper_numbers.json`.

## Abstract

{{sec:abstract}}

## 1 Introduction

Learned world models are increasingly used as simulators: a model trained on recorded trajectories is rolled forward to
answer "what happens next?" and, more ambitiously, "what would happen if we acted?" [@ha2018worldmodels; @hafner2020dreamer;
@hafner2023dreamerv3; @hafner2025dreamerv3nature; @bruce2024genie; @valevski2025gamengen]. The second question is a causal one. A model that
forecasts the factual future well can still be useless for decisions if it does not respond correctly to interventions,
and recorded trajectories rarely contain the counterfactual branch against which such responses can be checked.

We study this question in a setting where the counterfactual branch is available exactly: a deterministic agent-based
village simulation (WorldSim) in which a "god" can act on a settlement — gift or spoil food, rot wood — and in which a
recorded world can be forked at any tick into branches that share their whole past and differ only in the act (and, if
desired, in post-fork noise). This makes it possible to measure, per world, the true effect of an act on food,
population, deaths and settlement splits at one week and one month, and to ask whether a model trained on the world's
observable history predicts it.

The answer so far has been sobering. Autoregressive world models trained on the factual stream ignore the god's acts;
training on paired branches teaches the first week's effects only; and two established estimators of time-varying
treatment effects, the Counterfactual Recurrent Network (CRN) [@bica2020crn] and the Causal Transformer (CT)
[@melnychuk2022ct], did not beat "predict no effect" on either an interventional or a confounded observational split.
A reviewer could object to the last finding on three grounds: (1) the two methods were not trained as their authors train
them; (2) the observational split was too small, with a single act per world, for balancing to matter; (3) perhaps
nothing can beat "no effect", i.e. the benchmark may not discriminate between methods at all. This paper answers each.

**Contributions.**

1. **A benchmark with three splits from one frozen simulator** (Section 3): an interventional split with paired
   branches and two noise replicates per world (v2); a confounded observational split with one act per world (obs); and
   a new, three times larger observational split (obs2) in which a known, state-dependent policy acts up to five times
   per world, read sequentially from the trajectory it has already changed, with two reference rows per evaluation world
   and two noise replicates per test family. Every propensity is recorded.
2. **A measured headroom** (Section 5.2): for every split and horizon we place "predict no effect" against an effect
   floor (the other noise replicate's effect, i.e. the error of a predictor that knows the full state at the act and
   only lacks the post-fork noise) and against direct effect regressions on the observable and on the full (hidden)
   pre-act state. This tells how far from the ceiling the field's methods are, and how much of the per-world effect the
   observable state carries.
3. **A fair rematch of CRN and CT** (Sections 4–5): trained on their authors' optimizer-step budgets with their weight
   schedules on real progress, evaluated at their last checkpoint, with counts rounded at readout as the world models
   are — beside observational baselines (naive contrast, multi-cast inverse propensity weighting, matching), world
   models trained on factual and on paired data, and the direct regressions. Everything is pre-registered (W1 rule,
   world-level paired bootstrap).

{{sec:intro_findings}}

## 2 Related work

**World models and learned simulators.** Recurrent and latent-dynamics world models learn to predict an environment's
next observations from past observations and actions and are used for planning and policy learning
[@ha2018worldmodels; @hafner2020dreamer; @hafner2023dreamerv3]. Generative interactive environments and neural game
engines learn action-conditioned video models of games [@bruce2024genie; @valevski2025gamengen], and graph-network
simulators learn physical dynamics from particle trajectories [@sanchezgonzalez2020gns]. Their evaluation is mostly
factual (prediction error or downstream return); counterfactual correctness of the response to an action, measured
against the true branch, is rarely available. Our benchmark provides it for a social simulation and shows that factual
accuracy and counterfactual accuracy can diverge sharply.

**Time-varying treatment effects.** Estimating the effect of a sequence of treatments from observational trajectories
with time-varying confounding is the subject of marginal structural models and inverse probability weighting
[@robins2000msm; @hernan2020whatif], and of a line of neural estimators: recurrent marginal structural networks (RMSN)
[@lim2018rmsn], CRN with adversarially balanced representations via gradient reversal [@bica2020crn; @ganin2016dann],
G-Net based on g-computation [@li2021gnet], the Causal Transformer with counterfactual domain confusion
[@melnychuk2022ct], continuous-time variants [@seedat2022tecde], and methods for hidden confounding
[@bica2020deconfounder]. These methods are evaluated mainly on a pharmacokinetic–pharmacodynamic tumour-growth simulator
with a tunable confounding strength [@geng2017tumour] and on semi-synthetic data built from MIMIC-III
[@johnson2016mimic]. Our benchmark adds a simulator in which the dynamics are those of an agent society (thresholds,
splits, delayed reversals), the confounding comes from a god-player's policy with recorded propensities, and the truth
for every evaluation case is an exactly forked branch.

**Causal benchmarks and simulators.** Semi-synthetic benchmarks with known effects, such as IHDP [@hill2011bart] and the
ACIC data challenges [@dorie2019acic], made static effect estimators comparable; causal reinforcement-learning
environments such as CausalWorld [@ahmed2021causalworld] and the causal-confusion settings of imitation learning
[@dehaan2019causalconfusion] probe causal structure in control. We follow the same principle — simulate, so that the
counterfactual is known — for sequential interventions in a multi-agent system, and add a noise floor so that the
irreducible part of each per-world effect is measured rather than assumed.

**Agent-based social simulation.** Agent-based models have long been used to grow social phenomena from individual
rules [@epstein1996sugarscape], and recent work populates simulated towns with language-model agents
[@park2023generative]. WorldSim is a deterministic, rule-based village simulation of this kind; we use it not to model
any real society but as a controllable source of trajectories with exact counterfactual branches.

## 3 The benchmark

### 3.1 Simulator and scene

WorldSim is a deterministic village simulation written in Rust. A scene (`game_v19`) starts a few dozen agents on a
procedurally generated map with berry bushes, deadwood and water; agents forage, gather wood, keep a fire, eat from a
settlement store, age, die of starvation, dehydration, exposure, combat or old age, and give birth. A village whose food
shortfall persists through spring to autumn accumulates a hidden fission pressure and splits when it reaches a hidden
per-settlement threshold, so splits depend on weeks of history and on hidden state. A world is recorded for 120 display
days at one row per in-game hour (2,881 rows): 39 global columns (population, deaths by cause, trips, the calendar …),
14 columns per settlement (members, food and wood stores, fire …) and a few hidden tables (the winter's severity, the
fission pressure and threshold, the world's generation parameters). Every hidden column is marked in the recording;
no model or baseline in this paper reads one, except one regression that is labelled a diagnostic (Section 4.2).
The recorder is frozen (one binary, its source commit recorded in every episode), and the same arguments give
byte-identical files whatever the number of workers.

**Interventions** ("the god's hand") are applied to the home settlement at a chosen tick: `gift_food` 240,
`spoil_food` 240 (or 120), `rot_wood` 120. The recorder writes the requested and the applied amount.

**Forks.** A world can be recorded as a branch family: members share the whole simulation state up to a fork tick;
replicate 0 continues unchanged, replicate k ≥ 1 receives fresh randomness after the fork. A member with an act at or
after the fork equals its family's no-act member on every row up to the act. This yields, for every evaluation case, the
true effect of an act (the arm member minus the no-act member at the same replicate) and a second, independent
realization of it (the other replicate).

### 3.2 Tasks, cells and metrics

A **case** is a (world, reference row c, act). The history up to row c is observed; the act is applied at c; the
predictor reports the effect of the act on four outcomes — the settlement food store, the number alive, cumulative
deaths, the number of living settlements — at c + 7 days and c + 30 days. The split-day shift is reported but not scored.
Each act × outcome pair is a **cell** (12 per horizon). A cell's error is the mean absolute error of the predicted effect
divided by a fixed per-cell scale (the train standard deviation of that effect on v2, floored by the pooled standard
deviation for the observational splits so that a nearly constant cell cannot dominate); the **score** of a predictor at
a horizon is the mean over the 12 cells. Comparisons use a world-level paired bootstrap (B = 2,000, all cases of a world
resampled together) and the pre-registered **W1 rule**: KEEP if the model's seed mean is at most 0.8 × the baseline,
every seed is below the baseline and the 95 % interval of the difference lies below 0; WEAK if only the interval lies
below 0; TIE if it contains 0; NONE if it lies above 0. A single seed reaches WEAK at most.

### 3.3 Three splits

**v2 (interventional).** 560 worlds (400 train, 80 val, 80 test). Every world is forked at day 11 into two replicates,
and every arm is cast in both at a rule-fixed time: `gf240E` (gift 240) and `rw120E` (rot 120) on the eve of the first
winter, `sp240L` / `sp120L` (spoil) on the first morning the store falls below a line, when it exists. Assignment does not
depend on the state beyond these rules. Test: 604 cases (both replicates).

**obs (confounded, one act per world).** 560 further worlds. A god-player reads the factual trajectory at 06:00 of days
8–60 and acts at most once: with food per person in the lowest train tercile a gift is 15 times as likely as a spoil,
in the highest tercile the reverse, and rot is three times as likely in the highest wood tercile (daily probabilities
0.002–0.030). Training sees one observed episode per world; each val / test world has a family with the no-act member
and all three acts at its act row (or day 30 if it never acted). Test: 240 cases.

**obs2 (confounded, sequential, multi-act; new).** 1,640 worlds (1,200 train, 200 val, 240 test). The same tercile
structure with every daily probability doubled (so the confounding ratios are unchanged), up to five acts per world,
and a refractory period: after an act on day d the next decision is on day d + 8 or later, so no second act falls in the
week after an act. The policy reads the trajectory **already changed by its earlier acts**: the data are recorded in
rounds — round 0 the factual worlds; in round k every world that acted in round k − 1 is re-recorded with all its acts
so far, and the policy continues reading from the row after its last act — until a round adds no act. Replaying the
policy on a world's final (observed) episode reproduces every round's decisions, so the propensity of every decision row
is known. Every val / test world has two **reference rows**: c1 (its first act, or day 30) and c2 (its second act, or
c1 + 14 days). For each, a `pre` schedule (the world's acts before c) and three act schedules (`pre` plus the act at c)
are recorded as a family forked at c; test families have two replicates. {{sec:obs2_counts}}

### 3.4 The effect floor and the headroom ladder

For every test case with two replicates, the **floor** predicts the case's effect by the other replicate's true effect
(on v2 the family's other replicate; on obs2 the r0 case by its r1 twin). This is the error of a predictor that knows the
world's full state at the act and lacks only the post-fork noise. For symmetric noise its mean absolute error is about
√2 times the irreducible one, so it is a conservative stand-in for the ceiling: a perfect predictor could score below it,
never a predictor that lacks the state. The **ladder** for a split and horizon is: floor ← `ridge_priv` (a regression on
the full pre-act state, including hidden fields) ← `ridge_obs` (the same on the observable state) ← `zero` (no effect).
The gaps say how much headroom exists, how much of it the observable state carries linearly, and how much lies in hidden
state.

### 3.5 Pre-registration and leakage controls

Each study fixed its rules before any test number existed: the obs2 policy, roles, seeds and reference-row rule before
the first treated episode was recorded; the runs, budgets, checkpoint rules, cases, scales, floors and decision rules
before any test evaluation. Models see only observable inputs (74 action-aware or 67 action-blind columns of the hourly
table); training and model selection on the observational splits see only each train / val world's observed episode;
val families are used only to choose among the observational baselines; test families only in evaluation; propensities
are used by inverse probability weighting and analyses, never as model inputs.

## 4 Methods evaluated

### 4.1 Baselines

`zero` predicts no effect. On v2, `arm_mean` predicts the act's mean train effect and `an5_arm` the mean effect of the
same act in the five most similar train worlds (24-row window of eight observable variables before the act). On the
observational splits, from the observed train episodes and the recorded propensities only: `naive` (treated minus
control mean change over the risk set of decision rows); `ipw`, the Hájek-normalized inverse-propensity contrast
[@robins2000msm; @hernan2020whatif] — on obs2 with multi-act weights: a unit treated by act a at row c with no other act
in (c, c + h) has weight 1 / (p_a(c) ∏_{j ∈ (c, c+h)} (1 − p_tot(j))), a control (no act in [c, c + h)) weight
1 / ∏_{j ∈ [c, c+h)} (1 − p_tot(j)), so the post-act survival of the treated is weighted as well (a unit test shows that
dropping that factor biases the estimate); and `match5` (five nearest treated and five nearest controls). The
observational baseline of each horizon is chosen on the val families.

### 4.2 Direct effect regressions

`ridge_obs` fits one ridge regression per cell on the v2 train families (both replicates, every case): the target is the
true effect divided by the cell's scale, the features are the 37 observable state variables at c, their means over the
day and the week before c, the 30 calendar features at c, and (obs2 only) the number of earlier acts per kind and the
days since the last act; the penalty is chosen per cell on the v2 val families. It is applied unchanged to the v2 and
obs2 tests. `ridge_priv` adds every hidden field at c (winter severity, the hidden latent table, fission pressure and
threshold, the world's generation parameters); it is a **diagnostic** of how much a linear map of the full pre-act state
predicts and never enters a method ranking. Both regressions are trained on counterfactual labels of other worlds of the
same game, as is the paired world model below.

### 4.3 World models

**M1b** is an action-aware transformer over the hourly table (7-day context) trained to forecast the next hour's state
from scratch; its effect prediction is the difference between two autoregressive rollouts from the same history, one
with the act and one without, with counts rounded at readout when validation prefers it. We evaluate M1b trained on v2's
factual and arm members (`wm2_m1bact`, three seeds), on obs (`m1b_obs`, two seeds) and on obs2 (`m1b_obs2`, new), and
`wm3_full`, M1b fine-tuned on v2's paired branches with a paired-difference loss and a hazard head (three seeds).

### 4.4 CRN and CT as their authors train them

We use the authors' public implementation of both methods [@melnychuk2022ct] (its plain-PyTorch modules, unchanged) at
daily resolution: a token per day carries the 37 state variables (predicted as normalized daily changes and fed back),
the calendar as covariates, and a four-class treatment plus amount. CRN [@bica2020crn] is an LSTM encoder whose
representation is made treatment-invariant by a gradient-reversal classifier, followed by a decoder initialized from it
for multi-step prediction; CT [@melnychuk2022ct] is a three-stream transformer trained with counterfactual domain
confusion and EMA weights. Configurations are those selected by a pre-registered grid on the obs split. In contrast with
the earlier study, each stage now trains for **the authors' number of optimizer steps** (CRN encoder 15,700, decoder
53,800; CT 11,850), unless a 300-s cost check on the actual data predicted more than the stage's budget; the balancing
weight α follows the authors' schedule **on real progress** (so a finished run ends at α = α_max); the **last**
checkpoint is primary, as in the authors' code (CT's EMA weights), with the lowest-validation-loss checkpoint reported as
secondary; and **counts are rounded at readout** (food, alive, cumulative deaths), as for M1. All adaptations are listed
in Appendix A.

## 5 Results

{{sec:results}}

## 6 Analysis

{{sec:analysis}}

## 7 Limitations and broader impact

**Limitations.** One simulator and one scene; single acts of three kinds at one amount each; the observational policies
are synthetic, with known propensities and a refractory period that a real decision-maker would not have; obs2's
evaluation cases sit at the policy's act rows (and their fixed fallbacks), so they inherit its timing; the floor is the
other replicate's effect, which over-states the irreducible error by about √2 for symmetric noise; the direct
regressions are linear and trained on v2's counterfactual labels (another game split of the same simulator, not
another simulator); CRN and CT run at daily resolution with one configuration each, chosen on the smaller obs split;
the world models run at hourly resolution; GPU-level numerical reproducibility of the transformer rollouts is not
guaranteed (one re-scored run differed at the 1e-4 relative level); two seeds per trained method. Claims are limited to
the pre-registered rules; comparisons labelled post hoc are descriptive.

**Broader impact.** The benchmark is built on a toy simulation of a village and makes no claim about real societies or
real interventions. Its intended use is to test whether methods that learn dynamics from trajectories respond correctly
to interventions before they are trusted as simulators. A risk is over-reading simulated results: a method that does well
here is not thereby validated for policy or medical use, and one that does poorly may do well where its assumptions
hold. We release no personal data; the simulator and its recordings contain none.

## Data and code release

[OWNER: release plan]

## References

See `references.bib` (every entry verified; `REFERENCES_CHECK.md`).

## Appendix A. CRN and CT: deviations from the papers (D1–D24)

Source: the authors' implementation (`github.com/Valentyn1997/CausalTransformer`, commit `c49a24fa`, MIT licence),
plain-PyTorch modules vendored unchanged (`VariationalLSTM`, gradient reversal, `BRTreatmentOutcomeHead`,
`TransformerMultiInputBlock`, relative positional encodings); training loops, data pipeline, rollouts and checkpoint
bookkeeping are ours (no Lightning, Hydra or MLflow). The full list with reasons is in `docs/worldmodel/wm4/METHODS.md`
(D1–D15) and `docs/worldmodel/wm5/METHODS.md` (D16–D24).

| id | papers / authors' code | here |
|---|---|---|
| D1 | one step = one day (tumour simulator) | one token per day: the hourly episode sampled every 24 rows at a clock phase (06:00 on the observational splits; uniform per sequence in training and the act's phase at test on v2); the act falls in exactly one token |
| D2 | levels | tallies (deaths by cause, births) as daily sums; cumulative deaths recovered by summing predicted daily deaths |
| D3 | 1-d outcome | 37 state variables, predicted as normalized daily changes and fed back (clamped ≥ 0; flags ≤ 1); MSE over all 37 |
| D4 | vitals / static features | the calendar as time-varying covariates (history only; CT masks them after the last observed token, as the authors mask vitals); no static features |
| D5 | 4 treatment classes | 4 classes (none, gift, spoil, rot) plus the amount; cross-entropy without the deprecated reduction flag |
| D6 | τ ≤ 5 (CT), ≤ 10 (CRN) | 7- and 30-token horizons; CRN decoder trained on 29-step windows initialized from the encoder representation |
| D7 / D16 | authors' epochs on 10,000 sequences | the authors' number of optimizer steps (CRN 15,700 + 53,800, CT 11,850), reduced only if a 300-s cost check predicts more than the pre-registered budget; a 1.25 × budget safety cap |
| D8 / D18 | last epoch | primary: the last step (CT: EMA weights); secondary: the lowest validation factual-rollout loss |
| D9 | EMA weights swapped inside the forward pass | EMA (β 0.99) updated with the authors' rule but used for evaluation and the kept checkpoint only |
| D10 | small hidden sizes for the 1-d simulator | CRN encoder hidden 128, CT d_model 128, 2 layers, 4 heads; Adam, no weight decay, no clipping, no schedule |
| D11 / D22 | — | one pre-registered 4-configuration grid per method on the obs split (validation factual loss only); chosen: CRN BR 64 / FC 128, decoder BR 64 / FC 32, encoder dropout 0.1; CT lr 1e-3, BR 64, FC 128 |
| D12 / D21 | — | training data: v2 — every train-family member; obs / obs2 — one observed episode per train world |
| D13 / D20 | — | prediction contract: arm rollout − no-act rollout from the same history (the case's `pre` member); counts (food, alive, cumulative deaths) rounded at readout as for M1, unrounded reported as secondary |
| D17 | α ramp per epoch over the planned epochs | α = α_max (2 / (1 + e^(−10 p)) − 1) with p = step / planned steps, set before every step |
| D19 | per-epoch validation | at most 15 validation evaluations per stage |
| D24 | — | M1 effect rollouts zero the action history as in earlier studies; on obs2 the previous act is ≥ 8 days before c, outside M1's 7-day window |


## Appendix B. Hyper-parameters, budgets and training reached

{{sec:appendix_runs}}

## Appendix C. The observational policies

**obs (one act).** Decision rows: 06:00 of display days 8–60 (53 rows). State: food per person f = Σ store / max(1,
alive) and wood per person w, observable columns only. Terciles of f (q_f1 = 8.08, q_f2 = 12.1) and the 67th percentile
of w (q_w2 = 3.11) over the decision rows of the train worlds' factual episodes, frozen before any treated episode.
Daily probabilities: gift 0.030 / 0.008 / 0.002 and spoil 0.002 / 0.008 / 0.030 for the f terciles low / middle / high;
rot 0.012 if w ≥ q_w2 else 0.004. Draw u from a seeded generator per (world, day); gift if u < p_g, spoil if u < p_g + p_s,
rot if u < p_g + p_s + p_r. At most one act; no decision on a row with no one alive or no settlement.

**obs2 (sequential, up to five acts).** The same terciles (imported, not recomputed) and every probability doubled
(gift 0.060 / 0.016 / 0.004, spoil 0.004 / 0.016 / 0.060, rot 0.024 / 0.008); a separate random stream per (world, day);
at most five acts; after an act on day d the rows of days d + 1 … d + 7 have probability 0 (refractory). Each decision
reads the episode recorded with every earlier act of the world (rounds, Section 3.3). The propensity record lists, for
every world, every decision row's status (decision, refractory, cap, no decision) and (p_g, p_s, p_r), and every act with
the round that produced it.

**Reference rows and families (obs2).** c1 = first act row or day 30; c2 = second act row or c1 + 14 days; `pre` = the
acts before c; arms = `pre` + the act at c; nothing after c; val families one replicate, test families two.

## Appendix D. Datasheet (after Gebru et al. [@gebru2021datasheets])

**Motivation.** Created to test whether methods that learn dynamics from trajectories predict the effects of
interventions in a multi-agent system, against exact counterfactual branches, and how far they are from the attainable
error. Created by [AUTHORS]; funding [OWNER].

**Composition.** Recorded episodes of a deterministic village simulation (`game_v19`): per episode 2,881 hourly rows of
39 global and 14 per-settlement columns plus hidden tables (marked) and the list of interventions. Three splits:
v2 (560 worlds; families of the no-act member and four arms, two replicates each; 5,316 family episodes plus 560
`game_v16` control episodes and noise-floor families), obs (560 worlds; 2,219 episodes), obs2 (1,640 worlds; round
episodes of a sequential policy and two families per val / test world). Instances are synthetic worlds; there are no
people, no personal or sensitive data and no labels beyond what the simulator records. Roles (train / val / test) are
fixed by seed ranges. Hidden columns must not be model inputs; they are kept for diagnostics.

**Collection process.** Every episode was produced by one frozen recorder binary (source commit recorded in every
episode's metadata; sha256 of the binary recorded), with byte-identical output across worker counts; every array is
sha256-hashed in its episode's metadata and in the split's manifest. The observational policies are seeded and
pre-registered; their propensities are recorded. Counterfactual families are forks of the same world state.

**Preprocessing.** None beyond recording: the harness reads the arrays directly; cases, truths and scales are computed
by committed code from the arrays.

**Uses.** Intended: evaluating counterfactual / treatment-effect prediction and world models under interventions,
with the pre-registered harness (cells, scales, world-level bootstrap). Not intended: any claim about real societies,
real interventions or policy.

**Distribution and maintenance.** [OWNER: release plan]. The recorder, the harness and every manifest are in the
project repository; the episode arrays live outside version control (several GB per split).


## Appendix E. Compute

{{sec:appendix_compute}}
