use chrono::{Duration, Local, Utc};
use eframe::egui;
use rs_watson::config::{Config, StorageProvider};
use rs_watson::{ActiveFrame, Frame, Watson};
use rs_watson_storage::sqlite::SqliteStorage;

fn main() -> eframe::Result {
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_title("rs_watson")
            .with_inner_size([860.0, 580.0])
            .with_min_inner_size([600.0, 400.0]),
        ..Default::default()
    };
    eframe::run_native(
        "rs_watson",
        options,
        Box::new(|_cc| match WatsonApp::new() {
            Ok(app) => Ok(Box::new(app) as Box<dyn eframe::App>),
            Err(e) => Ok(Box::new(ErrorApp(e.to_string())) as Box<dyn eframe::App>),
        }),
    )
}

// ---------------------------------------------------------------------------
// Error fallback shown when the app fails to initialise
// ---------------------------------------------------------------------------

struct ErrorApp(String);

impl eframe::App for ErrorApp {
    fn ui(&mut self, ui: &mut egui::Ui, _frame: &mut eframe::Frame) {
        egui::CentralPanel::default().show_inside(ui, |ui| {
            ui.heading("Failed to start rs_watson");
            ui.separator();
            ui.label(egui::RichText::new(&self.0).color(egui::Color32::RED));
        });
    }
}

// ---------------------------------------------------------------------------
// Tab
// ---------------------------------------------------------------------------

#[derive(PartialEq)]
enum Tab {
    Log,
    Report,
}

// ---------------------------------------------------------------------------
// Main app
// ---------------------------------------------------------------------------

struct WatsonApp {
    watson: Watson<SqliteStorage>,
    #[allow(dead_code)] // reserved for epics, week_start, limit features
    config: Config,

    // Cached state — refreshed after every mutating action
    status: Option<ActiveFrame>,
    frames: Vec<Frame>,

    // Start-form inputs
    input_project: String,
    input_tags: String,

    // Inline feedback shown below the toolbar
    message: Option<String>,
    message_is_error: bool,

    active_tab: Tab,
}

impl WatsonApp {
    fn new() -> anyhow::Result<Self> {
        let config = Config::load()?;

        let data_dir = if let Ok(dir) = std::env::var("RS_WATSON_DATA_DIR") {
            std::path::PathBuf::from(dir)
        } else if let Some(dir) = &config.storage.data_dir {
            std::path::PathBuf::from(dir)
        } else {
            dirs::data_dir()
                .ok_or_else(|| anyhow::anyhow!("Could not determine data directory"))?
                .join("rs_watson")
        };
        std::fs::create_dir_all(&data_dir)?;

        let storage = match config.storage.provider {
            StorageProvider::Sqlite => SqliteStorage::new(data_dir.join("watson.db"))
                .map_err(|e| anyhow::anyhow!("SQLite: {e}"))?,
        };

        let watson = Watson::new(storage);
        let status = watson.status().ok().flatten();
        let frames = watson.log().unwrap_or_default();

        Ok(Self {
            watson,
            config,
            status,
            frames,
            input_project: String::new(),
            input_tags: String::new(),
            message: None,
            message_is_error: false,
            active_tab: Tab::Log,
        })
    }

    fn refresh(&mut self) {
        self.status = self.watson.status().ok().flatten();
        self.frames = self.watson.log().unwrap_or_default();
    }

    fn set_ok(&mut self, msg: impl Into<String>) {
        self.message = Some(msg.into());
        self.message_is_error = false;
    }

    fn set_err(&mut self, msg: impl Into<String>) {
        self.message = Some(msg.into());
        self.message_is_error = true;
    }

    fn do_start(&mut self) {
        let project = self.input_project.trim().to_string();
        if project.is_empty() {
            self.set_err("Project name is required.");
            return;
        }
        let tags: Vec<String> = self
            .input_tags
            .split(',')
            .map(|s| s.trim().to_string())
            .filter(|s| !s.is_empty())
            .collect();

        match self.watson.start_or_replace(&project, tags, Utc::now()) {
            Ok(result) => {
                let msg = if result.replaced.is_some() {
                    format!(
                        "Stopped previous session, now tracking \"{}\".",
                        result.active.project
                    )
                } else {
                    format!("Started tracking \"{}\".", result.active.project)
                };
                self.set_ok(msg);
                self.input_project.clear();
                self.input_tags.clear();
                self.refresh();
            }
            Err(e) => self.set_err(e.to_string()),
        }
    }

