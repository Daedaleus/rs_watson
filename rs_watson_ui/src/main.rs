use std::collections::BTreeMap;

use chrono::{DateTime, Duration, Local, NaiveDate, NaiveDateTime, NaiveTime, TimeZone, Utc};
use eframe::egui;
use rs_watson::config::{Config, StorageProvider};
use rs_watson::{ActiveFrame, Frame, Report, Watson, resolve_epic};
use rs_watson_storage::sqlite::SqliteStorage;
use uuid::Uuid;

// ---------------------------------------------------------------------------
// Entry point
// ---------------------------------------------------------------------------

fn main() -> eframe::Result {
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_title("rs_watson")
            .with_inner_size([900.0, 620.0])
            .with_min_inner_size([640.0, 420.0]),
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
// Error fallback
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
// State types
// ---------------------------------------------------------------------------

#[derive(PartialEq, Clone, Copy)]
enum Tab {
    Log,
    Add,
    Report,
}

struct EditState {
    id: Uuid,
    project: String,
    tags: String,
    start: String,
    end: String,
    error: Option<String>,
}

impl EditState {
    fn from_frame(f: &Frame) -> Self {
        Self {
            id: f.id,
            project: f.project.clone(),
            tags: f.tags.join(", "),
            start: fmt_local_dt(f.start),
            end: fmt_local_dt(f.end),
            error: None,
        }
    }
}

// ---------------------------------------------------------------------------
// App struct
// ---------------------------------------------------------------------------

struct WatsonApp {
    watson: Watson<SqliteStorage>,
    config: Config,

    // Cached — refreshed after every mutation
    status: Option<ActiveFrame>,
    frames: Vec<Frame>,
    projects: Vec<String>,

    // Toolbar (start/stop)
    input_project: String,
    input_tags: String,
    message: Option<String>,
    message_is_error: bool,

    active_tab: Tab,

    // Log tab
    log_from: String,
    log_to: String,
    delete_confirm_id: Option<Uuid>,
    edit_state: Option<EditState>,

    // Add tab
    add_project: String,
    add_tags: String,
    add_from: String,
    add_to: String,
    add_message: Option<String>,
    add_message_is_error: bool,

    // Report tab
    report_from: String,
    report_to: String,
    report_use_epics: bool,
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
        let projects = collect_projects(&frames);

        Ok(Self {
            watson,
            config,
            status,
            frames,
            projects,
            input_project: String::new(),
            input_tags: String::new(),
            message: None,
            message_is_error: false,
            active_tab: Tab::Log,
            log_from: String::new(),
            log_to: String::new(),
            delete_confirm_id: None,
            edit_state: None,
            add_project: String::new(),
            add_tags: String::new(),
            add_from: String::new(),
            add_to: String::new(),
            add_message: None,
            add_message_is_error: false,
            report_from: String::new(),
            report_to: String::new(),
            report_use_epics: false,
        })
    }

    fn refresh(&mut self) {
        self.status = self.watson.status().ok().flatten();
        self.frames = self.watson.log().unwrap_or_default();
        self.projects = collect_projects(&self.frames);
    }

    fn set_msg(&mut self, ok: bool, msg: impl Into<String>) {
        self.message = Some(msg.into());
        self.message_is_error = !ok;
    }

    fn set_add_msg(&mut self, ok: bool, msg: impl Into<String>) {
        self.add_message = Some(msg.into());
        self.add_message_is_error = !ok;
    }

    // --- Mutations -----------------------------------------------------------

    fn do_start(&mut self) {
        let project = self.input_project.trim().to_string();
        if project.is_empty() {
            self.set_msg(false, "Project name is required.");
            return;
        }
        let tags = parse_tags(&self.input_tags);
        match self.watson.start_or_replace(&project, tags, Utc::now()) {
            Ok(r) => {
                let msg = if r.replaced.is_some() {
                    format!(
                        "Stopped previous session, now tracking \"{}\".",
                        r.active.project
                    )
                } else {
                    format!("Started tracking \"{}\".", r.active.project)
                };
                self.set_msg(true, msg);
                self.input_project.clear();
                self.input_tags.clear();
                self.refresh();
            }
            Err(e) => self.set_msg(false, e.to_string()),
        }
    }

    fn do_stop(&mut self) {
        match self.watson.stop(Utc::now()) {
            Ok(f) => {
                self.set_msg(
                    true,
                    format!(
                        "Stopped \"{}\" — {}.",
                        f.project,
                        fmt_duration(f.end - f.start)
                    ),
                );
                self.refresh();
            }
            Err(e) => self.set_msg(false, e.to_string()),
        }
    }

    fn do_cancel(&mut self) {
        match self.watson.cancel() {
            Ok(f) => {
                self.set_msg(true, format!("Cancelled \"{}\".", f.project));
                self.refresh();
            }
            Err(e) => self.set_msg(false, e.to_string()),
        }
    }

    fn do_add(&mut self) {
        let project = self.add_project.trim().to_string();
        if project.is_empty() {
            self.set_add_msg(false, "Project name is required.");
            return;
        }
        let Some(start) = parse_local_dt(&self.add_from) else {
            self.set_add_msg(false, "Invalid start time. Use YYYY-MM-DD HH:MM or HH:MM.");
            return;
        };
        let Some(end) = parse_local_dt(&self.add_to) else {
            self.set_add_msg(false, "Invalid end time. Use YYYY-MM-DD HH:MM or HH:MM.");
            return;
        };
        let tags = parse_tags(&self.add_tags);
        match self.watson.add(&project, tags, start, end) {
            Ok(f) => {
                self.set_add_msg(
                    true,
                    format!(
                        "Added \"{}\" — {}.",
                        f.project,
                        fmt_duration(f.end - f.start)
                    ),
                );
                self.add_project.clear();
                self.add_tags.clear();
                self.add_from.clear();
                self.add_to.clear();
                self.refresh();
            }
            Err(e) => self.set_add_msg(false, e.to_string()),
        }
    }

    fn do_edit_save(&mut self) {
        let Some(state) = &mut self.edit_state else {
            return;
        };

        let project = state.project.trim().to_string();
        if project.is_empty() {
            state.error = Some("Project name is required.".into());
            return;
        }
        let Some(start) = parse_local_dt(&state.start) else {
            state.error = Some("Invalid start time.".into());
            return;
        };
        let Some(end) = parse_local_dt(&state.end) else {
            state.error = Some("Invalid end time.".into());
            return;
        };
        let tags = parse_tags(&state.tags);
        let id = state.id;

        match self.watson.edit(id, project, tags, start, end) {
            Ok(_) => {
                self.edit_state = None;
                self.set_msg(true, "Frame updated.");
                self.refresh();
            }
            Err(e) => {
                if let Some(s) = &mut self.edit_state {
                    s.error = Some(e.to_string());
                }
            }
        }
    }

    fn do_remove(&mut self, id: Uuid) {
        match self.watson.remove(id) {
            Ok(_) => {
                self.set_msg(true, "Frame removed.");
                self.delete_confirm_id = None;
                self.refresh();
            }
            Err(e) => self.set_msg(false, e.to_string()),
        }
    }

    // --- Filtering -----------------------------------------------------------

    fn filtered_frames<'a>(frames: &'a [Frame], from: &str, to: &str) -> Vec<&'a Frame> {
        let from_date = parse_local_date(from);
        let to_date = parse_local_date(to);
        frames
            .iter()
            .filter(|f| {
                let d = f.start.with_timezone(&Local).date_naive();
                from_date.is_none_or(|fd| d >= fd) && to_date.is_none_or(|td| d <= td)
            })
            .collect()
    }
}

