# Trinket - Design Document

## Overview

Trinket is a high-performance system tray application for Windows that provides lightning-fast text snippet storage and retrieval. The application runs continuously in the system tray and responds to global hotkeys for instant access to snippet management.

### Core Requirements
- **Add Snippet**: WIN+PgUp opens a text entry window with syntax highlighting
- **Get Snippet**: WIN+PgDown opens a searchable snippet browser
- **Performance**: Sub-100ms window launch, instant search across thousands of snippets
- **Persistence**: Snippets stored as individual .txt files for portability

## Architecture

### Application Structure

```
trinket/
├── src/
│   ├── main.rs              # Entry point, system tray setup
│   ├── app.rs               # Main application state and logic
│   ├── ui/
│   │   ├── mod.rs           # UI module exports
│   │   ├── add_window.rs    # Add snippet window
│   │   └── get_window.rs    # Get snippet window
│   ├── storage/
│   │   ├── mod.rs           # Storage module exports
│   │   ├── file_ops.rs      # File operations
│   │   └── indexer.rs       # Search indexing
│   ├── hotkeys.rs           # Global hotkey management
│   └── clipboard.rs         # Clipboard operations
├── snippets/                # Default snippet storage directory
└── Cargo.toml
```

### State Management

```rust
#[derive(Default)]
pub struct TrinketApp {
    // UI State
    mode: AppMode,
    add_window: AddWindowState,
    get_window: GetWindowState,
    
    // Data State
    snippets: Vec<Snippet>,
    search_index: SearchIndex,
    
    // System State
    hotkey_receiver: mpsc::Receiver<HotkeyEvent>,
    file_watcher: FileWatcher,
}

#[derive(Default)]
enum AppMode {
    #[default]
    Hidden,
    AddingSnippet,
    GettingSnippet,
}

struct Snippet {
    id: String,           // UUID
    content: String,      // Full text content
    preview: String,      // First 3 lines
    created: SystemTime,  // File creation time
    modified: SystemTime, // Last modified
    file_path: PathBuf,   // Absolute path
}
```

## Implementation Details

### 1. System Tray and Window Management

```rust
// main.rs - Application entry point
use eframe::egui;
use tray_icon::{TrayIcon, TrayIconBuilder};
use global_hotkey::{GlobalHotKeyManager, hotkey};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize logging
    env_logger::init();
    
    // Create channels for hotkey events
    let (hotkey_tx, hotkey_rx) = mpsc::channel();
    
    // Setup global hotkeys
    let manager = GlobalHotKeyManager::new()?;
    let add_hotkey = hotkey!(windows + pageup);
    let get_hotkey = hotkey!(windows + pagedown);
    
    manager.register(add_hotkey)?;
    manager.register(get_hotkey)?;
    
    // Setup system tray
    let tray_icon = TrayIconBuilder::new()
        .with_tooltip("Trinket - Text Snippets")
        .with_icon(icon)
        .build()?;
    
    // Run egui application
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_decorations(false)
            .with_transparent(true)
            .with_always_on_top()
            .with_visible(false), // Start hidden
        ..Default::default()
    };
    
    eframe::run_native(
        "Trinket",
        options,
        Box::new(|cc| Box::new(TrinketApp::new(cc, hotkey_rx))),
    )?;
    
    Ok(())
}
```

### 2. Add Snippet Window

```rust
// ui/add_window.rs
use egui_code_editor::{CodeEditor, ColorTheme};

pub struct AddWindowState {
    text_buffer: String,
    editor_theme: ColorTheme,
}

impl AddWindowState {
    pub fn show(&mut self, ctx: &egui::Context) -> Option<String> {
        let mut save_triggered = false;
        let mut close_triggered = false;
        
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.heading("Add New Snippet");
            
            // Code editor with syntax highlighting
            CodeEditor::default()
                .id_source("snippet_editor")
                .with_rows(20)
                .with_fontsize(14.0)
                .with_theme(self.editor_theme)
                .with_syntax(egui_code_editor::Syntax::markdown())
                .with_numlines(true)
                .show(ui, &mut self.text_buffer);
            
            ui.separator();
            
            // Buttons
            ui.horizontal(|ui| {
                if ui.button("Save and Close (Ctrl+Enter)").clicked() {
                    save_triggered = true;
                }
                if ui.button("Cancel (Esc)").clicked() {
                    close_triggered = true;
                }
            });
        });
        
        // Handle keyboard shortcuts
        ctx.input_mut(|i| {
            if i.consume_key(egui::Modifiers::CTRL, egui::Key::Enter) {
                save_triggered = true;
            }
            if i.consume_key(egui::Modifiers::NONE, egui::Key::Escape) {
                close_triggered = true;
            }
        });
        
        if save_triggered && !self.text_buffer.is_empty() {
            Some(std::mem::take(&mut self.text_buffer))
        } else if close_triggered {
            self.text_buffer.clear();
            None
        } else {
            None
        }
    }
}
```

### 3. Get Snippet Window

