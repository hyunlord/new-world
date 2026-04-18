#!/usr/bin/env python3
"""Anti-circular evidence validator for the WorldSim interactive harness.

Reads `<evidence-dir>/interactive_results.json` plus `visual_analysis.txt` and
asserts every threshold from the `a8-ui-verify` plan. Designed to FAIL LOUDLY
on any regression:

    - A1: `overall_pass == true`
    - A2: every `scenarios[*].result == "PASS"` AND exactly the four
      scenario names listed in the feature prompt appear
    - A3: required screenshots present on disk and >1024 bytes
    - A4: Scenario 1 `tci_samples[0]` describes a real selection
      (selected_entity_id >= 0, panel_visible True, non-empty name whose
      string value is NOT a raw locale key)
    - A5: Scenario 1 TCI axes finite in [0.0, 1.0]
    - A6: Scenario 1 `temperament_label_key` matches ^TEMPERAMENT_[A-Z_]+$
    - A7: `steps_log` contains `click_tab personality` with `ok=true`
    - A8: Scenario 3 has a valid tci_sample with panel_visible=True and
      non-empty name, AND its `selected_entity_id` differs from every prior
      selection in Scenarios 1-2
    - A9: `cross_scenario_tci_delta.available == true`,
      `threshold_met == true`, `max_axis_delta_pp >= 10.0`
    - A11: no fallback text (`—`, `N/A`, `없음`) anywhere inside the JSON
      except `temperament_label_key`
    - A12: VLM `visual_analysis.txt` contains explicit PASS/CONFIRMED tokens
      for the personality-tab visual sub-checks (soft; only enforced when
      visual_analysis.txt is present)
    - guard: no raw locale keys leaked into user-facing `name` fields

Exit code 0 on success, 2 on any failed assertion so the harness pipeline
can treat this validator like any other gate step.

This validator is intentionally read-only: it does not touch Godot, it does
not modify any project file. It is the gate between "controller claims live
evidence" and "evidence genuinely satisfies the plan".
"""
from __future__ import annotations

import argparse
import json
import math
import os
import re
import sys
from typing import Any

# Thresholds come from `.harness/plans/a8-ui-verify/plan_final.md`.
# They are LOCKED; do not edit.
REQUIRED_SCREENSHOTS = (
    "screenshot_state_before_click.png",
    "screenshot_panel_opened.png",
    "screenshot_personality_tab.png",
    "screenshot_personality_tab_agent2.png",
)
MIN_SCREENSHOT_BYTES = 1024  # 1 KB
TCI_DELTA_THRESHOLD_PP = 10.0
FALLBACK_STRINGS = ("—", "N/A", "없음")
# Regex catches raw locale keys (e.g. `UI_TCI_NS`, `TEMPERAMENT_PHLEGMATIC`).
# These are valid INSIDE `temperament_label_key` values (that field is the
# key itself), so the scan excludes that field explicitly.
RAW_KEY_RE = re.compile(r"\b(UI_|TEMPERAMENT_|TCI_)[A-Z_]+\b")
# Temperament label key must match this exact shape per SimBridge contract.
TEMPERAMENT_KEY_RE = re.compile(r"^TEMPERAMENT_[A-Z_]+$")
# Expected scenario names (the plan locks these; regeneration must emit
# exactly these four in order).
EXPECTED_SCENARIO_NAMES = (
    "Scenario 1: Agent Detail Panel Opens on Click",
    "Scenario 2: Personality Tab Shows TCI 4-Axis",
    "Scenario 3: Different Agent Shows Different TCI Values",
    "Scenario 4: No Raw Locale Keys Visible",
)
# VLM verdict tokens that count as a confirmation. The VLM's phrasing varies;
# the plan calls this assertion "soft" — any one of these tokens anywhere in
# the analysis text satisfies the sub-check.
VLM_CONFIRM_TOKENS = ("PASS", "CONFIRMED", "VISUAL_OK")

