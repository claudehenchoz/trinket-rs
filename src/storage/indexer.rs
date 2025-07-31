use super::Snippet;

#[derive(Default)]
pub struct SearchIndex {
    // Simple implementation for now - could be expanded with proper indexing
}

impl SearchIndex {
    pub fn new() -> Self {
        Self::default()
    }
    
    pub fn search(&self, query: &str, snippets: &[Snippet]) -> Vec<usize> {
        if query.is_empty() {
            return (0..snippets.len()).collect();
        }
        
        let query_lower = query.to_lowercase();
        snippets.iter()
            .enumerate() 
            .filter_map(|(idx, snippet)| {
                let content_lower = snippet.content.to_lowercase();
                if content_lower.contains(&query_lower) {
                    Some(idx)
                } else {
                    None
                }
            })
            .collect()
    }
}