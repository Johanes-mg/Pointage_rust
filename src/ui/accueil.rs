use crate::database::Database;
use crate::excel_export;
use crate::theme;
use chrono::{Datelike, Local, NaiveDate};
use std::time::{Duration, Instant};

const DUREE_MESSAGE: Duration = Duration::from_secs(8);
const CASE_LARGEUR: f32 = 46.0;
const CASE_HAUTEUR: f32 = 46.0;

pub struct AccueilState {
    pub message: String,
    pub message_ok: bool,
    pub message_expire: Option<Instant>,
    pub mois_affiche: NaiveDate,
    pub date_debut: Option<NaiveDate>,
    pub date_fin: Option<NaiveDate>,
}

impl AccueilState {
    pub fn new() -> Self {
        let t = Local::now().date_naive();
        Self {
            message: String::new(),
            message_ok: true,
            message_expire: None,
            mois_affiche: NaiveDate::from_ymd_opt(t.year(), t.month(), 1).unwrap(),
            date_debut: None,
            date_fin: None,
        }
    }

    fn afficher_message(&mut self, texte: String, ok: bool) {
        self.message = texte;
        self.message_ok = ok;
        self.message_expire = Some(Instant::now() + DUREE_MESSAGE);
    }

    fn message_visible(&self) -> bool {
        if let Some(exp) = self.message_expire {
            Instant::now() < exp
        } else {
            false
        }
    }

    fn purger_message_si_expire(&mut self) {
        if let Some(exp) = self.message_expire {
            if Instant::now() >= exp {
                self.message.clear();
                self.message_expire = None;
            }
        }
    }
}

pub fn afficher(ui: &mut egui::Ui, db: &Database, state: &mut AccueilState) {
    state.purger_message_si_expire();

    let dispo = ui.available_width();
    let ecart = 16.0_f32;
    let seuil = 720.0_f32;

    if dispo < seuil {
        calendrier_gauche(ui, state);
        ui.add_space(ecart);
        panneau_droit(ui, db, state);
    } else {
        let largeur_gauche = (dispo * 0.42).clamp(340.0, dispo - 380.0);
        let largeur_droite = dispo - largeur_gauche - ecart;

        ui.horizontal_top(|ui| {
            ui.allocate_ui_with_layout(
                egui::vec2(largeur_gauche, 0.0),
                egui::Layout::top_down(egui::Align::LEFT),
                |ui| {
                    calendrier_gauche(ui, state);
                },
            );

            ui.add_space(ecart);

            ui.allocate_ui_with_layout(
                egui::vec2(largeur_droite, 0.0),
                egui::Layout::top_down(egui::Align::LEFT),
                |ui| {
                    panneau_droit(ui, db, state);
                },
            );
        });
    }

    ui.ctx().request_repaint_after(Duration::from_millis(500));
}

