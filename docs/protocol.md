# Protocol version 1

`adoc-engine lsp` uses Content-Length framed JSON-RPC on stdio. Send `initialize`,
then `initialized`; finish with `shutdown` and `exit`. Requests before initialization
return -32002. Invalid parameters return -32602 without terminating the server.
Unknown methods return -32601. Cancelled requests return -32800; invalidated document
snapshots return -32801. Worker queues and messages are bounded; timeout is 60 seconds.

Capabilities advertise UTF-16 incremental synchronization, document symbols,
completion, hover, definition, and experimental AsciiDoc rendering/export.
`didOpen`/`didChange` versions must increase. Changes apply atomically. Invalid UTF-16
ranges leave the current document intact and produce a log notification. Closing a
document clears its diagnostics. Unsaved open files overlay disk content for includes.

## Custom requests

`asciidoc/render` accepts `{textDocument: {uri}, options}` or explicit `{text, options}`.
Options use camelCase: `safeMode`, `baseDir`, `inputFile`, `attributes`, `overlays`
(absolute file paths to text), `includeAliases` (logical target to absolute path),
`diagrams` (diagram source to passive SVG), `standalone`, `sourceLocations`.
The result contains `html`, `title`, `diagnostics`, `sourceMap`, `dependencies`, and
`version`. Source mapping records use `output_line`, `file`, `line`; lines are 1-based.
Language-server positions and symbol lines use 0-based UTF-16 positions.

`asciidoc/export` additionally accepts `format` and `outputDirectory`; after conversion
succeeds and the snapshot is still current, artifacts are written there and returned
as paths. Rendering work cancelled before completion does not publish artifacts.

## Persistent worker

`adoc-engine worker` is a separate JSON-lines transport for the benchmark and native
plugin client. Each request contains `source`, optional `options`, and optional
`action` (`render`, `analyze`, or `export`). Export also specifies `format` and returns
named byte arrays; analysis returns the parsed semantic symbol index. Render responses
include conversion `nanos`; timing excludes request parsing and response serialization.
Malformed requests return `{error: ...}` and the worker accepts subsequent requests.
