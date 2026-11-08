#!/usr/bin/env python3
"""Differential regression checks for list boundaries, captions and section numbering."""
import argparse
import hashlib
import json
from pathlib import Path
import shutil
import sys

ROOT=Path(__file__).resolve().parents[1]
sys.path.insert(0,str(ROOT/'benchmarks'))
from run import Worker,canonical


def cases():
    for depth in range(5):
        for siblings in [1,2]:
            prefix='* Parent\n'+''.join('*'*(level+1)+f' Child {level}\n' for level in range(1,depth+1))
            if depth and siblings==2:prefix+='*'*(depth+1)+' Another child\n'
            if not depth and siblings==2:prefix+='* Another parent\n'
            for title in ['.Release checklist','[title="Release checklist"]','[[release]]\n.Release checklist','.Release {counter:release-title}']:
                for checklist in [True,False]:
                    following='* [x] Build artifacts\n* [ ] Promote release\n' if checklist else '* Build artifacts\n* Promote release\n'
                    yield f'list-{depth}-{siblings}-{title}-{checklist}',prefix+'\n'+title+'\n'+following,{}
    for blank in ['', '\n']:
        yield f'list-boundary-{bool(blank)}','* Parent\n** Child\n'+blank+'.Release checklist\n* [x] Build artifacts\n',{}
    for label in ['NOTE','TIP','IMPORTANT','CAUTION','WARNING']:
        for delimiter in ['====','--',None]:
            for override in ['',',caption="Notice: "']:
                body='Verify readiness.\n'
                content=body if delimiter is None else delimiter+'\n'+body+delimiter+'\n'
                source=f'[{label}{override}]\n.Before switching traffic\n{content}\n.First example\n====\nOrdinary example.\n====\n\n.Second example\n====\nAnother example.\n====\n\nCounter: {{example-number}}\n'
                yield f'caption-{label}-{delimiter}-{override}',source,{}
        yield f'caption-nested-{label}',f'[{label}]\n.Outer admonition\n====\n.Nested example\n=====\nNested example body.\n=====\n====\n\n.Next example\n====\nNext body.\n====\n',{}
        yield f'caption-table-{label}',f'[{label}]\n.Capacity\n|===\n|One |Two\n|===\n\n.Next example\n====\nNext body.\n====\n',{}
    for doctype in ['article','book']:
        for level in [1,2,3]:
            for mode in ['', 'all', None]:
                attrs=f':doctype: {doctype}\n:toc:\n:toclevels: 4\n'+(':sectnums: '+mode+'\n' if mode is not None else '')
                if doctype=='article':attrs+=':xrefstyle: full\n'
                heading='='*(level+1)
                parents=''.join('='*(parent+1)+f' Parent {parent}\n\n' for parent in range(1,level))
                source=f'= Document\n{attrs}\n{parents}{heading} Before\n\n[bibliography]\n{heading} Special\n\n* [[[reference]]] A reference.\n\n{heading} After\n\nSee <<_special>> and <<_after>>.\n'
                yield f'bibliography-{doctype}-{level}-{mode}',source,{}
    for style in ['preface','glossary','index','colophon','dedication','acknowledgments','partintro','abstract']:
        for mode in ['', 'all']:
            source=f'= Document\n:sectnums: {mode}\n:toc:\n\n== Before\n\n[{style}]\n== Special\n\nA paragraph.\n\n== After\n'
            yield f'special-{style}-{mode}',source,{}
    for mode in ['', 'all']:
        yield f'preface-descendants-{mode}',f'= Document\n:sectnums: {mode}\n:toc:\n:toclevels: 4\n\n[preface]\n== Preface\n\n=== Child\n\n==== Grandchild\n\n== After\n',{}
    fixture=ROOT/'benchmarks/fixtures/showcase'
    yield 'complete-showcase',(fixture/'main.adoc').read_text(),{'safeMode':'safe','baseDir':str(fixture)}


def main():
    parser=argparse.ArgumentParser(description=__doc__)
    parser.add_argument('binary',type=Path)
    parser.add_argument('--node',default=shutil.which('node'))
    parser.add_argument('--ruby',default=shutil.which('ruby'))
    parser.add_argument('--output',type=Path,default=ROOT/'verification/showcase-compatibility/reference')
    args=parser.parse_args();args.output.mkdir(parents=True,exist_ok=True)
    workers={'ruby':Worker([args.ruby,str(ROOT/'benchmarks/ruby-worker.rb')]),'js':Worker([args.node,str(ROOT/'benchmarks/js-worker.cjs')]),'rust':Worker([str(args.binary.resolve()),'worker'])}
    results=[]
    try:
        for index,(name,source,options) in enumerate(cases()):
            request={'source':source,'options':{'safeMode':'secure','attributes':{},**options}}
            outputs={engine:worker.render(request)['html'] for engine,worker in workers.items()}
            expected=canonical(outputs['ruby']);parity={engine:canonical(html)==expected for engine,html in outputs.items()}
            results.append({'case':name,'sourceSha256':hashlib.sha256(source.encode()).hexdigest(),'parity':parity})
            if not all(parity.values()):
                (args.output/f'failure-{index}.adoc').write_text(source)
                for engine,html in outputs.items():(args.output/f'failure-{index}-{engine}.html').write_text(html)
                print('Mismatch:',index,name,flush=True)
    finally:
        for worker in workers.values():worker.close()
    report={'binarySha256':hashlib.sha256(args.binary.read_bytes()).hexdigest(),'cases':len(results),'passed':sum(all(r['parity'].values()) for r in results),'results':results}
    (args.output/'results.json').write_text(json.dumps(report,indent=2)+'\n')
    print(f"Showcase/reference parity: {report['passed']}/{report['cases']} cases")
    if report['passed']!=report['cases']:raise SystemExit(1)


if __name__=='__main__':main()