// ---------------------------------------------------------------------------
// eframe::App
// ---------------------------------------------------------------------------

impl eframe::App for WatsonApp {
    fn ui(&mut self, ui: &mut egui::Ui, _frame: &mut eframe::Frame) {
        ui.ctx()
            .request_repaint_after(std::time::Duration::from_secs(1));

        // Edit modal is a floating window — must be called before panels.
        if self.edit_state.is_some() {
            let ctx = ui.ctx().clone();
            self.show_edit_modal(&ctx);
        }

        egui::Panel::top("status_bar").show_inside(ui, |ui| {
            ui.add_space(5.0);
            ui.horizontal(|ui| match &self.status {
                Some(active) => {
                    let elapsed = Utc::now() - active.start;
                    ui.label(egui::RichText::new("● TRACKING").color(CLR_GREEN).strong());
                    ui.label(
                        egui::RichText::new(format!(
                            "  {}{}  —  {}",
                            active.project,
                            fmt_tags(&active.tags),
                            fmt_duration(elapsed)
                        ))
                        .strong(),
                    );
                }
                None => {
                    ui.label(egui::RichText::new("○ Not tracking").color(egui::Color32::GRAY));
                }
            });
            ui.add_space(4.0);
        });

        egui::Panel::top("toolbar").show_inside(ui, |ui| {
            ui.add_space(5.0);
            ui.horizontal(|ui| {
                // Project field with suggestion popup
                ui.label("Project");
                let proj_id = egui::Id::new("proj_popup");
                let proj_resp = ui.add(
                    egui::TextEdit::singleline(&mut self.input_project)
                        .hint_text("backend")
                        .desired_width(150.0),
                );
                if proj_resp.gained_focus() && !self.projects.is_empty() {
                    #[allow(deprecated)]
                    ui.memory_mut(|m| m.open_popup(proj_id));
                }
                #[allow(deprecated)]
                egui::popup_below_widget(
                    ui,
                    proj_id,
                    &proj_resp,
                    egui::PopupCloseBehavior::CloseOnClickOutside,
                    |ui| {
                        ui.set_min_width(150.0);
                        let filter = self.input_project.to_lowercase();
                        for proj in self
                            .projects
                            .clone()
                            .iter()
                            .filter(|p| filter.is_empty() || p.to_lowercase().contains(&filter))
                        {
                            if ui.selectable_label(false, proj).clicked() {
                                self.input_project = proj.clone();
                                ui.memory_mut(|m| m.close_popup(proj_id));
                            }
                        }
                    },
                );

                ui.label("Tags");
                ui.add(
                    egui::TextEdit::singleline(&mut self.input_tags)
                        .hint_text("api, auth")
                        .desired_width(140.0),
                );

                let start_clicked = ui
                    .button(egui::RichText::new("▶  Start").color(CLR_GREEN))
                    .clicked();
                let enter = proj_resp.lost_focus() && ui.input(|i| i.key_pressed(egui::Key::Enter));
                if start_clicked || enter {
                    self.do_start();
                }

                let tracking = self.status.is_some();
                ui.add_enabled_ui(tracking, |ui| {
                    if ui
                        .button(egui::RichText::new("■  Stop").color(CLR_RED))
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
                ui.add_space(2.0);
                let color = if self.message_is_error {
                    CLR_RED
                } else {
                    CLR_GREEN
                };
                ui.label(egui::RichText::new(msg).color(color).small());
            }
            ui.add_space(4.0);
        });

        egui::Panel::top("tabs").show_inside(ui, |ui| {
            ui.horizontal(|ui| {
                ui.selectable_value(&mut self.active_tab, Tab::Log, "Log");
                ui.selectable_value(&mut self.active_tab, Tab::Add, "Add");
                ui.selectable_value(&mut self.active_tab, Tab::Report, "Report");
            });
        });

        egui::CentralPanel::default().show_inside(ui, |ui| match self.active_tab {
            Tab::Log => self.show_log(ui),
            Tab::Add => self.show_add(ui),
            Tab::Report => self.show_report(ui),
        });
    }
}

// ---------------------------------------------------------------------------
// Log tab
// ---------------------------------------------------------------------------

impl WatsonApp {
    fn show_log(&mut self, ui: &mut egui::Ui) {
        // Date filter
        ui.horizontal(|ui| {
            ui.label("From");
            ui.add(
                egui::TextEdit::singleline(&mut self.log_from)
                    .hint_text("YYYY-MM-DD")
                    .desired_width(100.0),
            );
            ui.label("To");
            ui.add(
                egui::TextEdit::singleline(&mut self.log_to)
                    .hint_text("YYYY-MM-DD")
                    .desired_width(100.0),
            );
            if ui.small_button("Clear").clicked() {
                self.log_from.clear();
                self.log_to.clear();
            }
        });
        ui.separator();

        let visible: Vec<&Frame> =
            Self::filtered_frames(&self.frames, &self.log_from, &self.log_to);
        if visible.is_empty() {
            ui.centered_and_justified(|ui| {
                ui.label(egui::RichText::new("No frames.").color(egui::Color32::GRAY));
            });
            return;
        }

        // Collect mutations requested during rendering — avoids borrow conflicts.
        let mut to_edit: Option<Uuid> = None;
        let mut to_delete: Option<Uuid> = None;
        let mut confirmed_delete: Option<Uuid> = None;
        let mut cancel_delete = false;

        let mut by_day: BTreeMap<NaiveDate, Vec<&Frame>> = BTreeMap::new();
        for f in visible.iter().rev() {
            by_day
                .entry(f.start.with_timezone(&Local).date_naive())
                .or_default()
                .push(f);
        }

        egui::ScrollArea::vertical().show(ui, |ui| {
            for (date, day_frames) in by_day.iter().rev() {
                let day_total = day_frames
                    .iter()
                    .fold(Duration::zero(), |a, f| a + (f.end - f.start));
                ui.add_space(4.0);
                ui.horizontal(|ui| {
                    ui.label(egui::RichText::new(date.format("%A, %d %B %Y").to_string()).strong());
                    ui.label(
                        egui::RichText::new(format!("({})", fmt_duration(day_total)))
                            .color(egui::Color32::GRAY)
                            .small(),
                    );
                });
                ui.separator();

                for frame in day_frames {
                    if self.delete_confirm_id == Some(frame.id) {
                        // Inline delete confirmation row
                        ui.horizontal(|ui| {
                            ui.label(
                                egui::RichText::new(format!(
                                    "Delete \"{}\" {}  →  {} ?",
                                    frame.project,
                                    fmt_time(frame.start),
                                    fmt_time(frame.end)
                                ))
                                .color(CLR_RED),
                            );
                            if ui
                                .button(egui::RichText::new("Yes, delete").color(CLR_RED))
                                .clicked()
                            {
                                confirmed_delete = Some(frame.id);
                            }
                            if ui.button("Cancel").clicked() {
                                cancel_delete = true;
                            }
                        });
                    } else {
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
                                    .color(CLR_PURPLE),
                            );
                            ui.label(egui::RichText::new(&frame.project).strong());
                            if !frame.tags.is_empty() {
                                ui.label(
                                    egui::RichText::new(format!("[{}]", frame.tags.join(", ")))
                                        .color(CLR_CYAN)
                                        .small(),
                                );
                            }
                            ui.with_layout(
                                egui::Layout::right_to_left(egui::Align::Center),
                                |ui| {
                                    if ui
                                        .small_button(egui::RichText::new("✕").color(CLR_RED))
                                        .clicked()
                                    {
                                        to_delete = Some(frame.id);
                                    }
                                    if ui.small_button("Edit").clicked() {
                                        to_edit = Some(frame.id);
                                    }
                                },
                            );
                        });
                    }
                }
            }
            ui.add_space(8.0);
        });

        // Apply mutations after rendering
        if let Some(id) = confirmed_delete {
            self.do_remove(id);
        } else if cancel_delete {
            self.delete_confirm_id = None;
        } else if let Some(id) = to_delete {
            self.delete_confirm_id = Some(id);
        }
        if let Some(id) = to_edit.and_then(|id| {
            self.frames
                .iter()
                .find(|f| f.id == id)
                .map(|f| (id, f.clone()))
        }) {
            self.edit_state = Some(EditState::from_frame(&id.1));
            self.delete_confirm_id = None;
        }
    }
}