fn calendrier_gauche(ui: &mut egui::Ui, state: &mut AccueilState) {
    theme::carte(ui, |ui| {
        ui.set_width(ui.available_width());

        ui.horizontal_wrapped(|ui| {
            if theme::bouton_image_seule(ui, "annee_gauche.png", 40.0, 22.0) {
                state.mois_affiche = NaiveDate::from_ymd_opt(
                    state.mois_affiche.year() - 1,
                    state.mois_affiche.month(),
                    1,
                )
                .unwrap();
            }
            ui.add_space(4.0);
            if theme::bouton_image_seule(ui, "mois_gauche.png", 40.0, 22.0) {
                let (y, m) = mois_precedent(state.mois_affiche);
                state.mois_affiche = NaiveDate::from_ymd_opt(y, m, 1).unwrap();
            }
            ui.add_space(4.0);
            if theme::bouton_image_seule(ui, "mois_droite.png", 40.0, 22.0) {
                let (y, m) = mois_suivant(state.mois_affiche);
                state.mois_affiche = NaiveDate::from_ymd_opt(y, m, 1).unwrap();
            }
            ui.add_space(4.0);
            if theme::bouton_image_seule(ui, "annee_droite.png", 40.0, 22.0) {
                state.mois_affiche = NaiveDate::from_ymd_opt(
                    state.mois_affiche.year() + 1,
                    state.mois_affiche.month(),
                    1,
                )
                .unwrap();
            }

            ui.add_space(10.0);

            let mois_fr = [
                "Janvier", "Février", "Mars", "Avril", "Mai", "Juin", "Juillet", "Août",
                "Septembre", "Octobre", "Novembre", "Décembre",
            ];
            ui.label(
                egui::RichText::new(format!(
                    "{} {}",
                    mois_fr[state.mois_affiche.month0() as usize],
                    state.mois_affiche.year()
                ))
                .size(15.0)
                .color(theme::PRIMAIRE)
                .strong(),
            );
        });

        ui.add_space(8.0);

        if theme::bouton_secondaire(ui, "Aujourd'hui", 140.0) {
            let t = Local::now().date_naive();
            state.mois_affiche = NaiveDate::from_ymd_opt(t.year(), t.month(), 1).unwrap();
        }

        ui.add_space(10.0);
        theme::separateur(ui);
        ui.add_space(10.0);

        let jours_sem = ["L", "M", "M", "J", "V", "S", "D"];
        ui.horizontal(|ui| {
            for j in jours_sem.iter() {
                ui.add_sized(
                    [CASE_LARGEUR, 20.0],
                    egui::Label::new(
                        egui::RichText::new(*j)
                            .size(12.0)
                            .color(theme::TEXTE_SECONDAIRE)
                            .strong(),
                    ),
                );
            }
        });
        ui.add_space(4.0);

        let nb_jours = jours_dans_mois(state.mois_affiche.year(), state.mois_affiche.month());

        let premier = NaiveDate::from_ymd_opt(
            state.mois_affiche.year(),
            state.mois_affiche.month(),
            1,
        )
        .unwrap();
        let jour_semaine_premier = premier.weekday().num_days_from_monday();

        let nb_colonnes = 7;
        let aujourdhui = Local::now().date_naive();

        let mut jour_courant = 1;
        let mut ligne = 0;
        let mut nouvelle_date: Option<NaiveDate> = None;

        while jour_courant <= nb_jours {
            ui.horizontal(|ui| {
                for col in 0..nb_colonnes {
                    if ligne == 0 && col < jour_semaine_premier as usize {
                        ui.add_sized([CASE_LARGEUR, CASE_HAUTEUR], egui::Label::new(""));
                        continue;
                    }

                    if jour_courant > nb_jours {
                        ui.add_sized([CASE_LARGEUR, CASE_HAUTEUR], egui::Label::new(""));
                        continue;
                    }

                    let jour = jour_courant;
                    jour_courant += 1;

                    let date = NaiveDate::from_ymd_opt(
                        state.mois_affiche.year(),
                        state.mois_affiche.month(),
                        jour,
                    )
                    .unwrap();

                    let est_aujourdhui = date == aujourdhui;
                    let est_debut = state.date_debut == Some(date);
                    let est_fin = state.date_fin == Some(date);
                    let dans_intervalle = match (state.date_debut, state.date_fin) {
                        (Some(d), Some(f)) => date > d && date < f,
                        _ => false,
                    };

                    let (fond, texte, bordure) = if est_debut {
                        (theme::ACCENT, egui::Color32::WHITE, theme::PRIMAIRE)
                    } else if est_fin {
                        (theme::SUCCES, egui::Color32::WHITE, theme::SUCCES)
                    } else if dans_intervalle {
                        (theme::SELECTION_JOUR, theme::ACCENT, theme::ACCENT)
                    } else if est_aujourdhui {
                        (theme::SELECTION_JOUR, theme::ACCENT, theme::ACCENT)
                    } else {
                        (theme::FOND, theme::PRIMAIRE, theme::BORDURE)
                    };

                    let (rect, resp) = ui
                        .allocate_exact_size(egui::vec2(CASE_LARGEUR, CASE_HAUTEUR), egui::Sense::click());

                    if ui.is_rect_visible(rect) {
                        let painter = ui.painter();
                        painter.rect_filled(rect, theme::r_sm(), fond);
                        painter.rect_stroke(rect, theme::r_sm(), egui::Stroke::new(1.0_f32, bordure));
                        painter.text(
                            rect.center(),
                            egui::Align2::CENTER_CENTER,
                            jour.to_string(),
                            egui::FontId::proportional(15.0),
                            texte,
                        );
                    }

                    if resp.clicked() {
                        nouvelle_date = Some(date);
                    }
                    if resp.hovered() {
                        ui.ctx().set_cursor_icon(egui::CursorIcon::PointingHand);
                    }
                }
            });
            ligne += 1;
        }

        if let Some(d) = nouvelle_date {
            appliquer_selection(state, d);
        }
    });
}

