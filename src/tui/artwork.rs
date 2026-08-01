use crossbeam::channel::Sender;
use lofty::file::TaggedFileExt;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::sync::atomic::{AtomicU64, Ordering};

/// Badge glyphs for the album list.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Badge {
    Checking,
    Missing,
    SidecarOnly,
    Partial,
    Complete,
}

impl Badge {
    pub fn glyph(&self) -> &'static str {
        match self {
            Badge::Checking => "\u{2026}",
            Badge::Missing => "\u{25CB}",
            Badge::SidecarOnly => "\u{25C6}",
            Badge::Partial => "\u{25D0}",
            Badge::Complete => "\u{25CF}",
        }
    }
}

#[derive(Debug, Clone)]
pub struct ArtworkStatus {
    pub sidecar: Option<PathBuf>,
    pub embedded: EmbeddedState,
}

#[derive(Debug, Clone)]
pub enum EmbeddedState {
    Checking,
    None,
    Partial { with: usize, total: usize },
    All { total: usize },
}

impl Default for ArtworkStatus {
    fn default() -> Self {
        Self {
            sidecar: None,
            embedded: EmbeddedState::Checking,
        }
    }
}

impl ArtworkStatus {
    pub fn badge(&self) -> Badge {
        match (&self.embedded, &self.sidecar) {
            (EmbeddedState::Checking, _) => Badge::Checking,
            (EmbeddedState::None, None) => Badge::Missing,
            (EmbeddedState::None, Some(_)) => Badge::SidecarOnly,
            (EmbeddedState::Partial { .. }, _) => Badge::Partial,
            (EmbeddedState::All { .. }, _) => Badge::Complete,
        }
    }
}

/// Inspect an album's artwork state using Lofty for tag reading.
pub fn inspect_album(dir: &Path, tracks: &[PathBuf]) -> (ArtworkStatus, Option<Vec<u8>>) {
    let mut status = ArtworkStatus::default();

    // Sidecar: cover.{jpg,jpeg,png} in album dir
    for ext in &["jpg", "jpeg", "png"] {
        let candidate = dir.join(format!("cover.{}", ext));
        if candidate.exists() {
            status.sidecar = Some(candidate.clone());
            break;
        }
    }

    // Embedded: check every track using Lofty
    let total = tracks.len();
    let mut with_cover = 0usize;
    let mut preview_bytes = None;

    if total == 0 {
        status.embedded = EmbeddedState::None;
        return (status, None);
    }

    for track in tracks {
        if let Ok((has_cover, bytes)) = quick_inspect_lofty(track)
            && has_cover
        {
            with_cover += 1;
            if preview_bytes.is_none() {
                preview_bytes = bytes;
            }
        }
    }

    // Fall back to sidecar for preview
    if preview_bytes.is_none()
        && let Some(ref sidecar_path) = status.sidecar
    {
        preview_bytes = std::fs::read(sidecar_path).ok();
    }

    status.embedded = if with_cover == 0 {
        EmbeddedState::None
    } else if with_cover == total {
        EmbeddedState::All { total }
    } else {
        EmbeddedState::Partial {
            with: with_cover,
            total,
        }
    };

    (status, preview_bytes)
}

