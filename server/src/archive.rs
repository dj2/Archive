use rocket::response::NamedFile;
use std::path::Path;

pub struct Archive {
    data_dir: String,
    asset_dir: String,
}

impl Archive {
    pub fn new(data_dir: &str, asset_dir: &str) -> Self {
        Self {
            data_dir: data_dir.to_string(),
            asset_dir: asset_dir.to_string(),
        }
    }

    async fn retrieve(&self, prefix: &str, file: &Path) -> Option<NamedFile> {
        NamedFile::open(Path::new(prefix).join(file)).await.ok()
    }

    pub async fn retrieve_asset(&self, file: &Path) -> Option<NamedFile> {
        self.retrieve(&self.asset_dir, &file).await
    }

    pub async fn retrieve_note(&self, file: &Path) -> Option<NamedFile> {
        self.retrieve(&self.data_dir, &file).await
    }
}
