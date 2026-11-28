use eframe::egui;
use ferinth::Ferinth;
use std::sync::{Arc, Mutex};

fn main() {
    let args: Vec<String> = std::env::args().collect();

    if args.len() < 3 {
        println!("Usage: ocelot <ocelot pack url> <minecraft instance path>");
        std::process::exit(1);
    }

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
        Box::new(move |cc| {
            let state = Arc::new(Mutex::new(OcelotState::default()));
            let ctx = cc.egui_ctx.clone();
            let worker_state = state.clone();

            std::thread::spawn(move || {
                let rt = tokio::runtime::Runtime::new().unwrap();
                rt.block_on(run(ocpack_url, minecraft_instance_path, worker_state, ctx));
            });

            Ok(Box::new(OcelotApp { state }))
        }),
    );
}

async fn run(
    ocpack_url: String,
    minecraft_instance_path: String,
    state: Arc<Mutex<OcelotState>>,
    ctx: egui::Context,
) {
    let result: Result<(), Box<dyn std::error::Error + Send + Sync>> = async {
        {
            let mut s = state.lock().unwrap();
            s.mod_name = "Downloading ocpack".to_owned();
        }
        ctx.request_repaint();
        let ocpack_text = reqwest::get(&ocpack_url)
            .await?
            .error_for_status()?
            .text()
            .await?;
        let pack = parse_ocpack(&ocpack_text)?;
        println!("{pack:?}");
        // add downloads
        let _ = minecraft_instance_path;
        let modrinth = Ferinth::<()>::new("ocelot", Some(env!("CARGO_PKG_VERSION")), None);
        let mods_dir = std::path::Path::new(&minecraft_instance_path).join("mods");
        tokio::fs::create_dir_all(&mods_dir).await?; // the mods dir might not exist on a new instance
        
        let mut dir = tokio::fs::read_dir(&mods_dir).await?;
        while let Some(entry) = dir.next_entry().await? { // delete stuck .part files (maybe if ocelot is closed/crashed while downloading)
            let path = entry.path();
            if path.extension().is_some_and(|e| e == "part") {
                let _ = tokio::fs::remove_file(&path).await;
            }
        }

        for (i, entry) in pack.iter().enumerate() {
            {
                let mut s = state.lock().unwrap();
                s.mod_name = entry.slug.clone();
                s.mod_progress = 0.0;
                s.download_progress = i as f32 / pack.len() as f32;
            }
            ctx.request_repaint();
            download_mod(&modrinth, entry, &mods_dir, |p| {
                state.lock().unwrap().mod_progress = p;
                ctx.request_repaint();
            })
            .await?;
        }
        state.lock().unwrap().download_progress = 1.0;
        Ok(())
    }
    .await;

    match result {
        Ok(()) => {
            {
                let mut s = state.lock().unwrap();
                s.main_text = "Done!".to_owned();
                s.mod_name = "Launching Minecraft in 3 seconds".to_owned();
                s.mod_progress = 1.0;
            }
            ctx.request_repaint();
            tokio::time::sleep(std::time::Duration::from_secs(3)).await;
            state.lock().unwrap().done = true;
        }
        Err(err) => state.lock().unwrap().error = Some(err.to_string()),
    }
    ctx.request_repaint();
}

struct OcelotState {
    main_text: String,
    download_progress: f32,
    mod_name: String,
    mod_progress: f32,
    done: bool,
    error: Option<String>,
}

struct OcelotApp {
    state: Arc<Mutex<OcelotState>>,
}

#[derive(Debug, Clone)]
struct ModpackEntry {
    slug: String,
    version: String,
    loader: String,
    required: bool,
}

impl Default for OcelotState {
    fn default() -> Self {
        Self {
            main_text: "Preparing Minecraft".to_owned(),
            download_progress: 0.0,
            mod_name: "Waiting for thread".to_owned(),
            mod_progress: 0.0,
            done: false,
            error: None,
        }
    }
}

impl eframe::App for OcelotApp {
    fn ui(&mut self, ui: &mut egui::Ui, _frame: &mut eframe::Frame) {
        egui::CentralPanel::default().show(ui, |ui| {
            let state = self.state.lock().unwrap();
            wavy_text(ui, &state.main_text, 18.0);
            ui.add_space(5.0);
            ui.add(
                egui::ProgressBar::new(state.download_progress)
                    .show_percentage()
                    .animate(true),
            );
            ui.add_space(5.0);
            ui.horizontal(|ui| {
                ui.label(
                    egui::RichText::new(format!(
                        "{} ({}%)...",
                        state.mod_name,
                        (state.mod_progress * 100.0) as i32
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
            if state.done {
                ui.ctx().send_viewport_cmd(egui::ViewportCommand::Close);
            } else if let Some(_err) = &state.error {
                //show error
            }
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

async fn download_mod(
    modrinth: &Ferinth<()>,
    entry: &ModpackEntry,
    mods_dir: &std::path::Path,
    mut on_progress: impl FnMut(f32),
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let versions = modrinth
        .version_list_filtered(&entry.slug, Some(&[entry.loader.as_str()]), None, None)
        .await?;
    let version = versions
        .iter()
        .find(|v| v.version_number == entry.version)
        .ok_or(format!(
            "mod {} has no {} version {}",
            entry.slug, entry.loader, entry.version
        ))?;

    let file = version
        .files
        .iter()
        .find(|f| f.primary)
        .or(version.files.first())
        .ok_or("version has no files")?;

    let dest = mods_dir.join(&file.filename);
    if dest.exists() {
        on_progress(1.0);
        return Ok(());
    }

    let tmp = mods_dir.join(format!("{}.part", file.filename));
    let mut resp = reqwest::get(file.url.clone()).await?.error_for_status()?;
    let mut out = tokio::fs::File::create(&tmp).await?;
    let mut done = 0usize;
    while let Some(chunk) = resp.chunk().await? {
        tokio::io::AsyncWriteExt::write_all(&mut out, &chunk).await?;
        done += chunk.len();
        on_progress(done as f32 / file.size as f32);
    }
    drop(out);
    tokio::fs::rename(&tmp, &dest).await?;
    Ok(())
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
