use adoc_engine::{RenderOptions, export, render};
#[test]
fn semantics() {
    let r = render(
        "= Title\n:name: Rust\n\n== Section\n\nHello *{name}*.\n\n* First\n* Second",
        &RenderOptions::default(),
    )
    .unwrap();
    assert_eq!(r.title, "Title");
    assert!(r.html.contains("<strong>Rust</strong>"));
    assert!(r.html.contains("id=\"_section\""));
    assert!(r.html.contains("<ul>"));
}
#[test]
fn safe_modes() {
    assert!(
        render(
            "hello",
            &RenderOptions {
                safe_mode: "invalid".into(),
                ..Default::default()
            }
        )
        .is_err()
    );
}
#[test]
fn source_lines() {
    let r = render(
        "= Title\n\nBody",
        &RenderOptions {
            source_locations: true,
            ..Default::default()
        },
    )
    .unwrap();
    assert!(r.html.contains("data-source-line=\"3\""));
}
#[cfg(feature = "epub")]
#[test]
fn epub_container() {
    let artifacts = export(
        "= Book\n\n== Chapter\n\nHello *reader*.\n\n|===\n|One |Two\n|===",
        "epub",
        &RenderOptions::default(),
    )
    .unwrap();
    let mut zip = zip::ZipArchive::new(std::io::Cursor::new(&artifacts[0].1)).unwrap();
    let first = zip.by_index(0).unwrap();
    assert_eq!(first.name().unwrap(), "mimetype");
    assert_eq!(first.compression(), zip::CompressionMethod::Stored);
    drop(first);
    for name in ["EPUB/_preamble.xhtml", "EPUB/package.opf", "EPUB/nav.xhtml"] {
        let mut text = String::new();
        std::io::Read::read_to_string(&mut zip.by_name(name).unwrap(), &mut text).unwrap();
        roxmltree::Document::parse(&text).unwrap();
    }
}
#[cfg(any(feature = "pdf", feature = "svg", feature = "png"))]
#[test]
fn publication_formats() {
    for format in ["pdf", "svg", "png"]
        .into_iter()
        .filter(|format| adoc_engine::supported_formats().contains(format))
    {
        let artifacts = export(
            "= Book\n\n== Chapter\n\nHello *reader*.",
            format,
            &RenderOptions::default(),
        )
        .unwrap();
        let b = &artifacts[0].1;
        match format {
            "pdf" => assert!(b.starts_with(b"%PDF")),
            "svg" => {
                roxmltree::Document::parse(std::str::from_utf8(b).unwrap()).unwrap();
            }
            _ => assert!(b.starts_with(b"\x89PNG")),
        }
    }
}
#[test]
fn unsupported_format() {
    assert!(export("text", "docx", &RenderOptions::default()).is_err());
}

