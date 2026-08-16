//! Predictive context pre-loading (OSS stub).
//!
//! Enterprise pre-caches files based on task triage predictions.
//! OSS: no-op (no prefetching).

/// Warms the cache for predicted paths (OSS: no-op).
pub fn warm_predictions<T>(_paths: &[String], _cache: Option<&T>) {}

/// Plans prefetch after triage (OSS: no-op).
pub fn plan_after_triage(_task_class: &str) {}

/// Count of successfully warmed predictions (OSS: 0).
pub fn warmed_count() -> u64 {
    0
}

/// Count of skipped predictions (OSS: 0).
pub fn skipped_count() -> u64 {
    0
}

/// An entry in the prefetch plan.
#[derive(Debug, Clone)]
pub struct PrefetchEntry {
    pub path: String,
    pub score: f64,
}

/// A prefetch plan containing predicted files.
#[derive(Debug, Clone, Default)]
pub struct PrefetchPlan {
    pub files: Vec<PrefetchEntry>,
}

/// File trajectory tracker (OSS stub).
pub struct FileTrajectory {
    paths: Vec<(String, u32)>,
    capacity: usize,
}

impl FileTrajectory {
    pub fn new(capacity: usize) -> Self {
        Self {
            paths: Vec::new(),
            capacity,
        }
    }

    pub fn record(&mut self, path: &str) {
        if let Some(entry) = self.paths.iter_mut().find(|(p, _)| p == path) {
            entry.1 += 1;
        } else {
            if self.paths.len() >= self.capacity {
                self.paths.remove(0);
            }
            self.paths.push((path.to_owned(), 1));
        }
    }

    pub fn predict(&self, top_k: usize) -> Vec<(String, f64)> {
        let mut scored: Vec<(String, f64)> = self
            .paths
            .iter()
            .map(|(path, count)| (path.clone(), *count as f64))
            .collect();
        scored.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));
        scored.truncate(top_k);
        scored
    }
}

/// Builds a prefetch plan from trajectory data (OSS: basic frequency-based).
pub fn build_prefetch_plan(
    trajectory: &FileTrajectory,
    _exclude: &[String],
    top_k: usize,
    _min_score: f64,
) -> PrefetchPlan {
    let files = trajectory
        .predict(top_k)
        .into_iter()
        .map(|(path, score)| PrefetchEntry { path, score })
        .collect();
    PrefetchPlan { files }
}

/// Records a file read for trajectory learning (OSS: no-op).
pub fn record_file_read(_path: &str) {}

/// Checks if a path was predicted by the prefetch engine (OSS: always false).
pub fn is_prefetch_prediction(_path: &str) -> bool {
    false
}
