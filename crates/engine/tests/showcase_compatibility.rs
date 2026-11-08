use adoc_engine::{RenderOptions, render};
use asciidoc_parser::blocks::{Block, FindBlocks, IsBlock};

#[test]
fn title_after_nested_list_starts_a_separate_checklist() {
    let source = "* Control plane\n** Gateway\n* Workers\n** Storage adapter\n** Rendering service\n\n.Release checklist\n* [x] Build reproducible artifacts\n* [ ] Promote to production\n";
    let options = RenderOptions::default();
    let doc = asciidoc_html5::load_with(source, &options.parser_options().unwrap());
    let lists: Vec<_> = doc
        .child_blocks()
        .filter_map(|block| match block {
            Block::List(list) => Some(list),
            _ => None,
        })
        .collect();
    assert_eq!(
        lists.len(),
        2,
        "a title after the nested list must start a separate list"
    );
    assert!(!lists[0].is_checklist());
    assert!(lists[1].is_checklist());
    assert_eq!(lists[1].title(), Some("Release checklist"));
    assert_eq!(lists[0].child_blocks().count(), 2);
    assert_eq!(lists[1].child_blocks().count(), 2);
}

#[test]
fn titled_admonitions_do_not_advance_example_counter() {
    for label in ["NOTE", "TIP", "IMPORTANT", "CAUTION", "WARNING"] {
        let source = format!(
            "[{label}]\n.Before switching traffic\n====\nVerify readiness.\n====\n\n.First example\n====\nAn ordinary example.\n====\n\n.Second example\n====\nAnother example.\n====\n\nCounter: {{example-number}}\n"
        );
        let html = render(&source, &RenderOptions::default()).unwrap().html;
        assert!(html.contains("Example 1. First example"), "{html}");
        assert!(html.contains("Example 2. Second example"), "{html}");
        assert!(html.contains("Counter: 2"), "{html}");
        assert!(!html.contains("Example 3."), "{html}");
    }
}

#[test]
fn bibliography_is_unnumbered_and_does_not_consume_a_sibling_number() {
    for (heading, following) in [("==", "2. After"), ("===", "1.1. After")] {
        let source = format!(
            "= Document\n:sectnums:\n:toc:\n\n== Before\n\n[bibliography]\n{heading} Further reading\n\n* [[[ref]]] A reference.\n\n{heading} After\n"
        );
        let options = RenderOptions::default();
        let doc = asciidoc_html5::load_with(&source, &options.parser_options().unwrap());
        let bibliography = doc
            .descendant_blocks()
            .find_map(|block| match block {
                Block::Section(section) if section.declared_style() == Some("bibliography") => {
                    Some(section)
                }
                _ => None,
            })
            .unwrap();
        assert!(
            bibliography.section_number().is_none(),
            "{source}\n{bibliography:?}"
        );
        let html = render(&source, &options).unwrap().html;
        assert!(
            html.contains("<a href=\"#_further_reading\">Further reading</a>"),
            "{html}"
        );
        assert!(html.contains(&format!(">{following}</h")), "{html}");
    }
}

#[test]
fn explicit_all_numbers_bibliography_and_keeps_full_xref_label() {
    let source = "= Document\n:sectnums: all\n:toc:\n:xrefstyle: full\n\n== Before\n\n[bibliography]\n== Special\n\n* [[[ref]]] A reference.\n\n== After\n\nSee <<_special>>.\n";
    let html = render(source, &RenderOptions::default()).unwrap().html;
    assert!(html.contains(">2. Special</h2>"), "{html}");
    assert!(html.contains(">3. After</h2>"), "{html}");
    assert!(html.contains("href=\"#_special\">2,"), "{html}");
    assert!(!html.contains("Section 2"), "{html}");
}

#[test]
fn publication_backends_preserve_checklist_titles_and_caption_numbers() {
    let source = "= Document\n:sectnums:\n\n== Before\n\n* Workers\n** Storage\n\n.Release checklist\n* [x] Build\n* [ ] Promote\n\n[IMPORTANT]\n.Before switching traffic\n====\nVerify readiness.\n====\n\n.First example\n====\nAn ordinary example.\n====\n\n[bibliography]\n== Special\n\n* [[[ref]]] A reference.\n\n== After\n";
    let options = RenderOptions::default();
    let doc = asciidoc_html5::load_with(source, &options.parser_options().unwrap());
    let (markup, _) = adocers_typst::markup(&doc, std::path::Path::new("."), &Default::default());
    for expected in ["Release checklist", "Example 1. First example", "2. After"] {
        assert!(markup.contains(expected), "missing {expected}: {markup}");
    }
    assert!(!markup.contains("2. Special"), "{markup}");
    let artifacts = adoc_engine::export(source, "epub", &options).unwrap();
    let mut zip = zip::ZipArchive::new(std::io::Cursor::new(&artifacts[0].1)).unwrap();
    let mut xhtml = String::new();
    xhtml.push_str(
        "<root xmlns=\"http://www.w3.org/1999/xhtml\" xmlns:epub=\"http://www.idpf.org/2007/ops\">",
    );
    for i in 0..zip.len() {
        let mut entry = zip.by_index(i).unwrap();
        let name = entry.name().unwrap().to_string();
        if !name.ends_with(".xhtml")
            || matches!(
                name.as_str(),
                "EPUB/nav.xhtml" | "EPUB/toc.xhtml" | "EPUB/cover.xhtml"
            )
        {
            continue;
        }
        let mut source = String::new();
        std::io::Read::read_to_string(&mut entry, &mut source).unwrap();
        let parsed = roxmltree::Document::parse(&source).unwrap();
        let body = parsed
            .descendants()
            .find(|n| n.tag_name().name() == "body")
            .unwrap();
        xhtml.push_str(&source[body.range()]);
    }
    xhtml.push_str("</root>");
    let document = roxmltree::Document::parse(&xhtml).unwrap();
    let checklist = document
        .descendants()
        .find(|node| node.attribute("class") == Some("ulist checklist"))
        .expect("a separate checklist container in EPUB");
    assert!(
        checklist
            .children()
            .any(|node| node.attribute("class") == Some("title")
                && node.text() == Some("Release checklist"))
    );
    assert!(
        document
            .descendants()
            .any(|node| node.attribute("class") == Some("title")
                && node.text() == Some("Example 1. First example"))
    );
    assert!(
        document
            .descendants()
            .any(|node| node.has_tag_name("h2") && node.text() == Some("Special"))
    );
}

#[test]
fn complete_showcase_matches_pinned_ruby_and_js_html() {
    let fixture =
        std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../benchmarks/fixtures/showcase");
    let source =
        include_str!("../../../benchmarks/fixtures/showcase/main.adoc").replace("\r\n", "\n");
    let expected =
        include_str!("../../../benchmarks/fixtures/showcase/expected.html").replace("\r\n", "\n");
    let rendered = render(
        &source,
        &RenderOptions {
            base_dir: fixture,
            safe_mode: "safe".into(),
            ..Default::default()
        },
    )
    .unwrap();
    assert!(rendered.diagnostics.is_empty());
    // The pinned reference engines emit no final newline; Rust emits one.
    assert_eq!(rendered.html.trim_end(), expected.trim_end());
}