fn appliquer_selection(state: &mut AccueilState, d: NaiveDate) {
    match (state.date_debut, state.date_fin) {
        (None, _) => state.date_debut = Some(d),
        (Some(debut), None) => {
            if d < debut {
                state.date_fin = Some(debut);
                state.date_debut = Some(d);
            } else {
                state.date_fin = Some(d);
            }
        }
        (Some(_), Some(_)) => {
            state.date_debut = Some(d);
            state.date_fin = None;
        }
    }
}

fn panneau_droit(ui: &mut egui::Ui, db: &Database, state: &mut AccueilState) {
    theme::carte(ui, |ui| {
        ui.set_width(ui.available_width());

        section_horloge(ui);

        ui.add_space(24.0);
        theme::separateur(ui);
        ui.add_space(24.0);

        section_export_rapide(ui, db, state);

        ui.add_space(24.0);
        theme::separateur(ui);
        ui.add_space(24.0);

        section_export_plage(ui, db, state);

        if state.message_visible() && !state.message.is_empty() {
            ui.add_space(20.0);
            section_message(ui, &state.message, state.message_ok);
        }
    });
}

fn section_horloge(ui: &mut egui::Ui) {
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

    ui.vertical_centered(|ui| {
        ui.label(
            egui::RichText::new(jour_nom.to_uppercase())
                .size(13.0)
                .color(theme::TEXTE_SECONDAIRE)
                .strong()
                .extra_letter_spacing(1.5),
        );
        ui.add_space(8.0);
        ui.label(
            egui::RichText::new(now.format("%H:%M:%S").to_string())
                .size(56.0)
                .color(theme::ACCENT)
                .strong(),
        );
        ui.add_space(4.0);
        ui.label(
            egui::RichText::new(date_str)
                .size(15.0)
                .color(theme::TEXTE_SECONDAIRE),
        );
    });
}

fn section_export_rapide(ui: &mut egui::Ui, db: &Database, state: &mut AccueilState) {
    sous_titre(ui, "Export rapide");
    ui.add_space(12.0);

    let largeur = ui.available_width();

    if theme::bouton_image_texte(ui, "exporter.png", "Exporter aujourd'hui", largeur, 44.0) {
        let t = Local::now().date_naive();
        match excel_export::exporter_tout(db, t, t) {
            Ok(chemin) => {
                state.afficher_message(format!("Export réussi : {}", chemin), true);
            }
            Err(e) => {
                state.afficher_message(format!("Erreur : {}", e), false);
            }
        }
    }
}

fn section_export_plage(ui: &mut egui::Ui, db: &Database, state: &mut AccueilState) {
    sous_titre(ui, "Export par plage");
    ui.add_space(16.0);

    ui.horizontal_wrapped(|ui| {
        champ_date(
            ui,
            "DU",
            state.date_debut,
            theme::SELECTION_JOUR,
            theme::PRIMAIRE,
        );

        ui.add_space(10.0);
        ui.label(
            egui::RichText::new("-")
                .size(16.0)
                .color(theme::TEXTE_SECONDAIRE),
        );
        ui.add_space(10.0);

        champ_date(
            ui,
            "AU",
            state.date_fin,
            theme::JOUR_BIEN_BG,
            theme::SUCCES,
        );

        if state.date_debut.is_some() || state.date_fin.is_some() {
            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                if theme::bouton_image_seule(ui, "effacer.png", 32.0, 16.0) {
                    state.date_debut = None;
                    state.date_fin = None;
                }
            });
        }
    });

    ui.add_space(16.0);

    let pret = matches!((state.date_debut, state.date_fin), (Some(_), Some(_)));
    let largeur = ui.available_width();

    let clique;
    if pret {
        clique = theme::bouton_image_texte(
            ui,
            "exporter.png",
            "Exporter la plage",
            largeur,
            44.0,
        );
    } else {
        bouton_desactive(ui, "exporter.png", "Exporter la plage", largeur, 44.0);
        clique = false;
    }

    if clique {
        if let (Some(d), Some(f)) = (state.date_debut, state.date_fin) {
            match excel_export::exporter_tout(db, d, f) {
                Ok(chemin) => {
                    state.afficher_message(format!("Export réussi : {}", chemin), true);
                }
                Err(e) => {
                    state.afficher_message(format!("Erreur : {}", e), false);
                }
            }
        }
    }
}

