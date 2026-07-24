use lofty::ape::ApeTag;
use lofty::picture::{MimeType, Picture, PictureType};
use lofty::tag::{Tag, TagType};

#[test]
fn test_debug_ape() {
    let mut tag = Tag::new(TagType::Ape);
    let pic = Picture::new_unchecked(
        PictureType::CoverFront,
        Some(MimeType::Jpeg),
        Some(String::from("Front Cover")),
        vec![0xFF, 0xD8, 0xFF, 0xE0],
    );
    tag.push_picture(pic);

    let ape_tag = ApeTag::from(tag);
    let tag_again = Tag::from(ape_tag);
    println!("Tag items after roundtrip:");
    for item in tag_again.items() {
        println!("{:?}", item.key());
    }
}
