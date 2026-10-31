use std::{
    path::PathBuf,
    sync::atomic::{AtomicUsize, Ordering},
};

static NEXT_TEMP_DIR: AtomicUsize = AtomicUsize::new(0);

pub(crate) struct TestDir(pub PathBuf);

impl TestDir {
    pub(crate) fn new() -> Self {
        let id = NEXT_TEMP_DIR.fetch_add(1, Ordering::Relaxed);
        let path =
            std::env::temp_dir().join(format!("riftbound_test_{}_{}", std::process::id(), id));
        std::fs::create_dir_all(&path).expect("create test directory");
        Self(path)
    }
}

impl Drop for TestDir {
    fn drop(&mut self) {
        let _ = std::fs::remove_dir_all(&self.0);
    }
}