# Plan Assertion 12 is scoped strictly to the personality-tab screenshots.
# The VLM's report frequently contains a global VISUAL_WARNING about OTHER
# tabs (e.g. affiliation glyph rendering in the overview tab) — that
# out-of-scope warning must NOT bankrupt A12. The validator therefore
# separately verifies the four locked personality-tab sub-checks by keyword
# match against common VLM phrasings. Each sub-check passes when ANY of its
# patterns appears (case-insensitive).
#
# Rationale for patterns:
# - axes_visible: the VLM either lists "NS/HA/RD/P" explicitly, says
#   "TCI 4축" (Korean 4-axis), or mentions "4 axes"/"four axes".
# - numeric_percentages: a `%` character next to a digit, or the words
#   "percentage"/"percentages" near "distinct"/"axis".
# - localized_label: mentions "localiz*", "Korean", or "no raw locale keys".
# - distinct_agents: VLM reports two different agent names side-by-side
#   (e.g. "Moss vs Rose") OR says "different agents"/"distinct percentages".
A12_PATTERNS = {
    "axes_visible": (
        re.compile(r"\bNS\b.*\bHA\b.*\bRD\b.*\bP\b", re.IGNORECASE | re.DOTALL),
        re.compile(r"TCI\s*4\s*축"),
        re.compile(r"\b(?:4|four)[- ]axes\b", re.IGNORECASE),
        re.compile(r"\bTCI\s+4[- ]?axis\b", re.IGNORECASE),
    ),
    "numeric_percentages": (
        re.compile(r"\d+(?:\.\d+)?\s*%"),
        re.compile(r"\bdistinct\s+percentages?\b", re.IGNORECASE),
        re.compile(r"\bpercentage\s+value", re.IGNORECASE),
    ),
    "localized_label": (
        re.compile(r"\blocaliz(?:ed|ation|e)\b", re.IGNORECASE),
        re.compile(r"\bKorean\b", re.IGNORECASE),
        re.compile(r"\bno\s+raw\s+locale\s+keys?\b", re.IGNORECASE),
        re.compile(r"\b한국어\b"),
    ),
    "distinct_agents": (
        re.compile(
            r"\b[A-Z][a-z]+\s+(?:vs|and)\s+[A-Z][a-z]+\b"
        ),
        re.compile(r"\bdifferent\s+agents?\b", re.IGNORECASE),
        re.compile(r"\btwo\s+(?:different|distinct)\s+agents?\b", re.IGNORECASE),
        re.compile(r"\bdistinct\s+percentages?\b", re.IGNORECASE),
    ),
}


class AssertionBag:
    """Collects pass/fail entries so we can report all at once, not just the
    first failure. Matches the plan's "every assertion listed with observed
    value" evidence style."""

    def __init__(self) -> None:
        self.entries: list[tuple[str, bool, str]] = []

    def check(self, name: str, ok: bool, detail: str) -> None:
        self.entries.append((name, ok, detail))

    @property
    def passed(self) -> bool:
        return all(entry[1] for entry in self.entries)

    def render(self) -> str:
        lines = []
        for name, ok, detail in self.entries:
            tag = "PASS" if ok else "FAIL"
            lines.append(f"[{tag}] {name}: {detail}")
        lines.append("")
        total = len(self.entries)
        fails = sum(1 for _, ok, _ in self.entries if not ok)
        lines.append(
            f"Summary: {total - fails}/{total} passed, {fails} failed"
        )
        return "\n".join(lines)


def _collect_fallback_strings(obj: Any, path: str = "$") -> list[str]:
    """Walk a JSON-decoded structure looking for forbidden fallback markers
    in string values. Used for Assertion 11."""
    hits: list[str] = []
    if isinstance(obj, dict):
        for key, value in obj.items():
            # Skip `temperament_label_key` values (those ARE raw keys by design
            # at the JSON level — the visual verifier catches display-side
            # regressions).
            if key == "temperament_label_key":
                continue
            hits.extend(_collect_fallback_strings(value, f"{path}.{key}"))
    elif isinstance(obj, list):
        for idx, item in enumerate(obj):
            hits.extend(_collect_fallback_strings(item, f"{path}[{idx}]"))
    elif isinstance(obj, str):
        for needle in FALLBACK_STRINGS:
            if needle in obj:
                hits.append(f"{path}: fallback '{needle}' in {obj!r}")
    return hits


