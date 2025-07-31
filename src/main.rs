use eframe::egui;
use global_hotkey::{GlobalHotKeyManager, hotkey::{HotKey, Modifiers, Code}};
use std::sync::mpsc;
use tray_icon::TrayIconBuilder;

mod app;
mod clipboard;
mod hotkeys;
mod storage;
mod ui;

use app::TrinketApp;
use hotkeys::HotkeyEvent;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    env_logger::init();

    let (hotkey_tx, hotkey_rx) = mpsc::channel();

    let manager = GlobalHotKeyManager::new()?;
    let add_hotkey = HotKey::new(Some(Modifiers::SUPER | Modifiers::CONTROL), Code::PageUp);
    let get_hotkey = HotKey::new(Some(Modifiers::SUPER | Modifiers::CONTROL), Code::PageDown);

    manager.register(add_hotkey)?;
    manager.register(get_hotkey)?;

    let _hotkey_handler = std::thread::spawn(move || {
        loop {
            if let Ok(event) = global_hotkey::GlobalHotKeyEvent::receiver().try_recv() {
                if event.id == add_hotkey.id() {
                    let _ = hotkey_tx.send(HotkeyEvent::Add);
                } else if event.id == get_hotkey.id() {
                    let _ = hotkey_tx.send(HotkeyEvent::Get);
                }
            }
            std::thread::sleep(std::time::Duration::from_millis(10));
        }
    });

    let icon = tray_icon::Icon::from_rgba(vec![0; 32*32*4], 32, 32)?;

    let _tray_icon = TrayIconBuilder::new()
        .with_tooltip("Trinket - Text Snippets")
        .with_icon(icon)
        .build()?;

    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_decorations(false)
            .with_transparent(true)
            .with_always_on_top()
            .with_visible(false)
            .with_resizable(true)
            .with_inner_size([600.0, 400.0]),
        ..Default::default()
    };

    eframe::run_native(
        "Trinket",
        options,
        Box::new(|cc| Ok(Box::new(TrinketApp::new(cc, hotkey_rx)))),
    )?;

    Ok(())
}