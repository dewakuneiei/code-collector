use eframe::egui;
use crate::models::{dir_node::DirNode, file_node::FileNode};
use crate::operations::{selection, search};
use super::styles::get_file_color;

pub fn render_tree_main(ui: &mut egui::Ui, dir: &mut DirNode, is_root: bool, query: &str) -> Option<FileNode> {
    let mut recent_update: Option<FileNode> = None;

    let render_content = |ui: &mut egui::Ui, dir: &mut DirNode, recent_update: &mut Option<FileNode>| {
        // Subdirectories
        for sub_dir in &mut dir.children_dirs {
            if search::matches_search(sub_dir, query) {
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
            let mut is_checked = selection::is_dir_fully_selected(dir);
            if ui.checkbox(&mut is_checked, "").changed() {
                selection::set_dir_selection(dir, is_checked, query);
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

pub fn render_selected_list(ui: &mut egui::Ui, dir: &mut DirNode) {
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