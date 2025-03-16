use std::sync::Arc;

use crate::models::image::Image;
use anyhow::Error;
use axum::async_trait;
use serde::{Deserialize, Serialize};

#[async_trait]
pub trait ImageRepo: Send + Sync {
    async fn get_image(&self, id: &str) -> Result<Image, Error>;
    async fn create_image(
        &self,
        id: String,
        bytes: Vec<u8>,
        mime_type: String,
    ) -> Result<String, Error>;
    async fn delete_image(&self, id: &str) -> Result<(), Error>;
}

#[derive(Clone)]
pub struct LocalImageRepo {
    image_dir: String,
}

impl LocalImageRepo {
    pub fn new(image_dir: String) -> Arc<LocalImageRepo> {
        Arc::new(LocalImageRepo { image_dir })
    }
}

#[derive(Deserialize, Serialize)]
struct ImageMetadataFile {
    file_name: String,
    mime_type: String,
}

#[async_trait]
impl ImageRepo for LocalImageRepo {
    async fn get_image(&self, id: &str) -> Result<Image, Error> {
        let path = format!("{}/{}", self.image_dir, id);
        let meta_path = format!("{}/{}.meta", self.image_dir, id);

        let bytes = tokio::fs::read(path).await?;
        let metadata_str = tokio::fs::read_to_string(meta_path).await?;

        let metadata: ImageMetadataFile = serde_json::from_str(&metadata_str)?;

        Ok(Image {
            id: Some(id.to_string()),
            file_name: metadata.file_name,
            mime_type: metadata.mime_type,
            bytes,
        })
    }

    async fn create_image(
        &self,
        file_name: String,
        bytes: Vec<u8>,
        mime_type: String,
    ) -> Result<String, Error> {
        let image_id = uuid::Uuid::new_v4().to_string();
        let path = format!("{}/{}", self.image_dir, image_id);
        let meta_path = format!("{}/{}.meta", self.image_dir, image_id);

        let meta = ImageMetadataFile {
            file_name,
            mime_type: mime_type.clone(),
        };

        tokio::fs::write(path, bytes).await?;
        tokio::fs::write(meta_path, serde_json::to_string(&meta)?).await?;

        Ok(image_id)
    }

    async fn delete_image(&self, id: &str) -> Result<(), Error> {
        let path = format!("{}/{}", self.image_dir, id);
        let meta_path = format!("{}/{}.meta", self.image_dir, id);

        tokio::fs::remove_file(path).await?;
        tokio::fs::remove_file(meta_path).await?;

        Ok(())
    }
}
