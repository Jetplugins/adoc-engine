use adoc_engine::{RenderOptions, export, render, supported_formats};

#[test]
fn capabilities_match_executable_backends_and_missing_features_are_actionable() {
    let options = RenderOptions::default();
    let mut expected = vec!["html"];
    for (format, enabled) in [
        ("pdf", cfg!(feature = "pdf")),
        ("epub", cfg!(feature = "epub")),
        ("svg", cfg!(feature = "svg")),
        ("svg-merged", cfg!(feature = "svg")),
        ("png", cfg!(feature = "png")),
    ] {
        if enabled {
            expected.push(format);
            let artifacts = export("= Features\n\nHello *reader*.", format, &options).unwrap();
            assert!(!artifacts.is_empty());
            for (_, bytes) in artifacts {
                match format {
                    "pdf" => assert!(bytes.starts_with(b"%PDF")),
                    "epub" => assert!(bytes.starts_with(b"PK")),
                    "png" => assert!(bytes.starts_with(b"\x89PNG")),
                    _ => {
                        roxmltree::Document::parse(std::str::from_utf8(&bytes).unwrap()).unwrap();
                    }
                }
            }
        } else {
            // A disabled backend is diagnosed before parsing or reading assets.
            let error = export(
                "include::missing.adoc[]",
                format,
                &RenderOptions {
                    safe_mode: "invalid".into(),
                    ..Default::default()
                },
            )
            .unwrap_err()
            .to_string();
            let feature = if format == "svg-merged" {
                "svg"
            } else {
                format
            };
            assert!(
                error.contains(&format!("'{feature}' Cargo feature")),
                "{error}"
            );
        }
    }
    expected.push("json");
    assert_eq!(supported_formats(), expected);
    assert!(
        render("= HTML\n\nHello *reader*.", &options)
            .unwrap()
            .html
            .contains("<strong>reader</strong>")
    );
    let json = export("= JSON\n\nHello.", "json", &options).unwrap();
    assert_eq!(
        serde_json::from_slice::<serde_json::Value>(&json[0].1).unwrap()["title"],
        "JSON"
    );
    assert!(
        export("", "docx", &options)
            .unwrap_err()
            .to_string()
            .contains("unsupported format")
    );
}

#[test]
fn native_mathml_is_explicit_and_never_silently_ignored() {
    let source = "= Maths\n:stem: latexmath\n:stem-renderer: native\n\nlatexmath:[\\frac{a}{b}]\n";
    let result = render(source, &Default::default());
    assert_eq!(result.is_ok(), cfg!(feature = "mathml"));
    match result {
        Ok(result) => assert!(result.html.contains("<mfrac>")),
        Err(error) => assert!(error.to_string().contains("'mathml' Cargo feature")),
    }
}

#[cfg(feature = "epub")]
#[test]
fn epub_highlighting_is_explicit_and_never_silently_ignored() {
    let source =
        "= Code\n:source-highlighter: highlightjs\n\n[source,rust]\n----\nfn main() {}\n----\n";
    let mut options = RenderOptions::default();
    options
        .attributes
        .insert("source-highlighter".into(), "highlightjs".into());
    let result = export(source, "epub", &options);
    assert_eq!(result.is_ok(), cfg!(feature = "highlighting"));
    if let Err(error) = result {
        assert!(error.to_string().contains("'highlighting' Cargo feature"));
    }
}
