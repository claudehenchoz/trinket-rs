use egui;
use egui_extras::{Column, TableBuilder};
use crate::storage::Snippet;
use chrono::{DateTime, Local};

#[derive(Default)]
pub struct GetWindowState {
    search_query: String,
    filtered_indices: Vec<usize>,
    selected_index: usize,
    first_frame: bool,
}

pub struct SnippetView {
    pub snippet: Snippet,
    pub match_score: f32,
    pub highlighted_preview: String,
}

impl GetWindowState {
    pub fn new() -> Self {
        Self {
            search_query: String::new(),
            filtered_indices: Vec::new(),
            selected_index: 0,
            first_frame: true,
        }
    }
    
    pub fn show(&mut self, ctx: &egui::Context, snippets: &[Snippet]) -> Option<String> {
        let mut selected_content = None;
        
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.horizontal(|ui| {
                ui.label("Search:");
                let search_response = ui.text_edit_singleline(&mut self.search_query);
                
                if self.first_frame {
                    search_response.request_focus();
                    self.first_frame = false;
                }
            });
            
            ui.separator();
            
            self.update_filtered_results(snippets);
            
            let table = TableBuilder::new(ui)
                .striped(true)
                .resizable(true)
                .cell_layout(egui::Layout::left_to_right(egui::Align::Center))
                .column(Column::auto().at_least(120.0))
                .column(Column::remainder())
                .min_scrolled_height(300.0);
            
            table
                .header(20.0, |mut header| {
                    header.col(|ui| { ui.strong("Date"); });
                    header.col(|ui| { ui.strong("Preview"); });
                })
                .body(|body| {
                    body.rows(
                        25.0, 
                        self.filtered_indices.len(),
                        |mut row| {
                            let list_index = row.index();
                            if list_index < self.filtered_indices.len() {
                                let snippet_index = self.filtered_indices[list_index];
                                if snippet_index < snippets.len() {
                                    let snippet = &snippets[snippet_index];
                                    let is_selected = list_index == self.selected_index;
                                    
                                    row.set_selected(is_selected);
                                    
                                    row.col(|ui| {
                                        ui.label(format_timestamp(snippet.created));
                                    });
                                    
                                    row.col(|ui| {
                                        let highlighted = highlight_matches(&snippet.preview, &self.search_query);
                                        ui.label(highlighted);
                                    });
                                    
                                    if row.response().clicked() {
                                        self.selected_index = list_index;
                                        selected_content = Some(snippet.content.clone());
                                    }
                                }
                            }
                        }
                    );
                });
        });
        
        ctx.input_mut(|i| {
            if i.key_pressed(egui::Key::ArrowUp) && self.selected_index > 0 {
                self.selected_index -= 1;
            }
            if i.key_pressed(egui::Key::ArrowDown) && self.selected_index < self.filtered_indices.len().saturating_sub(1) {
                self.selected_index += 1;
            }
            if i.key_pressed(egui::Key::Enter) && !self.filtered_indices.is_empty() && self.selected_index < self.filtered_indices.len() {
                let snippet_index = self.filtered_indices[self.selected_index];
                if snippet_index < snippets.len() {
                    selected_content = Some(snippets[snippet_index].content.clone());
                }
            }
            if i.key_pressed(egui::Key::Escape) {
                selected_content = Some(String::new());
            }
        });
        
        selected_content
    }
    
    fn update_filtered_results(&mut self, snippets: &[Snippet]) {
        if self.search_query.is_empty() {
            self.filtered_indices = (0..snippets.len()).collect();
        } else {
            let query_lower = self.search_query.to_lowercase();
            self.filtered_indices = snippets.iter()
                .enumerate()
                .filter_map(|(idx, snippet)| {
                    let content_lower = snippet.content.to_lowercase();
                    if content_lower.contains(&query_lower) {
                        Some(idx)
                    } else {
                        None
                    }
                })
                .collect();
        }
        
        if self.selected_index >= self.filtered_indices.len() {
            self.selected_index = self.filtered_indices.len().saturating_sub(1);
        }
    }
    
    pub fn reset(&mut self) {
        self.first_frame = true;
        self.search_query.clear();
        self.selected_index = 0;
        self.filtered_indices.clear();
    }
}

fn format_timestamp(time: std::time::SystemTime) -> String {
    let datetime: DateTime<Local> = time.into();
    datetime.format("%m/%d %H:%M").to_string()
}

fn highlight_matches(text: &str, query: &str) -> String {
    if query.is_empty() {
        return text.to_string();
    }
    
    text.to_string()
}