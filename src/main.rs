use eframe::egui;
use global_hotkey::{GlobalHotKeyManager, hotkey::{HotKey, Modifiers, Code}};
use std::sync::mpsc;
use tray_icon::TrayIconBuilder;
use image::ImageFormat;

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

    let icon_bytes = include_bytes!("../assets/trinket.ico");
    let img = image::load_from_memory_with_format(icon_bytes, ImageFormat::Ico)
        .map_err(|e| format!("Failed to load tray icon: {}", e))?;
    let rgba_img = img.to_rgba8();
    let (width, height) = rgba_img.dimensions();
    let icon = tray_icon::Icon::from_rgba(rgba_img.into_raw(), width, height)?;

    let _tray_icon = TrayIconBuilder::new()
        .with_tooltip("Trinket - Text Snippets")
        .with_icon(icon)
        .build()?;

    let egui_icon_bytes = include_bytes!("../assets/trinket.ico");
    let img = image::load_from_memory_with_format(egui_icon_bytes, ImageFormat::Ico)
        .map_err(|e| format!("Failed to load icon: {}", e))?;
    let rgba_img = img.to_rgba8();
    let (width, height) = rgba_img.dimensions();
    let egui_icon = egui::IconData {
        rgba: rgba_img.into_raw(),
        width: width as u32,
        height: height as u32,
    };
    
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_decorations(false)
            .with_transparent(true)
            .with_always_on_top()
            .with_visible(false)
            .with_resizable(true)
            .with_inner_size([600.0, 400.0])
            .with_icon(egui_icon),
        ..Default::default()
    };

    eframe::run_native(
        "Trinket",
        options,
        Box::new(|cc| Ok(Box::new(TrinketApp::new(cc, hotkey_rx)))),
    )?;

    Ok(())
}