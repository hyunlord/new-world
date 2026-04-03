# Visual Verification Checklist — {{FEATURE}}

Answer each question based on the screenshots and data provided.
If a screenshot is not available, answer based on entity_summary.txt and performance.txt.

## General Health
1. Are agents visible on the map? (not all at 0,0, not invisible)
2. Are agents spread across the map? (not all clustered in one spot)
3. Is the map terrain rendered? (not black, not uniform color)
4. Are there any visual glitches? (z-ordering issues, flickering, missing textures)
5. Is FPS acceptable? (avg tick < 50ms for 20 TPS target)
6. Are there console errors? (any ERROR lines in console_log.txt)

## Feature-Specific
{{VISUAL_CHECKS}}

## Output Format
```
## Visual Analysis: {{FEATURE}}

### General Health
1. Agents visible: YES/NO — <detail>
2. Agent spread: YES/NO — spread X=<n> Y=<n>
3. Terrain rendered: YES/NO — <detail>
4. Visual glitches: NONE/<description>
5. Performance: avg <n>ms — OK/WARNING
6. Console errors: <count> — <summary if any>

### Feature-Specific
<answers to feature-specific questions>

### Visual Verdict
VISUAL_OK | VISUAL_WARNING(<reason>) | VISUAL_FAIL(<reason>)
```
