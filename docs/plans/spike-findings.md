# Spike Findings: Tagging and Embedding Engine Core

This document outlines the findings from Spike 1.2, implementing cover art embedding for the Rust rewrite of the COV toolkit using `lofty` and `id3`.

## Format-Specific Findings

### MP3 (ID3v2)
- **lofty limitation**: `lofty` does not provide an API to reliably serialize tags explicitly as ID3v2.3 (it parses v2.3 but often upgrades frames or writes v2.4 when saving). Because the legacy semantics strictly required `v2_version=3`, `id3` crate was introduced for `.mp3` files.
- **id3 crate**: We handle MP3 specifically using `id3::Tag::read_from_path`, `remove_picture_by_type`, and `tag.write_to_path(path, Version::Id3v23)`.

### FLAC, Ogg/Opus, WAV/AIFF
- **lofty compatibility**: These formats map cleanly to `lofty`'s `Tag::push_picture` and `PictureType::CoverFront` abstractions.
- **WAV/AIFF**: The ID3v2 tags inside RIFF/AIFF chunks are correctly modified by `lofty`, avoiding the versioning strictness needed for MP3 because the legacy Python code didn't force v2.3 specifically inside WAV files (or if it did, the toolchain accepts lofty's default ID3v2 handling within chunks).

### MP4 / M4A
- **PictureType handling**: MP4 metadata can represent images via `PictureType::CoverFront` or `PictureType::Other`. When clearing old covers, both must be removed using `tag.remove_picture_type(...)` to guarantee the old cover is removed before the new one is pushed.
- **Codec magic bytes**: M4A `covr` boxes depend on JPEG/PNG magic bytes which `lofty` handles seamlessly.

### APE / WavPack (.wv, .ape)
- **Binary item representation**: APE tags represent covers as binary items with keys like `"Cover Art (Front)"` instead of typical picture structs. `lofty`'s generic `Tag::push_picture` writes this item correctly to a standalone `ApeTag`, but due to how `lofty` writes APE tags within WavPack files (`wavpack::write::write_to` using `ApeTag::write_to` via `lofty_attr` macro), using the generic `Tag::push_picture` causes the cover to be silently dropped during the macro-generated conversion.
- **Workaround**: We detect `lofty::tag::TagType::Ape`, manually convert the `Tag` to `lofty::ape::ApeTag`, explicitly insert `lofty::ape::ApeItem::new("Cover Art (Front)", ItemValue::Binary(pic.as_ape_bytes()))`, and save the `ApeTag` directly to the file using `ape_tag.save_to_path(...)`. This guarantees the APE tag correctly embeds the cover art.
- **Reading APE covers**: When counting covers in test assertions, APE files do not populate `tag.pictures()`. Instead, one must loop through `tag.items()` to look for `ItemKey::Unknown("Cover Art (Front)")`.

## Testing
- **Test matrix coverage**: The `tests/embed_matrix.rs` script successfully tests replacement idempotency (JPEG, then PNG) across MP3, FLAC, M4A, Ogg, Opus, WAV, AIFF, and WavPack (and optionally APE and DSF when fixtures are present). All tests retain preexisting seeded title tags while replacing only the cover, passing the spike criteria.

## Matcher Benchmark: nucleo vs frizbee (Task 1.3)

We benchmarked `nucleo` and `frizbee` on a synthetic dataset of 5,000 album titles.

### Results
Throughput/latency to match 5,000 items:
- **"fleet foxes"**: `frizbee` ~36.6µs | `nucleo` ~38.3µs
- **"shore"**: `frizbee` ~37.4µs | `nucleo` ~42.7µs
- **"mac"**: `frizbee` ~38.7µs | `nucleo` ~51.4µs
- **"xyz"**: `frizbee` ~37.6µs | `nucleo` ~42.2µs

### Decision: `nucleo`
While `frizbee`'s SIMD approach gives it a raw throughput advantage in a simple sequential loop (matching 5k items in ~38µs vs `nucleo`'s ~45µs), **we are choosing `nucleo` for the COV toolkit rewrite**. 

**Reasoning:**
1. **Streaming Readiness**: The TUI scanner channels batches of albums over time. `nucleo` natively exposes `nucleo::Nucleo` which spins up a background thread pool, incrementally ingests items, and automatically computes/recomputes matches as new items stream in. Frizbee would require us to manually build this concurrent synchronization and incremental re-sorting architecture.
2. **Latency is negligible**: Both matchers process 5,000 items in less than 0.1 milliseconds. Since the bottleneck will be UI rendering and disk I/O, the <15µs difference between the two is entirely negligible compared to the massive architectural benefit of `nucleo`'s built-in background incremental streaming.