    fn do_stop(&mut self) {
        match self.watson.stop(Utc::now()) {
            Ok(frame) => {
                self.set_ok(format!(
                    "Stopped \"{}\" — {}.",
                    frame.project,
                    fmt_duration(frame.end - frame.start)
                ));
                self.refresh();
            }
            Err(e) => self.set_err(e.to_string()),
        }
    }

    fn do_cancel(&mut self) {
        match self.watson.cancel() {
            Ok(frame) => {
                self.set_ok(format!("Cancelled \"{}\".", frame.project));
                self.refresh();
            }
            Err(e) => self.set_err(e.to_string()),
        }
    }
}

// ---------------------------------------------------------------------------
// eframe::App impl
// ---------------------------------------------------------------------------

impl eframe::App for WatsonApp {
    fn ui(&mut self, ui: &mut egui::Ui, _frame: &mut eframe::Frame) {
        // Repaint once per second so the elapsed timer stays live.
        ui.ctx().request_repaint_after(std::time::Duration::from_secs(1));

        egui::Panel::top("status_bar").show_inside(ui, |ui| {
            ui.add_space(6.0);
            ui.horizontal(|ui| match &self.status {
                Some(active) => {
                    let elapsed = Utc::now() - active.start;
                    ui.label(
                        egui::RichText::new("● TRACKING")
                            .color(egui::Color32::from_rgb(80, 200, 120))
                            .strong(),
                    );
                    ui.label(
                        egui::RichText::new(format!(
                            "  {}{}  —  {}",
                            active.project,
                            fmt_tags(&active.tags),
                            fmt_duration(elapsed),
                        ))
                        .strong(),
                    );
                }
                None => {
                    ui.label(
                        egui::RichText::new("○ Not tracking").color(egui::Color32::GRAY),
                    );
                }
            });
            ui.add_space(4.0);
        });

        egui::Panel::top("toolbar").show_inside(ui, |ui| {
            ui.add_space(6.0);
            ui.horizontal(|ui| {
                ui.label("Project");
                let proj_resp = ui.add(
                    egui::TextEdit::singleline(&mut self.input_project)
                        .hint_text("backend")
                        .desired_width(160.0),
                );
                ui.label("Tags");
                ui.add(
                    egui::TextEdit::singleline(&mut self.input_tags)
                        .hint_text("api, auth")
                        .desired_width(160.0),
                );

                let start_clicked = ui
                    .button(egui::RichText::new("▶  Start").color(egui::Color32::from_rgb(80, 200, 120)))
                    .clicked();
                let enter_pressed = proj_resp.lost_focus()
                    && ui.input(|i| i.key_pressed(egui::Key::Enter));
                if start_clicked || enter_pressed {
                    self.do_start();
                }

                let tracking = self.status.is_some();
                ui.add_enabled_ui(tracking, |ui| {
                    if ui
                        .button(egui::RichText::new("■  Stop").color(egui::Color32::from_rgb(220, 80, 80)))
                        .clicked()
                    {
                        self.do_stop();
                    }
                    if ui
                        .button(egui::RichText::new("✕  Cancel").color(egui::Color32::GRAY))
                        .clicked()
                    {
                        self.do_cancel();
                    }
                });
            });

            if let Some(msg) = &self.message {
                let color = if self.message_is_error {
                    egui::Color32::from_rgb(220, 80, 80)
                } else {
                    egui::Color32::from_rgb(80, 200, 120)
                };
                ui.add_space(2.0);
                ui.label(egui::RichText::new(msg).color(color).small());
            }
            ui.add_space(4.0);
        });

        egui::Panel::top("tabs").show_inside(ui, |ui| {
            ui.horizontal(|ui| {
                ui.selectable_value(&mut self.active_tab, Tab::Log, "Log");
                ui.selectable_value(&mut self.active_tab, Tab::Report, "Report");
            });
        });

        egui::CentralPanel::default().show_inside(ui, |ui| match self.active_tab {
            Tab::Log => self.show_log(ui),
            Tab::Report => self.show_report(ui),
        });
    }
}

// ---------------------------------------------------------------------------
// Log view
// ---------------------------------------------------------------------------

