#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use eframe::egui;
use std::collections::VecDeque;
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::{Arc, atomic::{AtomicBool, Ordering}};
use std::thread;
use std::sync::mpsc::{Receiver, Sender, channel};

// --- CONFIGURATION ---
const DEFAULT_OUTPUT_FILENAME: &str = "full_code.txt";

const IGNORE_DIRS: &[&str] = &[
    ".git", ".vscode", "node_modules", "vendor", "__pycache__", 
    ".idea", "target", "dist", "build", "coverage", ".next", ".nuxt", "storage"
];

// --- DATA STRUCTURES ---

#[derive(Clone, PartialEq)]
enum ExportMode {
    OneFile,
    SeparateFiles,
}

#[derive(Clone)]
struct FileNode {
    name: String,
    path: PathBuf,
    rel_path: String,
    extension: String,
    selected: bool,
    size_bytes: u64,
}

#[derive(Clone)]
struct DirNode {
    name: String,
    path: PathBuf,
    children_dirs: Vec<DirNode>,
    children_files: Vec<FileNode>,
}

// Messages sent from the Background Thread to the GUI
enum ScanMessage {
    Progress(usize),       // "I found X files so far"
    Finished(DirNode),     // "Here is the completed tree"
    Cancelled,             // "User stopped me"
}

struct CodeCollectorApp {
    project_path: Option<PathBuf>,
    root_node: Option<DirNode>,
    status_text: String,
    export_mode: ExportMode,
    search_query: String,
    recent_files: VecDeque<FileNode>,

    // --- LOADING STATE ---
    is_loading: bool,
    loading_count: usize, // How many files found so far (for progress)
    loading_channel: Option<Receiver<ScanMessage>>, // Receiver for thread messages
    cancel_flag: Option<Arc<AtomicBool>>, // The "Stop" switch
}

impl Default for CodeCollectorApp {
    fn default() -> Self {
        Self {
            project_path: None,
            root_node: None,
            status_text: String::from("Ready to scan."),
            export_mode: ExportMode::OneFile,
            search_query: String::new(),
            recent_files: VecDeque::with_capacity(3),
            is_loading: false,
            loading_count: 0,
            loading_channel: None,
            cancel_flag: None,
        }
    }
}

// --- CORE LOGIC ---

impl CodeCollectorApp {
    
    fn open_folder_dialog(&mut self) {
        if let Some(path) = rfd::FileDialog::new().pick_folder() {
            self.project_path = Some(path.clone());
            self.start_scanning_thread(path);
        }
    }

