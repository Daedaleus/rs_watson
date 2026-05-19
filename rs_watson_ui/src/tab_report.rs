use chrono::Duration;
use eframe::egui;
use rs_watson::{Frame, Report, resolve_epic};

use crate::app::WatsonApp;
use crate::colors::{CLR_CYAN, CLR_PURPLE, CLR_YELLOW};
use crate::format::fmt_duration;
use crate::widgets::{date_filter_bar, empty_frames};

impl WatsonApp {
    pub(crate) fn show_report(&mut self, ui: &mut egui::Ui) {
        ui.horizontal(|ui| {
            date_filter_bar(ui, &mut self.report_from, &mut self.report_to);
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
            empty_frames(ui);
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

pub(crate) fn render_project_report(ui: &mut egui::Ui, report: &Report, show_total: bool) {
    for project in &report.projects {
        ui.add_space(2.0);
        if project.tags.is_empty() {
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
        } else {
            let id = ui.make_persistent_id(&project.name);
            egui::collapsing_header::CollapsingState::load_with_default_open(ui.ctx(), id, true)
                .show_header(ui, |ui| {
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
                })
                .body(|ui| {
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
