# T-2014-06: UI Entity Detail Panel — HEXACO 24-Facet Display

## Objective
Update entity_detail_panel.gd to display HEXACO 6 axes with expandable facets + trait badges.

## Non-goals
- Do NOT modify any other files
- Do NOT add new UI nodes or scenes — this is pure `_draw()` code
- Do NOT change the panel layout for sections other than Personality

## Scope
Files to MODIFY:
- `scripts/ui/entity_detail_panel.gd` — Personality + Traits display

## Current File Structure
The file uses Godot's `_draw()` for all rendering (no Control nodes).
Personality section currently at lines ~193-200:
```gdscript
cy = _draw_section_header(font, cx, cy, "Personality")
var p_keys: Array = ["openness", "agreeableness", "extraversion", "diligence", "emotional_stability"]
var p_labels: Array = ["Open", "Agree", "Extra", "Dilig", "Stab"]
for i in range(p_keys.size()):
    var val: float = entity.personality.get(p_keys[i], 0.5)
    cy = _draw_bar(font, cx + 10, cy, bar_w, p_labels[i], val, PERSONALITY_COLORS[p_keys[i]])
cy += 6.0
```

Deceased section similarly at lines ~584-593 reads `personality` dict with same keys.

## New PersonalityData Format
`entity.personality` is now a PersonalityData object (RefCounted) with:
```gdscript
var facets: Dictionary = {}      # 24 keys, 0.0~1.0
var axes: Dictionary = {}        # 6 keys: H, E, X, A, C, O
var active_traits: Array = []    # ["honest", "fearful", ...]
const AXIS_IDS = ["H", "E", "X", "A", "C", "O"]
const FACET_KEYS: Dictionary = { "H": [...], "E": [...], ... }
```

For deceased records, personality is stored as Dictionary:
```gdscript
{"facets": {...}, "active_traits": [...]}
```
Use PersonalityData.from_dict() to reconstruct.

## TraitSystem Interface (from T-2014-03)
```gdscript
const TraitSystem = preload("res://scripts/systems/trait_system.gd")
# TraitSystem.get_trait_definition(trait_id) -> Dictionary with name_kr, name_en, sentiment
# TraitSystem.get_trait_sentiment(trait_id) -> "positive" / "negative" / "neutral"
```

## Detailed Changes

### 1. Update PERSONALITY_COLORS constant
Replace old Big Five colors with HEXACO axis colors:
```gdscript
const PERSONALITY_COLORS: Dictionary = {
    "H": Color(0.9, 0.7, 0.2),   # Gold — Honesty-Humility
    "E": Color(0.4, 0.6, 0.9),   # Blue — Emotionality
    "X": Color(0.9, 0.5, 0.2),   # Orange — Extraversion
    "A": Color(0.3, 0.8, 0.5),   # Green — Agreeableness
    "C": Color(0.2, 0.6, 0.9),   # Teal — Conscientiousness
    "O": Color(0.7, 0.4, 0.9),   # Purple — Openness
}

const FACET_COLOR_DIM: float = 0.7  # Facet bars are dimmer than axis bars

## Trait sentiment colors
const TRAIT_COLORS: Dictionary = {
    "positive": Color(0.3, 0.8, 0.4),
    "negative": Color(0.9, 0.3, 0.3),
    "neutral": Color(0.9, 0.8, 0.3),
}
```

### 2. Add expand/collapse state
```gdscript
## Which axes are expanded (show facets)
var _expanded_axes: Dictionary = {}  # "H": true, "E": false, ...
```

