# Publication backend patch

adocers-typst 0.3.2 is vendored under its original MIT OR Apache-2.0 license.
Changes: recursively render AsciiDoc table cells; preserve semantic HTML passthrough
text, emphasis, and links through the existing inline translator.
Arbitrary browser CSS and JavaScript have no paginated equivalent and are not executed.

Further release fixes: explicit destinations for non-section block anchors; controlled
resource maps prevent filesystem reads in the emitter; supplied SVG diagrams are
embedded without invoking an external diagram runtime.
