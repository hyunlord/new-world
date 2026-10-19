#!/usr/bin/env python3
# /// script
# requires-python = ">=3.11"
# dependencies = []
# ///
# Run: python3 tools/art_pipeline/generate_agent_sections_p10.py SOURCE OUTPUT.gd
"""Generate opt-in diagnostics; never modify the ordinary renderer."""
from __future__ import annotations

import hashlib
import json
import sys
from pathlib import Path
from typing import Final

NAMES: Final = (
    "reads", "checksum_talk", "chill_ingest", "activity_context", "tile_contexts",
    "agent_setup", "look", "pose", "frame_choice", "variant", "meta", "children",
    "draw_rows", "blocked", "seats", "footprints", "declump", "easing",
    "walker_offsets", "walker_apply", "post_loop", "retire", "selection", "causal", "other",
)
# Ordered prefixes are unique inside the named function, including indentation.
PROCESS: Final = (
    ('\tvar world_info:', 'reads'), ('\t_move_duration =', 'other'),
    ('\tvar snap:', 'reads'), ('\tvar founders:', 'other'),
    ('\t_prepare_talk_cache(', 'checksum_talk'), ('\tvar lerp_t:', 'other'),
    ('\tvar context:', 'activity_context'), ('\tvar tile_contexts:', 'other'),
    ('\tvar time:', 'reads'), ('\tvar tick:', 'other'),
    ('\tingest_chill_snapshot(', 'chill_ingest'), ('\tvar hour:', 'other'),
    ('\t\tvar agent_id: int = int(agent_ids[i])', 'agent_setup'), ('\t\tvar look:', 'look'),
    ('\t\tvar previous_position:', 'agent_setup'), ('\t\tvar tile_position', 'tile_contexts'),
    ('\t\tvar state:', 'pose'), ('\t\tvar body:', 'look'),
    ('\t\tvar choice:', 'frame_choice'), ('\t\tSpriteLibrary.apply_variant(', 'variant'),
    ('\t\tsprite.flip_h =', 'meta'), ('\t\tCarryLoad.apply(', 'children'),
    ('\t\tdraw_agents.append(', 'draw_rows'), ('\t_apply_drawn_positions(', 'other'),
    ('\tfor row: Dictionary in draw_agents:', 'post_loop'),
    ('\t_retire_missing(', 'retire'), ('\t_update_selection_ring(', 'selection'),
    ('\t_ingest_causal_events(', 'causal'),
)
DRAW: Final = (
    ('\tvar hearths:', 'blocked'), ('\tvar seats:', 'seats'),
    ('\tvar spread:', 'footprints'), ('\tvar declump:', 'declump'),
    ('\tvar lateral:', 'walker_offsets'),
)


class MarkerError(RuntimeError):
    """Source shape changed; refresh markers only after inspecting the new code."""


def unique_line(lines: list[str], prefix: str) -> int:
    """Resolve exactly one line with its indentation-sensitive prefix."""
    indexes = [index for index, line in enumerate(lines) if line.startswith(prefix)]
    if len(indexes) != 1:
        raise MarkerError(f"Expected one marker {prefix!r}; found {len(indexes)}")
    return indexes[0]


def inject(body: str, markers: tuple[tuple[str, str], ...]) -> str:
    """Add checkpoints without replacing or reordering any original statement."""
    lines = body.splitlines(keepends=True)
    additions: dict[int, list[str]] = {}
    previous = -1
    for prefix, section in markers:
        index = unique_line(lines, prefix)
        if index <= previous:
            raise MarkerError(f"Reordered section marker: {prefix!r}")
        previous = index
        indent = lines[index][:len(lines[index]) - len(lines[index].lstrip('\t'))]
        additions.setdefault(index, []).append(f'{indent}_p10_sections.mark({NAMES.index(section)}) # P10_PROFILE\n')
    if lines[0].startswith('func _process('):
        additions.setdefault(1, []).insert(0, '\t_p10_sections.begin() # P10_PROFILE\n')
        returns = [index for index, line in enumerate(lines) if line.strip() == 'return']
        if len(returns) != 2:
            raise MarkerError(f"Expected two guarded process returns, found {len(returns)}")
        for index in returns:
            indent = lines[index][:len(lines[index]) - len(lines[index].lstrip('\t'))]
            additions.setdefault(index, []).append(f'{indent}_p10_sections.finish(false, "early_return_line_{index + 1}") # P10_PROFILE\n')
        count = unique_line(lines, '\tvar n:') + 1
        additions.setdefault(count, []).append('\t_p10_sections.agent_count = n # P10_PROFILE\n')
        miss = unique_line(lines, '\t\t\ttile_contexts[tile_position] =')
        additions.setdefault(miss, []).append('\t\t\t_p10_sections.tile_context_misses += 1 # P10_PROFILE\n')
        ending = '\t_p10_sections.finish(true) # P10_PROFILE\n'
    else:
        declump_end = unique_line(lines, '\tvar declump:') + 1
        additions.setdefault(declump_end, []).append(f'\t_p10_sections.mark({NAMES.index("easing")}) # P10_PROFILE\n')
        lateral_end = unique_line(lines, '\tvar lateral:') + 1
        additions.setdefault(lateral_end, []).append(f'\t_p10_sections.mark({NAMES.index("walker_apply")}) # P10_PROFILE\n')
        if any(line.strip().startswith('return') for line in lines):
            raise MarkerError('Drawing function acquired an unhandled early return')
        ending = '\t_p10_sections.mark(24) # P10_PROFILE\n'
    result = ''.join(''.join(additions.get(index, [])) + line for index, line in enumerate(lines))
    return result.rstrip('\n') + '\n' + ending + '\n'



