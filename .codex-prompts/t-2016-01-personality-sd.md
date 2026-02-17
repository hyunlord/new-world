# T-2016-01: Personality SD 0.15 → 0.25

## Objective
Change personality standard deviation from 0.15 to 0.25 across all personality-related code, and increase intra-axis facet variance from 0.25 to 0.35.

## Background
- Academic HEXACO-60 SD ≈ 15% of range (SD=0.15 on 0~1 scale)
- With SD=0.15, most agents cluster at 35~65%, Trait emergence <0.1%
- SD=0.25 spreads distribution to 5~95%, Trait emergence ~15%, agents get 2~4 traits each
- This is a deliberate gameplay decision; distribution structure (normal, correlations, sex diffs) stays academic

## Files to Modify

### 1. `scripts/core/personality_data.gd`

**Current code (lines 56-63):**
```gdscript
## Convert 0.0~1.0 trait value to z-score (mean=0.5, sd=0.15)
func to_zscore(trait01: float) -> float:
	return (trait01 - 0.5) / 0.15


## Convert z-score back to 0.0~1.0 (clamped to [0.05, 0.95])
func from_zscore(z: float) -> float:
	return clampf(0.5 + 0.15 * z, 0.05, 0.95)
```

**Required change:**

Add a constant at the top of the file (after `const ALL_FACET_KEYS`) with academic comment:
```gdscript
## Personality standard deviation (gameplay-widened)
## Academic: HEXACO-60 SD ≈ 15% of range (Ashton & Lee 2009)
## Gameplay: SD=0.25 for agent differentiation & trait emergence (~15%)
## Distribution structure (normal, correlations, sex diffs) unchanged
const PERSONALITY_SD: float = 0.25
```

Then update both functions to use the constant:
```gdscript
## Convert 0.0~1.0 trait value to z-score (mean=0.5, sd=PERSONALITY_SD)
func to_zscore(trait01: float) -> float:
	return (trait01 - 0.5) / PERSONALITY_SD


## Convert z-score back to 0.0~1.0 (clamped to [0.05, 0.95])
func from_zscore(z: float) -> float:
	return clampf(0.5 + PERSONALITY_SD * z, 0.05, 0.95)
```

### 2. `scripts/systems/personality_generator.gd`

**Current code (line 134):**
```gdscript
		var facet_z: float = z_axis + _randfn(0.0, 0.25)
```

**Required change:**
Change the intra-axis facet variance from 0.25 to 0.35. With SD=0.25, facet variance of 0.35 keeps facets meaningfully different within an axis without overwhelming the axis signal.

```gdscript
		# Intra-axis facet variance (0.35 balances differentiation vs axis coherence at SD=0.25)
		var facet_z: float = z_axis + _randfn(0.0, 0.35)
```

## Non-goals
- Do NOT change the correlation matrix, heritability, or sex difference values
- Do NOT change anything in trait_system.gd or trait_definitions.json (that's T-2016-02)
- Do NOT change any 0.15 values that are NOT personality SD (e.g., frailty SD, colors, camera zoom, job ratios)
- Do NOT modify any UI files
- Do NOT add migration code for existing saves (values are stored as 0~1, no conversion needed)

## Acceptance Criteria
- [ ] `PERSONALITY_SD` constant added to personality_data.gd with academic comment
- [ ] `to_zscore()` uses `PERSONALITY_SD` instead of hardcoded 0.15
- [ ] `from_zscore()` uses `PERSONALITY_SD` instead of hardcoded 0.15
- [ ] Facet variance in personality_generator.gd changed from 0.25 to 0.35
- [ ] No other 0.15 values changed anywhere (grep to verify)
- [ ] No GDScript parse errors

## Godot 4.6 Notes
- personality_data.gd uses `extends RefCounted`, NO `class_name`
- personality_generator.gd uses `extends RefCounted`, NO `class_name`
- Use `preload()` for cross-script references, not class_name
