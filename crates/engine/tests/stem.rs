use adoc_engine::{RenderOptions, export, render};
use std::io::{Cursor, Read};

const SOURCE: &str = r"= Equations
:stem: latexmath
:stem-renderer: native

Inline stem:[\frac{a}{b}] then asciimath:[sqrt(x^2+y^2)].

[stem]
++++
\sum_{i=1}^{n} i = \frac{n(n+1)}{2}
++++

[latexmath]
++++
\begin{pmatrix}a & b \\ c & d\end{pmatrix}
++++

[source,latex]
----
\(\frac{literal}{code}\)
----
";

fn math_nodes(html: &str) -> Vec<(String, String)> {
    // The generated math fragments are also valid XML.
    let re = regex::Regex::new(r"(?s)<math\b.*?</math>").unwrap();
    re.find_iter(html)
        .map(|capture| {
            let doc = roxmltree::Document::parse(capture.as_str()).unwrap();
            let root = doc.root_element();
            (
                root.attribute("display").unwrap_or("inline").into(),
                root.attribute("alttext").unwrap().into(),
            )
        })
        .collect()
}

#[test]
fn native_html_renders_mixed_inline_and_block_math_without_scripts() {
    for standalone in [false, true] {
        let options = RenderOptions {
            standalone,
            source_locations: true,
            ..Default::default()
        };
        let result = render(SOURCE, &options).unwrap();
        assert!(!result.html.contains("MathJax"));
        assert!(result.html.contains("<mfrac>"), "{}", result.html);
        assert!(result.html.contains("<msqrt>"));
        assert!(result.html.contains("<mtable"));
        assert!(result.html.contains(r"\(\frac{literal}{code}\)"));
        let equations = math_nodes(&result.html);
        assert_eq!(equations.len(), 4);
        assert_eq!(
            equations.iter().map(|eq| eq.0.as_str()).collect::<Vec<_>>(),
            ["inline", "inline", "block", "block"]
        );
        assert!(result.html.contains(" then "));
        assert!(!result.source_map.is_empty());
        let exported = export(SOURCE, "html", &options).unwrap();
        assert_eq!(exported[0].1, result.html.as_bytes());
        if standalone {
            assert!(result.html.contains("<head>"));
        }
    }
}

#[test]
fn default_html_retains_asciidoctor_mathjax_contract() {
    let source = SOURCE.replace(":stem-renderer: native\n", "");
    let result = render(
        &source,
        &RenderOptions {
            standalone: true,
            ..Default::default()
        },
    )
    .unwrap();
    assert!(result.html.contains("MathJax"));
    assert!(result.html.contains(r"\(\frac{a}{b}\)"));
    assert!(!result.html.contains("<math"));
}

#[test]
fn epub_has_static_latex_mathml_and_manifest_properties() {
    let artifacts = export(SOURCE, "epub", &RenderOptions::default()).unwrap();
    let mut zip = zip::ZipArchive::new(Cursor::new(&artifacts[0].1)).unwrap();
    let mut body = String::new();
    zip.by_name("EPUB/_preamble.xhtml")
        .unwrap()
        .read_to_string(&mut body)
        .unwrap();
    assert_eq!(math_nodes(&body).len(), 4);
    assert!(body.contains("<mfrac>"));
    assert!(body.contains("<mtable"));
    assert!(body.contains("<mstyle scriptlevel=\"0\""));
    assert!(body.contains("http://www.w3.org/1998/Math/MathML"));
    roxmltree::Document::parse(&body).unwrap();
    let mut opf = String::new();
    zip.by_name("EPUB/package.opf")
        .unwrap()
        .read_to_string(&mut opf)
        .unwrap();
    assert!(opf.contains("properties=\"mathml\""));
}

#[test]
fn invalid_latex_and_excessive_math_return_errors() {
    for expression in [
        r"\thiscommanddoesnotexist{x}".into(),
        "x".repeat(32 * 1024 + 1),
    ] {
        let source = format!(
            "= Invalid\n:stem: latexmath\n:stem-renderer: native\n\nlatexmath:[{expression}]\n"
        );
        for format in ["html", "epub"] {
            assert!(
                export(&source, format, &RenderOptions::default()).is_err(),
                "{format}"
            );
        }
    }
}

#[test]
fn tex_alias_and_block_notation_override_typeset_in_publications() {
    let source = "= Maths\n:stem: tex\n\n[stem]\n++++\n\\frac{a}{b}\n++++\n\n[stem,latexmath]\n++++\n\\sqrt{4}\n++++\n\n[asciimath]\n++++\nsqrt(9)\n++++\n";
    for format in ["pdf", "svg", "svg-merged", "png"] {
        assert!(
            !export(source, format, &RenderOptions::default()).unwrap()[0]
                .1
                .is_empty(),
            "{format}"
        );
    }
}

#[test]
fn native_html_can_be_selected_by_api_without_changing_prose_or_code() {
    let mut options = RenderOptions::default();
    options
        .attributes
        .insert("stem-renderer".into(), "native".into());
    let source = "= Text\n\nBefore & after latexmath:[x^2] with *bold* text.\n\n`\\(literal\\)`\n\n[asciimath]\n++++\nsqrt(4)\n++++\n";
    let result = render(source, &options).unwrap();
    assert!(result.html.contains("Before &amp; after "));
    assert!(result.html.contains(" with <strong>bold</strong> text."));
    assert!(result.html.contains("<code>\\(literal\\)</code>"));
    assert_eq!(
        math_nodes(&result.html)
            .iter()
            .map(|eq| eq.0.as_str())
            .collect::<Vec<_>>(),
        ["inline", "block"]
    );
}
