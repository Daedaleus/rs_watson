use std::collections::BTreeMap;

use chrono::{Duration, Local, NaiveDate};
use eframe::egui;
use rs_watson::Frame;
use uuid::Uuid;

use crate::app::WatsonApp;
use crate::colors::{CLR_CYAN, CLR_PURPLE, CLR_RED};
use crate::format::{fmt_duration, fmt_time};
use crate::types::EditState;
use crate::widgets::{date_filter_bar, empty_frames};

impl WatsonApp {
    pub(crate) fn show_log(&mut self, ui: &mut egui::Ui) {
        ui.horizontal(|ui| {
            date_filter_bar(ui, &mut self.log_from, &mut self.log_to);
        });
        ui.separator();

        let visible: Vec<&Frame> =
            Self::filtered_frames(&self.frames, &self.log_from, &self.log_to);
        if visible.is_empty() {
            empty_frames(ui);
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

    pub(crate) fn show_edit_modal(&mut self, ctx: &egui::Context) {
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