### 3. Replace Personality drawing section (living entity)
Replace the old 5-bar personality section with:
```gdscript
# ── Personality (HEXACO 6-axis + expandable facets) ──
cy = _draw_section_header(font, cx, cy, "Personality")

# Axis labels
var axis_labels: Dictionary = {
    "H": "H (정직)", "E": "E (감정)", "X": "X (외향)",
    "A": "A (우호)", "C": "C (성실)", "O": "O (개방)"
}

var pd = entity.personality  # PersonalityData object
for axis_id in pd.AXIS_IDS:
    var axis_val: float = pd.axes.get(axis_id, 0.5)
    var color: Color = PERSONALITY_COLORS.get(axis_id, Color.GRAY)
    var is_expanded: bool = _expanded_axes.get(axis_id, false)
    var arrow: String = "▼" if is_expanded else "▶"
    var label: String = "%s %s" % [arrow, axis_labels.get(axis_id, axis_id)]

    # Draw axis bar (clickable to expand)
    var axis_y: float = cy
    cy = _draw_bar(font, cx + 10, cy, bar_w, label, axis_val, color)

    # Register click region for expand/collapse
    _click_regions.append({
        "rect": Rect2(cx + 10, axis_y, bar_w, 16.0),
        "entity_id": -1,  # special: axis toggle
        "axis_id": axis_id,
    })

    # Draw facets if expanded
    if is_expanded:
        var fkeys: Array = pd.FACET_KEYS[axis_id]
        for fk in fkeys:
            var fval: float = pd.facets.get(fk, 0.5)
            var fname: String = fk.substr(fk.find("_") + 1).replace("_", " ").capitalize()
            var dim_color: Color = Color(color.r * FACET_COLOR_DIM, color.g * FACET_COLOR_DIM, color.b * FACET_COLOR_DIM)
            cy = _draw_bar(font, cx + 25, cy, bar_w - 15, fname, fval, dim_color)
cy += 4.0

# ── Traits ──
if pd.active_traits.size() > 0:
    cy = _draw_section_header(font, cx, cy, "Traits")
    var trait_x: float = cx + 10
    for trait_id in pd.active_traits:
        var tdef: Dictionary = TraitSystem.get_trait_definition(trait_id)
        var tname: String = tdef.get("name_kr", trait_id)
        var sentiment: String = tdef.get("sentiment", "neutral")
        var tcolor: Color = TRAIT_COLORS.get(sentiment, Color.GRAY)
        # Draw tag/badge
        var text_w: float = font.get_string_size(tname, HORIZONTAL_ALIGNMENT_LEFT, -1, GameConfig.get_font_size("popup_body")).x
        if trait_x + text_w + 16 > size.x - 20:
            cy += 16.0
            trait_x = cx + 10
        # Badge background
        draw_rect(Rect2(trait_x, cy + 1, text_w + 10, 14), Color(tcolor.r, tcolor.g, tcolor.b, 0.2))
        draw_rect(Rect2(trait_x, cy + 1, text_w + 10, 14), tcolor, false, 1.0)
        draw_string(font, Vector2(trait_x + 5, cy + 12), tname, HORIZONTAL_ALIGNMENT_LEFT, -1, GameConfig.get_font_size("popup_body"), tcolor)
        trait_x += text_w + 16
    cy += 20.0
cy += 6.0
```

### 4. Handle axis expand/collapse clicks
In `_gui_input()`, when left click on an axis region:
```gdscript
# In the click region check loop, add:
if region.has("axis_id"):
    var aid: String = region.axis_id
    _expanded_axes[aid] = not _expanded_axes.get(aid, false)
    accept_event()
    return
```

### 5. Deceased personality display
In `_draw_deceased()`, replace the old personality bar section with equivalent code.
For deceased, personality is stored as Dictionary, so reconstruct PersonalityData:
```gdscript
var PersonalityDataScript = load("res://scripts/core/personality_data.gd")
var p_dict: Dictionary = r.get("personality", {})
var pd: RefCounted
if p_dict.has("facets"):
    pd = PersonalityDataScript.from_dict(p_dict)
else:
    # Old format migration
    pd = PersonalityDataScript.new()
    pd.migrate_from_big_five(p_dict)
```
Then use the same axis bar + facet + trait drawing code.

### 6. Add TraitSystem preload at top of file
```gdscript
const TraitSystem = preload("res://scripts/systems/trait_system.gd")
```

## IMPORTANT: Preserve ALL other sections
The file is ~619 lines. Only modify:
1. The PERSONALITY_COLORS constant (lines 12-18)
2. Add TRAIT_COLORS constant and _expanded_axes var
3. Add TraitSystem preload at top
4. The Personality section in _draw() (~lines 193-200)
5. The Personality section in _draw_deceased() (~lines 584-593)
6. The click handler in _gui_input()

Do NOT change: Status, Needs, Emotions, Family, Relationships, Stats, Action History sections.

## Acceptance Criteria
- [ ] 6 HEXACO axis bars with expand/collapse arrows
- [ ] Clicking axis toggles facet visibility
- [ ] 4 facet sub-bars shown when expanded (dimmer color, indented)
- [ ] Trait badges at bottom with color coding (green/red/yellow)
- [ ] Korean trait names displayed
- [ ] Deceased view uses same HEXACO display
- [ ] Old Big Five saves handled via migration in deceased view
- [ ] All other panel sections unchanged