// ---------------------------------------------------------------------------
// Edit modal
// ---------------------------------------------------------------------------

impl WatsonApp {
    fn show_edit_modal(&mut self, ctx: &egui::Context) {
        let Some(state) = &mut self.edit_state else {
            return;
        };
        let mut save = false;
        let mut close = false;

        egui::Window::new("Edit Frame")
            .collapsible(false)
            .resizable(false)
            .anchor(egui::Align2::CENTER_CENTER, [0.0, 0.0])
            .show(ctx, |ui| {
                egui::Grid::new("edit_grid")
                    .num_columns(2)
                    .spacing([12.0, 8.0])
                    .show(ui, |ui| {
                        ui.label("Project");
                        ui.add(egui::TextEdit::singleline(&mut state.project).desired_width(220.0));
                        ui.end_row();

                        ui.label("Tags");
                        ui.add(
                            egui::TextEdit::singleline(&mut state.tags)
                                .hint_text("api, auth")
                                .desired_width(220.0),
                        );
                        ui.end_row();

                        ui.label("Start");
                        ui.add(egui::TextEdit::singleline(&mut state.start).desired_width(220.0));
                        ui.end_row();

                        ui.label("End");
                        ui.add(egui::TextEdit::singleline(&mut state.end).desired_width(220.0));
                        ui.end_row();
                    });

                if let Some(err) = &state.error {
                    ui.colored_label(CLR_RED, err);
                }

                ui.separator();
                ui.horizontal(|ui| {
                    if ui.button(egui::RichText::new("Save").strong()).clicked() {
                        save = true;
                    }
                    if ui.button("Cancel").clicked() {
                        close = true;
                    }
                });
            });

        if save {
            self.do_edit_save();
        } else if close {
            self.edit_state = None;
        }
    }
}

