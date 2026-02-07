use std::collections::HashMap;
use std::sync::{Arc, RwLock};
use std::time::{Duration, Instant};

#[derive(Clone)]
pub struct CacheEntry {
    pub html: String,
    pub generated_at: Instant,
    pub revalidate_after: Duration,
}

impl CacheEntry {
    pub fn new(html: String, revalidate_seconds: u64) -> Self {
        Self {
            html,
            generated_at: Instant::now(),
            revalidate_after: Duration::from_secs(revalidate_seconds),
        }
    }

    pub fn is_stale(&self) -> bool {
        self.generated_at.elapsed() > self.revalidate_after
    }

    pub fn age_seconds(&self) -> u64 {
        self.generated_at.elapsed().as_secs()
    }
}

pub struct IncrementalCache {
    entries: Arc<RwLock<HashMap<String, CacheEntry>>>,
    default_revalidate: u64,
}

impl IncrementalCache {
    pub fn new(default_revalidate_seconds: u64) -> Self {
        Self {
            entries: Arc::new(RwLock::new(HashMap::new())),
            default_revalidate: default_revalidate_seconds,
        }
    }

    pub fn get(&self, path: &str) -> Option<CacheEntry> {
        let entries = self.entries.read().unwrap();
        entries.get(path).cloned()
    }

    pub fn get_if_fresh(&self, path: &str) -> Option<CacheEntry> {
        self.get(path).filter(|entry| !entry.is_stale())
    }

    pub fn set(&self, path: &str, html: String) {
        self.set_with_revalidate(path, html, self.default_revalidate);
    }

    pub fn set_with_revalidate(&self, path: &str, html: String, revalidate_seconds: u64) {
        let mut entries = self.entries.write().unwrap();
        entries.insert(path.to_string(), CacheEntry::new(html, revalidate_seconds));
    }

    pub fn invalidate(&self, path: &str) {
        let mut entries = self.entries.write().unwrap();
        entries.remove(path);
    }

    pub fn invalidate_all(&self) {
        let mut entries = self.entries.write().unwrap();
        entries.clear();
    }

    pub fn stale_paths(&self) -> Vec<String> {
        let entries = self.entries.read().unwrap();
        entries
            .iter()
            .filter(|(_, entry)| entry.is_stale())
            .map(|(path, _)| path.clone())
            .collect()
    }

    pub fn cache_size(&self) -> usize {
        let entries = self.entries.read().unwrap();
        entries.len()
    }
}

impl Clone for IncrementalCache {
    fn clone(&self) -> Self {
        Self {
            entries: Arc::clone(&self.entries),
            default_revalidate: self.default_revalidate,
        }
    }
}

pub struct IsrConfig {
    pub revalidate_seconds: u64,
    pub on_demand_revalidation: bool,
}

impl Default for IsrConfig {
    fn default() -> Self {
        Self {
            revalidate_seconds: 60,
            on_demand_revalidation: true,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::thread::sleep;

    #[test]
    fn test_cache_entry_creation() {
        let entry = CacheEntry::new("<html></html>".to_string(), 60);
        assert!(!entry.is_stale());
        assert_eq!(entry.html, "<html></html>");
    }

    #[test]
    fn test_cache_entry_staleness() {
        let entry = CacheEntry::new("<html></html>".to_string(), 0);
        sleep(Duration::from_millis(10));
        assert!(entry.is_stale());
    }

    #[test]
    fn test_incremental_cache_operations() {
        let cache = IncrementalCache::new(60);

        assert!(cache.get("/").is_none());

        cache.set("/", "<html>home</html>".to_string());
        let entry = cache.get("/").unwrap();
        assert_eq!(entry.html, "<html>home</html>");

        cache.invalidate("/");
        assert!(cache.get("/").is_none());
    }

    #[test]
    fn test_cache_with_revalidate() {
        let cache = IncrementalCache::new(60);

        cache.set_with_revalidate("/fast", "fast page".to_string(), 0);
        sleep(Duration::from_millis(10));

        assert!(cache.get("/fast").unwrap().is_stale());
        assert!(cache.get_if_fresh("/fast").is_none());
    }

    #[test]
    fn test_stale_paths() {
        let cache = IncrementalCache::new(60);

        cache.set_with_revalidate("/stale1", "page1".to_string(), 0);
        cache.set_with_revalidate("/stale2", "page2".to_string(), 0);
        cache.set("/fresh", "fresh page".to_string());

        sleep(Duration::from_millis(10));

        let stale = cache.stale_paths();
        assert_eq!(stale.len(), 2);
        assert!(stale.contains(&"/stale1".to_string()));
        assert!(stale.contains(&"/stale2".to_string()));
    }

    #[test]
    fn test_cache_clone_shares_data() {
        let cache1 = IncrementalCache::new(60);
        let cache2 = cache1.clone();

        cache1.set("/shared", "shared content".to_string());

        assert!(cache2.get("/shared").is_some());
    }
}