impl WatsonApp {
    fn show_log(&self, ui: &mut egui::Ui) {
        if self.frames.is_empty() {
            ui.centered_and_justified(|ui| {
                ui.label(
                    egui::RichText::new("No frames recorded yet.").color(egui::Color32::GRAY),
                );
            });
            return;
        }

        use std::collections::BTreeMap;
        let mut by_day: BTreeMap<chrono::NaiveDate, Vec<&Frame>> = BTreeMap::new();
        for frame in self.frames.iter().rev() {
            by_day
                .entry(frame.start.with_timezone(&Local).date_naive())
                .or_default()
                .push(frame);
        }

        egui::ScrollArea::vertical().show(ui, |ui| {
            for (date, day_frames) in by_day.iter().rev() {
                let day_total = day_frames
                    .iter()
                    .fold(Duration::zero(), |acc, f| acc + (f.end - f.start));

                ui.add_space(6.0);
                ui.horizontal(|ui| {
                    ui.label(
                        egui::RichText::new(date.format("%A, %d %B %Y").to_string()).strong(),
                    );
                    ui.label(
                        egui::RichText::new(format!("({})", fmt_duration(day_total)))
                            .color(egui::Color32::GRAY)
                            .small(),
                    );
                });
                ui.separator();

                for frame in day_frames {
                    ui.horizontal(|ui| {
                        ui.label(
                            egui::RichText::new(format!(
                                "{}  →  {}",
                                fmt_time(frame.start),
                                fmt_time(frame.end)
                            ))
                            .monospace()
                            .color(egui::Color32::LIGHT_GRAY),
                        );
                        ui.label(
                            egui::RichText::new(fmt_duration(frame.end - frame.start))
                                .monospace()
                                .color(egui::Color32::from_rgb(180, 100, 220)),
                        );
                        ui.label(egui::RichText::new(&frame.project).strong());
                        if !frame.tags.is_empty() {
                            ui.label(
                                egui::RichText::new(format!("[{}]", frame.tags.join(", ")))
                                    .color(egui::Color32::from_rgb(100, 180, 220))
                                    .small(),
                            );
                        }
                    });
                }
            }
            ui.add_space(8.0);
        });
    }
}

// ---------------------------------------------------------------------------
// Report view
// ---------------------------------------------------------------------------

impl WatsonApp {
    fn show_report(&self, ui: &mut egui::Ui) {
        if self.frames.is_empty() {
            ui.centered_and_justified(|ui| {
                ui.label(
                    egui::RichText::new("No frames recorded yet.").color(egui::Color32::GRAY),
                );
            });
            return;
        }

        let report = rs_watson::Report::from_frames(&self.frames);

        egui::ScrollArea::vertical().show(ui, |ui| {
            ui.add_space(6.0);
            ui.horizontal(|ui| {
                ui.label(egui::RichText::new("Total").strong());
                ui.label(
                    egui::RichText::new(fmt_duration(report.total))
                        .color(egui::Color32::from_rgb(180, 100, 220))
                        .strong(),
                );
            });
            ui.separator();

            for project in &report.projects {
                ui.add_space(4.0);
                ui.horizontal(|ui| {
                    ui.label(
                        egui::RichText::new(&project.name)
                            .strong()
                            .color(egui::Color32::from_rgb(220, 180, 80)),
                    );
                    ui.label(
                        egui::RichText::new(fmt_duration(project.total))
                            .color(egui::Color32::from_rgb(180, 100, 220))
                            .strong(),
                    );
                });
                for tag in &project.tags {
                    ui.horizontal(|ui| {
                        ui.add_space(16.0);
                        ui.label(
                            egui::RichText::new(format!("[{}]", tag.name))
                                .color(egui::Color32::from_rgb(100, 180, 220))
                                .small(),
                        );
                        ui.label(
                            egui::RichText::new(fmt_duration(tag.total))
                                .color(egui::Color32::from_rgb(180, 100, 220))
                                .small(),
                        );
                    });
                }
            }
            ui.add_space(8.0);
        });
    }
}

// ---------------------------------------------------------------------------
// Formatting helpers
// ---------------------------------------------------------------------------

fn fmt_duration(d: Duration) -> String {
    let total = d.num_seconds().max(0);
    let h = total / 3600;
    let m = (total % 3600) / 60;
    let s = total % 60;
    if h > 0 {
        format!("{h}h {m:02}m {s:02}s")
    } else if m > 0 {
        format!("{m}m {s:02}s")
    } else {
        format!("{s}s")
    }
}

fn fmt_time(dt: chrono::DateTime<Utc>) -> String {
    dt.with_timezone(&Local).format("%H:%M:%S").to_string()
}

fn fmt_tags(tags: &[String]) -> String {
    if tags.is_empty() {
        String::new()
    } else {
        format!("  [{}]", tags.join(", "))
    }
}
