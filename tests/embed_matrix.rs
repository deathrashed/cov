use anyhow::Result;
use cov::embed::embed_file;
use cov::testutil::{read_cover_count_and_title, seed_title};
use std::fs;
use std::path::Path;
use tempfile::tempdir;

#[test]
fn test_embed_matrix() -> Result<()> {
    let exts = vec![
        "mp3", "flac", "m4a", "ogg", "opus", "wav", "aiff", "dsf", "wv", "ape",
    ];
    let artwork_jpg = fs::read("tests/fixtures/artwork.jpg")?;
    let artwork_png = fs::read("tests/fixtures/artwork.png")?;

    let dir = tempdir()?;
    for ext in exts {
        let fixture_path = Path::new("tests/fixtures/").join(format!("fixture.{}", ext));
        if !fixture_path.exists() {
            eprintln!("SKIPPED: {} (fixture not present)", ext);
            continue;
        }
        let test_path = dir.path().join(format!("test.{}", ext));
        fs::copy(&fixture_path, &test_path)?;

        seed_title(&test_path, "Keep Me")?;

        // Pass 1: embed jpeg
        embed_file(&test_path, &artwork_jpg, "image/jpeg")?;

        let (covers, title) = read_cover_count_and_title(&test_path)?;
        assert_eq!(
            covers, 1,
            "{} should have exactly 1 cover after first embed",
            ext
        );
        assert_eq!(
            title.as_deref(),
            Some("Keep Me"),
            "{} should retain seeded title",
            ext
        );

        // Pass 2: embed png
        embed_file(&test_path, &artwork_png, "image/png")?;

        let (covers, title) = read_cover_count_and_title(&test_path)?;
        assert_eq!(
            covers, 1,
            "{} should have exactly 1 cover after replacing with PNG",
            ext
        );
        assert_eq!(
            title.as_deref(),
            Some("Keep Me"),
            "{} should retain seeded title",
            ext
        );
    }

    Ok(())
}
