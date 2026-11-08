#!/usr/bin/env python3
"""Fail a confirmed benchmark regression; compare runs on the same machine."""
import argparse,json,statistics
p=argparse.ArgumentParser();p.add_argument('baseline');p.add_argument('candidate');p.add_argument('--threshold',type=float,default=.10);a=p.parse_args()
b=json.load(open(a.baseline));c=json.load(open(a.candidate))
for key in ('machine','platform','cpu'):
 if b['metadata'][key]!=c['metadata'][key]:raise SystemExit(f'Incomparable host metadata: {key}')
failed=[]
for case in b['metadata']['cases']:
 name=case['case'];other=next((x for x in c['metadata']['cases']if x['case']==name),None)
 if not other or case['sha256']!=other['sha256']:raise SystemExit(f'Corpus mismatch: {name}')
 for mode in ('warm','cold','edit'):
  old=[x['nanos']for x in b['samples']if x['case']==name and x['engine']=='rust'and x['mode']==mode]
  new=[x['nanos']for x in c['samples']if x['case']==name and x['engine']=='rust'and x['mode']==mode]
  if old and new:
   ratio=statistics.median(new)/statistics.median(old)
   print(f'{name}/{mode}: {ratio:.3f}x baseline')
   if ratio>1+a.threshold:failed.append(f'{name}/{mode}')
if failed:raise SystemExit('Regressions need an isolated confirmation run: '+', '.join(failed))