fn section_message(ui: &mut egui::Ui, texte: &str, ok: bool) {
    let (fond, couleur) = if ok {
        (egui::Color32::from_rgb(209, 250, 229), theme::SUCCES)
    } else {
        (egui::Color32::from_rgb(254, 226, 226), theme::DANGER)
    };

    egui::Frame::none()
        .fill(fond)
        .rounding(theme::r_sm())
        .inner_margin(egui::Margin::symmetric(12.0, 10.0))
        .show(ui, |ui| {
            ui.set_width(ui.available_width());
            ui.label(egui::RichText::new(texte).size(12.5).color(couleur));
        });
}

fn sous_titre(ui: &mut egui::Ui, texte: &str) {
    ui.label(
        egui::RichText::new(texte)
            .size(14.5)
            .color(theme::PRIMAIRE)
            .strong(),
    );
}

fn champ_date(
    ui: &mut egui::Ui,
    label: &str,
    date: Option<NaiveDate>,
    fond: egui::Color32,
    couleur_texte: egui::Color32,
) {
    ui.vertical(|ui| {
        ui.label(
            egui::RichText::new(label)
                .size(10.5)
                .color(theme::TEXTE_SECONDAIRE)
                .strong()
                .extra_letter_spacing(1.0),
        );
        ui.add_space(4.0);

        match date {
            Some(d) => {
                egui::Frame::none()
                    .fill(fond)
                    .rounding(theme::r_sm())
                    .inner_margin(egui::Margin::symmetric(12.0, 8.0))
                    .show(ui, |ui| {
                        ui.label(
                            egui::RichText::new(d.format("%d/%m/%Y").to_string())
                                .size(13.0)
                                .color(couleur_texte)
                                .strong(),
                        );
                    });
            }
            None => {
                egui::Frame::none()
                    .fill(theme::FOND)
                    .rounding(theme::r_sm())
                    .stroke(egui::Stroke::new(1.0_f32, theme::BORDURE))
                    .inner_margin(egui::Margin::symmetric(12.0, 8.0))
                    .show(ui, |ui| {
                        ui.label(
                            egui::RichText::new("-")
                                .size(13.0)
                                .color(theme::TEXTE_SECONDAIRE),
                        );
                    });
            }
        }
    });
}

fn bouton_desactive(
    ui: &mut egui::Ui,
    fichier: &str,
    texte: &str,
    largeur: f32,
    hauteur: f32,
) {
    let ctx = ui.ctx().clone();
    let (rect, _) = ui.allocate_exact_size(egui::vec2(largeur, hauteur), egui::Sense::hover());

    let painter = ui.painter();
    painter.rect_filled(rect, theme::r_md(), theme::FOND);
    painter.rect_stroke(rect, theme::r_md(), egui::Stroke::new(1.0_f32, theme::BORDURE));

    let mut x = rect.left() + 14.0;
    if let Some(tex) = theme::image(&ctx, fichier) {
        let img_size = egui::vec2(20.0, 20.0);
        let img_rect = egui::Rect::from_min_size(
            egui::pos2(x, rect.center().y - img_size.y / 2.0),
            img_size,
        );
        painter.image(
            tex.id(),
            img_rect,
            egui::Rect::from_min_max(egui::pos2(0.0, 0.0), egui::pos2(1.0, 1.0)),
            egui::Color32::from_white_alpha(100),
        );
        x += img_size.x + 10.0;
    }

    painter.text(
        egui::pos2(x, rect.center().y),
        egui::Align2::LEFT_CENTER,
        texte,
        egui::FontId::proportional(13.0),
        theme::TEXTE_SECONDAIRE,
    );
}

fn mois_precedent(d: NaiveDate) -> (i32, u32) {
    if d.month() == 1 {
        (d.year() - 1, 12)
    } else {
        (d.year(), d.month() - 1)
    }
}

fn mois_suivant(d: NaiveDate) -> (i32, u32) {
    if d.month() == 12 {
        (d.year() + 1, 1)
    } else {
        (d.year(), d.month() + 1)
    }
}

fn jours_dans_mois(annee: i32, mois: u32) -> u32 {
    match mois {
        1 | 3 | 5 | 7 | 8 | 10 | 12 => 31,
        4 | 6 | 9 | 11 => 30,
        2 => {
            if (annee % 4 == 0 && annee % 100 != 0) || annee % 400 == 0 {
                29
            } else {
                28
            }
        }
        _ => 30,
    }
}