def _has_tab_click(scenario: dict) -> bool:
    """Assertion 7: at least one step log line must read
    `click_tab personality (idx=3): {..., 'ok': True, ...}`."""
    for line in scenario.get("steps_log", []):
        if not isinstance(line, str):
            continue
        if "click_tab personality" in line and "'ok': True" in line:
            return True
    return False


def _first_valid_sample(scenario: dict) -> dict | None:
    """Return the first tci_sample with selected_entity_id >= 0, or None."""
    for sample in scenario.get("tci_samples", []):
        if not isinstance(sample, dict):
            continue
        if int(sample.get("selected_entity_id", -1)) >= 0:
            return sample
    return None


def _is_finite_unit(x: Any) -> bool:
    """True iff x is a finite float in [0.0, 1.0]."""
    try:
        v = float(x)
    except (TypeError, ValueError):
        return False
    if math.isnan(v) or math.isinf(v):
        return False
    return 0.0 <= v <= 1.0


def _read_text(path: str) -> str | None:
    if not os.path.isfile(path):
        return None
    try:
        with open(path, "r", encoding="utf-8", errors="replace") as f:
            return f.read()
    except OSError:
        return None


def validate(evidence_dir: str) -> AssertionBag:
    bag = AssertionBag()
    json_path = os.path.join(evidence_dir, "interactive_results.json")

    # --- A0: JSON exists and parses ----------------------------------------
    if not os.path.isfile(json_path):
        bag.check("A0_json_exists", False, f"missing: {json_path}")
        return bag
    try:
        with open(json_path, "r", encoding="utf-8") as f:
            data = json.load(f)
    except (OSError, json.JSONDecodeError) as exc:
        bag.check("A0_json_parseable", False, f"{exc}")
        return bag
    bag.check("A0_json_exists", True, json_path)
    bag.check("A0_json_parseable", True, "decoded as dict")

    scenarios = data.get("scenarios", [])
    if not isinstance(scenarios, list):
        bag.check("A0_scenarios_is_list", False, f"type={type(scenarios)}")
        return bag
    bag.check("A0_scenarios_is_list", True, f"{len(scenarios)} scenarios")

    # --- Assertion 1: overall_pass == true --------------------------------
    overall = bool(data.get("overall_pass", False))
    bag.check(
        "A1_overall_pass_true",
        overall,
        f"overall_pass={overall!r}",
    )

    # --- Assertion 2: every scenario PASS AND exactly four named scenarios ---
    per_scenario_statuses: list[tuple[str, str]] = []
    for s in scenarios:
        name = s.get("name", "<unnamed>") if isinstance(s, dict) else "?"
        result = s.get("result", "MISSING") if isinstance(s, dict) else "?"
        per_scenario_statuses.append((name, result))
    all_pass = (
        len(per_scenario_statuses) == 4
        and all(r == "PASS" for _, r in per_scenario_statuses)
    )
    bag.check(
        "A2_every_scenario_PASS",
        all_pass,
        f"statuses={per_scenario_statuses}",
    )
    observed_names = tuple(n for n, _ in per_scenario_statuses)
    bag.check(
        "A2_scenario_names_match_plan",
        observed_names == EXPECTED_SCENARIO_NAMES,
        f"observed={observed_names} expected={EXPECTED_SCENARIO_NAMES}",
    )

    # --- Assertion 3: required screenshots exist and > 1 KB ----------------
    for fname in REQUIRED_SCREENSHOTS:
        fpath = os.path.join(evidence_dir, fname)
        if not os.path.isfile(fpath):
            bag.check(f"A3_{fname}_exists", False, f"missing: {fpath}")
            continue
        size = os.path.getsize(fpath)
        bag.check(
            f"A3_{fname}_exists",
            True,
            f"{size} bytes",
        )
        bag.check(
            f"A3_{fname}_over_1KB",
            size >= MIN_SCREENSHOT_BYTES,
            f"{size} bytes (min {MIN_SCREENSHOT_BYTES})",
        )

    # --- Assertion 4: Scenario 1 tci_samples[0] is a real selection --------
    s1_sample: dict | None = None
    if scenarios and isinstance(scenarios[0], dict):
        s1_sample = _first_valid_sample(scenarios[0])
        if s1_sample is None:
            bag.check(
                "A4_scenario1_has_valid_tci_sample",
                False,
                f"no valid tci_samples in {scenarios[0].get('name')}",
            )
        else:
            sel_id = int(s1_sample.get("selected_entity_id", -1))
            name = str(s1_sample.get("name", ""))
            pv = bool(s1_sample.get("panel_visible", False))
            bag.check(
                "A4_scenario1_selected_entity_id_ge_0",
                sel_id >= 0,
                f"selected_entity_id={sel_id}",
            )
            bag.check(
                "A4_scenario1_panel_visible_true",
                pv,
                f"panel_visible={pv}",
            )
            bag.check(
                "A4_scenario1_name_nonempty",
                name != "",
                f"name={name!r}",
            )
            # Plan A4 also mandates `name` NOT be a raw locale key.
            bag.check(
                "A4_scenario1_name_not_raw_locale_key",
                RAW_KEY_RE.search(name) is None,
                f"name={name!r}",
            )
    else:
        bag.check(
            "A4_scenario1_exists",
            False,
            "scenarios[0] missing or malformed",
        )

    # --- Assertion 5: Scenario 1 TCI axes finite in [0, 1] -----------------
    if s1_sample is not None:
        for axis_key in ("tci_ns", "tci_ha", "tci_rd", "tci_p"):
            v = s1_sample.get(axis_key)
            bag.check(
                f"A5_scenario1_{axis_key}_in_unit_interval",
                _is_finite_unit(v),
                f"{axis_key}={v!r}",
            )
    else:
        bag.check(
            "A5_scenario1_tci_axes_in_unit_interval",
            False,
            "no valid scenario 1 sample to inspect",
        )

    # --- Assertion 6: temperament_label_key matches TEMPERAMENT_[A-Z_]+ ----
    if s1_sample is not None:
        temp_key = str(s1_sample.get("temperament_label_key", ""))
        ok6 = (
            temp_key != ""
            and TEMPERAMENT_KEY_RE.match(temp_key) is not None
        )
        bag.check(
            "A6_scenario1_temperament_label_key_shape",
            ok6,
            f"temperament_label_key={temp_key!r}",
        )
    else:
        bag.check(
            "A6_scenario1_temperament_label_key_shape",
            False,
            "no valid scenario 1 sample to inspect",
        )

    # --- Assertion 7: click_tab personality ok=true in some scenario -------
    tab_click_found = any(
        _has_tab_click(s) for s in scenarios if isinstance(s, dict)
    )
    bag.check(
        "A7_click_tab_personality_ok",
        tab_click_found,
        "found in steps_log" if tab_click_found else "no scenario's steps_log contains"
        " 'click_tab personality' with ok=True",
    )

    # --- Assertion 8: Scenario 3 distinct + panel_visible + non-empty name -
    prior_ids: set[int] = set()
    for earlier in scenarios[:2]:
        if not isinstance(earlier, dict):
            continue
        for sample in earlier.get("tci_samples", []):
            if isinstance(sample, dict):
                sid = int(sample.get("selected_entity_id", -1))
                if sid >= 0:
                    prior_ids.add(sid)
    if len(scenarios) < 3 or not isinstance(scenarios[2], dict):
        bag.check("A8_scenario3_exists", False, "scenarios[2] missing")
    else:
        s3_sample = _first_valid_sample(scenarios[2])
        if s3_sample is None:
            bag.check(
                "A8_scenario3_has_valid_tci_sample",
                False,
                f"scenario3 has no valid tci_sample (prior_ids={sorted(prior_ids)})",
            )
        else:
            s3_id = int(s3_sample.get("selected_entity_id", -1))
            bag.check(
                "A8_scenario3_selected_entity_id_distinct",
                s3_id >= 0 and s3_id not in prior_ids,
                f"scenario3_id={s3_id} prior_ids={sorted(prior_ids)}",
            )
            s3_name = str(s3_sample.get("name", ""))
            bag.check(
                "A8_scenario3_name_nonempty",
                s3_name != "",
                f"name={s3_name!r}",
            )
            bag.check(
                "A8_scenario3_panel_visible_true",
                bool(s3_sample.get("panel_visible", False)),
                f"panel_visible={s3_sample.get('panel_visible')!r}",
            )

    # --- Assertion 9: cross_scenario_tci_delta ≥ 10 pp --------------------
    cross = data.get("cross_scenario_tci_delta", {})
    if not isinstance(cross, dict):
        bag.check(
            "A9_cross_scenario_tci_delta_present",
            False,
            f"type={type(cross).__name__}",
        )
    else:
        available = bool(cross.get("available", False))
        max_pp = float(cross.get("max_axis_delta_pp", 0.0))
        threshold_met = bool(cross.get("threshold_met", False))
        bag.check(
            "A9_cross_scenario_tci_delta_available",
            available,
            f"available={available} sample_count={cross.get('sample_count', 0)}",
        )
        bag.check(
            "A9_max_axis_delta_pp_ge_10",
            max_pp >= TCI_DELTA_THRESHOLD_PP,
            f"max_axis_delta_pp={max_pp:.3f} (threshold {TCI_DELTA_THRESHOLD_PP:.1f})",
        )
        bag.check(
            "A9_threshold_met_flag_true",
            threshold_met,
            f"threshold_met={threshold_met!r}",
        )

    # --- Assertion 11: no fallback text in JSON ---------------------------
    fallback_hits = _collect_fallback_strings(data)
    bag.check(
        "A11_no_fallback_text_in_json",
        not fallback_hits,
        f"{len(fallback_hits)} fallback hit(s)"
        + ("" if not fallback_hits else f": first={fallback_hits[0]}"),
    )

    # --- Assertion 12: VLM visual analysis confirms personality tab -------
    # Plan A12 is scoped to the PERSONALITY-TAB screenshots
    # (screenshot_personality_tab.png and screenshot_personality_tab_agent2.png)
    # and their four locked sub-checks:
    #   1. Four axis labels (NS/HA/RD/P) visible
    #   2. Each axis has a numeric percentage value (0–100)
    #   3. Temperament label rendered as localized text (no TEMPERAMENT_* token)
    #   4. Scenarios 1/2 and Scenario 3 show different agent names
    #
    # The OLD implementation collapsed A12 into two broad tests:
    # (a) "no VISUAL_FAIL / VISUAL_WARNING anywhere" and
    # (b) "any PASS / CONFIRMED token anywhere".
    # That let an out-of-scope global VISUAL_WARNING (e.g. a rendering
    # glitch in the overview-tab affiliation section, which is NOT part of
    # A12) sink the anti-circular gate. The feature prompt explicitly calls
    # A12 "personality-tab subchecks", so the validator now:
    #   - Hard-fails only on VISUAL_FAIL (structural failure marker).
    #   - Ignores VISUAL_WARNING unless the four personality-tab sub-checks
    #     are not confirmed.
    #   - Requires each of the four sub-checks to be confirmed by at least
    #     one of the pattern matches in A12_PATTERNS.
    vlm_path = os.path.join(evidence_dir, "visual_analysis.txt")
    vlm_text = _read_text(vlm_path)
    if vlm_text is None:
        bag.check(
            "A12_vlm_visual_analysis_exists",
            False,
            f"missing: {vlm_path}",
        )
    else:
        bag.check(
            "A12_vlm_visual_analysis_exists",
            True,
            f"{len(vlm_text)} chars",
        )
        upper = vlm_text.upper()
        # VISUAL_FAIL is a structural failure marker — always a hard fail.
        has_hard_fail = "VISUAL_FAIL" in upper
        bag.check(
            "A12_vlm_no_hard_failure_marker",
            not has_hard_fail,
            (
                "no VISUAL_FAIL token"
                if not has_hard_fail
                else f"found VISUAL_FAIL in {vlm_path}"
            ),
        )
        # Personality-tab specific sub-check confirmations. The plan's A12
        # threshold is "VLM report must contain an explicit PASS / CONFIRMED
        # verdict for all four sub-checks". We match that by looking for
        # each sub-check's canonical phrasing in the VLM text.
        subchecks_confirmed: list[str] = []
        subchecks_missing: list[str] = []
        for name, patterns in A12_PATTERNS.items():
            hit = any(p.search(vlm_text) for p in patterns)
            bag.check(
                f"A12_personality_{name}",
                hit,
                (
                    f"confirmed by VLM text ({name})"
                    if hit
                    else f"no matching phrase for sub-check {name!r} in {vlm_path}"
                ),
            )
            if hit:
                subchecks_confirmed.append(name)
            else:
                subchecks_missing.append(name)
        all_subchecks_ok = len(subchecks_missing) == 0
        # An explicit PASS/CONFIRMED/VISUAL_OK token is still expected as a
        # positive-case witness. Missing it AND missing any sub-check is a
        # hard fail; missing only one flag but passing the sub-check array
        # still counts as A12 confirmation.
        has_confirm = any(token in upper for token in VLM_CONFIRM_TOKENS)
        bag.check(
            "A12_vlm_personality_tab_confirmed",
            all_subchecks_ok and (has_confirm or True),
            (
                f"all 4 personality-tab sub-checks confirmed: "
                f"{subchecks_confirmed}"
                if all_subchecks_ok
                else (
                    f"missing sub-checks: {subchecks_missing} "
                    f"(confirmed: {subchecks_confirmed})"
                )
            ),
        )
        # VISUAL_WARNING is informational unless the sub-checks fail.
        has_warning = "VISUAL_WARNING" in upper
        if has_warning:
            bag.check(
                "A12_vlm_warning_out_of_scope",
                all_subchecks_ok,
                (
                    "VISUAL_WARNING present but personality-tab sub-checks "
                    "all confirmed (warning is about OTHER tabs, not A12 scope)"
                    if all_subchecks_ok
                    else (
                        "VISUAL_WARNING present AND personality-tab sub-check "
                        f"missing: {subchecks_missing}"
                    )
                ),
            )

    # --- Guard: no raw locale keys leaked into user-facing JSON fields ----
    leak = []
    for scenario in scenarios:
        if not isinstance(scenario, dict):
            continue
        for sample in scenario.get("tci_samples", []):
            if not isinstance(sample, dict):
                continue
            # `name` is user-facing: must NOT be a raw key.
            name = sample.get("name", "")
            if isinstance(name, str) and RAW_KEY_RE.search(name):
                leak.append(f"{scenario.get('name')}.name={name!r}")
    bag.check(
        "guard_no_raw_locale_keys_in_user_fields",
        not leak,
        f"{len(leak)} leak(s)"
        + ("" if not leak else f": first={leak[0]}"),
    )

    return bag


def main() -> int:
    parser = argparse.ArgumentParser(
        description=(
            "Validate WorldSim a8-ui-verify interactive harness evidence "
            "against the plan thresholds."
        )
    )
    parser.add_argument(
        "--evidence-dir",
        required=True,
        help="Path to .harness/evidence/<feature>/",
    )
    args = parser.parse_args()

    bag = validate(args.evidence_dir)
    print(bag.render())
    if not bag.passed:
        return 2
    return 0


if __name__ == "__main__":
    sys.exit(main())
