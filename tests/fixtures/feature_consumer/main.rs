use adoc_engine::{RenderOptions, analyze, export, render, supported_formats};
use std::io::{Read, Write};

fn main() {
    let args: Vec<_> = std::env::args().collect();
    if args.get(1).is_some_and(|arg| arg == "render") {
        let mut source = String::new();
        std::io::stdin().read_to_string(&mut source).unwrap();
        let artifacts = export(&source, &args[2], &Default::default()).unwrap();
        std::io::stdout().write_all(&artifacts[0].1).unwrap();
        return;
    }
    let expected: Vec<_> = args[1].split(',').collect();
    assert_eq!(supported_formats(), expected);
    let source = "= Consumer\n\n== Section\n\nA *small* document.";
    assert!(
        render(source, &Default::default())
            .unwrap()
            .html
            .contains("<strong>small</strong>")
    );
    assert!(
        !analyze(source, &Default::default())
            .unwrap()
            .symbols
            .is_empty()
    );
    for format in ["html", "json", "epub", "pdf", "svg", "svg-merged", "png"] {
        let result = export(source, format, &Default::default());
        if expected.contains(&format) {
            let artifacts = result.unwrap();
            assert!(!artifacts.is_empty());
            let bytes = &artifacts[0].1;
            match format {
                "pdf" => assert!(bytes.starts_with(b"%PDF")),
                "epub" => assert!(bytes.starts_with(b"PK")),
                "png" => assert!(bytes.starts_with(b"\x89PNG")),
                "svg" | "svg-merged" => {
                    assert!(std::str::from_utf8(bytes).unwrap().contains("<svg"))
                }
                "html" => assert!(
                    std::str::from_utf8(bytes)
                        .unwrap()
                        .contains("<strong>small</strong>")
                ),
                "json" => assert!(
                    std::str::from_utf8(bytes)
                        .unwrap()
                        .contains("\"title\": \"Consumer\"")
                ),
                _ => unreachable!(),
            }
        } else {
            let error = result.unwrap_err().to_string();
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
    let native = "= Maths\n:stem: latexmath\n:stem-renderer: native\n\nlatexmath:[\\frac{a}{b}]";
    let native = render(native, &RenderOptions::default());
    if args[2] == "true" {
        assert!(native.unwrap().html.contains("<mfrac>"));
    } else {
        assert!(
            native
                .unwrap_err()
                .to_string()
                .contains("'mathml' Cargo feature")
        );
    }
    if expected.contains(&"epub") {
        let mut options = RenderOptions::default();
        options
            .attributes
            .insert("source-highlighter".into(), "highlightjs".into());
        let highlighted = export(
            "= Code\n\n[source,rust]\n----\nfn main() {}\n----",
            "epub",
            &options,
        );
        if args[3] == "true" {
            highlighted.unwrap();
        } else {
            assert!(
                highlighted
                    .unwrap_err()
                    .to_string()
                    .contains("'highlighting' Cargo feature")
            );
        }
    }
    println!("Verified {}", args[1]);
}
