use crate::database::Database;
use crate::excel_export;
use crate::theme;
use chrono::{Datelike, Local};

pub struct AccueilState {
    pub message: String,
    pub message_ok: bool,
}

impl AccueilState {
    pub fn new() -> Self {
        Self {
            message: String::new(),
            message_ok: true,
        }
    }
}

pub fn afficher(ui: &mut egui::Ui, db: &Database, state: &mut AccueilState) {
    let hauteur = ui.available_height();

    ui.vertical_centered(|ui| {
        ui.add_space((hauteur / 2.0 - 200.0).max(20.0));

        let now = Local::now();
        let jours_fr = [
            "Lundi", "Mardi", "Mercredi", "Jeudi", "Vendredi", "Samedi", "Dimanche",
        ];
        let mois_fr = [
            "janvier", "février", "mars", "avril", "mai", "juin", "juillet", "août",
            "septembre", "octobre", "novembre", "décembre",
        ];
        let jour_nom = jours_fr[now.weekday().num_days_from_monday() as usize];
        let mois_nom = mois_fr[now.month0() as usize];
        let date_str = format!("{} {} {}", now.day(), mois_nom, now.year());

        ui.label(
            egui::RichText::new(jour_nom)
                .size(28.0)
                .color(theme::PRIMAIRE)
                .strong(),
        );
        ui.add_space(4.0);

        ui.label(
            egui::RichText::new(now.format("%H:%M:%S").to_string())
                .size(64.0)
                .color(theme::ACCENT)
                .strong(),
        );
        ui.add_space(4.0);

        ui.label(
            egui::RichText::new(date_str)
                .size(18.0)
                .color(theme::TEXTE_SECONDAIRE),
        );

        ui.add_space(40.0);

        if theme::bouton_image_texte(ui, "exporter.png", "Exporter en Excel", 220.0, 44.0) {
            let t = Local::now().date_naive();
            match excel_export::exporter_tout(db, t, t) {
                Ok(chemin) => {
                    state.message = format!("Export réussi : {}", chemin);
                    state.message_ok = true;
                }
                Err(e) => {
                    state.message = format!("Erreur : {}", e);
                    state.message_ok = false;
                }
            }
        }

        if !state.message.is_empty() {
            ui.add_space(16.0);
            ui.vertical_centered(|ui| {
                ui.set_max_width(500.0);
                theme::message_inline(ui, &state.message, state.message_ok);
            });
        }
    });

    ui.ctx()
        .request_repaint_after(std::time::Duration::from_millis(500));
}