// ---------------------------------------------------------------------------
// Add tab
// ---------------------------------------------------------------------------

impl WatsonApp {
    fn show_add(&mut self, ui: &mut egui::Ui) {
        ui.add_space(12.0);

        egui::Grid::new("add_grid")
            .num_columns(2)
            .spacing([12.0, 10.0])
            .show(ui, |ui| {
                ui.label("Project");
                // Project field with autocomplete popup
                let add_proj_id = egui::Id::new("add_proj_popup");
                let add_proj_resp = ui.add(
                    egui::TextEdit::singleline(&mut self.add_project)
                        .hint_text("backend")
                        .desired_width(240.0),
                );
                if add_proj_resp.gained_focus() && !self.projects.is_empty() {
                    #[allow(deprecated)]
                    ui.memory_mut(|m| m.open_popup(add_proj_id));
                }
                #[allow(deprecated)]
                egui::popup_below_widget(
                    ui,
                    add_proj_id,
                    &add_proj_resp,
                    egui::PopupCloseBehavior::CloseOnClickOutside,
                    |ui| {
                        ui.set_min_width(240.0);
                        let filter = self.add_project.to_lowercase();
                        for proj in self
                            .projects
                            .clone()
                            .iter()
                            .filter(|p| filter.is_empty() || p.to_lowercase().contains(&filter))
                        {
                            if ui.selectable_label(false, proj).clicked() {
                                self.add_project = proj.clone();
                                ui.memory_mut(|m| m.close_popup(add_proj_id));
                            }
                        }
                    },
                );
                ui.end_row();

                ui.label("Tags");
                ui.add(
                    egui::TextEdit::singleline(&mut self.add_tags)
                        .hint_text("api, auth  (comma-separated)")
                        .desired_width(240.0),
                );
                ui.end_row();

                ui.label("Start");
                ui.add(
                    egui::TextEdit::singleline(&mut self.add_from)
                        .hint_text("YYYY-MM-DD HH:MM  or  HH:MM")
                        .desired_width(240.0),
                );
                ui.end_row();

                ui.label("End");
                ui.add(
                    egui::TextEdit::singleline(&mut self.add_to)
                        .hint_text("YYYY-MM-DD HH:MM  or  HH:MM")
                        .desired_width(240.0),
                );
                ui.end_row();
            });

        ui.add_space(8.0);
        if ui
            .button(egui::RichText::new("Add Frame").strong())
            .clicked()
        {
            self.do_add();
        }

        if let Some(msg) = &self.add_message {
            ui.add_space(6.0);
            let color = if self.add_message_is_error {
                CLR_RED
            } else {
                CLR_GREEN
            };
            ui.label(egui::RichText::new(msg).color(color));
        }
    }
}

