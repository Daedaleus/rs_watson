use eframe::egui;

use crate::app::WatsonApp;
use crate::widgets::{feedback_label, project_autocomplete};

impl WatsonApp {
    pub(crate) fn show_add(&mut self, ui: &mut egui::Ui) {
        ui.add_space(12.0);
        let projects = self.projects.clone();

        egui::Grid::new("add_grid")
            .num_columns(2)
            .spacing([12.0, 10.0])
            .show(ui, |ui| {
                ui.label("Project");
                let add_proj_id = egui::Id::new("add_proj_popup");
                let add_proj_resp = ui.add(
                    egui::TextEdit::singleline(&mut self.add_project)
                        .hint_text("backend")
                        .desired_width(240.0),
                );
                project_autocomplete(
                    ui,
                    add_proj_id,
                    &add_proj_resp,
                    &mut self.add_project,
                    &projects,
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

        if let Some(msg) = &self.add_message.clone() {
            ui.add_space(6.0);
            feedback_label(ui, msg, self.add_message_is_error, false);
        }
    }
}