def write_skip_markers(source: str, markers: tuple[tuple[str, str], ...]) -> tuple[tuple[str, str], ...]:
    """Recognize the reviewed guard; hits never execute miss-only checkpoints."""
    if '\t\tvar skip_writes_p13:' not in source:
        return markers
    guarded = list(markers)
    variant = guarded.index(('\t\tSpriteLibrary.apply_variant(', 'variant'))
    guarded[variant:variant + 1] = [
        ('\t\tvar write_key_p13:', 'other'),
        ('\t\tif not skip_writes_p13:', 'other'),
        ('\t\t\tif sprite_id.is_empty():', 'frame_choice'),
        ('\t\t\tSpriteLibrary.apply_variant(', 'variant'),
    ]
    guarded[guarded.index(('\t\tsprite.flip_h =', 'meta'))] = ('\t\t\tsprite.flip_h =', 'meta')
    children = guarded.index(('\t\tCarryLoad.apply(', 'children'))
    guarded[children:children + 1] = [
        ('\t\t\tCarryLoad.apply(', 'children'),
        ('\t\tif _activity_oracle_enabled:', 'other'),
    ]
    return tuple(guarded)


def process_markers(source: str) -> tuple[tuple[str, str], ...]:
    """Use reviewed anchors for tick-cache and daily-activity renderer shapes."""
    if '\tvar cache_tick: bool =' not in source:
        return write_skip_markers(source, PROCESS)
    markers = list(PROCESS)
    markers.remove(('\tvar time:', 'reads'))
    markers.remove(('\tvar tick:', 'other'))
    talk = markers.index(('\t_prepare_talk_cache(', 'checksum_talk'))
    markers[talk:talk] = [('\tvar time:', 'reads'), ('\tvar tick:', 'other')]
    markers[markers.index(('\tvar context:', 'activity_context'))] = ('\tif not cache_tick or _tick_context.is_empty():', 'activity_context')
    markers[markers.index(('\tingest_chill_snapshot(', 'chill_ingest'))] = ('\tif not cache_tick or not _tick_chill_ingested:', 'chill_ingest')
    markers[markers.index(('\t\tvar look:', 'look'))] = ('\t\tvar life_stage:', 'look')
    if '\tvar activity_snapshot: Dictionary =' in source:
        tick = markers.index(('\tvar tick:', 'other')) + 1
        markers[tick:tick] = [
            ('\tvar activity_snapshot:', 'reads'),
            ('\tif not cache_tick or not _tick_activities_ready:', 'pose'),
            ('\t_prepare_frame_choices()', 'frame_choice'),
        ]
        body = markers.index(('\t\tvar body:', 'look')) + 1
        markers[body:body] = [('\t\tif daily_active and not bool(moments["pickup"]):', 'pose')]
    return write_skip_markers(source, tuple(markers))


def generate(source: str) -> str:
    """Instrument two known functions and append the diagnostic readout API."""
    if 'P10_PROFILE' in source or '_p10_sections' in source:
        raise MarkerError('Input is already instrumented or conflicts with diagnostic names')
    result = source
    for name, markers in (('_process', process_markers(source)), ('_apply_drawn_positions', DRAW)):
        prefix = f'func {name}('
        lines = result.splitlines(keepends=True)
        start = unique_line(lines, prefix)
        end = next((index for index in range(start + 1, len(lines)) if lines[index].startswith('func ')), len(lines))
        body = ''.join(lines[start:end])
        result = ''.join(lines[:start]) + inject(body, markers) + ''.join(lines[end:])
    declaration = 'var _p10_sections = preload("res://scripts/test/art/agent_section_clock_p10.gd").new() # P10_PROFILE\n'
    result = result.replace('extends Node2D\n', 'extends Node2D\n' + declaration, 1)
    if declaration not in result:
        raise MarkerError('Expected extends Node2D declaration')
    result += '\nfunc section_profile_readout() -> Dictionary: # P10_PROFILE\n\treturn _p10_sections.readout() # P10_PROFILE\n'
    original = '\n'.join(line for line in source.splitlines() if line.strip())
    recovered = '\n'.join(line for line in result.splitlines() if line.strip() and '# P10_PROFILE' not in line)
    if recovered != original:
        raise MarkerError('Instrumentation changed ordinary statements')
    return result


def main() -> int:
    """Write the generated script with a SHA-bound provenance sidecar."""
    if len(sys.argv) != 3:
        print('Usage: generate_agent_sections_p10.py SOURCE OUTPUT.gd', file=sys.stderr)
        return 2
    source, output = (Path(value).resolve() for value in sys.argv[1:])
    if source == output or output.name == 'agent_renderer.gd':
        raise MarkerError('Refusing to overwrite production renderer')
    text = source.read_text()
    generated = generate(text)
    output.parent.mkdir(parents=True, exist_ok=True)
    output.write_text(generated)
    clock = Path(__file__).resolve().parents[2] / 'scripts/test/art/agent_section_clock_p10.gd'
    manifest = {'schema_version': 1, 'source': str(source), 'output': str(output),
                'source_sha256': hashlib.sha256(source.read_bytes()).hexdigest(),
                'output_sha256': hashlib.sha256(output.read_bytes()).hexdigest(),
                'clock_sha256': hashlib.sha256(clock.read_bytes()).hexdigest(),
                'generator_sha256': hashlib.sha256(Path(__file__).read_bytes()).hexdigest(),
                'exclusive_sections': [name + '_usec' for name in NAMES],
                'founding_groups_status': 'unavailable_separate_callback',
                'ordinary_statements_preserved': True}
    output.with_suffix('.manifest.json').write_text(json.dumps(manifest, indent=2) + '\n')
    print(json.dumps(manifest, indent=2))
    return 0


if __name__ == '__main__':
    raise SystemExit(main())
