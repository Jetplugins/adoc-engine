const asciidoctor = require('@asciidoctor/core')();
const lines = require('node:readline').createInterface({input:process.stdin});
lines.on('line', line => {
 const {source, options = {}} = JSON.parse(line);
 const start = process.hrtime.bigint();
 const html = asciidoctor.convert(source, {safe:options.safeMode || 'secure',header_footer:false,base_dir:options.baseDir || '.',attributes:options.attributes || {}});
 const nanos = Number(process.hrtime.bigint()-start);
 process.stdout.write(JSON.stringify({html,nanos})+'\n');
});
