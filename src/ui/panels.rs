use eframe::egui;
use crate::app::CodeCollectorApp;
use crate::models::export_mode::ExportMode;
use crate::models::theme::ThemePreference;
use crate::operations::export;
use super::styles::get_file_color;
use super::tree::render_selected_list;

pub fn show_top_panel(ctx: &egui::Context, app: &mut CodeCollectorApp) {
    egui::TopBottomPanel::top("top_panel").show(ctx, |ui| {
        ui.add_space(5.0);
        ui.horizontal(|ui| {
            ui.heading("📂 Code Collector");
            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                // Open Project
                if ui.add_enabled(!app.is_loading, egui::Button::new("Open Project")).clicked() {
                    app.open_folder_dialog();
                }
                ui.add_space(5.0);

                // --- Refresh Button ---
                // Only show if a project is loaded
                if app.project_path.is_some() {
                    if ui.add_enabled(!app.is_loading, egui::Button::new("🔄")).on_hover_text("Refresh Folder").clicked() {
                        app.refresh_project();
                    }
                }

                ui.add_space(10.0);
                ui.separator();

                // --- THEME SELECTOR ---
                egui::ComboBox::from_id_salt("theme_selector")
                    .selected_text(format!("{:?}", app.theme))
                    .show_ui(ui, |ui| {
                        ui.selectable_value(&mut app.theme, ThemePreference::System, "System");
                        ui.selectable_value(&mut app.theme, ThemePreference::Light, "Light");
                        ui.selectable_value(&mut app.theme, ThemePreference::Dark, "Dark");
                    });
                
                ui.label("Theme:");
            });
        });

        if app.root_node.is_some() && !app.is_loading {
            ui.add_space(5.0);
            ui.separator();
            ui.horizontal(|ui| {
                ui.label("🔍");
                ui.add(egui::TextEdit::singleline(&mut app.search_query).hint_text("Search files...").desired_width(f32::INFINITY));
            });
        }
        ui.add_space(5.0);
    });
}

pub fn show_bottom_panel(ctx: &egui::Context, app: &mut CodeCollectorApp) {
    egui::TopBottomPanel::bottom("bottom_panel").show(ctx, |ui| {
        ui.add_space(5.0);
        ui.horizontal(|ui| {
            if ui.button("Select All").clicked() { app.select_all(true); }
            if ui.button("None").clicked() { app.select_all(false); }
            ui.separator();
            ui.label(&app.status_text);
        });
        ui.separator();
        ui.horizontal(|ui| {
            ui.label("Export Mode:");
            ui.radio_value(&mut app.export_mode, ExportMode::OneFile, "Single File (.txt)");
            ui.radio_value(&mut app.export_mode, ExportMode::SeparateFiles, "Separate Files");
        });
        ui.add_space(5.0);
        ui.horizontal_centered(|ui| {
            let w = (ui.available_width() / 2.0) - 5.0;
            let copy_enabled = app.export_mode == ExportMode::OneFile && !app.is_loading;
            if ui.add_enabled_ui(copy_enabled, |ui| {
                ui.add_sized([w, 40.0], egui::Button::new("📋 Copy to Clipboard"))
            }).inner.clicked() { app.copy_to_clipboard(); }

            if ui.add_sized([w, 40.0], egui::Button::new("💾 Save Selected")).clicked() { app.handle_save(); }
        });
        ui.add_space(5.0);
    });
}

pub fn show_side_panel(ctx: &egui::Context, app: &mut CodeCollectorApp) {
    egui::SidePanel::right("right_panel")
        .resizable(true)
        .default_width(250.0)
        .show(ctx, |ui| {
            ui.add_space(10.0);
            ui.heading("Tools");
            ui.separator();

            egui::CollapsingHeader::new("🕒 Recent Files").default_open(true).show(ui, |ui| {
                if app.recent_files.is_empty() {
                    ui.label(egui::RichText::new("No recent selections").italics().weak());
                } else {
                    for file in &app.recent_files {
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
                    if let Some(root) = &mut app.root_node {
                        render_selected_list(ui, root);
                    }
                });
            });

            ui.separator();
            egui::CollapsingHeader::new("📊 Selection Stats").default_open(true).show(ui, |ui| {
                if let Some(root) = &app.root_node {
                    let (bytes, count) = export::calculate_stats(root);
                    let kb = bytes as f64 / 1024.0;
                    let est_lines = bytes / 30; 
                    ui.label(format!("Files: {}", count));
                    ui.label(format!("Size: {:.2} KB", kb));
                    ui.label(format!("Est. Lines: ~{}", est_lines));
                }
            });
        });
}