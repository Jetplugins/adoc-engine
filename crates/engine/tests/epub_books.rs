#![cfg(feature = "epub")]

use adoc_engine::{RenderOptions, export};
#[cfg(feature = "highlighting")]
use std::path::PathBuf;
use std::{
    collections::BTreeMap,
    io::{Cursor, Read},
};

fn book(source: &str, attributes: &[(&str, &str)]) -> BTreeMap<String, Vec<u8>> {
    let mut options = RenderOptions::default();
    options
        .attributes
        .insert("epub-fonts".into(), "none".into());
    options
        .attributes
        .insert("epub-modified".into(), "2026-11-09T00:00:00Z".into());
    options
        .attributes
        .extend(attributes.iter().map(|(k, v)| ((*k).into(), (*v).into())));
    let artifacts = export(source, "epub", &options).unwrap();
    let mut zip = zip::ZipArchive::new(Cursor::new(&artifacts[0].1)).unwrap();
    let mut files = BTreeMap::new();
    for i in 0..zip.len() {
        let mut entry = zip.by_index(i).unwrap();
        let name = entry.name().unwrap().to_string();
        let mut bytes = Vec::new();
        entry.read_to_end(&mut bytes).unwrap();
        files.insert(name, bytes);
    }
    files
}
fn text<'a>(files: &'a BTreeMap<String, Vec<u8>>, name: &str) -> &'a str {
    std::str::from_utf8(&files[name]).unwrap()
}

#[test]
fn book_chapters_relocate_forward_back_and_footnote_links() {
    let files = book(
        "= Book\n:doctype: book\n\nSee <<one>>. footnote:[Shared note]\n\n[#one]\n== One\n\nSee <<two>>.\n\n[#two]\n== Two\n\nReturn <<one>>.",
        &[],
    );
    assert!(files.contains_key("EPUB/one.xhtml"));
    assert!(files.contains_key("EPUB/two.xhtml"));
    assert!(text(&files, "EPUB/_preamble.xhtml").contains("one.xhtml#one"));
    assert!(text(&files, "EPUB/one.xhtml").contains("two.xhtml#two"));
    assert!(text(&files, "EPUB/two.xhtml").contains("one.xhtml#one"));
    assert!(
        text(&files, "EPUB/_preamble.xhtml").contains("two.xhtml#_footnotedef_1")
            || text(&files, "EPUB/_preamble.xhtml").contains("two.xhtml#_footnote_1")
    );
    assert!(text(&files, "EPUB/_preamble.xhtml").contains("epub:type=\"noteref\""));
    assert!(text(&files, "EPUB/two.xhtml").contains("epub:type=\"footnote\""));
}
#[test]
fn article_stays_in_one_file_despite_chapter_level_override() {
    let files = book(
        "= Article\nJane Writer\n:doctype: article\n:epub-chapter-level: 2\n\n== A\n\n=== Nested\n\n== B",
        &[],
    );
    let opf = roxmltree::Document::parse(text(&files, "EPUB/package.opf")).unwrap();
    assert_eq!(
        opf.descendants()
            .filter(|n| n.has_tag_name(("http://www.idpf.org/2007/opf", "itemref")))
            .count(),
        1
    );
    let content = text(&files, "EPUB/content.xhtml");
    assert!(content.contains("class=\"byline\""));
    assert!(content.contains("Jane Writer"));
}
#[test]
fn splitting_depth_and_unicode_ids_produce_safe_names_and_links() {
    let files = book(
        "= Book\n:epub-chapter-level: 2\n\n== Part\n\n[#café]\n=== Café\n\nText\n\n[#next]\n=== Next\n\nSee <<café>>.",
        &[],
    );
    assert!(files.contains_key("EPUB/caf-e9.xhtml"));
    assert!(text(&files, "EPUB/next.xhtml").contains("caf-e9.xhtml#caf%C3%A9"));
}
#[test]
fn metadata_aliases_csv_series_and_source_header_language_are_preserved() {
    let files = book(
        "= Book\nJane Writer; John Reader\n:lang: nl\n:producer: Example Press\n:copyright: Copyright Authors\n:keywords: Rust, \"books, EPUB\"\n:revdate: 2026-11-09\n:series-name: Native\n:series-volume: 3\n:series-id: urn:uuid:example\n:source: https://example.org/book\n\nText",
        &[],
    );
    let xml = roxmltree::Document::parse(text(&files, "EPUB/package.opf")).unwrap();
    let values = |name| {
        xml.descendants()
            .filter(|n| n.tag_name().name() == name)
            .filter_map(|n| n.text())
            .collect::<Vec<_>>()
    };
    assert_eq!(values("language"), ["nl"]);
    assert_eq!(values("publisher"), ["Example Press"]);
    assert_eq!(values("rights"), ["Copyright Authors"]);
    assert_eq!(values("subject"), ["Rust", "books, EPUB"]);
    assert_eq!(values("date"), ["2026-11-09"]);
    assert!(values("creator").contains(&"Jane Writer"));
    assert!(values("creator").contains(&"John Reader"));
    assert!(text(&files, "EPUB/package.opf").contains("belongs-to-collection"));
    assert!(text(&files, "EPUB/_preamble.xhtml").contains("xml:lang=\"nl\""));
}
#[test]
fn outline_and_visible_toc_have_independent_depths_and_ncx_fallback() {
    let files = book(
        "= Book\n:toc:\n:toclevels: 1\n:outlinelevels: 3\n\n== Chapter\n\n=== Detail\n\n==== Deep\n\nText",
        &[],
    );
    assert!(text(&files, "EPUB/nav.xhtml").contains("Deep"));
    assert!(!text(&files, "EPUB/toc.xhtml").contains("Detail"));
    let ncx = roxmltree::Document::parse(text(&files, "EPUB/toc.ncx")).unwrap();
    assert!(ncx.descendants().any(|n| {
        n.tag_name().name() == "navPoint"
            && n.parent()
                .is_some_and(|p| p.tag_name().name() == "navPoint")
    }));
    assert!(text(&files, "EPUB/nav.xhtml").contains("landmarks"));
}
#[test]
fn math_is_static_mathml_and_manifest_marks_mathml_content() {
    let files = book(
        "= Maths\n:stem: asciimath\n\nInline stem:[x^2/2].\n\n[asciimath]\n++++\nsqrt(x^2+y^2)\n++++",
        &[],
    );
    let body = text(&files, "EPUB/_preamble.xhtml");
    assert!(body.contains("http://www.w3.org/1998/Math/MathML"));
    assert!(body.contains("<mfrac>"));
    assert!(body.contains("<msqrt>"));
    assert!(!body.contains("\\$"));
    assert!(text(&files, "EPUB/package.opf").contains("properties=\"mathml\""));
}
#[cfg(feature = "highlighting")]
#[test]
fn highlighting_preserves_source_and_produces_static_token_classes() {
    let files = book(
        "= Code\n\n[source,rust]\n----\nfn ready() -> bool { true }\n----",
        &[("source-highlighter", "rouge")],
    );
    let body = text(&files, "EPUB/_preamble.xhtml");
    assert!(body.contains("storage type function rust"), "{body}");
    let parsed = roxmltree::Document::parse(body).unwrap();
    let code = parsed
        .descendants()
        .find(|n| n.tag_name().name() == "code")
        .unwrap();
    assert_eq!(
        code.descendants()
            .filter(|n| n.is_text())
            .filter_map(|n| n.text())
            .collect::<String>()
            .trim(),
        "fn ready() -> bool { true }"
    );
}
#[test]
fn font_embedding_is_optional_and_carries_redistribution_notices() {
    let files = book("= Fonts\n\nText", &[("epub-fonts", "bundled")]);
    assert!(files.contains_key("EPUB/fonts/LibertinusSerif-Regular.otf"));
    assert!(files.contains_key("EPUB/fonts/DejaVuSansMono.ttf"));
    assert!(text(&files, "EPUB/font-licenses.txt").contains("SIL OPEN FONT LICENSE"));
    let plain = book("= Fonts\n\nText", &[]);
    assert!(!plain.keys().any(|k| k.contains(".otf")));
}
#[test]
fn reproducible_metadata_gives_byte_identical_output() {
    let options = RenderOptions {
        attributes: BTreeMap::from([("epub-modified".into(), "2026-11-09T00:00:00Z".into())]),
        ..Default::default()
    };
    assert_eq!(
        export("= Repeat\n\nText", "epub", &options).unwrap(),
        export("= Repeat\n\nText", "epub", &options).unwrap()
    );
}
#[test]
fn invalid_references_and_metadata_fail_before_an_archive_is_returned() {
    for source in [
        "= Bad\n\nSee <<missing>>.",
        "= Bad\n:series-name: Book\n:series-volume: nope\n\nText",
        "= Bad\n:keywords: \"unclosed\n\nText",
    ] {
        assert!(
            export(source, "epub", &RenderOptions::default()).is_err(),
            "{source}"
        );
    }
}
#[cfg(feature = "highlighting")]
#[test]
fn book_fixture_preserves_includes_assets_cover_and_media() {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../tests/fixtures/epub")
        .canonicalize()
        .unwrap();
    for name in ["book.adoc", "article.adoc", "media.adoc"] {
        let source = std::fs::read_to_string(root.join(name)).unwrap();
        let options = RenderOptions {
            base_dir: root.clone(),
            input_file: Some(root.join(name)),
            safe_mode: "safe".into(),
            ..Default::default()
        };
        let result = export(&source, "epub", &options).unwrap();
        assert!(!result[0].1.is_empty());
    }
}

