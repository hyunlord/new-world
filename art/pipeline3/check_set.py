# /// script
# requires-python = ">=3.11"
# dependencies = ["pillow", "numpy"]
# ///
# How to run: python -m art.pipeline3.check_set manifest.json --output output_directory
"""Route-wide structural report, inventory closure, and rejection-only board."""
import hashlib
import json
import sys
from dataclasses import asdict
from pathlib import Path
from PIL import Image, ImageDraw
from .check_layer import Finding, Result, check_layer
from .layer_schema import Layer, load_layer
from .failure_marks import marked_preview
from .set_checks import combinations, fixed_upper, fixed_overlays
from .structure import FRAMES

def inventory(layers: list[Layer], directions: list[str], route: str = 'A') -> list[Result]:
    """Fail closed for absent directions, frames, derived bodies or variant counts."""
    failures: list[Finding] = []
    if not directions or not {'S','E'}.issubset(directions) or not set(directions).issubset({'S','E','N'}):
        failures.append(Finding('inventory','directions require S,E and optional N'))
    ids = [l.id for l in layers]
    if len(ids) != len(set(ids)):
        failures.append(Finding('inventory','duplicate layer ids'))
    if route not in ('A', 'B'):
        failures.append(Finding('inventory', 'route must be A or B'))
    bodies = ('adult',) if route == 'B' else ('adult','child','elder')
    if route == 'B' and any(layer.body_type != 'adult' for layer in layers):
        failures.append(Finding('inventory', 'TRIAL3B scope is adult only'))
    for body in bodies:
        subset = [l for l in layers if l.body_type == body]
        for direction in directions:
            for frame in FRAMES:
                count = sum(l.kind == 'body' and l.direction == direction and l.frame == frame for l in subset)
                if count != 1:
                    failures.append(Finding('inventory',f'{body}/{direction}/{frame}: expected1 body, found{count}'))
            for kind in ('top','hair'):
                count = sum(l.kind == kind and l.direction == direction for l in subset)
                required = {'hide_wrap','fur_cloak'} if kind == 'top' else {'short','long_tied'}
                variants = {l.variant for l in subset if l.kind == kind and l.direction == direction}
                if count < 2 or not required.issubset(variants):
                    failures.append(Finding('inventory',f'{body}/{direction}/{kind}: required variants {sorted(required)}, found {sorted(variants)}'))
        if sum(l.kind == 'body' and l.frame == 'sleep' for l in subset) != 1:
            failures.append(Finding('inventory',f'{body}: expected1 sleep'))
        for kind in ('item','carry'):
            variant = 'stone_axe' if kind == 'item' else 'log_bundle'
            if not any(l.kind == kind and l.variant == variant for l in subset):
                failures.append(Finding('inventory',f'{body}: missing{kind}'))
    return [Result('minimum_set',not failures,tuple(failures))]

