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

    /// Replace all items (clear + repopulate).
    pub fn replace_items(&mut self, items: Vec<Arc<Album>>) {
        self.items = items;
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