/// Use Lofty to inspect a single file for embedded cover art.
fn quick_inspect_lofty(path: &Path) -> anyhow::Result<(bool, Option<Vec<u8>>)> {
    let tagged_file = lofty::probe::Probe::open(path)?
        .read()
        .map_err(|e| anyhow::anyhow!("lofty read error: {e}"))?;

    // Lofty v2: iterate over properties without tags() method.
    // Check each known tag type.
    for tag_type in &[
        lofty::tag::TagType::Id3v2,
        lofty::tag::TagType::Ape,
        lofty::tag::TagType::VorbisComments,
        lofty::tag::TagType::Mp4Ilst,
    ] {
        if let Some(tag) = tagged_file.tag(*tag_type) {
            for pic in tag.pictures() {
                let pic_type = pic.pic_type();
                if pic_type == lofty::picture::PictureType::CoverFront
                    || pic_type == lofty::picture::PictureType::Other
                {
                    return Ok((true, Some(pic.data().to_vec())));
                }
            }
        }
    }

    Ok((false, None))
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Filter {
    All,
    Missing,
    NeedsEmbed,
}

impl Filter {
    pub fn next(self) -> Self {
        match self {
            Filter::All => Filter::Missing,
            Filter::Missing => Filter::NeedsEmbed,
            Filter::NeedsEmbed => Filter::All,
        }
    }

    pub fn allows(&self, badge: Badge) -> bool {
        match self {
            Filter::All => true,
            Filter::Missing => badge == Badge::Missing,
            Filter::NeedsEmbed => badge == Badge::SidecarOnly || badge == Badge::Partial,
        }
    }
}

/// A background artwork-inspection job for a single album.
pub struct InspectJob {
    pub epoch: u64,
    pub dir: PathBuf,
    pub tracks: Vec<PathBuf>,
}

/// Result of inspecting one album's artwork state, tagged with the scan
/// epoch it was produced for so stale results can be discarded.
pub struct ArtworkMsg {
    pub epoch: u64,
    pub dir: PathBuf,
    pub status: ArtworkStatus,
}

/// Spawn a fixed pool of worker threads that consume `InspectJob`s and report
/// `ArtworkMsg` results back over `result_tx`. Returns the sender used to
/// submit jobs.
///
/// Each worker checks `cancel` before doing any work: if the job's epoch no
/// longer matches the live epoch (i.e. a rescan happened), the job is
/// dropped instead of wasting time inspecting stale tracks.
pub fn spawn_inspector_pool(
    cancel: Arc<AtomicU64>,
    result_tx: Sender<ArtworkMsg>,
    workers: usize,
) -> Sender<InspectJob> {
    let (job_tx, job_rx) = crossbeam::channel::unbounded::<InspectJob>();

    for _ in 0..workers.max(1) {
        let job_rx = job_rx.clone();
        let result_tx = result_tx.clone();
        let cancel = cancel.clone();
        std::thread::spawn(move || {
            while let Ok(job) = job_rx.recv() {
                if cancel.load(Ordering::Relaxed) != job.epoch {
                    continue; // stale: a rescan happened, drop this job
                }
                let (status, _preview) = inspect_album(&job.dir, &job.tracks);
                if result_tx
                    .send(ArtworkMsg {
                        epoch: job.epoch,
                        dir: job.dir,
                        status,
                    })
                    .is_err()
                {
                    return;
                }
            }
        });
    }

    job_tx
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[test]
    fn test_badge_glyphs() {
        assert_eq!(Badge::Checking.glyph(), "\u{2026}");
        assert_eq!(Badge::Missing.glyph(), "\u{25CB}");
        assert_eq!(Badge::Complete.glyph(), "\u{25CF}");
    }

    #[test]
    fn test_artwork_status_badge() {
        let mut s = ArtworkStatus::default();
        assert_eq!(s.badge(), Badge::Checking);

        s.embedded = EmbeddedState::None;
        assert_eq!(s.badge(), Badge::Missing);

        s.sidecar = Some(PathBuf::from("cover.jpg"));
        assert_eq!(s.badge(), Badge::SidecarOnly);

        s.embedded = EmbeddedState::Partial { with: 1, total: 2 };
        assert_eq!(s.badge(), Badge::Partial);

        s.embedded = EmbeddedState::All { total: 2 };
        assert_eq!(s.badge(), Badge::Complete);
    }

    #[test]
    fn test_filter_cycle() {
        let mut f = Filter::All;
        f = f.next();
        assert_eq!(f, Filter::Missing);
        f = f.next();
        assert_eq!(f, Filter::NeedsEmbed);
        f = f.next();
        assert_eq!(f, Filter::All);
    }

    #[test]
    fn test_filter_allows() {
        assert!(Filter::All.allows(Badge::Complete));
        assert!(Filter::All.allows(Badge::Missing));
        assert!(!Filter::Missing.allows(Badge::Complete));
        assert!(Filter::Missing.allows(Badge::Missing));
        assert!(!Filter::NeedsEmbed.allows(Badge::Complete));
        assert!(!Filter::NeedsEmbed.allows(Badge::Missing));
        assert!(Filter::NeedsEmbed.allows(Badge::SidecarOnly));
        assert!(Filter::NeedsEmbed.allows(Badge::Partial));
    }

    #[test]
    fn test_inspect_empty_dir() {
        let dir = tempdir().unwrap();
        let (status, preview) = inspect_album(dir.path(), &[]);
        assert!(matches!(status.embedded, EmbeddedState::None));
        assert!(preview.is_none());
    }
}
