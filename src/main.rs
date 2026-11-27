use eframe::egui;

fn main() {
    let args: Vec<String> = std::env::args().collect();

    let ocpack_url = args[1].clone();
    let minecraft_instance_path = args[2].clone();

    if minecraft_instance_path == "" {
        println!("No minecraft instance path provided, exiting");
        std::process::exit(1);
    }

    if ocpack_url == "" {
        println!("No ocelot pack url provided, exiting");
        std::process::exit(1);
    }

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

    let ocpack_text = match reqwest::blocking::get(&ocpack_url)
        .and_then(|response| response.error_for_status())
        .and_then(|response| response.text())
    {
        Ok(text) => text,
        Err(err) => {
            println!("Failed to download ocpack from {ocpack_url}: {err}");
            std::process::exit(1);
        }
    };

    println!("{:?}", parse_ocpack(&ocpack_text));
}

struct OcelotApp {
    main_text: String,
    download_progress: f32,
    mod_name: String,
    mod_progress: f32,
}

#[derive(Debug, Clone)]
struct ModpackEntry {
    slug: String,
    version: String,
    loader: String,
    required: bool,
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
                ui.label(
                    egui::RichText::new(format!(
                        "{} ({}%)...",
                        self.mod_name,
                        (self.mod_progress * 100.0) as i32
                    ))
                    .strong(),
                );
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

fn parse_ocpack(text: &str) -> Result<Vec<ModpackEntry>, String> {
    text.lines()
        .enumerate()
        .skip(1) // the ocpacks header (cause its a csv)
        .filter(|(_, line)| !line.trim().is_empty())
        .map(|(i, line)| {
            let cols: Vec<&str> = line.split(',').map(str::trim).collect();
            let [url, version, platform, required] = cols[..] else {
                return Err(format!(
                    "line {}: incorrect number of cols: {}",
                    i + 1,
                    cols.len()
                ));
            };
            let slug = url
                .trim_end_matches('/')
                .rsplit('/')
                .next()
                .filter(|s| !s.is_empty())
                .ok_or(format!("line {}: not a mod url", i + 1))?;
            Ok(ModpackEntry {
                slug: slug.to_owned(),
                version: version.to_owned(),
                loader: platform.to_owned(),
                required: required.eq("required"),
            })
        })
        .collect()
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
