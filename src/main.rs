use eframe::egui;

fn main() {
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([420.0, 95.0])
            .with_min_inner_size([420.0, 95.0])
            .with_max_inner_size([420.0, 95.0])
            .with_resizable(false)
            .with_maximize_button(false),
        ..Default::default()
    };
    let _ = eframe::run_native(
        &format!("Ocelot v{}", env!("CARGO_PKG_VERSION")),
        options,
        Box::new(|_creation_context| Ok(Box::<OcelotApp>::default())),
    );
}

struct OcelotApp {
    main_text: String,
    download_progress: f32,
    mod_name: String,
    mod_progress: f32,
}

impl Default for OcelotApp {
    fn default() -> Self {
        Self {
            main_text: "Preparing Minecraft".to_owned(),
            download_progress: 0.65,
            mod_name: "This Mod".to_owned(),
            mod_progress: 0.65,
        }
    }
}

impl eframe::App for OcelotApp {
    fn ui(&mut self, ui: &mut egui::Ui, _frame: &mut eframe::Frame) {
        egui::CentralPanel::default().show(ui, |ui| {
            wavy_text(ui, &self.main_text, 18.0);
            ui.add_space(5.0);
            ui.add(
                egui::ProgressBar::new(self.download_progress)
                    .show_percentage()
                    .animate(true),
            );
            ui.add_space(5.0);
            ui.horizontal(|ui| {
                ui.label(egui::RichText::new(format!("{} ({}%)...", self.mod_name, (self.mod_progress * 100.0) as i32)).strong());
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    if ui.button("Cancel").clicked() {
                        // cancel it
                        // for now just exit
                        std::process::exit(1);
                    }
                    if ui.button("Optional mods").clicked() {
                        // cancel it
                        // for now just exit
                        std::process::exit(1);
                    }
                });
            });
        });
    }
}

pub fn parse_ocpack(ocpack: &str) -> Vec<String> {
    let mut mods = Vec::new();
    for line in ocpack.lines() {
        let line = line.trim();
        if line.is_empty() || line.starts_with('#') {
            continue;
        }
        // something i forgot
        mods.push(line.to_string());
    }
    mods
}

fn wavy_text(ui: &mut egui::Ui, text: &str, size: f32) {
    let font = egui::FontId::proportional(size);
    let time = ui.input(|i| i.time);
    let painter = ui.painter();
    let amplitude: f32 = 2.0;
    let speed: f64 = 7.0;
    let phase_per_char: f32 = 0.5;
    let hue_speed: f64 = 0.3;
    let hue_per_char: f32 = 0.06;
    let glyphs: Vec<_> = text
        .chars()
        .enumerate()
        .map(|(i, c)| {
            let hue = ((time * hue_speed) as f32 + i as f32 * hue_per_char).rem_euclid(1.0);
            let color: egui::Color32 = egui::ecolor::Hsva::new(hue, 0.8, 1.0, 1.0).into();
            painter.layout_no_wrap(c.to_string(), font.clone(), color)
        })
        .collect();
    let width: f32 = glyphs.iter().map(|g| g.size().x).sum();
    let height = glyphs.iter().map(|g| g.size().y).fold(0.0, f32::max);
    let (rect, _) = ui.allocate_exact_size(
        egui::vec2(width, height + amplitude * 2.0),
        egui::Sense::hover(),
    );
    let t = time * speed;
    let mut x = rect.left();
    for (i, galley) in glyphs.into_iter().enumerate() {
        let dy = (t as f32 + i as f32 * phase_per_char).sin() * amplitude;
        let pos = egui::pos2(x, rect.top() + amplitude + dy);
        x += galley.size().x;
        ui.painter().galley(pos, galley, egui::Color32::PLACEHOLDER);
    }
    ui.ctx().request_repaint();
}
