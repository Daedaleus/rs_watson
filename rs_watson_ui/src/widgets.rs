use eframe::egui;

use crate::colors::{CLR_GREEN, CLR_RED};

/// Project autocomplete popup — call after adding the `TextEdit` for `input`.
pub(crate) fn project_autocomplete(
    ui: &mut egui::Ui,
    popup_id: egui::Id,
    text_resp: &egui::Response,
    input: &mut String,
    projects: &[String],
) {
    if text_resp.gained_focus() && !projects.is_empty() {
        #[allow(deprecated)]
        ui.memory_mut(|m| m.open_popup(popup_id));
    }
    #[allow(deprecated)]
    egui::popup_below_widget(
        ui,
        popup_id,
        text_resp,
        egui::PopupCloseBehavior::CloseOnClickOutside,
        |ui| {
            ui.set_min_width(text_resp.rect.width());
            let filter = input.to_lowercase();
            for proj in projects
                .iter()
                .filter(|p| filter.is_empty() || p.to_lowercase().contains(&filter))
            {
                if ui.selectable_label(false, proj.as_str()).clicked() {
                    *input = proj.clone();
                    ui.memory_mut(|m| m.close_popup(popup_id));
                }
            }
        },
    );
}

/// "From … To … Clear" filter row — call inside a `ui.horizontal` closure.
pub(crate) fn date_filter_bar(ui: &mut egui::Ui, from: &mut String, to: &mut String) {
    ui.label("From");
    ui.add(
        egui::TextEdit::singleline(from)
            .hint_text("YYYY-MM-DD")
            .desired_width(100.0),
    );
    ui.label("To");
    ui.add(
        egui::TextEdit::singleline(to)
            .hint_text("YYYY-MM-DD")
            .desired_width(100.0),
    );
    if ui.small_button("Clear").clicked() {
        from.clear();
        to.clear();
    }
}

/// Centered "No frames." placeholder.
pub(crate) fn empty_frames(ui: &mut egui::Ui) {
    ui.centered_and_justified(|ui| {
        ui.label(egui::RichText::new("No frames.").color(egui::Color32::GRAY));
    });
}

/// One-line green/red feedback label.
pub(crate) fn feedback_label(ui: &mut egui::Ui, msg: &str, is_error: bool, small: bool) {
    let color = if is_error { CLR_RED } else { CLR_GREEN };
    let rt = egui::RichText::new(msg).color(color);
    ui.label(if small { rt.small() } else { rt });
}
