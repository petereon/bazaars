use serde::{Serialize, Serializer};

pub struct Image {
    pub id: Option<String>,
    pub file_name: String,
    pub mime_type: String,
    pub bytes: Vec<u8>,
}

impl Serialize for Image {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_some(&self.id)
    }
}