// ---------------------------------------------------------------------------
// Report tab
// ---------------------------------------------------------------------------

impl WatsonApp {
    fn show_report(&mut self, ui: &mut egui::Ui) {
        // Filter + epics toggle
        ui.horizontal(|ui| {
            ui.label("From");
            ui.add(
                egui::TextEdit::singleline(&mut self.report_from)
                    .hint_text("YYYY-MM-DD")
                    .desired_width(100.0),
            );
            ui.label("To");
            ui.add(
                egui::TextEdit::singleline(&mut self.report_to)
                    .hint_text("YYYY-MM-DD")
                    .desired_width(100.0),
            );
            if ui.small_button("Clear").clicked() {
                self.report_from.clear();
                self.report_to.clear();
            }
            if !self.config.epics.is_empty() {
                ui.separator();
                ui.checkbox(&mut self.report_use_epics, "By Epic");
            }
        });
        ui.separator();

        let visible: Vec<Frame> =
            Self::filtered_frames(&self.frames, &self.report_from, &self.report_to)
                .into_iter()
                .cloned()
                .collect();

        if visible.is_empty() {
            ui.centered_and_justified(|ui| {
                ui.label(egui::RichText::new("No frames.").color(egui::Color32::GRAY));
            });
            return;
        }

        egui::ScrollArea::vertical().show(ui, |ui| {
            if self.report_use_epics && !self.config.epics.is_empty() {
                self.render_epic_report(ui, &visible);
            } else {
                let report = Report::from_frames(&visible);
                render_project_report(ui, &report, true);
            }
        });
    }