```rust
// ui/get_window.rs
use egui_extras::{Column, TableBuilder};

pub struct GetWindowState {
    search_query: String,
    filtered_snippets: Vec<SnippetView>,
    selected_index: usize,
    table_state: TableState,
}

struct SnippetView {
    snippet: Snippet,
    match_score: f32,
    highlighted_preview: String,
}

impl GetWindowState {
    pub fn show(&mut self, ctx: &egui::Context, snippets: &[Snippet]) -> Option<String> {
        let mut selected_content = None;
        
        egui::CentralPanel::default().show(ctx, |ui| {
            // Search box with auto-focus
            ui.horizontal(|ui| {
                ui.label("Search:");
                let search_response = ui.text_edit_singleline(&mut self.search_query);
                
                // Auto-focus on first frame
                if self.table_state.first_frame {
                    search_response.request_focus();
                    self.table_state.first_frame = false;
                }
            });
            
            ui.separator();
            
            // Update filtered results when search changes
            if self.search_query != self.table_state.last_query {
                self.update_filtered_results(snippets);
                self.table_state.last_query = self.search_query.clone();
            }
            
            // Snippet table
            let table = TableBuilder::new(ui)
                .striped(true)
                .resizable(true)
                .cell_layout(egui::Layout::left_to_right(egui::Align::Center))
                .column(Column::auto()) // Date
                .column(Column::remainder()) // Preview
                .min_scrolled_height(0.0);
            
            table
                .header(20.0, |mut header| {
                    header.col(|ui| { ui.strong("Date"); });
                    header.col(|ui| { ui.strong("Preview"); });
                })
                .body(|mut body| {
                    body.rows(
                        20.0, 
                        self.filtered_snippets.len(),
                        |mut row| {
                            let index = row.index();
                            let snippet_view = &self.filtered_snippets[index];
                            let is_selected = index == self.selected_index;
                            
                            row.set_selected(is_selected);
                            
                            row.col(|ui| {
                                ui.label(format_timestamp(snippet_view.snippet.created));
                            });
                            
                            row.col(|ui| {
                                // Render highlighted preview
                                ui.label(&snippet_view.highlighted_preview);
                            });
                            
                            // Handle click
                            if row.response().clicked() {
                                self.selected_index = index;
                                selected_content = Some(snippet_view.snippet.content.clone());
                            }
                        }
                    );
                });
        });
        
        // Handle keyboard navigation
        ctx.input_mut(|i| {
            if i.key_pressed(egui::Key::ArrowUp) && self.selected_index > 0 {
                self.selected_index -= 1;
            }
            if i.key_pressed(egui::Key::ArrowDown) && self.selected_index < self.filtered_snippets.len() - 1 {
                self.selected_index += 1;
            }
            if i.key_pressed(egui::Key::Enter) && !self.filtered_snippets.is_empty() {
                selected_content = Some(self.filtered_snippets[self.selected_index].snippet.content.clone());
            }
            if i.key_pressed(egui::Key::Escape) {
                selected_content = Some(String::new()); // Signal to close
            }
        });
        
        selected_content
    }
    
    fn update_filtered_results(&mut self, snippets: &[Snippet]) {
        if self.search_query.is_empty() {
            self.filtered_snippets = snippets.iter()
                .map(|s| SnippetView {
                    snippet: s.clone(),
                    match_score: 1.0,
                    highlighted_preview: s.preview.clone(),
                })
                .collect();
        } else {
            // Simple substring search for now
            self.filtered_snippets = snippets.iter()
                .filter_map(|s| {
                    let content_lower = s.content.to_lowercase();
                    let query_lower = self.search_query.to_lowercase();
                    
                    if content_lower.contains(&query_lower) {
                        Some(SnippetView {
                            snippet: s.clone(),
                            match_score: 1.0,
                            highlighted_preview: highlight_matches(&s.preview, &self.search_query),
                        })
                    } else {
                        None
                    }
                })
                .collect();
        }
        
        // Reset selection
        self.selected_index = 0;
    }
}
```

### 4. Storage and Indexing

```rust
// storage/file_ops.rs
use std::fs;
use std::path::{Path, PathBuf};
use uuid::Uuid;

pub struct FileStorage {
    base_path: PathBuf,
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
        
        // Atomic write using tempfile
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
        
        // Sort by creation date, newest first
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
```

### 5. Clipboard Integration

```rust
// clipboard.rs
use arboard::Clipboard;

pub fn copy_to_clipboard(text: &str) -> Result<(), Box<dyn std::error::Error>> {
    let mut clipboard = Clipboard::new()?;
    clipboard.set_text(text)?;
    Ok(())
}
```

### 6. Main Application Logic

