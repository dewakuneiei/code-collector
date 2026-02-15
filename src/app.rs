use eframe::egui;
use std::collections::VecDeque;
use std::fs;
use std::path::PathBuf;
use std::sync::{Arc, atomic::{AtomicBool, Ordering}};
use std::thread;
use std::sync::mpsc::{Receiver, channel};

use crate::models::{dir_node::DirNode, file_node::FileNode, export_mode::ExportMode, theme::ThemePreference};
use crate::scanner::{ScanMessage, thread::read_dir_recursive_threaded};
use crate::operations::{selection, export};
use crate::ui::{panels, tree};

pub struct CodeCollectorApp {
    pub project_path: Option<PathBuf>,
    pub root_node: Option<DirNode>,
    pub status_text: String,
    pub export_mode: ExportMode,
    pub theme: ThemePreference,
    pub search_query: String,
    pub recent_files: VecDeque<FileNode>,

    // --- LOADING STATE ---
    pub is_loading: bool,
    pub loading_count: usize,
    pub loading_channel: Option<Receiver<ScanMessage>>,
    pub cancel_flag: Option<Arc<AtomicBool>>,
}

impl Default for CodeCollectorApp {
    fn default() -> Self {
        Self {
            project_path: None,
            root_node: None,
            status_text: String::from("Ready to scan."),
            export_mode: ExportMode::OneFile,
            theme: ThemePreference::System,
            search_query: String::new(),
            recent_files: VecDeque::with_capacity(3),
            is_loading: false,
            loading_count: 0,
            loading_channel: None,
            cancel_flag: None,
        }
    }
}

impl CodeCollectorApp {
    
    pub fn open_folder_dialog(&mut self) {
        if let Some(path) = rfd::FileDialog::new().pick_folder() {
            self.project_path = Some(path.clone());
            self.start_scanning_thread(path);
        }
    }

    fn start_scanning_thread(&mut self, path: PathBuf) {
        self.is_loading = true;
        self.loading_count = 0;
        self.root_node = None;
        self.recent_files.clear();
        self.search_query.clear();

        let (tx, rx) = channel();
        self.loading_channel = Some(rx);

        let cancel_flag = Arc::new(AtomicBool::new(false));
        self.cancel_flag = Some(cancel_flag.clone());

        thread::spawn(move || {
            let root_path = path.clone();
            if let Some(node) = read_dir_recursive_threaded(&path, &root_path, &tx, &cancel_flag) {
                if !cancel_flag.load(Ordering::Relaxed) {
                    let _ = tx.send(ScanMessage::Finished(node));
                } else {
                    let _ = tx.send(ScanMessage::Cancelled);
                }
            } else {
                let _ = tx.send(ScanMessage::Cancelled);
            }
        });
    }

    fn cancel_loading(&mut self) {
        if let Some(flag) = &self.cancel_flag {
            flag.store(true, Ordering::Relaxed);
        }
    }

    pub fn select_all(&mut self, state: bool) {
        if let Some(root) = &mut self.root_node {
            selection::set_dir_selection(root, state, &self.search_query);
            self.update_status();
        }
    }

    pub fn add_to_recents(&mut self, file: FileNode) {
        self.recent_files.retain(|f| f.path != file.path);
        self.recent_files.push_front(file);
        if self.recent_files.len() > 3 {
            self.recent_files.pop_back();
        }
    }

    pub fn handle_save(&mut self) {
        match self.export_mode {
            ExportMode::OneFile => self.save_single_file(),
            ExportMode::SeparateFiles => self.save_separate_files(),
        }
    }

    fn save_single_file(&mut self) {
        if let Some(root) = &self.root_node {
            let mut content = String::new();
            export::collect_content_string(root, &mut content);
            
            if content.is_empty() { 
                self.status_text = "No files selected!".to_string();
                return; 
            }

            if let Some(path) = rfd::FileDialog::new().set_file_name(export::DEFAULT_OUTPUT_FILENAME).save_file() {
                if let Err(e) = fs::write(&path, content) {
                    self.status_text = format!("Error saving: {}", e);
                } else {
                    self.status_text = "Saved successfully!".to_string();
                    let _ = open::that(&path);
                }
            }
        }
    }

    fn save_separate_files(&mut self) {
        if let Some(root) = &self.root_node {
            if let Some(target_dir) = rfd::FileDialog::new().pick_folder() {
                if let Err(e) = export::write_files_recursive(root, &target_dir) {
                    self.status_text = format!("Error: {}", e);
                } else {
                    self.status_text = "Files exported successfully!".to_string();
                    let _ = open::that(&target_dir);
                }
            }
        }
    }

