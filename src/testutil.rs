use anyhow::{Context, Result};
use lofty::file::TaggedFileExt;
use lofty::picture::PictureType;
use lofty::probe::Probe;
use lofty::tag::{Accessor, ItemKey, Tag, TagExt};
use std::path::Path;

pub fn seed_title(path: &Path, title: &str) -> Result<()> {
    let mut tagged_file = Probe::open(path)
        .context("Failed to open file")?
        .read()
        .context("Failed to read file")?;

    let tag = match tagged_file.primary_tag_mut() {
        Some(primary_tag) => primary_tag,
        None => {
            let tag_type = tagged_file.primary_tag_type();
            tagged_file.insert_tag(Tag::new(tag_type));
            tagged_file.primary_tag_mut().unwrap()
        }
    };

    tag.insert_text(ItemKey::TrackTitle, title.to_string());
    tag.save_to_path(path, lofty::config::WriteOptions::new())
        .context("Failed to save title")?;
    Ok(())
}

pub fn read_cover_count_and_title(path: &Path) -> Result<(usize, Option<String>)> {
    let tagged_file = Probe::open(path)
        .context("Failed to open file")?
        .read()
        .context("Failed to read file")?;

    let mut title = None;
    let mut covers = 0;

    if let Some(tag) = tagged_file.primary_tag() {
        title = tag.title().map(|s| s.into_owned());
    }

    for tag in tagged_file.tags() {
        let c = tag
            .pictures()
            .iter()
            .filter(|p| {
                p.pic_type() == PictureType::CoverFront || p.pic_type() == PictureType::Other
            })
            .count();
        covers += c;

        if tag.tag_type() == lofty::tag::TagType::Ape {
            for item in tag.items() {
                if let ItemKey::Unknown(k) = item.key()
                    && k == "Cover Art (Front)"
                {
                    covers += 1;
                }
            }
        }
    }

    Ok((covers, title))
}