    fn render_epic_report(&self, ui: &mut egui::Ui, frames: &[Frame]) {
        let epics = &self.config.epics;
        let mut buckets: Vec<(&str, Vec<&Frame>)> =
            epics.iter().map(|e| (e.name.as_str(), vec![])).collect();
        let mut unassigned: Vec<&Frame> = vec![];

        for frame in frames {
            match resolve_epic(frame, epics) {
                Some(name) => {
                    if let Some(b) = buckets.iter_mut().find(|(n, _)| *n == name) {
                        b.1.push(frame);
                    }
                }
                None => unassigned.push(frame),
            }
        }

        let grand_total = frames
            .iter()
            .fold(Duration::zero(), |a, f| a + (f.end - f.start));

        for (name, epic_frames) in buckets.iter().filter(|(_, f)| !f.is_empty()) {
            let owned: Vec<Frame> = epic_frames.iter().copied().cloned().collect();
            let report = Report::from_frames(&owned);
            ui.add_space(4.0);
            ui.horizontal(|ui| {
                ui.label(
                    egui::RichText::new(format!("◆  {name}"))
                        .color(CLR_CYAN)
                        .strong(),
                );
                ui.label(
                    egui::RichText::new(format!("({})", fmt_duration(report.total)))
                        .color(egui::Color32::GRAY)
                        .small(),
                );
            });
            render_project_report(ui, &report, false);
            ui.add_space(4.0);
        }

        if !unassigned.is_empty() {
            let owned: Vec<Frame> = unassigned.iter().copied().cloned().collect();
            let report = Report::from_frames(&owned);
            ui.add_space(4.0);
            ui.horizontal(|ui| {
                ui.label(
                    egui::RichText::new("◆  Unassigned")
                        .color(egui::Color32::GRAY)
                        .strong(),
                );
                ui.label(
                    egui::RichText::new(format!("({})", fmt_duration(report.total)))
                        .color(egui::Color32::GRAY)
                        .small(),
                );
            });
            render_project_report(ui, &report, false);
        }

        ui.separator();
        ui.horizontal(|ui| {
            ui.label(egui::RichText::new("Total").strong());
            ui.label(
                egui::RichText::new(fmt_duration(grand_total))
                    .color(CLR_PURPLE)
                    .strong(),
            );
        });
        ui.add_space(8.0);
    }
}

