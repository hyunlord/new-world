#!/usr/bin/env python3
"""PERF2-0: markdown tables from perf2-sim JSON (timing/, sub/) and perf2_frame.gd JSON (render/).

usage: summarize.py <data-dir>   (data-dir holds sim/timing, sim/sub, render/<matrix>/*.json)
"""
import glob
import json
import math
import os
import statistics
import sys


def load(path):
    with open(path) as fh:
        return json.load(fh)


def sim_tables(root):
    out = []
    runs = {}
    for f in sorted(glob.glob(os.path.join(root, 'sim/timing/n*-r*.json'))):
        d = load(f)
        n = int(os.path.basename(f).split('-')[0][1:])
        runs.setdefault(n, []).append(d)
    out.append('| n | reps | tick mean ms (median [min-max]) | p50 ms | p99 ms | max ms | TPS (1 core) | allocs/tick mean (p99) | alloc MB/tick | RSS end MB | live heap end MB | save MB | save ms | load ms |')
    out.append('|---:|---:|---|---:|---:|---:|---:|---|---:|---:|---:|---:|---:|---:|')
    for n in sorted(runs):
        rs = runs[n]
        means = [r['tick_us']['mean'] / 1000 for r in rs]
        med = lambda key, sub: statistics.median(r[key][sub] for r in rs)
        r0 = rs[0]
        out.append('| %d | %d | %.3f [%.3f-%.3f] | %.2f | %.2f | %.1f | %.0f | %.0f (%.0f) | %.2f | %.0f | %.0f | %.2f | %.0f | %.0f |' % (
            n, len(rs), statistics.median(means), min(means), max(means), med('tick_us', 'p50') / 1000, med('tick_us', 'p99') / 1000,
            med('tick_us', 'max') / 1000, statistics.median(r['ticks_per_s_achieved'] for r in rs),
            med('alloc_count_per_tick', 'mean'), med('alloc_count_per_tick', 'p99'), med('alloc_bytes_per_tick', 'mean') / 1e6,
            r0['memory']['rss_kib_end'] / 1024, r0['memory']['live_heap_end'] / 1e6, r0['save']['bytes'] / 1e6,
            statistics.median(r['save']['save_ms'] for r in rs), statistics.median(r['save']['load_ms'] for r in rs)))
    out.append('')
    # Determinism: reps 1 and 2.
    out.append('| n | day fingerprints equal (r1 vs r2) | end save fnv64 r1 / r2 | load round trip fingerprint |')
    out.append('|---:|---|---|---|')
    for n in sorted(runs):
        rs = {r.get('_rep'): r for r in runs[n]}
        a = load(os.path.join(root, f'sim/timing/n{n}-r1.json'))
        b = load(os.path.join(root, f'sim/timing/n{n}-r2.json'))
        fa = [x['fingerprint'] for x in a['days_rows']]
        fb = [x['fingerprint'] for x in b['days_rows']]
        out.append('| %d | %s (%d/%d) | %s / %s | %s |' % (n, 'IDENTICAL' if fa == fb else 'DIFFER', sum(x == y for x, y in zip(fa, fb)), len(fa),
                                                     a['save']['fnv64'], b['save']['fnv64'], a['save']['load_fingerprint_equal']))
    out.append('')
    # Per-system at each n (rep 1), with the scaling exponent 100 -> 1000.
    base = {s['name']: s for s in load(os.path.join(root, 'sim/timing/n100-r1.json'))['systems']}
    mid = {s['name']: s for s in load(os.path.join(root, 'sim/timing/n300-r1.json'))['systems']}
    top = load(os.path.join(root, 'sim/timing/n1000-r1.json'))
    out.append('| system (n=1000) | interval | share | mean us/tick | p50 | p99 | max | us/call | n100 mean | n300 mean | alpha 100->1000 |')
    out.append('|---|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|')
    for s in top['systems'][:24]:
        x = s['ns_per_tick']
        b = base.get(s['name'], {}).get('ns_per_tick', {}).get('mean', 0)
        m = mid.get(s['name'], {}).get('ns_per_tick', {}).get('mean', 0)
        alpha = math.log(x['mean'] / b) / math.log(10) if b > 500 and x['mean'] > 0 else float('nan')
        out.append('| %s | %d | %.1f %% | %.1f | %.1f | %.1f | %.1f | %.1f | %.1f | %.1f | %s |' % (
            s['name'], s['interval'], 100 * s['share'], x['mean'] / 1e3, x['p50'] / 1e3, x['p99'] / 1e3, x['max'] / 1e3,
            s['mean_us_per_call'], b / 1e3, m / 1e3, '-' if math.isnan(alpha) else '%.2f' % alpha))
    out.append('')
    out.append('Slowest steady ticks (n=1000, rep 1): ' + '; '.join(
        'tick %d %.1f ms (%s)' % (t['tick'], t['ms'], ', '.join('%s %.1f' % (a, b) for a, b in t['top'][:2])) for t in top['slowest_ticks'][:8]))
    out.append('')
    out.append('| day (n=1000) | alive | archetypes | RSS MB | live heap MB | chronicle entries | buildings | tick ms (day sum) | nav requests | A* searches | nodes expanded |')
    out.append('|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|')
    for r in top['days_rows']:
        if r['day'] in (1, 2, 5, 10, 15, 20, 25, 30) or r is top['days_rows'][-1]:
            out.append('| %d | %d | %d | %.0f | %.1f | %d | %d | %.0f | %d | %d | %d |' % (
                r['day'], r['alive'], r['archetypes'], r['rss_kib'] / 1024, r['live_heap_bytes'] / 1e6, r['chronicle_entries'],
                r['buildings'], r['tick_ms_sum'], r['nav']['requests'], r['nav']['searches'], r['nav']['expanded']))
    out.append('')
    return out


