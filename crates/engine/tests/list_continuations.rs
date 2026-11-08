use adoc_engine::{RenderOptions, export, render};
use asciidoc_parser::{
    HasSpan,
    blocks::{Block, FindBlocks},
};

fn assert_sibling_after_admonition(source: &str, expected_items: usize) {
    let options = RenderOptions {
        source_locations: true,
        ..Default::default()
    };
    let doc = asciidoc_html5::load_with(source, &options.parser_options().unwrap());
    let items: Vec<_> = doc
        .descendant_blocks()
        .filter_map(|block| match block {
            Block::ListItem(item) => Some(item),
            _ => None,
        })
        .collect();
    assert_eq!(items.len(), expected_items, "{source}");
    let promoted = items
        .iter()
        .find(|item| {
            item.span().data().contains("Promote the verified release.")
                && !item.span().data().contains("Keep the previous release")
        })
        .expect("promotion must be a separate list item");
    let expected_line = source
        .lines()
        .position(|line| line.contains("Promote the verified release."))
        .unwrap()
        + 1;
    assert_eq!(doc.origin_of(promoted.span()).line, expected_line);
    let admonitions: Vec<_> = doc
        .descendant_blocks()
        .filter_map(|block| match block {
            Block::Admonition(admonition) => Some(admonition),
            _ => None,
        })
        .collect();
    assert_eq!(admonitions.len(), 1, "{source}");
    assert!(
        !admonitions[0]
            .span()
            .data()
            .contains("Promote the verified release."),
        "{source}"
    );
    let rendered = render(source, &options).unwrap();
    assert!(
        rendered.diagnostics.is_empty(),
        "{source}\n{:?}",
        rendered.diagnostics
    );
    assert!(
        rendered.html.contains("Promote the verified release.</p>"),
        "{}",
        rendered.html
    );
}

#[test]
fn continued_admonitions_preserve_sibling_items() {
    for label in ["NOTE", "TIP", "IMPORTANT", "CAUTION", "WARNING"] {
        for marker in [".", "*", "-", "1."] {
            for style in [
                format!("{label}: Keep the previous release.\nWait for health checks."),
                format!("[{label}]\nKeep the previous release.\nWait for health checks."),
            ] {
                let next_marker = if marker == "1." { "2." } else { marker };
                let source = format!(
                    "{marker} Deploy the application.\n+\n{style}\n{next_marker} Promote the verified release.\n"
                );
                assert_sibling_after_admonition(&source, 2);
                assert_sibling_after_admonition(&source.replace('\n', "\r\n"), 2);
            }
        }
    }
}

#[test]
fn continued_admonition_can_return_to_ancestor_list() {
    let source = ". Deploy the application.\n.. Check the workers.\n+\nTIP: Keep the previous release.\n. Promote the verified release.\n";
    assert_sibling_after_admonition(source, 3);
}

#[test]
fn showcase_listing_then_tip_preserves_final_item() {
    assert_sibling_after_admonition(
        include_str!("../../../benchmarks/fixtures/list-continuations.adoc"),
        5,
    );
}

#[test]
fn top_level_admonition_keeps_list_like_paragraph_text() {
    let source = "TIP: Keep the previous release.\n. This is still paragraph text.\n";
    let options = RenderOptions::default();
    let doc = asciidoc_html5::load_with(source, &options.parser_options().unwrap());
    assert_eq!(
        doc.descendant_blocks()
            .filter(|block| matches!(block, Block::ListItem(_)))
            .count(),
        0
    );
    let html = render(source, &options).unwrap().html;
    assert!(html.contains(". This is still paragraph text."));
}

#[test]
fn epub_preserves_the_separate_list_item() {
    let source = ". Deploy the application.\n+\nTIP: Keep the previous release.\n. Promote the verified release.\n";
    let artifacts = export(source, "epub", &RenderOptions::default()).unwrap();
    let mut archive = zip::ZipArchive::new(std::io::Cursor::new(&artifacts[0].1)).unwrap();
    let mut xhtml = String::new();
    std::io::Read::read_to_string(
        &mut archive.by_name("EPUB/content.xhtml").unwrap(),
        &mut xhtml,
    )
    .unwrap();
    let doc = roxmltree::Document::parse(&xhtml).unwrap();
    let promoted = doc
        .descendants()
        .find(|node| {
            node.is_element()
                && node.tag_name().name() == "p"
                && node.text() == Some("Promote the verified release.")
        })
        .expect("promotion paragraph in EPUB");
    assert_eq!(promoted.parent().unwrap().tag_name().name(), "li");
    assert!(!promoted.ancestors().any(|node| node.has_tag_name("td")));
}
