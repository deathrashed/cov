use anyhow::{Context, Result};
use id3::{TagLike, Version};
use lofty::config::WriteOptions;
use lofty::file::TaggedFileExt;
use lofty::picture::{MimeType, Picture, PictureType};
use lofty::probe::Probe;
use lofty::tag::{Tag, TagExt};
use std::path::Path;

pub fn embed_file(path: &Path, data: &[u8], mime: &str) -> Result<()> {
    let ext = path
        .extension()
        .and_then(|s| s.to_str())
        .unwrap_or("")
        .to_lowercase();

    if ext == "mp3" {
        let mut tag = match id3::Tag::read_from_path(path) {
            Ok(t) => t,
            Err(id3::Error {
                kind: id3::ErrorKind::NoTag,
                ..
            }) => id3::Tag::new(),
            Err(e) => anyhow::bail!("Failed to read MP3 tag from {:?}: {}", path, e),
        };
        tag.remove_picture_by_type(id3::frame::PictureType::CoverFront);
        tag.add_frame(id3::frame::Picture {
            mime_type: mime.to_string(),
            picture_type: id3::frame::PictureType::CoverFront,
            description: "Front Cover".to_string(),
            data: data.to_vec(),
        });
        tag.write_to_path(path, Version::Id3v23)
            .context("Failed to write id3v2.3")?;
        return Ok(());
    }

    let mut tagged_file = Probe::open(path)
        .context("Failed to open file")?
        .read()
        .context("Failed to read file")?;

    let tag_type = tagged_file.primary_tag_type();

    if tagged_file.primary_tag_mut().is_none() {
        tagged_file.insert_tag(Tag::new(tag_type));
    }

    let tag = tagged_file.primary_tag_mut().unwrap();

    tag.remove_picture_type(PictureType::CoverFront);
    tag.remove_picture_type(PictureType::Other); // For m4a

    if tag_type == lofty::tag::TagType::Ape {
        tag.remove_key(&lofty::tag::ItemKey::Unknown(
            "Cover Art (Front)".to_string(),
        ));
    }

    let mime_type = match mime {
        "image/png" => MimeType::Png,
        "image/jpeg" => MimeType::Jpeg,
        _ => MimeType::Jpeg,
    };

    let picture = Picture::new_unchecked(
        PictureType::CoverFront,
        Some(mime_type),
        Some(String::from("Cover Art (Front)")),
        data.to_vec(),
    );

    if tag_type == lofty::tag::TagType::Ape {
        let mut ape_tag = lofty::ape::ApeTag::from(tag.clone());
        ape_tag.insert(
            lofty::ape::ApeItem::new(
                String::from("Cover Art (Front)"),
                lofty::tag::ItemValue::Binary(picture.as_ape_bytes()),
            )
            .unwrap(),
        );

        ape_tag
            .save_to_path(path, WriteOptions::new())
            .context("Failed to save ApeTag via lofty")?;
        return Ok(());
    }

    tag.push_picture(picture);

    // Save
    tag.save_to_path(path, WriteOptions::new())
        .context("Failed to save tag via lofty")?;

    Ok(())
}