```rust
// app.rs
impl TrinketApp {
    pub fn new(cc: &eframe::CreationContext<'_>, hotkey_rx: mpsc::Receiver<HotkeyEvent>) -> Self {
        // Setup file storage
        let storage_path = dirs::data_local_dir()
            .unwrap_or_else(|| PathBuf::from("."))
            .join("trinket")
            .join("snippets");
        
        let storage = FileStorage::new(storage_path).expect("Failed to create storage");
        let snippets = storage.load_all_snippets().unwrap_or_default();
        
        // Setup file watcher
        let (watcher_tx, watcher_rx) = mpsc::channel();
        let mut watcher = notify::recommended_watcher(move |evt| {
            let _ = watcher_tx.send(evt);
        }).expect("Failed to create file watcher");
        
        watcher.watch(&storage.base_path, RecursiveMode::NonRecursive)
            .expect("Failed to watch snippet directory");
        
        Self {
            mode: AppMode::Hidden,
            add_window: AddWindowState::default(),
            get_window: GetWindowState::default(),
            snippets,
            search_index: SearchIndex::new(),
            hotkey_receiver: hotkey_rx,
            file_watcher: watcher,
            storage,
        }
    }
}

impl eframe::App for TrinketApp {
    fn update(&mut self, ctx: &egui::Context, frame: &mut eframe::Frame) {
        // Check for hotkey events
        if let Ok(event) = self.hotkey_receiver.try_recv() {
            match event {
                HotkeyEvent::Add => {
                    self.mode = AppMode::AddingSnippet;
                    frame.set_visible(true);
                    frame.focus();
                }
                HotkeyEvent::Get => {
                    self.mode = AppMode::GettingSnippet;
                    self.get_window.table_state.first_frame = true;
                    frame.set_visible(true);
                    frame.focus();
                }
            }
        }
        
        // Handle different modes
        match self.mode {
            AppMode::Hidden => {
                frame.set_visible(false);
            }
            AppMode::AddingSnippet => {
                if let Some(content) = self.add_window.show(ctx) {
                    // Save snippet
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
                    
                    // Hide window
                    self.mode = AppMode::Hidden;
                    frame.set_visible(false);
                }
            }
            AppMode::GettingSnippet => {
                if let Some(content) = self.get_window.show(ctx, &self.snippets) {
                    if !content.is_empty() {
                        // Copy to clipboard
                        if let Err(e) = copy_to_clipboard(&content) {
                            log::error!("Failed to copy to clipboard: {}", e);
                        }
                    }
                    
                    // Hide window
                    self.mode = AppMode::Hidden;
                    frame.set_visible(false);
                }
            }
        }
        
        // Check for file system changes
        if let Ok(event) = self.file_watcher_rx.try_recv() {
            // Reload snippets if files changed
            if let Ok(snippets) = self.storage.load_all_snippets() {
                self.snippets = snippets;
            }
        }
    }
}
```

## Cargo Dependencies

```toml
[package]
name = "trinket"
version = "0.1.0"
edition = "2021"

[dependencies]
# Core egui framework
eframe = { version = "0.29", features = ["default"] }
egui = "0.29"
egui_extras = { version = "0.29", features = ["all"] }
egui_code_editor = "0.2"

# System integration
tray-icon = "0.21"
global-hotkey = "0.7"
arboard = "3.4"

# File operations
notify = "7.0"
notify-debouncer-full = "0.5"
tempfile = "3.14"

# Utilities
uuid = { version = "1.11", features = ["v4"] }
dirs = "5.0"
serde = { version = "1.0", features = ["derive"] }
serde_json = "1.0"
chrono = "0.4"
log = "0.4"
env_logger = "0.11"

# Platform-specific
[target.'cfg(windows)'.dependencies]
windows = { version = "0.60", features = ["Win32_Foundation", "Win32_UI_WindowsAndMessaging"] }
```

## Performance Considerations

1. **Startup Performance**
   - Load snippets asynchronously after window creation
   - Cache snippet metadata in a binary format (using bincode)
   - Lazy-load full snippet content only when needed

2. **Search Performance**
   - For <1000 snippets: Simple in-memory substring search
   - For 1000+ snippets: Consider adding tantivy for full-text indexing
   - Implement debounced search to avoid excessive filtering

3. **Memory Usage**
   - Store only preview + metadata in memory
   - Load full content on-demand from disk
   - Implement LRU cache for recently accessed snippets

## Platform-Specific Notes

### Windows
- Global hotkeys work natively with the chosen crate
- System tray requires no special handling
- File paths use standard Windows conventions

### Future Linux/macOS Support
- Would require platform-specific window management code
- Tray icon implementation differs on Linux (GTK vs Qt)
- Global hotkeys may require additional permissions on macOS

## Security Considerations

1. **File Storage**
   - Snippets stored as plain text files
   - No encryption by default (could be added)
   - Relies on OS file permissions

2. **Clipboard Access**
   - Uses standard clipboard APIs
   - No clipboard history tracking
   - Clear clipboard after timeout (optional feature)

## Testing Strategy

1. **Unit Tests**
   - File operations (save/load/delete)
   - Search/filter logic
   - Preview generation

2. **Integration Tests**
   - Hotkey registration/handling
   - Window show/hide cycles
   - Clipboard operations

3. **Performance Tests**
   - Load time with 1000+ snippets
   - Search response time
   - Memory usage monitoring

## Deployment

1. **Build Configuration**
   ```toml
   [profile.release]
   opt-level = 3
   lto = true
   strip = true
   ```
