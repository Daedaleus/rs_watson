#![cfg_attr(target_os = "windows", windows_subsystem = "windows")]

mod app;
mod colors;
mod format;
mod tab_add;
mod tab_log;
mod tab_report;
mod types;
mod widgets;

use chrono::Utc;
use eframe::egui;

use app::WatsonApp;
use colors::{CLR_GREEN, CLR_RED};
use format::{fmt_duration, fmt_tags};
use types::Tab;
use widgets::{feedback_label, project_autocomplete};

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

impl eframe::App for WatsonApp {
    fn ui(&mut self, ui: &mut egui::Ui, _frame: &mut eframe::Frame) {
        ui.ctx()
            .request_repaint_after(std::time::Duration::from_secs(1));

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
            let projects = self.projects.clone();
            ui.horizontal(|ui| {
                ui.label("Project");
                let proj_id = egui::Id::new("proj_popup");
                let proj_resp = ui.add(
                    egui::TextEdit::singleline(&mut self.input_project)
                        .hint_text("backend")
                        .desired_width(150.0),
                );
                project_autocomplete(ui, proj_id, &proj_resp, &mut self.input_project, &projects);

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

            if let Some(msg) = self.message.clone() {
                ui.add_space(2.0);
                feedback_label(ui, &msg, self.message_is_error, true);
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
