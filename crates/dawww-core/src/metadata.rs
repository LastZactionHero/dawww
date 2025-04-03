use serde::{Deserialize, Serialize};
use std::time::SystemTime;
use std::time::{UNIX_EPOCH};

#[derive(Debug, Serialize, Deserialize)]
pub struct Metadata {
    pub title: String,
    pub creation_date: String,
    pub modification_date: String,
    pub revision: u32,
}

impl Metadata {
    /// Create new metadata with the given title
    pub fn new(title: String) -> Self {
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs();
        let iso_date = chrono::DateTime::from_timestamp(now as i64, 0)
            .unwrap()
            .to_rfc3339();

        Self {
            title,
            creation_date: iso_date.clone(),
            modification_date: iso_date,
            revision: 0,
        }
    }

    /// Update the title and modification date
    pub fn set_title(&mut self, title: String) {
        self.title = title;
        // Sleep briefly to ensure different timestamp
        std::thread::sleep(std::time::Duration::from_secs(1));
        self.update_modification_date();
    }

    /// Update the modification date to the current time
    pub fn update_modification_date(&mut self) {
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs();
        self.modification_date = chrono::DateTime::from_timestamp(now as i64, 0)
            .unwrap()
            .to_rfc3339();
    }

    /// Increment the revision number
    pub fn increment_revision(&mut self) {
        self.revision += 1;
    }

    /// Get the modification date as a DateTime
    pub fn modification_date(&self) -> chrono::DateTime<chrono::Utc> {
        chrono::DateTime::parse_from_rfc3339(&self.modification_date)
            .unwrap()
            .with_timezone(&chrono::Utc)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_metadata_new() {
        let title = "Test Song".to_string();
        let metadata = Metadata::new(title.clone());
        
        assert_eq!(metadata.title, title);
        assert_eq!(metadata.revision, 0);
        
        // Verify dates are valid ISO 8601
        assert!(chrono::DateTime::parse_from_rfc3339(&metadata.creation_date).is_ok());
        assert!(chrono::DateTime::parse_from_rfc3339(&metadata.modification_date).is_ok());
        assert_eq!(metadata.creation_date, metadata.modification_date);
    }

    #[test]
    fn test_metadata_set_title() {
        let mut metadata = Metadata::new("Original Title".to_string());
        let original_date = metadata.modification_date.clone();
        
        metadata.set_title("New Title".to_string());
        
        assert_eq!(metadata.title, "New Title");
        assert!(metadata.modification_date != original_date);
        assert!(chrono::DateTime::parse_from_rfc3339(&metadata.modification_date).is_ok());
    }

    #[test]
    fn test_metadata_increment_revision() {
        let mut metadata = Metadata::new("Test Song".to_string());
        assert_eq!(metadata.revision, 0);
        
        metadata.increment_revision();
        assert_eq!(metadata.revision, 1);
        
        metadata.increment_revision();
        assert_eq!(metadata.revision, 2);
    }
} 