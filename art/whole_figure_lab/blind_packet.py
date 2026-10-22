# /// script
# requires-python = ">=3.11"
# dependencies = []
# ///
# How to run: python3 -m art.whole_figure_lab.blind_packet LAB PACKET
"""Copy anonymous observed playback sequences only, never private attribute keys."""
import json
import shutil
import sys
from pathlib import Path

from .build import sha


def build(lab: Path, output: Path) -> None:
    """Publish two seconds of actual captured playback for every native-scale GIF."""
    playback=json.loads((lab/'browser/playback.json').read_text())
    if playback['page_errors'] or len(playback['specimens'])!=24:
        raise SystemExit('Playback incomplete or page errors present')
    output.mkdir(parents=True,exist_ok=False)
    records=[]
    for record in playback['specimens']:
        folder=lab/'browser/playback'/f'{record["id"]}_{record["zoom"]}x'
        files=[]
        for number in (1,2):
            source=folder/f'watched_contact_{number:02d}.png'
            destination=output/f'{record["id"]}_{record["zoom"]}x_sequence{number}.png'
            shutil.copyfile(source,destination)
            files.append({'file':destination.name,'sha256':sha(destination)})
        records.append({'id':record['id'],'zoom':record['zoom'],'observed_sequences':files,
                        'source_gif_sha256':record['source_sha256'],'unique_screenshots':record['unique_screenshots']})
    status=(lab/'DATASET_STATUS.txt').read_text().splitlines()[0]
    (output/'manifest.json').write_text(json.dumps({'dataset_status':status,
        'formal_R1_eligible':playback.get('formal_R1_eligible',False),
        'method':playback['method'],'target_capture_hz':8,'browser':playback['browser'],
        'records':records,'answer_key_included':False},indent=2)+'\n')
    (output/'README.md').write_text('''# Blind playback review

Do not inspect sibling directories, sample plans, code, or answer keys. This packet
contains only anonymous IDs and real Chromium playback screenshot contact sheets.
For each of 12 IDs at BOTH scales, inspect sequence1 and sequence2 in chronological
order (left-to-right, top-to-bottom). Each page shows eight observed samples with
actual elapsed timestamps, not decoded GIF frames or authored poses substituted
for playback. Each sample shows two facings across two ground conditions.

Report your independent reading of activity, carried item, top/clothing, hair,
sex presentation, and age group. Unknown/ambiguous is a valid answer and counts
as wrong in later scoring. Keep uncertainty verbatim. No answer key is provided.
Do not guess labels from filename or status. If status is nonconforming, this
review is exploratory only and cannot certify standard-canvas readability.

Write a JSON list with 24 rows: id, zoom, activity, item, top, hair, sex, age,
confidence, and notes. Also give a brief verbatim summary of visual problems.
''')
    print(output)


if __name__=='__main__':
    build(Path(sys.argv[1]),Path(sys.argv[2]))
