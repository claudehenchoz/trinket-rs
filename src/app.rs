use eframe::egui;
use std::path::PathBuf;
use std::sync::mpsc;

use crate::clipboard::copy_to_clipboard;
use crate::hotkeys::HotkeyEvent;
use crate::storage::{FileStorage, SearchIndex, Snippet};
use crate::ui::{AddWindowState, GetWindowState};

#[derive(Default)]
pub enum AppMode {
    #[default]
    Hidden,
    AddingSnippet,
    GettingSnippet,
}

pub struct TrinketApp {
    mode: AppMode,
    add_window: AddWindowState,
    get_window: GetWindowState,
    
    snippets: Vec<Snippet>,
    search_index: SearchIndex,
    
    hotkey_receiver: mpsc::Receiver<HotkeyEvent>,
    storage: FileStorage,
}

impl TrinketApp {
    pub fn new(_cc: &eframe::CreationContext<'_>, hotkey_rx: mpsc::Receiver<HotkeyEvent>) -> Self {
        let storage_path = dirs::data_local_dir()
            .unwrap_or_else(|| PathBuf::from("."))
            .join("trinket")
            .join("snippets");
        
        let storage = FileStorage::new(storage_path).expect("Failed to create storage");
        let snippets = storage.load_all_snippets().unwrap_or_default();
        
        Self {
            mode: AppMode::Hidden,
            add_window: AddWindowState::new(),
            get_window: GetWindowState::new(),
            snippets,
            search_index: SearchIndex::new(),
            hotkey_receiver: hotkey_rx,
            storage,
        }
    }
}

impl eframe::App for TrinketApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        if let Ok(event) = self.hotkey_receiver.try_recv() {
            match event {
                HotkeyEvent::Add => {
                    self.mode = AppMode::AddingSnippet;
                }
                HotkeyEvent::Get => {
                    self.mode = AppMode::GettingSnippet;
                    self.get_window.reset();
                }
            }
        }
        
        match self.mode {
            AppMode::Hidden => {
                // Window is controlled by hotkey events
            }
            AppMode::AddingSnippet => {
                if let Some(content) = self.add_window.show(ctx) {
                    if !content.is_empty() {
                        match self.storage.save_snippet(&content) {
                            Ok(snippet) => {
                                self.snippets.insert(0, snippet);
                                log::info!("Snippet saved successfully");
                            }
                            Err(e) => {
                                log::error!("Failed to save snippet: {}", e);
                            }
                        }
                    }
                    
                    self.mode = AppMode::Hidden;
                }
            }
            AppMode::GettingSnippet => {
                if let Some(content) = self.get_window.show(ctx, &self.snippets) {
                    if !content.is_empty() {
                        if let Err(e) = copy_to_clipboard(&content) {
                            log::error!("Failed to copy to clipboard: {}", e);
                        } else {
                            log::info!("Snippet copied to clipboard");
                        }
                    }
                    
                    self.mode = AppMode::Hidden;
                }
            }
        }
        
        ctx.request_repaint();
    }
}