def failure_board(layers: list[Layer], results: list[Result], destination: Path) -> None:
    """Draw every rejected layer with rules, red border and diagnostic region marks."""
    rejected = [(l,[f for r in results if r.id == l.id and not r.passed for f in r.findings]) for l in layers]
    rejected = [(l,f) for l,f in rejected if f]
    rows = max(1,(len(rejected)+3)//4)
    board = Image.new('RGB',(1024,rows*460),'#ebe6dc')
    draw = ImageDraw.Draw(board)
    if not rejected:
        draw.text((20,20),'No rejected layers. Inventory/combination failures remain in CHECKS.',fill='black')
    for index,(layer,findings) in enumerate(rejected):
        x,y = index%4*256,index//4*460
        try:
            preview = marked_preview(layer,findings)
            preview.thumbnail((248,350))
            board.paste(preview,(x+4,y+24),preview)
        except OSError as error:
            draw.text((x+4,y+40),str(error)[:30],fill='red')
        draw.rectangle((x+2,y+22,x+252,y+378),outline='red',width=3)
        draw.text((x+4,y+4),layer.id[:34],fill='black')
        for line,finding in enumerate(findings[:6]):
            draw.text((x+4,y+380+line*12),finding.rule+': '+finding.detail[:18],fill='#990000')
    board.save(destination)

def run(manifest: Path, output: Path) -> bool:
    """Produce reports even when the route rejects; return structural acceptance."""
    raw = json.loads(manifest.read_text())
    layers = [load_layer(manifest.parent / p) for p in raw['layers']]
    results = [check_layer(layer) for layer in layers]
    valid = [l for l,r in zip(layers,results,strict=True) if r.passed]
    route = raw.get('route')
    cross = fixed_overlays(layers) if route == 'B' else fixed_upper(valid)
    if route not in ('A', 'B'):
        results.append(Result('route',False,(Finding('route','route must be A or B'),)))
    results.extend(cross)
    minimum = inventory(layers,raw['directions'],route=route)
    if not layers:
        results.append(Result('nonempty',False,(Finding('inventory','no layers supplied'),)))
    combos = combinations(valid,route=route)
    if not combos:
        results.append(Result('combinations',False,(Finding('inventory','no combinations rendered'),)))
    subset_passed = all(r.passed for r in results) and all(c.passed for c in combos)
    complete = raw.get('require_minimum',True) and all(r.passed for r in minimum)
    passed = subset_passed and complete
    status = 'PASS' if passed else 'NOT_COMPLETE' if subset_passed else 'FAIL'
    results.extend(minimum)
    output.mkdir(parents=True,exist_ok=True)
    report = {'schema_version':1,'route':raw['route'],'passed':passed,'status':status,'subset_passed':subset_passed,'layer_count':len(layers),
              'layer_pass_count':sum(check_layer(l).passed for l in layers),
              'sleep_coverage':'bare_sleep_only; top/hair/item/carry attributes not covered during sleep',
              'combination_count':len(combos),'results':[asdict(r) for r in results],
              'combinations':[asdict(c) for c in combos],
              'hashes':{str(p.resolve()):hashlib.sha256(p.read_bytes()).hexdigest() for p in [manifest]+[manifest.parent / p for p in raw['layers']]+[p for l in layers for p in (l.image,l.mask)]+[l.anatomical_mask for l in layers if l.anatomical_mask is not None] if p.exists()}}
    report['input_hashes'] = report['hashes']
    (output/'CHECKS.json').write_text(json.dumps(report,indent=2)+'\n')
    lines = [f"# CHECKS {raw['route']}",f"\nRoute acceptance: {status}. Supplied subset structural pass: {subset_passed}.",
             f'\nLayers: {len(layers)}. Exhaustive valid-input combinations: {len(combos)}.',
             '\nSleep coverage: bare body only on96x64; top/hair/item/carry attributes are not covered in sleep.',
             '\nSynthetic checks do not establish art, readability, provenance, or FPS acceptance.\n']
    for result in results:
        lines.append(f"- {result.id}: {'PASS' if result.passed else 'FAIL'} "+'; '.join(f'{f.rule}: {f.detail}' for f in result.findings))
    for combo in combos:
        if not combo.passed:
            lines.append('- COMBINATION '+','.join(combo.ids)+': '+ '; '.join(f'{f.rule}: {f.detail}' for f in combo.findings))
    (output/'CHECKS.md').write_text('\n'.join(line.rstrip() for line in lines)+'\n')
    failure_board(layers,results,output/'failures.png')
    return passed

def main() -> int:
    """CLI intentionally accepts only manifest plus output directory."""
    if len(sys.argv) != 4 or sys.argv[2] != '--output':
        print('usage: python -m art.pipeline3.check_set MANIFEST --output DIRECTORY',file=sys.stderr)
        return 2
    try:
        return 0 if run(Path(sys.argv[1]),Path(sys.argv[3])) else 1
    except (OSError,ValueError,TypeError,KeyError) as error:
        print(f'Invalid route input: {error}',file=sys.stderr)
        return 2

if __name__ == '__main__':
    raise SystemExit(main())
