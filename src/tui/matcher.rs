use crate::tui::scanner::Album;
use nucleo::{Config, Nucleo, Utf32String};
use std::sync::Arc;

/// Wraps a nucleo matcher for album fuzzy filtering.
pub struct AlbumMatcher {
    nucleo: Nucleo<Arc<Album>>,
    items: Vec<Arc<Album>>,
}

impl AlbumMatcher {
    pub fn new() -> Self {
        let nucleo = Nucleo::new(Config::DEFAULT, Arc::new(|| {}), None, 1);
        Self {
            nucleo,
            items: Vec::new(),
        }
    }

    /// Replace all items (clear old items by recreating the Nucleo matcher + repopulate).
    pub fn replace_items(&mut self, items: Vec<Arc<Album>>) {
        self.items = items;
        self.nucleo = Nucleo::new(Config::DEFAULT, Arc::new(|| {}), None, 1);
        let injector = self.nucleo.injector();
        for album in &self.items {
            let a = album.clone();
            injector.push(a, move |item, cols| {
                cols[0] = Utf32String::from(item.rel.as_str());
            });
        }
        self.nucleo.tick(0);
    }

    /// Set or update the query pattern.
    pub fn query(&mut self, pattern: &str) {
        self.nucleo.pattern.reparse(
            0,
            pattern,
            nucleo::pattern::CaseMatching::Smart,
            nucleo::pattern::Normalization::Smart,
            false,
        );
        // Tick to process
        self.nucleo.tick(10);
        while self.nucleo.tick(10).running {}
    }

    /// Get current ranked results.
    pub fn results(&self) -> Vec<Arc<Album>> {
        let snapshot = self.nucleo.snapshot();
        let count = snapshot.matched_item_count() as usize;
        let mut results = Vec::with_capacity(count);
        for item in snapshot.matched_items(..) {
            results.push(item.data.clone());
        }
        results
    }
}

impl Default for AlbumMatcher {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    fn make_album(rel: &str) -> Arc<Album> {
        Arc::new(Album {
            dir: PathBuf::from(format!("/music/{}", rel)),
            rel: rel.to_string(),
            display: rel.to_string(),
            tracks: vec![],
        })
    }

    #[test]
    fn test_replace_items_clears_previous() {
        let mut matcher = AlbumMatcher::new();
        matcher.replace_items(vec![make_album("Album A"), make_album("Album B")]);
        matcher.query("");
        assert_eq!(matcher.results().len(), 2);

        // Replacing with 3 items should result in 3, NOT 2 + 3 = 5
        matcher.replace_items(vec![
            make_album("Album A"),
            make_album("Album B"),
            make_album("Album C"),
        ]);
        matcher.query("");
        assert_eq!(matcher.results().len(), 3, "replace_items must clear previous nucleo entries");
    }
}
