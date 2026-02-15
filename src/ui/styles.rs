use eframe::egui;

pub fn get_file_color(extension: &str, ui: &egui::Ui) -> egui::Color32 {
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