    // New: Spawns a thread to scan without freezing UI
    fn start_scanning_thread(&mut self, path: PathBuf) {
        self.is_loading = true;
        self.loading_count = 0;
        self.root_node = None;
        self.recent_files.clear();
        self.search_query.clear();

        let (tx, rx) = channel();
        self.loading_channel = Some(rx);

        // specific flag to control cancellation
        let cancel_flag = Arc::new(AtomicBool::new(false));
        self.cancel_flag = Some(cancel_flag.clone());

        // Spawn the worker thread
        thread::spawn(move || {
            let root_path = path.clone();
            
            // We use a helper function that we can pass the tx and cancel_flag to
            if let Some(node) = Self::read_dir_recursive_threaded(&path, &root_path, &tx, &cancel_flag) {
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

    // Static helper for the thread (doesn't use &self)
    fn read_dir_recursive_threaded(
        dir_path: &Path, 
        root_path: &Path, 
        tx: &Sender<ScanMessage>, 
        cancel_flag: &Arc<AtomicBool>
    ) -> Option<DirNode> {
        
        // 1. Check Cancellation
        if cancel_flag.load(Ordering::Relaxed) {
            return None;
        }

        let mut node = DirNode {
            name: dir_path.file_name().unwrap_or_default().to_string_lossy().to_string(),
            path: dir_path.to_path_buf(),
            children_dirs: Vec::new(),
            children_files: Vec::new(),
        };

        if let Ok(entries) = fs::read_dir(dir_path) {
            for entry in entries.flatten() {
                // Check cancellation inside the loop for faster response
                if cancel_flag.load(Ordering::Relaxed) { return None; }

                let path = entry.path();
                let name = entry.file_name().to_string_lossy().to_string();

                if name.starts_with('.') && name != ".env" { continue; }
                if IGNORE_DIRS.contains(&name.as_str()) { continue; }

                if path.is_dir() {
                    // Recurse
                    if let Some(child) = Self::read_dir_recursive_threaded(&path, root_path, tx, cancel_flag) {
                        node.children_dirs.push(child);
                    }
                } else {
                    let name_lower = name.to_lowercase();
                    let extension = if name_lower.ends_with(".blade.php") {
                        "blade.php".to_string()
                    } else {
                        path.extension()
                            .map(|e| e.to_string_lossy().to_string().to_lowercase())
                            .unwrap_or_default()
                    };
                    
                    let rel_path = path.strip_prefix(root_path)
                        .unwrap_or(&path)
                        .to_string_lossy()
                        .to_string();

                    if name == DEFAULT_OUTPUT_FILENAME { continue; }

                    let size_bytes = fs::metadata(&path).map(|m| m.len()).unwrap_or(0);

                    node.children_files.push(FileNode {
                        name,
                        path,
                        rel_path,
                        extension,
                        selected: false,
                        size_bytes,
                    });

                    // NOTIFY UI: Found a file
                    // We send 1 to increment count. 
                    // (Sending the full total every time is also fine, but sending 1 is simple)
                    let _ = tx.send(ScanMessage::Progress(1));
                }
            }
        }

        // Sort for clean display
        node.children_dirs.sort_by(|a, b| a.name.cmp(&b.name));
        node.children_files.sort_by(|a, b| a.name.cmp(&b.name));
        Some(node)
    }

    // --- SELECTION LOGIC ---

    fn set_dir_selection(dir: &mut DirNode, state: bool, query: &str) {
        let is_visible = |name: &str| query.is_empty() || name.to_lowercase().contains(&query.to_lowercase());

        for file in &mut dir.children_files {
            if is_visible(&file.name) {
                file.selected = state;
            }
        }
        for sub in &mut dir.children_dirs {
            Self::set_dir_selection(sub, state, query);
        }
    }

    fn select_all(&mut self, state: bool) {
        if let Some(root) = &mut self.root_node {
            Self::set_dir_selection(root, state, &self.search_query);
            self.update_status();
        }
    }

    fn is_dir_fully_selected(dir: &DirNode) -> bool {
        if dir.children_files.is_empty() && dir.children_dirs.is_empty() { return false; }
        for file in &dir.children_files {
            if !file.selected { return false; }
        }
        for sub in &dir.children_dirs {
            if !Self::is_dir_fully_selected(sub) { return false; }
        }
        true
    }

    // --- SEARCH HELPERS ---

    fn matches_search(dir: &DirNode, query: &str) -> bool {
        if query.is_empty() { return true; }
        let q = query.to_lowercase();
        
        for file in &dir.children_files {
            if file.name.to_lowercase().contains(&q) { return true; }
        }

        for sub in &dir.children_dirs {
            if Self::matches_search(sub, query) { return true; }
        }
        
        false
    }

    // --- RECENT FILES LOGIC ---
    fn add_to_recents(&mut self, file: FileNode) {
        self.recent_files.retain(|f| f.path != file.path);
        self.recent_files.push_front(file);
        if self.recent_files.len() > 3 {
            self.recent_files.pop_back();
        }
    }

    // --- STATS LOGIC ---
    fn calculate_stats(dir: &DirNode) -> (u64, usize) {
        let mut size = 0;
        let mut files = 0;
        for file in &dir.children_files {
            if file.selected {
                size += file.size_bytes;
                files += 1;
            }
        }
        for sub in &dir.children_dirs {
            let (s, f) = Self::calculate_stats(sub);
            size += s;
            files += f;
        }
        (size, files)
    }

    // --- EXPORT LOGIC ---

    fn handle_save(&mut self) {
        match self.export_mode {
            ExportMode::OneFile => self.save_single_file(),
            ExportMode::SeparateFiles => self.save_separate_files(),
        }
    }

    fn save_single_file(&mut self) {
        if let Some(root) = &self.root_node {
            let mut content = String::new();
            Self::collect_content_string(root, &mut content);
            
            if content.is_empty() { 
                self.status_text = "No files selected!".to_string();
                return; 
            }

            if let Some(path) = rfd::FileDialog::new().set_file_name(DEFAULT_OUTPUT_FILENAME).save_file() {
                if let Err(e) = fs::write(&path, content) {
                    self.status_text = format!("Error saving: {}", e);
                } else {
                    self.status_text = "Saved successfully!".to_string();
                    let _ = open::that(&path);
                }
            }
        }
    }

    fn collect_content_string(dir: &DirNode, content: &mut String) {
        for file in &dir.children_files {
            if file.selected {
                if let Ok(code) = fs::read_to_string(&file.path) {
                     content.push_str(&format!(
                        "\n\n{}\nFILE: {}\nLANGUAGE: {}\n{}\n\n",
                        "=".repeat(50), file.rel_path, file.extension, "=".repeat(50)
                    ));
                    content.push_str(&code);
                }
            }
        }
        for sub in &dir.children_dirs {
            Self::collect_content_string(sub, content);
        }
    }

    fn save_separate_files(&mut self) {
        if let Some(root) = &self.root_node {
            if let Some(target_dir) = rfd::FileDialog::new().pick_folder() {
                if let Err(e) = Self::write_files_recursive(root, &target_dir) {
                    self.status_text = format!("Error: {}", e);
                } else {
                    self.status_text = "Files exported successfully!".to_string();
                    let _ = open::that(&target_dir);
                }
            }
        }
    }

    fn write_files_recursive(dir: &DirNode, base_target_path: &Path) -> std::io::Result<()> {
        for file in &dir.children_files {
            if file.selected {
                let target_file_path = base_target_path.join(&file.rel_path);
                if let Some(parent) = target_file_path.parent() {
                    fs::create_dir_all(parent)?;
                }
                fs::copy(&file.path, target_file_path)?;
            }
        }
        for sub_dir in &dir.children_dirs {
            Self::write_files_recursive(sub_dir, base_target_path)?;
        }
        Ok(())
    }

    fn copy_to_clipboard(&mut self) {
        if let Some(root) = &self.root_node {
            let mut content = String::new();
            Self::collect_content_string(root, &mut content);
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
}

// --- UI HELPERS ---

fn get_file_color(extension: &str, ui: &egui::Ui) -> egui::Color32 {
    let is_dark = ui.visuals().dark_mode;
    match extension {
        "html" | "htm" => egui::Color32::from_rgb(227, 76, 38), 
        "css" | "scss" | "sass" => egui::Color32::from_rgb(86, 156, 214), 
        "js" | "mjs" => if is_dark { egui::Color32::from_rgb(241, 224, 90) } else { egui::Color32::from_rgb(210, 180, 0) }, 
        "ts" | "tsx" => egui::Color32::from_rgb(49, 120, 198), 
        "jsx" => egui::Color32::from_rgb(97, 218, 251), 
        "rs" | "rust" => egui::Color32::from_rgb(222, 165, 132), 
        "json" | "toml" | "xml" => egui::Color32::from_rgb(207, 145, 120), 
        "py" => egui::Color32::from_rgb(53, 114, 165),
        "md" => egui::Color32::from_rgb(150, 150, 150),
        "c" | "cpp" | "h" => egui::Color32::from_rgb(89, 108, 209),
        "java" => egui::Color32::from_rgb(176, 114, 25),
        "php" => egui::Color32::from_rgb(119, 123, 179),
        "blade.php" => egui::Color32::from_rgb(240, 83, 64),
        _ => if is_dark { egui::Color32::LIGHT_GRAY } else { egui::Color32::DARK_GRAY },
    }
}

// --- RECURSIVE RENDERERS ---

fn render_tree_main(ui: &mut egui::Ui, dir: &mut DirNode, is_root: bool, query: &str) -> Option<FileNode> {
    let mut recent_update: Option<FileNode> = None;

    let render_content = |ui: &mut egui::Ui, dir: &mut DirNode, recent_update: &mut Option<FileNode>| {
        // Subdirectories
        for sub_dir in &mut dir.children_dirs {
            if CodeCollectorApp::matches_search(sub_dir, query) {
                if let Some(node) = render_tree_main(ui, sub_dir, false, query) {
                    *recent_update = Some(node);
                }
            }
        }
        
        // Files
        for file in &mut dir.children_files {
            if !query.is_empty() && !file.name.to_lowercase().contains(&query.to_lowercase()) { continue; }

            ui.horizontal(|ui| {
                ui.add_space(24.0);
                if ui.checkbox(&mut file.selected, "").changed() {
                     if file.selected {
                        *recent_update = Some(file.clone());
                     }
                }
                
                let color = get_file_color(&file.extension, ui);
                if ui.selectable_label(false, egui::RichText::new(&file.name).color(color)).clicked() {
                    file.selected = !file.selected;
                    if file.selected {
                        *recent_update = Some(file.clone());
                    }
                }
            });
        }
    };

    if is_root {
        render_content(ui, dir, &mut recent_update);
    } else {
        ui.horizontal(|ui| {
            let mut is_checked = CodeCollectorApp::is_dir_fully_selected(dir);
            if ui.checkbox(&mut is_checked, "").changed() {
                CodeCollectorApp::set_dir_selection(dir, is_checked, query);
            }

            let mut header = egui::CollapsingHeader::new(egui::RichText::new(&dir.name).strong()).id_salt(&dir.path);
            if !query.is_empty() { header = header.default_open(true); }

            header.icon(|ui, open, _| {
                ui.label(egui::RichText::new(if open > 0.0 { "📂" } else { "📁" }).color(egui::Color32::GOLD));
            })
            .show(ui, |ui| render_content(ui, dir, &mut recent_update));
        });
    }

    recent_update
}

fn render_selected_list(ui: &mut egui::Ui, dir: &mut DirNode) {
    for file in &mut dir.children_files {
        if file.selected {
            ui.horizontal(|ui| {
                if ui.button("❌").on_hover_text("Deselect").clicked() {
                    file.selected = false;
                }
                let color = get_file_color(&file.extension, ui);
                ui.label(egui::RichText::new(&file.name).color(color));
            });
        }
    }
    for sub in &mut dir.children_dirs {
        render_selected_list(ui, sub);
    }
}

// --- APP ENTRY ---

impl eframe::App for CodeCollectorApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        
        // --- HANDLE THREAD MESSAGES ---
        // 1. We use a flag to defer the update call until after we stop reading the channel
        let mut scan_completed = false; 

        if self.is_loading {
            if let Some(rx) = &self.loading_channel {
                // Read all available messages without blocking
                while let Ok(msg) = rx.try_recv() {
                    match msg {
                        ScanMessage::Progress(count) => {
                            self.loading_count += count;
                            // Request repaint to animate spinner
                            ctx.request_repaint(); 
                        }
                        ScanMessage::Finished(root) => {
                            self.root_node = Some(root);
                            self.is_loading = false;
                            scan_completed = true; // Mark it as done, but don't call method yet
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

        // 2. Now that the `if let Some(rx)` block is closed, `self` is free to be mutated again.
        if scan_completed {
            self.update_status();
        }

        // --- TOP PANEL ---
        egui::TopBottomPanel::top("top_panel").show(ctx, |ui| {
            ui.add_space(5.0);
            ui.horizontal(|ui| {
                ui.heading("📂 Code Collector");
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    // Disable button if loading
                    if ui.add_enabled(!self.is_loading, egui::Button::new("Open Project")).clicked() {
                        self.open_folder_dialog();
                    }
                });
            });

            // Hide search bar if loading
            if self.root_node.is_some() && !self.is_loading {
                ui.add_space(5.0);
                ui.separator();
                ui.horizontal(|ui| {
                    ui.label("🔍");
                    ui.add(egui::TextEdit::singleline(&mut self.search_query).hint_text("Search files...").desired_width(f32::INFINITY));
                });
            }
            ui.add_space(5.0);
        });

        // --- BOTTOM PANEL ---
        egui::TopBottomPanel::bottom("bottom_panel").show(ctx, |ui| {
            ui.add_space(5.0);
            ui.horizontal(|ui| {
                if ui.button("Select All").clicked() { self.select_all(true); }
                if ui.button("None").clicked() { self.select_all(false); }
                ui.separator();
                ui.label(&self.status_text);
            });
            ui.separator();
            ui.horizontal(|ui| {
                ui.label("Export Mode:");
                ui.radio_value(&mut self.export_mode, ExportMode::OneFile, "Single File (.txt)");
                ui.radio_value(&mut self.export_mode, ExportMode::SeparateFiles, "Separate Files");
            });
            ui.add_space(5.0);
            ui.horizontal_centered(|ui| {
                let w = (ui.available_width() / 2.0) - 5.0;
                let copy_enabled = self.export_mode == ExportMode::OneFile && !self.is_loading;
                if ui.add_enabled_ui(copy_enabled, |ui| {
                    ui.add_sized([w, 40.0], egui::Button::new("📋 Copy to Clipboard"))
                }).inner.clicked() { self.copy_to_clipboard(); }

                if ui.add_sized([w, 40.0], egui::Button::new("💾 Save Selected")).clicked() { self.handle_save(); }
            });
            ui.add_space(5.0);
        });

        // --- RIGHT SIDE PANEL ---
        egui::SidePanel::right("right_panel")
            .resizable(true)
            .default_width(250.0)
            .show(ctx, |ui| {
                ui.add_space(10.0);
                ui.heading("Tools");
                ui.separator();

                egui::CollapsingHeader::new("🕒 Recent Files").default_open(true).show(ui, |ui| {
                    if self.recent_files.is_empty() {
                        ui.label(egui::RichText::new("No recent selections").italics().weak());
                    } else {
                        for file in &self.recent_files {
                            ui.horizontal(|ui| {
                                let color = get_file_color(&file.extension, ui);
                                ui.label(egui::RichText::new("•").color(color));
                                ui.label(egui::RichText::new(&file.name).color(color));
                            });
                        }
                    }
                });

                ui.separator();
                egui::CollapsingHeader::new("✅ Selected Files").default_open(true).show(ui, |ui| {
                    egui::ScrollArea::vertical().max_height(300.0).show(ui, |ui| {
                        if let Some(root) = &mut self.root_node {
                            render_selected_list(ui, root);
                        }
                    });
                });

                ui.separator();
                egui::CollapsingHeader::new("📊 Selection Stats").default_open(true).show(ui, |ui| {
                    if let Some(root) = &self.root_node {
                        let (bytes, count) = CodeCollectorApp::calculate_stats(root);
                        let kb = bytes as f64 / 1024.0;
                        let est_lines = bytes / 30; 
                        ui.label(format!("Files: {}", count));
                        ui.label(format!("Size: {:.2} KB", kb));
                        ui.label(format!("Est. Lines: ~{}", est_lines));
                    }
                });
            });

        // --- CENTRAL PANEL (File Tree) ---
        egui::CentralPanel::default().show(ctx, |ui| {
            if self.is_loading {
                // If loading, show empty bg, the popup handles the visual
                ui.vertical_centered(|ui| {
                   ui.add_space(50.0);
                });
            } else if self.root_node.is_some() {
                egui::ScrollArea::vertical().show(ui, |ui| {
                    if let Some(root) = &mut self.root_node {
                        if let Some(just_selected) = render_tree_main(ui, root, true, &self.search_query) {
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

        // --- LOADING POPUP MODAL ---
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

fn main() -> Result<(), eframe::Error> {
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default().with_inner_size([900.0, 800.0]),
        ..Default::default()
    };
    eframe::run_native("Code Collector", options, Box::new(|_cc| Ok(Box::new(CodeCollectorApp::default()))))
}