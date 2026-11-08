gem 'asciidoctor', '= 2.0.26'
require 'asciidoctor'
require 'json'
STDOUT.sync = true
STDIN.each_line do |line|
 request = JSON.parse(line)
 options = request.fetch('options', {})
 start = Process.clock_gettime(Process::CLOCK_MONOTONIC, :nanosecond)
 html = Asciidoctor.convert(request.fetch('source'), safe: options.fetch('safeMode','secure').to_sym, header_footer: false, base_dir: options.fetch('baseDir','.'), attributes: options.fetch('attributes',{}))
 nanos = Process.clock_gettime(Process::CLOCK_MONOTONIC, :nanosecond)-start
 puts JSON.generate({html: html, nanos: nanos})
end