    pub fn copy_to_clipboard(&mut self) {
        if let Some(root) = &self.root_node {
            let mut content = String::new();
            export::collect_content_string(root, &mut content);
            if !content.is_empty() {
                if let Ok(mut clipboard) = arboard::Clipboard::new() {
                    let _ = clipboard.set_text(content);
                    self.status_text = "Copied to clipboard!".to_string();
                }
            }
        }
    }

    fn update_status(&mut self) {
        if let Some(project) = &self.project_path {
             self.status_text = format!("Project: {}", project.file_name().unwrap_or_default().to_string_lossy());
        }
    }

    // --- Apply Theme ---
    fn configure_theme(&self, ctx: &egui::Context) {
        match self.theme {
            ThemePreference::Dark => {
                ctx.set_visuals(egui::Visuals::dark());
            }
            ThemePreference::Light => {
                // Customized "Apple-like" Light Theme
                let mut visuals = egui::Visuals::light();

                // 1. Main Background: Soft Mac-like Gray (not harsh white)
                visuals.panel_fill = egui::Color32::from_rgb(242, 242, 247); 
                visuals.window_fill = egui::Color32::from_rgb(255, 255, 255);

                // 2. Widgets (Buttons/Inputs): White to pop against the gray
                visuals.widgets.inactive.weak_bg_fill = egui::Color32::WHITE;
                visuals.widgets.inactive.bg_fill = egui::Color32::WHITE;
                
                // 3. Selection Color: Apple Blue
                visuals.selection.bg_fill = egui::Color32::from_rgb(0, 122, 255);
                visuals.selection.stroke = egui::Stroke::new(1.0, egui::Color32::from_rgb(0, 122, 255));

                // 4. Text: Soft Black (Startk black on white is hard on eyes)
                visuals.override_text_color = Some(egui::Color32::from_rgb(30, 30, 30));

                ctx.set_visuals(visuals);
            }
            ThemePreference::System => {
                // Reset to default
                ctx.set_visuals(egui::Visuals::default());
            }
        }
    }
}

impl eframe::App for CodeCollectorApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        let mut scan_completed = false; 

        if self.is_loading {
            if let Some(rx) = &self.loading_channel {
                while let Ok(msg) = rx.try_recv() {
                    match msg {
                        ScanMessage::Progress(count) => {
                            self.loading_count += count;
                            ctx.request_repaint(); 
                        }
                        ScanMessage::Finished(root) => {
                            self.root_node = Some(root);
                            self.is_loading = false;
                            scan_completed = true; 
                        }
                        ScanMessage::Cancelled => {
                            self.is_loading = false;
                            self.status_text = "Scanning cancelled.".to_string();
                            self.root_node = None; 
                        }
                    }
                }
            }
        }
        self.configure_theme(ctx); 

        if scan_completed {
            self.update_status();
        }

        // Panels
        panels::show_top_panel(ctx, self);
        panels::show_bottom_panel(ctx, self);
        panels::show_side_panel(ctx, self);

        // Central Panel
        egui::CentralPanel::default().show(ctx, |ui| {
            if self.is_loading {
                ui.vertical_centered(|ui| {
                   ui.add_space(50.0);
                });
            } else if self.root_node.is_some() {
                egui::ScrollArea::vertical().show(ui, |ui| {
                    if let Some(root) = &mut self.root_node {
                        if let Some(just_selected) = tree::render_tree_main(ui, root, true, &self.search_query) {
                            self.add_to_recents(just_selected);
                        }
                    }
                });
            } else {
                ui.centered_and_justified(|ui| {
                    ui.label("Open a project to begin.");
                });
            }
        });

        // Loading Modal
        if self.is_loading {
            egui::Window::new("Loading...")
                .collapsible(false)
                .resizable(false)
                .anchor(egui::Align2::CENTER_CENTER, [0.0, 0.0])
                .show(ctx, |ui| {
                    ui.set_min_width(250.0);
                    ui.vertical_centered(|ui| {
                        ui.add_space(10.0);
                        ui.spinner();
                        ui.add_space(10.0);
                        ui.label(egui::RichText::new("Scanning Directory...").strong().size(16.0));
                        ui.label(format!("Found {} files", self.loading_count));
                        ui.add_space(20.0);
                        if ui.add(egui::Button::new("Cancel").min_size([100.0, 30.0].into())).clicked() {
                            self.cancel_loading();
                        }
                        ui.add_space(10.0);
                    });
                });
        }
    }
}