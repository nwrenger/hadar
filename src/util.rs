use std::cmp::Ordering;

/// Wrapper for a key-value pair that is ordable by the key.
#[derive(Debug)]
pub struct OrdPair<K: Ord, V>(pub K, pub V);

impl<K: Ord, V> Eq for OrdPair<K, V> {}

impl<K: Ord, V> PartialEq for OrdPair<K, V> {
    fn eq(&self, other: &Self) -> bool {
        self.0.eq(&other.0)
    }
}

impl<K: Ord, V> PartialOrd for OrdPair<K, V> {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl<K: Ord, V> Ord for OrdPair<K, V> {
    fn cmp(&self, other: &Self) -> Ordering {
        self.0.cmp(&other.0)
    }
}