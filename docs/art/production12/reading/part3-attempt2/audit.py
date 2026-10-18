import collections, hashlib, json, pathlib, re, struct
ROOT=pathlib.Path('/Users/rexxa/work/art-production12-evidence/p3/live-attempt2')
OUT=pathlib.Path('/Users/rexxa/work/art-production12-evidence/p3/attempt2-audit')
CASES=['live','multi','estranged','crowd1','crowd2','crowd3']
FIELDS=['tick','kind','subject','other','third','settlement','settlement2','cause','value']
def digest(p): return hashlib.sha256(p.read_bytes()).hexdigest()
def rows(p): return [{k:p[k][i] for k in FIELDS} for i in range(len(p['kind']))]
def plain(t): return re.sub(r'\[[^]]+\]','',t).replace('\u2060','')
missing=[str(ROOT/(c+'-result.json')) for c in CASES if not (ROOT/(c+'-result.json')).exists()]
if not (ROOT/'private-reader-map.json').exists():missing.append('private-reader-map.json')
if missing:raise SystemExit('NOT_READY: '+', '.join(missing))
reg=json.loads((ROOT/'registration.json').read_text());mapping=json.loads((ROOT/'private-reader-map.json').read_text())
report={'head':reg['head'],'method':'Fresh attempt2 raw records and sidecars only. No reader answers read. PNG dimensions/hash checked without editing/rescaling. Text/scroll coverage is not pixel readability.','issues':[],'cases':{},'image_map':[]}
for case in CASES:
 refpath=ROOT/case/'reference-data.json';ref=json.loads(refpath.read_text());result=json.loads((ROOT/(case+'-result.json')).read_text())
 full=[r for w in ref['reference_worlds'] for p in w['chronicle_pages'] for r in rows(p)]
 entry={'result':result,'reference_sha256':digest(refpath),'reference_world_keys':[list(w) for w in ref['reference_worlds']], 'world_ticks':[w['clock']['tick'] for w in ref['reference_worlds']],'kind_counts':dict(collections.Counter(r['kind'] for r in full)),'departure_causes':sorted({r['cause'] for r in full if r['kind']==5}),'subjects':{},'raw_relations_events':[r for r in full if r['kind']>=17],'images':[],'scroll_groups':{}}
 groups=collections.defaultdict(list)
 for section,cap in ref['captures'].items():
  if not isinstance(cap,dict) or 'chronicle' not in cap:continue
  events=rows(cap['chronicle']);entry['subjects'][section]={'raw':cap,'sidecar_text':{}}
  for r in events:
   if r not in full:report['issues'].append({'case':case,'section':section,'person_row_missing_full':r})
 for f in sorted((ROOT/case).glob('*.json')):
  if f.name=='reference-data.json' or 'crowd-proof' in f.name:continue
  d=json.loads(f.read_text())
  if 'file' not in d:continue
  png=ROOT/case/d['file'];head=png.read_bytes()[:24];dims=struct.unpack('>II',head[16:24]);p=d['provenance'];c=d.get('page_context',{})
  check={'file':str(png),'dimensions':dims,'hash_matches':digest(png)==d['png_sha256'],'head_matches':p['head']==reg['head'],'bridge_matches':p['loaded_bridge_sha256']==reg['loaded_bridge_sha256'],'clock_tick':d['clock']['tick']}
  entry['images'].append(check)
  if not all(check[k] for k in ['hash_matches','head_matches','bridge_matches']):report['issues'].append(check)
  for key in ['life_text','rendered_text']:
   if key not in c:continue
   text=plain(c[key]);hits=re.findall(r'\b(?:CHRON|INSPECTOR|LIFE)_[A-Z0-9_]+\b|%[-+0-9.]*[sdf]',text)
   if hits:report['issues'].append({'file':str(f),'placeholder_hits':hits})
   if c.get('section') in entry['subjects']:entry['subjects'][c['section']]['sidecar_text'][c['language']]=text
  if 'scroll_value' in c:groups[(c['section'],c['language'])].append({'file':str(f),'value':c['scroll_value'],'page':c['scroll_page'],'maximum':c['scroll_max']})
 for key,gs in groups.items():
  gs.sort(key=lambda g:g['value']);covered=0;gaps=[]
  for g in gs:
   if g['value']>covered+1:gaps.append([covered,g['value']])
   covered=max(covered,g['value']+g['page'])
  maximum=max(g['maximum'] for g in gs)
  if covered<maximum-1:gaps.append([covered,maximum])
  entry['scroll_groups']['/'.join(key)]={'pages':gs,'starts_at_top':gs[0]['value']==0,'ends_at_bottom':covered>=maximum-1,'gaps':gaps}
  if gaps:report['issues'].append({'case':case,'scroll_group':key,'gaps':gaps})
 report['cases'][case]=entry
for m in mapping:
 p=pathlib.Path(m['source']);target=ROOT/'reader'/m['file'];ok=digest(p)==m['sha256']==digest(target);report['image_map'].append({'file':m['file'],'source':str(p),'hash_matches':ok})
 if not ok:report['issues'].append({'map_hash_mismatch':m['file']})
report['coverage_limits']=['Raw text presence/scroll range does not prove glyph legibility or absence of clipped lines.','Kind18/21, porter subtype and departure coverage are enumerated from raw records; absent cases are not inferred to pass.']
(OUT/'audit.json').write_text(json.dumps(report,ensure_ascii=False,indent=2)+'\n')
print(json.dumps({'cases':len(report['cases']),'images':len(report['image_map']),'issues':report['issues'],'raw_keys':{c:report['cases'][c]['reference_world_keys'] for c in CASES}},indent=2))