fn render_project_report(ui: &mut egui::Ui, report: &Report, show_total: bool) {
    for project in &report.projects {
        ui.add_space(2.0);
        ui.horizontal(|ui| {
            ui.label(
                egui::RichText::new(&project.name)
                    .strong()
                    .color(CLR_YELLOW),
            );
            ui.label(
                egui::RichText::new(fmt_duration(project.total))
                    .color(CLR_PURPLE)
                    .strong(),
            );
        });
        for tag in &project.tags {
            ui.horizontal(|ui| {
                ui.add_space(16.0);
                ui.label(
                    egui::RichText::new(format!("[{}]", tag.name))
                        .color(CLR_CYAN)
                        .small(),
                );
                ui.label(
                    egui::RichText::new(fmt_duration(tag.total))
                        .color(CLR_PURPLE)
                        .small(),
                );
            });
        }
    }
    if show_total && report.projects.len() > 1 {
        ui.separator();
        ui.horizontal(|ui| {
            ui.label(egui::RichText::new("Total").strong());
            ui.label(
                egui::RichText::new(fmt_duration(report.total))
                    .color(CLR_PURPLE)
                    .strong(),
            );
        });
    }
    ui.add_space(4.0);
}

// ---------------------------------------------------------------------------
// Colours
// ---------------------------------------------------------------------------

const CLR_GREEN: egui::Color32 = egui::Color32::from_rgb(80, 200, 120);
const CLR_RED: egui::Color32 = egui::Color32::from_rgb(220, 80, 80);
const CLR_PURPLE: egui::Color32 = egui::Color32::from_rgb(180, 100, 220);
const CLR_CYAN: egui::Color32 = egui::Color32::from_rgb(100, 180, 220);
const CLR_YELLOW: egui::Color32 = egui::Color32::from_rgb(220, 180, 80);

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

fn collect_projects(frames: &[Frame]) -> Vec<String> {
    let mut names: Vec<String> = frames.iter().map(|f| f.project.clone()).collect();
    names.sort();
    names.dedup();
    names
}

fn parse_tags(s: &str) -> Vec<String> {
    s.split(',')
        .map(|t| t.trim().to_string())
        .filter(|t| !t.is_empty())
        .collect()
}

fn parse_local_dt(s: &str) -> Option<DateTime<Utc>> {
    let s = s.trim();
    let try_naive = |fmt| NaiveDateTime::parse_from_str(s, fmt).ok();
    let naive = try_naive("%Y-%m-%d %H:%M:%S")
        .or_else(|| try_naive("%Y-%m-%d %H:%M"))
        .or_else(|| {
            NaiveTime::parse_from_str(s, "%H:%M:%S")
                .or_else(|_| NaiveTime::parse_from_str(s, "%H:%M"))
                .ok()
                .map(|t| Local::now().date_naive().and_time(t))
        })?;
    Local
        .from_local_datetime(&naive)
        .single()
        .map(|dt| dt.with_timezone(&Utc))
}

fn parse_local_date(s: &str) -> Option<NaiveDate> {
    let s = s.trim().to_lowercase();
    match s.as_str() {
        "" => None,
        "today" => Some(Local::now().date_naive()),
        "yesterday" => Some(Local::now().date_naive() - Duration::days(1)),
        _ => NaiveDate::parse_from_str(&s, "%Y-%m-%d").ok(),
    }
}

fn fmt_local_dt(dt: DateTime<Utc>) -> String {
    dt.with_timezone(&Local)
        .format("%Y-%m-%d %H:%M:%S")
        .to_string()
}

fn fmt_time(dt: DateTime<Utc>) -> String {
    dt.with_timezone(&Local).format("%H:%M:%S").to_string()
}

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

fn fmt_tags(tags: &[String]) -> String {
    if tags.is_empty() {
        String::new()
    } else {
        format!("  [{}]", tags.join(", "))
    }
}
