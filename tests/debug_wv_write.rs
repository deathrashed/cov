use lofty::ape::{ApeItem, ApeTag};
use lofty::file::TaggedFileExt;
use lofty::probe::Probe;
use lofty::tag::ItemValue;
use lofty::tag::TagExt;
use std::fs;
use std::path::Path;

#[test]
fn test_debug_wv_write() {
    fs::copy("tests/fixtures/fixture.wv", "tests/test.wv").unwrap();
    let path = Path::new("tests/test.wv");

    let mut tagged_file = Probe::open(path).unwrap().read().unwrap();
    let tag = tagged_file.primary_tag_mut().unwrap();

    let mut ape_tag = ApeTag::from(tag.clone());
    ape_tag.insert(
        ApeItem::new(
            String::from("Cover Art (Front)"),
            ItemValue::Binary(vec![1, 2, 3, 4]),
        )
        .unwrap(),
    );

    ape_tag
        .save_to_path(path, lofty::config::WriteOptions::new())
        .unwrap();

    // Now verify if it can still be opened as a WavPack file!
    let tagged_file2 = Probe::open(path).unwrap().read().unwrap();
    println!("File type: {:?}", tagged_file2.file_type());
}
