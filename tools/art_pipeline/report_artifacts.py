"""Publish real capture before/after plates and bounded animated GIFs."""
from __future__ import annotations
import argparse
import json
from pathlib import Path
from PIL import Image, ImageDraw, ImageFont


def before_after(raw: Path, output: Path, repo: Path, before_dir: Path | None = None) -> None:
    rows = [('d2_1200_F2','a_d2_1200_F2','Noon village'),
            ('d2_2200_F2N','a_d2_2300_F2N','Night village'),
            ('d2_1200_F1','a_d2_1200_F1','Overview'),
            ('d2_1200_F4','a_d2_1200_F4','Inspector')]
    if before_dir is not None:
        rows = [('a_d2_1200_F2','a_d2_1200_F2','Noon village'),('a_d2_2300_F2N','a_d2_2300_F2N','Night village'),('a_d2_1200_F1','a_d2_1200_F1','Overview')]
    canvas = Image.new('RGB',(1920,len(rows)*570),(25,27,28))
    draw = ImageDraw.Draw(canvas)
    font = ImageFont.truetype(str(repo/'assets/fonts/galmuri/Galmuri11.ttf'),18)
    for index,(old,new,title) in enumerate(rows):
        draw.text((12,index*570+5),title+' — before / after',font=font,fill='white')
        for column,path in enumerate([(before_dir/f'{old}.png' if before_dir is not None else repo/f'docs/revitalize/shots/final/{old}.jpg'),raw/f'{new}.png']):
            if not path.exists():
                raise FileNotFoundError(path)
            with Image.open(path) as source:
                panel = source.convert('RGB').resize((960,540),Image.Resampling.NEAREST)
            canvas.paste(panel,(column*960,index*570+30))
    canvas.save(output/'before_after.png')


def animation(raw: Path, output: Path, manifest_name: str, filename: str) -> dict[str,int|str]:
    frames_data = json.loads((raw/manifest_name).read_text())['frames']
    if len(frames_data)!=40:
        raise RuntimeError('GIF requires all 40 captured frames')
    frames = []
    for entry in frames_data:
        with Image.open(raw/entry['file']) as source:
            frames.append(source.convert('RGB').resize((960,540),Image.Resampling.NEAREST))
    target = output/filename
    for colors in [128,64,32]:
        palette = frames[0].quantize(colors=colors,method=Image.Quantize.MEDIANCUT)
        indexed = [frame.quantize(palette=palette,dither=Image.Dither.NONE) for frame in frames]
        timestamps = [entry['real_time_usec'] for entry in frames_data]
        durations = [max(10,round((b-a)/10000)*10) for a,b in zip(timestamps,timestamps[1:])] + [100]
        indexed[0].save(target,save_all=True,append_images=indexed[1:],duration=durations,loop=0,optimize=True,disposal=2)
        if target.stat().st_size<=8_000_000:
            with Image.open(target) as encoded:
                encoded_frames = encoded.n_frames
            return {'file':filename,'bytes':target.stat().st_size,'source_frames':40,'encoded_frames':encoded_frames,'colors':colors}
    raise RuntimeError('GIF exceeds 8 MB after bounded palette reductions')


def season_strip(raw: Path, output: Path, repo: Path) -> None:
    font = ImageFont.truetype(str(repo/'assets/fonts/galmuri/Galmuri11.ttf'),18)
    for filename, days in [('season_strip.png', [10,18,24,40]),
                           ('four_seasons_actual.png', [10,18,24,32])]:
        canvas = Image.new('RGB',(1920,310),(25,27,28))
        draw = ImageDraw.Draw(canvas)
        for index, day in enumerate(days):
            stem = f'l_d{day}_1200_F2'
            metadata = json.loads((raw/f'{stem}.entities.json').read_text())
            clock = metadata['time_snapshot']
            season = ['봄','여름','가을','겨울'][clock['season']]
            title = f'{day}일차 · {season} {clock["day_of_season"]}일째'
            draw.text((index*480+8,8),title,font=font,fill='white')
            with Image.open(raw/f'{stem}.png') as image:
                canvas.paste(image.convert('RGB').resize((480,270),Image.Resampling.NEAREST),(index*480,40))
        canvas.save(output/filename)

if __name__ == '__main__':
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('raw',type=Path)
    parser.add_argument('--output',type=Path,default=Path('docs/art/production'))
    parser.add_argument('--before-dir',type=Path)
    parser.add_argument('--long-raw',type=Path)
    parser.add_argument('--repo',type=Path,default=Path('.'))
    args = parser.parse_args()
    args.output.mkdir(parents=True,exist_ok=True)
    before_after(args.raw,args.output,args.repo,args.before_dir)
    if args.long_raw is not None:
        season_strip(args.long_raw,args.output,args.repo)
    evidence = [animation(args.raw,args.output,'strip.json','village_noon.gif'),
                animation(args.raw,args.output,'evening.json','village_evening.gif')]
    budgets = {str(folder):sum(path.stat().st_size for path in folder.rglob('*') if path.is_file()) for folder in [args.repo/'assets/art',args.output]}
    report = {'animations':evidence,'budget_bytes':budgets,'assets_limit_bytes':25_000_000,'docs_limit_bytes':40_000_000}
    (args.output/'report_artifacts.json').write_text(json.dumps(report,indent=2)+'\n')
    print(json.dumps(report))
    if budgets[str(args.repo/'assets/art')]>25_000_000 or budgets[str(args.output)]>40_000_000:
        raise RuntimeError('Committed artifact byte budget exceeded; see report_artifacts.json')
