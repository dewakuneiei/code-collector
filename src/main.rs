#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod app;
mod ui;
mod models;
mod scanner;
mod operations;

use app::CodeCollectorApp;
use eframe::egui;

fn main() -> Result<(), eframe::Error> {
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default().with_inner_size([900.0, 800.0]),
        ..Default::default()
    };
    eframe::run_native("Code Collector", options, Box::new(|_cc| Ok(Box::new(CodeCollectorApp::default()))))
}