use crate::templates::notebook::Toc;
use crate::{section, templates};
use color_eyre::eyre::Result;
use onenote_parser::notebook::Notebook;
use onenote_parser::property::common::Color;
use onenote_parser::section::{Section, SectionEntry};
use palette::rgb::Rgb;
use palette::{Alpha, ConvertFrom, Hsl, Saturate, Shade, Srgb};
use std::fs;
use std::path::Path;
use log::info;

pub(crate) type RgbColor = Alpha<Rgb<palette::encoding::Srgb, u8>, f32>;

pub(crate) struct Renderer;

impl Renderer {
    pub fn new() -> Self {
        Renderer
    }

    pub fn render(&mut self, notebook: &Notebook, name: &str, output_dir: &Path) -> Result<()> {
        info!("Notebook name: {:?} {:?}", name, output_dir);
        if !output_dir.is_dir() {
            fs::create_dir(&output_dir)?;
        }

        let notebook_dir = output_dir.join(sanitize_filename::sanitize(name));

        if !notebook_dir.is_dir() {
            fs::create_dir(&notebook_dir)?;
        }

        let mut toc = Vec::new();

        for entry in notebook.entries() {
            match entry {
                SectionEntry::Section(section) => {
                    toc.push(Toc::Section(self.render_section(
                        section,
                        &notebook_dir,
                        output_dir,
                    )?));
                }
                SectionEntry::SectionGroup(group) => {
                    let dir_name = sanitize_filename::sanitize(group.display_name());
                    let section_group_dir = notebook_dir.join(dir_name);
                    info!("Section group directory: {:?}", section_group_dir);
                    if !section_group_dir.is_dir() {
                        fs::create_dir(&section_group_dir)?;
                    }

                    let mut entries = Vec::new();

                    for entry in group.entries() {
                        if let SectionEntry::Section(section) = entry {
                            entries.push(self.render_section(
                                section,
                                &section_group_dir,
                                &output_dir,
                            )?);
                        }
                    }

                    toc.push(templates::notebook::Toc::SectionGroup(
                        group.display_name().to_string(),
                        entries,
                    ))
                }
            }
        }

        let toc_html = templates::notebook::render(name, &toc)?;
        let toc_file = output_dir.join(format!("{}.html", name));
        fs::write(toc_file, toc_html)?;

        Ok(())
    }

    fn render_section(
        &mut self,
        section: &Section,
        notebook_dir: &Path,
        base_dir: &Path,
    ) -> Result<templates::notebook::Section> {
        let mut renderer = section::Renderer::new();
        let section_path = renderer.render(section, notebook_dir)?;
        info!("section_path: {:?}", section_path);

        let path_from_base_dir = section_path.strip_prefix(base_dir)?.to_string_lossy().to_string();
        info!("path_from_base_dir: {:?}", path_from_base_dir);
        Ok(templates::notebook::Section {
            name: section.display_name().to_string(),
            path: path_from_base_dir,
            color: section.color().map(prepare_color),
        })
    }
}

fn prepare_color(color: Color) -> RgbColor {
    Alpha {
        alpha: color.alpha() as f32 / 255.0,
        color: Srgb::convert_from(
            Hsl::convert_from(Srgb::new(
                color.r() as f32 / 255.0,
                color.g() as f32 / 255.0,
                color.b() as f32 / 255.0,
            ))
            .darken(0.2)
            .saturate(1.0),
        )
        .into_format(),
    }
}
