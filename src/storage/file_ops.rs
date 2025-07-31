use std::fs;
use std::path::PathBuf;
use std::time::SystemTime;
use uuid::Uuid;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Snippet {
    pub id: String,
    pub content: String,
    pub preview: String,
    pub created: SystemTime,
    pub modified: SystemTime,
    pub file_path: PathBuf,
}

pub struct FileStorage {
    pub base_path: PathBuf,
}

impl FileStorage {
    pub fn new(base_path: PathBuf) -> Result<Self, std::io::Error> {
        fs::create_dir_all(&base_path)?;
        Ok(Self { base_path })
    }
    
    pub fn save_snippet(&self, content: &str) -> Result<Snippet, std::io::Error> {
        let id = Uuid::new_v4().to_string();
        let filename = format!("{}.txt", id);
        let file_path = self.base_path.join(&filename);
        
        use tempfile::NamedTempFile;
        let temp_file = NamedTempFile::new_in(&self.base_path)?;
        fs::write(&temp_file, content)?;
        temp_file.persist(&file_path)?;
        
        let metadata = fs::metadata(&file_path)?;
        let created = metadata.created().unwrap_or_else(|_| SystemTime::now());
        let modified = metadata.modified().unwrap_or_else(|_| SystemTime::now());
        
        Ok(Snippet {
            id,
            content: content.to_string(),
            preview: create_preview(content),
            created,
            modified,
            file_path,
        })
    }
    
    pub fn load_all_snippets(&self) -> Result<Vec<Snippet>, std::io::Error> {
        let mut snippets = Vec::new();
        
        for entry in fs::read_dir(&self.base_path)? {
            let entry = entry?;
            let path = entry.path();
            
            if path.extension().and_then(|s| s.to_str()) == Some("txt") {
                let content = fs::read_to_string(&path)?;
                let metadata = entry.metadata()?;
                
                let id = path.file_stem()
                    .and_then(|s| s.to_str())
                    .unwrap_or_default()
                    .to_string();
                
                snippets.push(Snippet {
                    id,
                    content: content.clone(),
                    preview: create_preview(&content),
                    created: metadata.created().unwrap_or_else(|_| SystemTime::now()),
                    modified: metadata.modified().unwrap_or_else(|_| SystemTime::now()),
                    file_path: path,
                });
            }
        }
        
        snippets.sort_by(|a, b| b.created.cmp(&a.created));
        
        Ok(snippets)
    }
}

fn create_preview(content: &str) -> String {
    content.lines()
        .take(3)
        .collect::<Vec<_>>()
        .join(" ")
        .chars()
        .take(200)
        .collect()
}