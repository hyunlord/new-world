# /// script
# requires-python = ">=3.10"
# dependencies = ["Pillow", "numpy"]
# ///
# How to run: python3 tools/art_pipeline/audit_manifest_v4.py --output REPORT.json
"""Audit installed PNG coverage; explicit flags register region pages or sizes."""
from __future__ import annotations

import argparse
from collections import Counter
from copy import deepcopy
import json
from pathlib import Path
import re

from PIL import Image
from salvage_verdicts import atomic_json


def normalized_path(root: Path, value: str) -> str:
    """Reject resources outside the asset root and collapse path aliases."""
    path=(root/value.removeprefix('res://')).resolve()
    path.relative_to((root/'assets/art').resolve())
    return path.relative_to(root.resolve()).as_posix()


def audit_manifest(root: Path, manifest: dict, *, register_region_pages: bool=False,
                   record_dimensions: bool=False) -> tuple[dict,dict]:
    """Return updated metadata and audit without writing or deleting any assets."""
    updated=deepcopy(manifest); entries=updated['assets'];registered=[];repairs=[]
    counts=Counter(normalized_path(root,entry['path']) for entry in entries)
    if register_region_pages:
        region_path=root/'assets/art/atlas/region_atlas.json'
        regions=json.loads(region_path.read_text())
        for color,region in sorted(regions['pages'].items()):
            path=normalized_path(root,region);color_path=normalized_path(root,color)
            if not re.fullmatch(r'assets/art/atlas/world_[0-9]+_regions\.png',path):
                raise ValueError(f'Not a named region atlas page: {path}')
            if color_path!=path.replace('_regions.png','.png') or not (root/color_path).is_file():
                raise ValueError(f'Region page lacks paired color atlas: {path}')
            if not (root/path).is_file():raise ValueError(f'Missing region page: {path}')
            if counts[path]:continue
            with Image.open(root/path) as image:size=list(image.size)
            entries.append({'path':path,'source':'procedural:tools/art_pipeline/regions.py',
                'native_size':size,'anchor':[0,0],'qa_status':'format_checked','role':'region_index_atlas'})
            counts[path]+=1;registered.append(path)
    disk={path.relative_to(root).as_posix():path for path in (root/'assets/art').rglob('*')
          if path.is_file() and path.suffix.lower()=='.png'}
    sizes={};nonbinary=[];unreadable={}
    for path,file in sorted(disk.items()):
        try:
            with Image.open(file) as source:
                sizes[path]=list(source.size)
                histogram=source.convert('RGBA').getchannel('A').histogram()
                if sum(histogram[1:255]):nonbinary.append(path)
        except (OSError,ValueError) as error:
            unreadable[path]=str(error)
    mismatches=[]
    for entry in entries:
        path=normalized_path(root,entry['path'])
        if path not in sizes:continue
        # Native dimensions of an outlined sprite describe a different image.
        field='canvas_size' if 'canvas_size' in entry or entry.get('sprite_id') else 'native_size'
        recorded=entry.get(field);actual=sizes[path]
        if recorded!=actual:
            change={'path':path,'field':field,'recorded':recorded,'actual':actual}
            if record_dimensions:entry[field]=actual;repairs.append(change)
            else:mismatches.append(change)
    atlas_path=root/'assets/art/atlas/world_atlas.json'
    atlas=json.loads(atlas_path.read_text()) if atlas_path.exists() else {}
    suppressed=set(updated.get('suppressed',{}))|set(atlas.get('suppressed',{}))
    suppressed.update(entry['sprite_id'] for entry in entries if entry.get('sprite_id') and
                      (entry.get('drawable') is False or entry.get('qa_status')=='suppressed'))
    drawable={identity for identity,row in atlas.get('sprites',{}).items() if row.get('drawable',True)}
    suppressed_drawable=sorted(suppressed & drawable)
    png_registered={path for path in counts if Path(path).suffix.lower()=='.png'}
    report={'schema_version':1,'png_count':len(disk),'registered_png_count':len(png_registered),
        'unregistered_pngs':sorted(set(disk)-png_registered),
        'missing_pngs':sorted(png_registered-set(disk)),
        'duplicate_paths':{path:count for path,count in sorted(counts.items()) if count!=1},
        'nonbinary_alpha':nonbinary,'unreadable_pngs':unreadable,'size_mismatches':mismatches,
        'atlas_manifest_missing':not atlas_path.exists(),'suppressed_drawable':suppressed_drawable,
        'registered_region_pages':registered,'dimension_repairs':repairs,
        'measured_sizes':sizes}
    report['passed']=not any(report[key] for key in ('unregistered_pngs','missing_pngs','duplicate_paths',
        'nonbinary_alpha','unreadable_pngs','size_mismatches','atlas_manifest_missing','suppressed_drawable'))
    updated.setdefault('audits',{})['manifest_v4']=deepcopy(report)
    return updated,report


def main() -> None:
    """Persist accurate audit metadata, with other mutations explicitly opt-in."""
    parser=argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--root',type=Path,default=Path.cwd())
    parser.add_argument('--register-region-pages',action='store_true')
    parser.add_argument('--record-dimensions',action='store_true')
    parser.add_argument('--output',type=Path,required=True)
    args=parser.parse_args();root=args.root.resolve();path=root/'assets/art/manifest.json'
    if args.output.resolve()==path:raise ValueError('Report must not overwrite manifest')
    updated,report=audit_manifest(root,json.loads(path.read_text()),
        register_region_pages=args.register_region_pages,record_dimensions=args.record_dimensions)
    atomic_json(args.output,report);atomic_json(path,updated)
    print(json.dumps({key:report[key] for key in ('passed','png_count','registered_png_count')}))
    raise SystemExit(0 if report['passed'] else 1)


if __name__=='__main__':main()
