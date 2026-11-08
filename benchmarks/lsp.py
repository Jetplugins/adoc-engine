#!/usr/bin/env python3
"""End-to-end LSP render latency with the two persistent native workers."""
import argparse,hashlib,json,math,queue,statistics,subprocess,threading,time
from pathlib import Path
p=argparse.ArgumentParser();p.add_argument('--rust',type=Path,default=Path('target/release/adoc-engine'));p.add_argument('--samples',type=int,default=50);p.add_argument('--update-readme',action='store_true');a=p.parse_args();binary=a.rust.resolve();root=Path(__file__).resolve().parents[1];results=[]
process=subprocess.Popen([str(binary),'lsp'],stdin=subprocess.PIPE,stdout=subprocess.PIPE,stderr=subprocess.PIPE);responses=queue.Queue()
def reader():
 while True:
  headers={}
  while True:
   line=process.stdout.readline()
   if not line:return
   if line==b'\r\n':break
   key,value=line.decode().split(':',1);headers[key.lower()]=value.strip()
  responses.put(json.loads(process.stdout.read(int(headers['content-length']))))
threading.Thread(target=reader,daemon=True).start();serial=0
def request(method,params=None):
 global serial
 serial+=1;payload=json.dumps({'jsonrpc':'2.0','id':serial,'method':method,'params':params or {}}).encode();started=time.perf_counter_ns();process.stdin.write(f'Content-Length: {len(payload)}\r\n\r\n'.encode()+payload);process.stdin.flush();response=responses.get(timeout=120);elapsed=time.perf_counter_ns()-started;assert response['id']==serial and 'error'not in response,response;return response['result'],elapsed
try:
 request('initialize')
 for count in [10,1000,10000]:
  source='= LSP benchmark\n\n'+('A *formatted* paragraph.\n\n'*count);values=[]
  for i in range(a.samples+10):
   result,elapsed=request('asciidoc/render',{'text':source});assert result['html']
   if i>=10:values.append(elapsed)
  results.append({'paragraphs':count,'inputBytes':len(source.encode()),'samplesNanos':values,'medianMs':statistics.median(values)/1e6,'p95Ms':sorted(values)[math.ceil(.95*len(values))-1]/1e6});print(f'LSP/{count}: {results[-1]["medianMs"]:.2f} ms',flush=True)
 request('shutdown');payload=b'{"jsonrpc":"2.0","method":"exit"}';process.stdin.write(f'Content-Length: {len(payload)}\r\n\r\n'.encode()+payload);process.stdin.flush();process.wait(timeout=10);assert process.returncode==0
finally:
 if process.poll()is None:process.kill();process.wait()
 for stream in (process.stdin,process.stdout,process.stderr):stream.close()
output=root/'benchmarks/baselines/release';output.mkdir(parents=True,exist_ok=True);(output/'lsp.json').write_text(json.dumps({'binarySha256':hashlib.sha256(binary.read_bytes()).hexdigest(),'method':'JSON-RPC round trip including IPC, native worker conversion and response decoding; ten warmups per case.','results':results},indent=2)+'\n')
if a.update_readme:
 section=['### Language-server performance','',f'End-to-end JSON-RPC render latency with persistent native workers, ten warm-ups and {a.samples} measured requests per case. IPC and client response decoding are included.','','| Paragraphs | Median ms | p95 ms |','|---|---:|---:|']+[f"| {r['paragraphs']} | {r['medianMs']:.2f} | {r['p95Ms']:.2f} |"for r in results]+['','[Raw LSP samples](benchmarks/baselines/release/lsp.json).']
 readme=root/'README.md';text=readme.read_text();start='<!-- LSP-PERF:START -->';end='<!-- LSP-PERF:END -->';block=start+'\n'+'\n'.join(section)+'\n'+end
 if start in text:text=text[:text.index(start)]+block+text[text.index(end)+len(end):]
 else:text+='\n'+block+'\n'
 readme.write_text(text)
