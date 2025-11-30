use eframe::egui;
use std::fs;
use std::path::{Path, PathBuf};

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
}

#[derive(Clone)]
struct DirNode {
    name: String,
    path: PathBuf,
    children_dirs: Vec<DirNode>,
    children_files: Vec<FileNode>,
}

struct CodeCollectorApp {
    project_path: Option<PathBuf>,
    root_node: Option<DirNode>,
    status_text: String,
    export_mode: ExportMode,
    search_query: String, // <--- NEW: Search state
}

impl Default for CodeCollectorApp {
    fn default() -> Self {
        Self {
            project_path: None,
            root_node: None,
            status_text: String::from("Ready to scan."),
            export_mode: ExportMode::OneFile,
            search_query: String::new(),
        }
    }
}

// --- CORE LOGIC ---

impl CodeCollectorApp {
    
    fn open_folder_dialog(&mut self) {
        if let Some(path) = rfd::FileDialog::new().pick_folder() {
            self.project_path = Some(path.clone());
            self.status_text = String::from("Scanning directory...");
            self.search_query.clear(); // Clear search on new open
            let root = self.read_dir_recursive(&path, &path);
            self.root_node = Some(root);
            self.update_status();
        }
    }

    fn read_dir_recursive(&self, dir_path: &Path, root_path: &Path) -> DirNode {
        let mut node = DirNode {
            name: dir_path.file_name().unwrap_or_default().to_string_lossy().to_string(),
            path: dir_path.to_path_buf(),
            children_dirs: Vec::new(),
            children_files: Vec::new(),
        };

        if let Ok(entries) = fs::read_dir(dir_path) {
            for entry in entries.flatten() {
                let path = entry.path();
                let name = entry.file_name().to_string_lossy().to_string();

                if name.starts_with('.') && name != ".env" { continue; }
                if IGNORE_DIRS.contains(&name.as_str()) { continue; }

                if path.is_dir() {
                    node.children_dirs.push(self.read_dir_recursive(&path, root_path));
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

                    node.children_files.push(FileNode {
                        name,
                        path,
                        rel_path,
                        extension,
                        selected: false,
                    });
                }
            }
        }

        node.children_dirs.sort_by(|a, b| a.name.cmp(&b.name));
        node.children_files.sort_by(|a, b| a.name.cmp(&b.name));
        node
    }

    // --- SELECTION LOGIC ---