#[cfg(feature = "full")]
#[test]
fn local_images_and_jail() {
    let dir = std::env::temp_dir().join(format!("adoc-engine-images-{}", std::process::id()));
    std::fs::create_dir_all(&dir).unwrap();
    let image = dir.join("picture.svg");
    std::fs::write(&image,r#"<svg xmlns="http://www.w3.org/2000/svg" width="10" height="10"><rect width="10" height="10" fill="red"/></svg>"#).unwrap();
    let source = "= Images\n\nimage::picture.svg[Square]";
    assert!(
        export(
            source,
            "epub",
            &RenderOptions {
                base_dir: dir.clone(),
                ..Default::default()
            }
        )
        .is_err()
    );
    let options = RenderOptions {
        base_dir: dir.clone(),
        safe_mode: "safe".into(),
        ..Default::default()
    };
    for format in ["epub", "pdf", "svg"] {
        assert!(!export(source, format, &options).unwrap()[0].1.is_empty());
    }
    let outside = dir
        .parent()
        .unwrap()
        .join(format!("outside-{}.svg", std::process::id()));
    std::fs::write(&outside, std::fs::read(&image).unwrap()).unwrap();
    let traversal = format!(
        "image::../{}[]",
        outside.file_name().unwrap().to_string_lossy()
    );
    assert!(export(&traversal, "epub", &options).is_err());
    std::fs::remove_dir_all(dir).unwrap();
    std::fs::remove_file(outside).unwrap();
}

#[test]
fn overlays_and_semantic_catalog() {
    let root = std::env::temp_dir();
    let child = root.join("adoc-engine-unsaved.adoc");
    let options = RenderOptions {
        base_dir: root,
        safe_mode: "safe".into(),
        source_locations: true,
        overlays: std::collections::BTreeMap::from([(
            child,
            "== Child\n\n[[anchor]]\nUnsaved.".into(),
        )]),
        ..Default::default()
    };
    let a =
        adoc_engine::analyze("= Main\n\ninclude::adoc-engine-unsaved.adoc[]", &options).unwrap();
    assert!(a.rendered.html.contains("Unsaved."));
    assert!(a.symbols.iter().any(|s| s.name == "anchor"));
}
#[cfg(feature = "pdf")]
#[test]
fn compound_cells_and_passthrough_are_preserved() {
    let source = "= Test\n\n[cols=\"1\"]\n|===\na|\n* Nested cell text\n|===\n\n++++\n<p>Visible <strong>HTML text</strong>.</p>\n++++";
    let opts = RenderOptions::default();
    let doc = asciidoc_html5::load_with(source, &opts.parser_options().unwrap());
    let (markup, _) = adocers_typst::markup(&doc, std::path::Path::new("."), &Default::default());
    assert!(markup.contains("Nested cell text"), "{markup}");
    assert!(markup.contains("HTML text"), "{markup}");
    assert!(
        export(source, "pdf", &opts).unwrap()[0]
            .1
            .starts_with(b"%PDF")
    );
}
#[cfg(feature = "epub")]
#[test]
fn epub_handles_entities_and_distinct_identifiers() {
    let opts = RenderOptions::default();
    for source in [
        "= First\n\n++++\n<p>&copy; &nbsp; <br> Text</p>\n++++",
        "= Second\n\nDifferent.",
    ] {
        let artifacts = export(source, "epub", &opts).unwrap();
        assert!(!artifacts[0].1.is_empty());
    }
}

#[test]
fn supplied_diagrams_all_publications() {
    let diagram = "@startuml\nAlice -> Bob\n@enduml";
    let source = format!("= Diagrams\n\n[plantuml]\n----\n{diagram}\n----");
    let svg = r#"<svg xmlns="http://www.w3.org/2000/svg" width="40" height="20"><rect width="40" height="20" fill="red"/></svg>"#;
    let options = RenderOptions {
        diagrams: std::collections::BTreeMap::from([(diagram.into(), svg.into())]),
        ..Default::default()
    };
    let result = render(&source, &options).unwrap();
    assert!(result.html.contains("<svg"), "{}", result.html);
    for format in ["epub", "pdf", "svg", "svg-merged", "png"]
        .into_iter()
        .filter(|format| adoc_engine::supported_formats().contains(format))
    {
        let artifacts = export(&source, format, &options).unwrap();
        assert!(!artifacts[0].1.is_empty());
    }
}
#[cfg(feature = "pdf")]
#[test]
fn passthrough_cannot_bypass_asset_policy() {
    let root = std::env::temp_dir();
    let file = root.join(format!("adoc-secret-{}.svg", std::process::id()));
    std::fs::write(
        &file,
        r#"<svg xmlns="http://www.w3.org/2000/svg" width="5" height="5"/>"#,
    )
    .unwrap();
    let source = format!("++++\n<img src=\"{}\">\n++++", file.display());
    assert!(export(&source, "pdf", &RenderOptions::default()).is_err());
    std::fs::remove_file(file).unwrap();
}
#[test]
fn hostile_diagram_rejected() {
    let options = RenderOptions {
        diagrams: std::collections::BTreeMap::from([(
            "source".into(),
            "<svg><script>alert(1)</script></svg>".into(),
        )]),
        ..Default::default()
    };
    assert!(export("[source]\n----\nsource\n----", "html", &options).is_err());
}

#[test]
fn aliases_preserve_nested_file_context() {
    let root = std::env::temp_dir().join(format!("adoc-alias-{}", std::process::id()));
    std::fs::create_dir_all(root.join("nested")).unwrap();
    std::fs::write(
        root.join("nested/part.adoc"),
        "== Included\n\ninclude::leaf.adoc[]",
    )
    .unwrap();
    std::fs::write(root.join("nested/leaf.adoc"), "Nested alias content.").unwrap();
    let options = RenderOptions {
        base_dir: root.clone(),
        safe_mode: "safe".into(),
        include_aliases: std::collections::BTreeMap::from([(
            "module:part.adoc".into(),
            root.join("nested/part.adoc"),
        )]),
        ..Default::default()
    };
    let result = render("include::module:part.adoc[]", &options).unwrap();
    assert!(result.html.contains("Nested alias content."), "{result:?}");
    std::fs::remove_dir_all(root).unwrap();
}

#[cfg(feature = "epub")]
#[test]
fn diagram_namespace_is_well_formed() {
    let options=RenderOptions{diagrams:std::collections::BTreeMap::from([("diagram".into(),r#"<svg xmlns="http://www.w3.org/2000/svg" width="5" height="5"><rect width="5" height="5"/></svg>"#.into())]),..Default::default()};
    let artifact = export("[source]\n----\ndiagram\n----", "epub", &options).unwrap();
    let mut zip = zip::ZipArchive::new(std::io::Cursor::new(&artifact[0].1)).unwrap();
    let mut content = String::new();
    std::io::Read::read_to_string(
        &mut zip.by_name("EPUB/_preamble.xhtml").unwrap(),
        &mut content,
    )
    .unwrap();
    assert!(!content.contains(":xmlns="), "{content}");
}

#[cfg(feature = "pdf")]
#[test]
fn publication_links_to_nonsection_blocks() {
    let source = "= References\n\nSee <<table-anchor,the table>> and <<paragraph-anchor,the paragraph>>.\n\n[[paragraph-anchor]]\nA destination.\n\n[[table-anchor]]\n|===\n|Cell\n|===";
    let options = RenderOptions::default();
    let doc = asciidoc_html5::load_with(source, &options.parser_options().unwrap());
    let (markup, _) = adocers_typst::markup(&doc, std::path::Path::new("."), &Default::default());
    assert!(
        markup.contains("#link(label(\"table-anchor\"))"),
        "{markup}"
    );
    assert!(
        markup.contains("#link(label(\"paragraph-anchor\"))"),
        "{markup}"
    );
    assert!(
        export(source, "pdf", &options).unwrap()[0]
            .1
            .starts_with(b"%PDF")
    );
}
#[test]
fn included_diagnostics_keep_their_file() {
    let dir = std::env::temp_dir();
    let options = RenderOptions {
        base_dir: dir.clone(),
        safe_mode: "safe".into(),
        overlays: std::collections::BTreeMap::from([(
            dir.join("diagnostic-child.adoc"),
            "include::absent-child-file.adoc[]".into(),
        )]),
        ..Default::default()
    };
    let result = render("include::diagnostic-child.adoc[]", &options).unwrap();
    assert!(
        result.diagnostics.iter().any(|d| d
            .file
            .as_ref()
            .is_some_and(|s| s.ends_with("diagnostic-child.adoc"))),
        "{result:?}"
    );
}
