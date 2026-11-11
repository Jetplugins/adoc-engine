const asciidoctor = require('@asciidoctor/core');
const lines = require('node:readline').createInterface({input:process.stdin});
async function main() {
 for await (const line of lines) {
  const {source, options = {}} = JSON.parse(line);
  const start = process.hrtime.bigint();
  const html = await asciidoctor.convert(source, {safe:options.safeMode || 'secure',header_footer:false,base_dir:options.baseDir || '.',attributes:options.attributes || {}});
  const nanos = Number(process.hrtime.bigint()-start);
  process.stdout.write(JSON.stringify({html,nanos})+'\n');
 }
}
main().catch(error => {console.error(error); process.exitCode = 1;});
