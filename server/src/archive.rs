//! The Archive object gives access to the files in the archive. It is used
//! to work with the notes and assets which are stored there.

use rocket::response::NamedFile;
use std::path::Path;

/// The Archive provides access to the notes and assets in the archive.
pub struct Archive {
    data_dir: String,
    asset_dir: String,
}

impl Archive {
    /// Create a new archive with `data_dir` and `asset_dir` as provided.
    pub fn new(data_dir: &str, asset_dir: &str) -> Self {
        Self {
            data_dir: data_dir.to_string(),
            asset_dir: asset_dir.to_string(),
        }
    }

    async fn retrieve(&self, prefix: &str, file: &Path) -> Option<NamedFile> {
        NamedFile::open(Path::new(prefix).join(file)).await.ok()
    }

    /// Retrieve the asset at `file`. The `file` is the file path from the root
    /// of the assets directory.
    pub async fn retrieve_asset(&self, file: &Path) -> Option<NamedFile> {
        self.retrieve(&self.asset_dir, &file).await
    }

    /// Retrieve the asset at `file`. The `file` is the file path from the root
    /// of the notes directory.
    pub async fn retrieve_note(&self, file: &Path) -> Option<NamedFile> {
        self.retrieve(&self.data_dir, &file).await
    }
}
