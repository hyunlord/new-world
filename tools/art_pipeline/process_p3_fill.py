# /// script
# requires-python = ">=3.10"
# dependencies = ["Pillow", "numpy"]
# ///
# How to run: python3 tools/art_pipeline/process_p3_fill.py --sheet P3_fill_01 --raw /path/attempt_1.png
"""Fit sixteen new physical cells and freeze independent review without acceptance."""
from __future__ import annotations

import argparse
from collections import defaultdict
from copy import deepcopy
import json
from pathlib import Path
import shutil

from fill_catalog import load_catalog
from fill_v4 import process
from regions_v2 import table_from_data
from reprocess_p3 import ROOT, OUTPUT as REPROCESS, Reprocessed, boards, digest, freeze_review_index, reprocess
from salvage_v4 import CandidateRecord, guard_data, guarded_candidate
from v4_contracts import guard_group


class FillProcessingError(ValueError):
    """Physical source coverage or preservation checks failed."""


def physical_sources(records: list[CandidateRecord], sheet_id: str) -> list[CandidateRecord]:
    """Require exactly sixteen unique original cells; exclude derivations and context."""
    physical = [row for row in records if row['source_kind'] == 'generated' and row['sheet'] == sheet_id]
    if sorted(row['cell'] for row in physical) != list(range(1, 17)):
        raise FillProcessingError('Expected exactly sixteen unique original physical cells')
    if any(row.get('budget_alias') is not None for row in physical):
        raise FillProcessingError('Part 3 villager fill cells cannot alias another physical sheet')
    return sorted(physical, key=lambda row: row['cell'])


def failed_source(ancestor: CandidateRecord, source_path: Path) -> Reprocessed:
    """Keep unconvertible raw cells visible without inventing a fitted image."""
    row: Reprocessed = deepcopy(ancestor)
    row.update(id=ancestor['id'] + ':p3', ancestor_id=ancestor['id'], source_candidate=ancestor['id'],
        ancestor_record=deepcopy(ancestor), ancestor_ledger=str(source_path.relative_to(ROOT)),
        ancestor_ledger_sha256=digest(source_path), raw_sha256=digest(Path(ancestor['raw_path'])),
        target_slot=ancestor['target_slot'], coverage_reasons=['new_fill_physical_cell'], images={},
        guards={'errors':['conversion_failure'], 'warnings':[], 'passed':False},
        review_status='pending_independent_visual_review')
    row.pop('group_guards', None)
    row.pop('cross_guards', None)
    return row


def run(sheet_id: str, raw_path: Path) -> Path:
    """Process one authorized raw sheet, preserving full ancestry and frozen maps."""
    catalog_path = ROOT / 'docs/art/production3/fills/catalog.json'
    catalog = load_catalog(catalog_path)
    if sheet_id not in {sheet['id'] for sheet in catalog['sheets']}:
        raise FillProcessingError('Sheet not present in the authorized catalog')
    output = ROOT / 'docs/art/production3/fills' / sheet_id
    ledger_path = output / 'candidates.json'
    if ledger_path.exists() or (output / 'review_index.json').exists():
        raise FillProcessingError('Refusing to change previously frozen review evidence')
    output.mkdir(parents=True, exist_ok=True)
    source_path = output / 'raw_candidates.json'
    if source_path.exists():
        source = json.loads(source_path.read_text())
        if source['raw_sha256'] != digest(raw_path):
            raise FillProcessingError('Raw input changed since the first processing attempt')
    else:
        source = process(ROOT, catalog, sheet_id, raw_path, 1, source_path, output / 'raw_cells')
    physical = physical_sources(source['candidates'], sheet_id)
    shutil.copytree(REPROCESS / 'references', output / 'references', dirs_exist_ok=True)
    frozen = json.loads((output / 'references/tables.json').read_text())
    tables = {body: table_from_data(data) for body, data in frozen['tables'].items()}
    records: list[Reprocessed] = []
    for ancestor in physical:
        if ancestor['factor'] is None:
            records.append(failed_source(ancestor, source_path))
        else:
            records.append(reprocess({'record': ancestor, 'ledger': str(source_path.relative_to(ROOT)),
                                      'reasons': ['new_fill_physical_cell']}, tables, output))
    groups: dict[tuple[str, str, int, str, str], list[Reprocessed]] = defaultdict(list)
    for row in records:
        if row['guards']['passed']:
            groups[(row['body'],row['sheet'],row['attempt'],row['view'],row['activity'])].append(row)
    for members in groups.values():
        report = guard_data(guard_group([guarded_candidate(row) for row in members]))
        for row in members:
            row['group_guards'] = report
    board_index = boards(records, output)
    ledger = {'schema_version':1, 'scale':source['scale'], 'references':source['references'],
        'candidates':records + source['candidates'], 'reprocessed_ids':[r['id'] for r in records],
        'reference_tables':str((output / 'references/tables.json').relative_to(ROOT)),
        'acceptance_status':'pending_independent_visual_review', 'boards':board_index,
        'physical_attempted_cells':16, 'source_ledger':str(source_path.relative_to(ROOT)),
        'source_ledger_sha256':digest(source_path), 'raw_sha256':digest(raw_path),
        'catalog_sha256':digest(catalog_path)}
    ledger_path.write_text(json.dumps(ledger,indent=2,allow_nan=False)+'\n')
    (output / 'board_index.json').write_text(json.dumps(board_index,indent=2)+'\n')
    freeze_review_index(output)
    return ledger_path


def main() -> None:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--sheet', required=True)
    parser.add_argument('--raw', type=Path, required=True)
    args = parser.parse_args()
    print(run(args.sheet, args.raw.resolve()))


if __name__ == '__main__':
    main()