    fn set_dir_selection(dir: &mut DirNode, state: bool, query: &str) {
        // If searching, only select visible items
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

    // Returns true if this folder OR any of its children match the query
    fn matches_search(dir: &DirNode, query: &str) -> bool {
        if query.is_empty() { return true; }
        let q = query.to_lowercase();
        
        // 1. Does folder name match?
        // if dir.name.to_lowercase().contains(&q) { return true; } // Optional: Enable if you want empty folders to show if they match name

        // 2. Do any files match?
        for file in &dir.children_files {
            if file.name.to_lowercase().contains(&q) { return true; }
        }

        // 3. Do any subfolders match?
        for sub in &dir.children_dirs {
            if Self::matches_search(sub, query) { return true; }
        }
        
        false
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
                    
                    // --- NEW: Open the file immediately ---
                    if let Err(e) = open::that(&path) {
                        eprintln!("Could not open file: {}", e);
                        self.status_text = format!("Saved, but couldn't open: {}", e);
                    }
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
                    let _ = open::that(&target_dir); // Open the folder after export
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

    fn count_selected_recursive(dir: &DirNode) -> (usize, usize) {
        let mut selected = 0;
        let mut total = dir.children_files.len();
        for file in &dir.children_files {
            if file.selected { selected += 1; }
        }
        for sub_dir in &dir.children_dirs {
            let (s, t) = Self::count_selected_recursive(sub_dir);
            selected += s;
            total += t;
        }
        (selected, total)
    }

    fn update_status(&mut self) {
        if let Some(root) = &self.root_node {
            let (selected, total) = Self::count_selected_recursive(root);
            let project_name = self.project_path.as_ref()
                .and_then(|p| p.file_name())
                .map(|n| n.to_string_lossy().to_string())
                .unwrap_or_default();
            
            self.status_text = format!("Project: {} | Selected: {} / {}", project_name, selected, total);
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

// --- RECURSIVE TREE RENDER ---

fn render_tree(ui: &mut egui::Ui, dir: &mut DirNode, is_root: bool, on_change: &mut bool, query: &str) {
    let render_content = |ui: &mut egui::Ui, dir: &mut DirNode, changed: &mut bool| {
        // 1. Render Subdirectories
        for sub_dir in &mut dir.children_dirs {
            // Filter: Only show sub-directories that contain matches
            if CodeCollectorApp::matches_search(sub_dir, query) {
                render_tree(ui, sub_dir, false, changed, query);
            }
        }
        
        // 2. Render Files
        for file in &mut dir.children_files {
            // Filter: Search check
            if !query.is_empty() && !file.name.to_lowercase().contains(&query.to_lowercase()) {
                continue;
            }

            ui.horizontal(|ui| {
                ui.add_space(24.0);
                if ui.checkbox(&mut file.selected, "").changed() { *changed = true; }
                
                let color = get_file_color(&file.extension, ui);
                // Standard file hover effect
                if ui.selectable_label(false, egui::RichText::new(&file.name).color(color)).clicked() {
                    file.selected = !file.selected;
                    *changed = true;
                }
            });
        }
    };

    if is_root {
        render_content(ui, dir, on_change);
    } else {
        ui.horizontal(|ui| {
            let mut is_checked = CodeCollectorApp::is_dir_fully_selected(dir);
            if ui.checkbox(&mut is_checked, "").changed() {
                CodeCollectorApp::set_dir_selection(dir, is_checked, query);
                *on_change = true;
            }

            // --- IMPROVED FOLDER HOVER UI ---
            // We use CollapsingHeader, but if there is a search query, we FORCE it open.
            let mut header = egui::CollapsingHeader::new(
                egui::RichText::new(&dir.name).strong()
            )
            .id_salt(&dir.path);

            // If searching, force open so user sees results
            if !query.is_empty() {
                header = header.default_open(true);
            }

            header.icon(|ui, open, _| {
                    ui.label(egui::RichText::new(if open > 0.0 { "📂" } else { "📁" }).color(egui::Color32::GOLD));
                })
                .show(ui, |ui| render_content(ui, dir, on_change));
        });
    }
}

// --- APP ENTRY ---

impl eframe::App for CodeCollectorApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        
        // --- TOP PANEL ---
        egui::TopBottomPanel::top("top_panel").show(ctx, |ui| {
            ui.add_space(5.0);
            ui.horizontal(|ui| {
                ui.heading("📂 Code Collector");
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                     if ui.button("Open Project").clicked() {
                        self.open_folder_dialog();
                     }
                });
            });

            // --- SEARCH BAR ---
            if self.root_node.is_some() {
                ui.add_space(5.0);
                ui.separator();
                ui.horizontal(|ui| {
                    ui.label("🔍");
                    ui.add(egui::TextEdit::singleline(&mut self.search_query)
                        .hint_text("Search files...")
                        .desired_width(f32::INFINITY)); // Full width
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
                
                let copy_enabled = self.export_mode == ExportMode::OneFile;
                if ui.add_enabled_ui(copy_enabled, |ui| {
                    ui.add_sized([w, 40.0], egui::Button::new("📋 Copy to Clipboard"))
                }).inner.clicked() {
                    self.copy_to_clipboard();
                }

                if ui.add_sized([w, 40.0], egui::Button::new("💾 Save Selected")).clicked() {
                    self.handle_save();
                }
            });
            ui.add_space(5.0);
        });

        // --- CENTRAL PANEL ---
        egui::CentralPanel::default().show(ctx, |ui| {
            if self.root_node.is_some() {
                egui::ScrollArea::vertical()
                    .auto_shrink([false, false])
                    .show(ui, |ui| {
                        let mut changed = false;
                        if let Some(root) = &mut self.root_node {
                            // Pass the search query down
                            render_tree(ui, root, true, &mut changed, &self.search_query);
                        }
                        if changed { self.update_status(); }
                    });
            } else {
                ui.centered_and_justified(|ui| {
                    ui.label("Open a project to begin.");
                });
            }
        });
    }
}

fn main() -> Result<(), eframe::Error> {
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default().with_inner_size([700.0, 800.0]),
        ..Default::default()
    };
    eframe::run_native("Code Collector", options, Box::new(|_cc| Ok(Box::new(CodeCollectorApp::default()))))
}