#[test]
fn publication_backend_and_soft_api_attributes_are_semantic() {
    let source =
        "= Book\n:product: Header\n\nifdef::backend-epub3[]\nEPUB only\nendif::[]\n\n{product}";
    let files = book(source, &[("product", "Default@")]);
    let content = text(&files, "EPUB/_preamble.xhtml");
    assert!(content.contains("EPUB only"));
    assert!(content.contains("Header"));
    assert!(!content.contains("Default"));
    let files = book(source, &[("product", "Pinned")]);
    assert!(text(&files, "EPUB/_preamble.xhtml").contains("Pinned"));
    let files = book(
        "= Book\n:product: Header\n\nifdef::product[]\nPresent\nendif::[]",
        &[("product!", "")],
    );
    assert!(!text(&files, "EPUB/_preamble.xhtml").contains("Present"));
}

#[test]
fn default_identifier_preserves_sha256_and_explicit_uuid_takes_precedence() {
    for (attributes, expected) in [
        (
            vec![],
            "urn:sha256:909cb81127c5e1944b16dc5944ade59facb822c7a3f250d1f0256293bfd3c055",
        ),
        (
            vec![("uuid", "urn:uuid:custom-book")],
            "urn:uuid:custom-book",
        ),
    ] {
        let files = book("= Book\n\nText", &attributes);
        let opf = roxmltree::Document::parse(text(&files, "EPUB/package.opf")).unwrap();
        let identifier = opf
            .descendants()
            .find(|node| node.has_tag_name(("http://purl.org/dc/elements/1.1/", "identifier")))
            .and_then(|node| node.text());
        assert_eq!(identifier, Some(expected));
        assert!(text(&files, "EPUB/toc.ncx").contains(expected));
    }
}
