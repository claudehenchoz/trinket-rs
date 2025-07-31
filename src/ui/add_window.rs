use egui;

#[derive(Default)]
pub struct AddWindowState {
    text_buffer: String,
}

impl AddWindowState {
    pub fn new() -> Self {
        Self {
            text_buffer: String::new(),
        }
    }
    
    pub fn show(&mut self, ctx: &egui::Context) -> Option<String> {
        let mut save_triggered = false;
        let mut close_triggered = false;
        
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.heading("Add New Snippet");
            ui.add_space(10.0);
            
            egui::ScrollArea::vertical()
                .max_height(300.0)
                .show(ui, |ui| {
                    ui.text_edit_multiline(&mut self.text_buffer)
                        .request_focus();
                });
            
            ui.add_space(10.0);
            ui.separator();
            ui.add_space(10.0);
            
            ui.horizontal(|ui| {
                if ui.button("Save and Close (Ctrl+Enter)").clicked() {
                    save_triggered = true;
                }
                if ui.button("Cancel (Esc)").clicked() {
                    close_triggered = true;
                }
            });
        });
        
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