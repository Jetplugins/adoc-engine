use crate::{Document, RenderOptions};
use anyhow::{Result, anyhow, bail};
use typst_as_lib::{TypstEngine, typst_kit_options::TypstKitFontOptions};
pub fn export(
    doc: &Document<'_>,
    format: &str,
    options: &RenderOptions,
) -> Result<Vec<(String, Vec<u8>)>> {
    crate::presentation::validate_diagrams(&options.diagrams)?;
    let html = asciidoc_html5::convert_document(doc);
    let mut resources = std::collections::BTreeMap::new();
    for target in crate::presentation::image_sources(&html) {
        let path = crate::assets::local_path(&target, options)?;
        resources.insert(target, std::fs::read(path)?);
    }
    let settings = adocers_typst::Options {
        math: true,
        ..Default::default()
    };
    let (source, images) = adocers_typst::markup_with_resources(
        doc,
        &options.base_dir,
        &settings,
        Some(resources),
        options.diagrams.clone(),
    );
    let engine = TypstEngine::builder()
        .main_file(source)
        .search_fonts_with(TypstKitFontOptions::default().include_system_fonts(false))
        .with_static_file_resolver(images.iter().map(|(p, b)| (p.as_str(), b.as_slice())))
        .build();
    let compiled: typst_layout::PagedDocument =
        engine.compile().output.map_err(|e| anyhow!("{e}"))?;
    match format {
        #[cfg(feature = "pdf")]
        "pdf" => {
            let bytes =
                typst_pdf::pdf(&compiled, &Default::default()).map_err(|e| anyhow!("{e:?}"))?;
            Ok(vec![("document.pdf".into(), bytes)])
        }
        #[cfg(feature = "svg")]
        "svg-merged" => Ok(vec![(
            "document.svg".into(),
            typst_svg::svg_merged(&compiled, &Default::default(), typst::layout::Abs::pt(12.0))
                .into_bytes(),
        )]),
        #[cfg(feature = "svg")]
        "svg" => Ok(compiled
            .pages()
            .iter()
            .enumerate()
            .map(|(i, page)| {
                (
                    format!("page-{}.svg", i + 1),
                    typst_svg::svg(page, &Default::default()).into_bytes(),
                )
            })
            .collect()),
        #[cfg(feature = "png")]
        "png" => compiled
            .pages()
            .iter()
            .enumerate()
            .map(|(i, page)| {
                Ok((
                    format!("page-{}.png", i + 1),
                    typst_render::render(page, &Default::default()).encode_png()?,
                ))
            })
            .collect(),
        _ => bail!("unsupported publication format: {format}"),
    }
}
