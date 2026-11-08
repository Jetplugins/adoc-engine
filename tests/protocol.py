#!/usr/bin/env python3
"""Black-box release gates. Every read has a deadline; no hanging test suite."""
import json,os,queue,subprocess,sys,tempfile,threading,time,unittest
from pathlib import Path
from urllib.parse import urljoin
BINARY=str(Path(sys.argv.pop(1)).resolve()) if len(sys.argv)>1 else str(Path('target/debug/adoc-engine').resolve())
class Client:
 def __init__(self):
  self.p=subprocess.Popen([BINARY,'lsp'],stdin=subprocess.PIPE,stdout=subprocess.PIPE,stderr=subprocess.PIPE);self.q=queue.Queue();self.serial=0
  def reader():
   try:
    while True:
     headers={}
     while True:
      line=self.p.stdout.readline()
      if not line:return
      if line==b'\r\n':break
      k,v=line.decode().split(':',1);headers[k.lower()]=v.strip()
     self.q.put(json.loads(self.p.stdout.read(int(headers['content-length']))))
   except BaseException as e:self.q.put(e)
  threading.Thread(target=reader,daemon=True).start()
 def send(self,method,params=None,id=None):
  v={'jsonrpc':'2.0','method':method,'params':params or {}}
  if id is not None:v['id']=id
  self.raw(json.dumps(v).encode())
 def raw(self,b):self.p.stdin.write(f'Content-Length: {len(b)}\r\n\r\n'.encode()+b);self.p.stdin.flush()
 def receive(self,id=None):
  deadline=time.monotonic()+15
  while True:
   result=self.q.get(timeout=max(.01,deadline-time.monotonic()))
   if isinstance(result,BaseException):raise result
   if id is None or result.get('id')==id:return result
 def request(self,method,params=None):
  self.serial+=1;self.send(method,params,self.serial);return self.receive(self.serial)
 def initialize(self):return self.request('initialize')
 def open(self,uri,text,version=1):self.send('textDocument/didOpen',{'textDocument':{'uri':uri,'version':version,'text':text}})
 def close(self):
  if self.p.poll()is None:
   try:self.request('shutdown');self.send('exit');self.p.wait(timeout=10)
   finally:
    if self.p.poll()is None:self.p.kill();self.p.wait()
  for stream in (self.p.stdin,self.p.stdout,self.p.stderr):stream.close()
class ProtocolTests(unittest.TestCase):
 def setUp(self):self.c=Client();self.tmp=tempfile.TemporaryDirectory();self.root=Path(self.tmp.name)
 def tearDown(self):self.c.close();self.tmp.cleanup()
 def test_lifecycle_and_invalid_requests(self):
  self.assertEqual(self.c.request('asciidoc/render',{'text':'hello'})['error']['code'],-32002)
  self.assertEqual(self.c.initialize()['result']['capabilities']['positionEncoding'],'utf-16')
  self.c.raw(b'{broken');self.assertEqual(self.c.receive()['error']['code'],-32700)
  for params in ({'text':'hello','options':{'safeMode':'bogus'}},{'text':'hello','options':42},{}):
   self.assertEqual(self.c.request('asciidoc/render',params)['error']['code'],-32602)
  self.assertIn('hello',self.c.request('asciidoc/render',{'text':'hello'})['result']['html'])
  self.assertEqual(self.c.request('unknown')['error']['code'],-32601)
 def test_unsaved_includes_semantics_and_navigation(self):
  self.c.initialize();root=(self.root/'main.adoc').as_uri();child=(self.root/'part.adoc').as_uri()
  self.c.open(child,'== Unsaved section\n\n[[target]]\nIncluded text.');self.c.open(root,'= Main\n\ninclude::part.adoc[]\n\nSee <<target>>.')
  params={'textDocument':{'uri':root},'options':{'safeMode':'safe','sourceLocations':True}}
  result=self.c.request('asciidoc/render',params)['result'];self.assertIn('Included text.',result['html']);self.assertTrue(result['dependencies']);self.assertTrue(result['sourceMap'])
  labels=[v['label']for v in self.c.request('textDocument/completion',params)['result']];self.assertIn('target',labels);self.assertIn('_unsaved_section',labels)
  definition=self.c.request('textDocument/definition',params|{'position':{'line':4,'character':8}})['result'];self.assertEqual(definition['uri'],child)
 def test_invalid_edits_atomic_and_unicode(self):
  self.c.initialize();uri=(self.root/'main.adoc').as_uri();self.c.open(uri,'= Main\n\nA😀B')
  self.c.send('textDocument/didChange',{'textDocument':{'uri':uri,'version':2},'contentChanges':[{'range':{'start':{'line':2,'character':1},'end':{'line':2,'character':2}},'text':'bad'}]})
  r=self.c.request('asciidoc/render',{'textDocument':{'uri':uri}})['result'];self.assertIn('A😀B',r['html']);self.assertEqual(r['version'],1)
  self.c.send('textDocument/didChange',{'textDocument':{'uri':uri,'version':3},'contentChanges':[{'range':{'start':{'line':2,'character':1},'end':{'line':2,'character':3}},'text':'X'}]})
  self.assertIn('AXB',self.c.request('asciidoc/render',{'textDocument':{'uri':uri}})['result']['html'])
 def test_cancel_and_stale_requests(self):
  self.c.initialize();uri=(self.root/'main.adoc').as_uri();self.c.open(uri,'= Main\n\nInitial')
  large='= Large\n\n'+('Paragraph *bold*.\n\n'*100000)
  self.c.send('asciidoc/render',{'text':large},100);self.c.send('$/cancelRequest',{'id':100})
  self.assertEqual(self.c.receive(100)['error']['code'],-32800)
  self.c.send('asciidoc/render',{'textDocument':{'uri':uri},'text':large},101)
  self.c.send('textDocument/didChange',{'textDocument':{'uri':uri,'version':2},'contentChanges':[{'text':'= Changed\n\nNew'}]})
  self.assertEqual(self.c.receive(101)['error']['code'],-32801)
  self.assertIn('New',self.c.request('asciidoc/render',{'textDocument':{'uri':uri}})['result']['html'])
 def test_included_diagnostics_use_child_uri(self):
  self.c.initialize();child=self.root/'child.adoc';child.write_text('include::absent.adoc[]');root=(self.root/'main.adoc').as_uri()
  self.c.send('textDocument/didOpen',{'textDocument':{'uri':root,'version':1,'text':'= Main\n\ninclude::child.adoc[]'},'options':{'safeMode':'safe'}})
  notifications=[self.c.receive(),self.c.receive()]
  self.assertTrue(any(n.get('params',{}).get('uri')==child.as_uri()and n['params']['diagnostics']for n in notifications),notifications)
 def test_outline_excludes_listing_and_attributes(self):
  self.c.initialize();uri=(self.root/'main.adoc').as_uri();self.c.open(uri,'= Main\n\n----\n== Fake\n:hidden: no\n----\n\n== Real\n\n:visible: yes')
  params={'textDocument':{'uri':uri}}
  self.assertEqual([v['name']for v in self.c.request('textDocument/documentSymbol',params)['result']],['Real'])
  labels=[v['label']for v in self.c.request('textDocument/completion',params)['result']];self.assertIn('visible',labels);self.assertNotIn('hidden',labels)
if __name__=='__main__':unittest.main(verbosity=2)
