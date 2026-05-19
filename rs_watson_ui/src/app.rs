use chrono::{Local, Utc};
use rs_watson::config::{Config, StorageProvider};
use rs_watson::{ActiveFrame, Frame, Watson};
use rs_watson_storage::sqlite::SqliteStorage;
use uuid::Uuid;

use crate::format::{collect_projects, fmt_duration, parse_local_date, parse_local_dt, parse_tags};
use crate::types::{EditState, Tab};

pub(crate) struct WatsonApp {
    pub(crate) watson: Watson<SqliteStorage>,
    pub(crate) config: Config,

    // Cached state — refreshed after every mutation
    pub(crate) status: Option<ActiveFrame>,
    pub(crate) frames: Vec<Frame>,
    pub(crate) projects: Vec<String>,

    // Toolbar
    pub(crate) input_project: String,
    pub(crate) input_tags: String,
    pub(crate) message: Option<String>,
    pub(crate) message_is_error: bool,

    pub(crate) active_tab: Tab,

    // Log tab
    pub(crate) log_from: String,
    pub(crate) log_to: String,
    pub(crate) delete_confirm_id: Option<Uuid>,
    pub(crate) edit_state: Option<EditState>,

    // Add tab
    pub(crate) add_project: String,
    pub(crate) add_tags: String,
    pub(crate) add_from: String,
    pub(crate) add_to: String,
    pub(crate) add_message: Option<String>,
    pub(crate) add_message_is_error: bool,

    // Report tab
    pub(crate) report_from: String,
    pub(crate) report_to: String,
    pub(crate) report_use_epics: bool,
}

impl WatsonApp {
    pub(crate) fn new() -> anyhow::Result<Self> {
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

    pub(crate) fn refresh(&mut self) {
        self.status = self.watson.status().ok().flatten();
        self.frames = self.watson.log().unwrap_or_default();
        self.projects = collect_projects(&self.frames);
    }

    pub(crate) fn set_msg(&mut self, ok: bool, msg: impl Into<String>) {
        self.message = Some(msg.into());
        self.message_is_error = !ok;
    }

    pub(crate) fn set_add_msg(&mut self, ok: bool, msg: impl Into<String>) {
        self.add_message = Some(msg.into());
        self.add_message_is_error = !ok;
    }

    pub(crate) fn filtered_frames<'a>(frames: &'a [Frame], from: &str, to: &str) -> Vec<&'a Frame> {
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

    // --- Mutations -----------------------------------------------------------

    pub(crate) fn do_start(&mut self) {
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

    pub(crate) fn do_stop(&mut self) {
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

    pub(crate) fn do_cancel(&mut self) {
        match self.watson.cancel() {
            Ok(f) => {
                self.set_msg(true, format!("Cancelled \"{}\".", f.project));
                self.refresh();
            }
            Err(e) => self.set_msg(false, e.to_string()),
        }
    }

    pub(crate) fn do_add(&mut self) {
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

    pub(crate) fn do_edit_save(&mut self) {
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

    pub(crate) fn do_remove(&mut self, id: Uuid) {
        match self.watson.remove(id) {
            Ok(_) => {
                self.set_msg(true, "Frame removed.");
                self.delete_confirm_id = None;
                self.refresh();
            }
            Err(e) => self.set_msg(false, e.to_string()),
        }
    }
}
