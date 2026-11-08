#!/usr/bin/env python3
"""End-to-end stdio protocol test against the built executable."""
import json,subprocess,sys
p=subprocess.Popen([sys.argv[1],'lsp'],stdin=subprocess.PIPE,stdout=subprocess.PIPE)
def send(method,params=None,id=None):
 v={'jsonrpc':'2.0','method':method,'params':params or {}}
 if id is not None:v['id']=id
 b=json.dumps(v).encode();p.stdin.write(f'Content-Length: {len(b)}\r\n\r\n'.encode()+b);p.stdin.flush()
def receive():
 headers={}
 while True:
  line=p.stdout.readline()
  assert line,'server exited'
  if line==b'\r\n':break
  k,v=line.decode().split(':',1);headers[k.lower()]=v.strip()
 return json.loads(p.stdout.read(int(headers['content-length'])))
try:
 send('initialize',id=1);assert receive()['result']['capabilities']['textDocumentSync']==2
 send('initialized')
 uri='file:///tmp/smoke.adoc'
 send('textDocument/didOpen',{'textDocument':{'uri':uri,'version':1,'text':'= Test\n:name: Rust\n\nHello 😀.'}})
 assert receive()['method']=='textDocument/publishDiagnostics'
 send('textDocument/didChange',{'textDocument':{'uri':uri,'version':2},'contentChanges':[{'range':{'start':{'line':3,'character':6},'end':{'line':3,'character':8}},'text':'World'}]})
 assert receive()['params']['version']==2
 send('asciidoc/render',{'textDocument':{'uri':uri}},id=2)
 r=receive()['result'];assert r['version']==2 and 'Hello World.'in r['html']
 send('textDocument/completion',{'textDocument':{'uri':uri},'position':{'line':3,'character':0}},id=3)
 assert any(e['label']=='name'for e in receive()['result'])
 send('shutdown',id=4);assert receive()['result']is None
 send('exit');p.wait(timeout=10);assert p.returncode==0
 print('LSP stdio smoke test passed')
finally:
 if p.poll()is None:p.kill();p.wait()