def sub_tables(root):
    out = []
    for f in sorted(glob.glob(os.path.join(root, 'sim/sub/n*.json'))):
        d = load(f)
        sc = d.get('subprofile_scopes', {})
        rows = sorted(sc.items(), key=lambda kv: -kv[1]['total_ms'])
        mind = sc.get('mind.total', {}).get('mean_us_per_tick', 0)
        out.append('Subprofile %s (tick mean %.3f ms with timers; ColonyMind total %.1f us/tick):' % (os.path.basename(f), d['tick_us']['mean'] / 1000, mind))
        out.append('')
        out.append('| scope | us/tick | calls/tick | us/call | max us/tick |')
        out.append('|---|---:|---:|---:|---:|')
        for name, v in rows[:30]:
            out.append('| %s | %.1f | %.1f | %.2f | %.0f |' % (name, v['mean_us_per_tick'], v['mean_calls_per_tick'], v['mean_us_per_call'], v['max_us_per_tick']))
        nav = [r['nav'] for r in d['days_rows'] if r['nav'].get('search_ms') is not None]
        if nav:
            ms = sum(x['search_ms'] for x in nav)
            ticks = d['ticks']
            out.append('')
            out.append('A* search: %.1f ms over %d days = %.1f us/tick; %d searches, %d expanded nodes, %d requests (%.0f %% memo hits).' % (
                ms, len(nav), ms * 1000 / ticks, sum(x['searches'] for x in nav), sum(x['expanded'] for x in nav),
                sum(x['requests'] for x in nav), 100 * sum(x['memo_hits'] for x in nav) / max(1, sum(x['requests'] for x in nav))))
        out.append('')
    return out


def render_tables(root, matrix):
    out = []
    rows = []
    for f in sorted(glob.glob(os.path.join(root, 'render', matrix, '*.json'))):
        try:
            rows.append((os.path.basename(f)[:-5], load(f)['summary']))
        except (json.JSONDecodeError, KeyError):
            continue
    out.append('| run | people | FPS | frame mean | p50 | p99 | max | >33 ms /min | >50 ms /min | draw calls | primitives (M) | objects | WorldSim ms | People ms | bridge ms (calls) | UI ms | VRAM MB | tex MB |')
    out.append('|---|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---|---:|---:|---:|')
    for name, s in rows:
        seg = s.get('segments_ms_mean', {})
        ui = sum(v for k, v in seg.items() if k.startswith('UI/') or k.startswith('MuxHud/'))
        fm = s['frame_ms']
        out.append('| %s | %d | %.1f | %.1f | %.1f | %.1f | %.1f | %.0f | %.0f | %.0f | %.1f | %.0f | %.2f | %.2f | %.2f (%.1f) | %.2f | %.0f | %.0f |' % (
            name, s['people'], s['fps'], fm['mean'], fm['p50'], fm['p99'], fm['max'], s['over_33ms_per_min'], s['over_50ms_per_min'],
            s['draw_calls_mean'], s['primitives_mean'] / 1e6, s['objects_mean'], seg.get('Main/WorldSim', 0), seg.get('World3D/People', 0),
            s['bridge_ms_mean'], s['bridge_calls_mean'], ui, s['memory']['video_mem'] / 1e6, s['memory']['texture_mem'] / 1e6))
    out.append('')
    return out, rows


if __name__ == '__main__':
    root = sys.argv[1]
    lines = sim_tables(root) + sub_tables(root)
    for m in sys.argv[2:]:
        t, _ = render_tables(root, m)
        lines += ['Render matrix ' + m + ':', ''] + t
    print('\n'.join(lines))
