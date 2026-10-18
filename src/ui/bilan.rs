use crate::database::Database;
use crate::models::{EnregistrementPointage, Personnel};
use crate::theme;
use chrono::{Datelike, Local, NaiveDate, NaiveTime};
use std::collections::HashMap;

pub struct BilanState {
    pub mois_affiche: NaiveDate,
    pub personnel_selectionne: Option<i64>,
    pub date_detaillee: Option<NaiveDate>,
    pub cache_personnel: Option<Vec<Personnel>>,
    pub cache_personnel_id: Option<i64>,
    pub cache_mois: Option<NaiveDate>,
    pub cache_pointages: HashMap<NaiveDate, EnregistrementPointage>,
    pub cache_jours_pointes: Vec<u32>,
}

impl BilanState {
    pub fn new() -> Self {
        let t = Local::now().date_naive();
        Self {
            mois_affiche: NaiveDate::from_ymd_opt(t.year(), t.month(), 1).unwrap(),
            personnel_selectionne: None,
            date_detaillee: None,
            cache_personnel: None,
            cache_personnel_id: None,
            cache_mois: None,
            cache_pointages: HashMap::new(),
            cache_jours_pointes: Vec::new(),
        }
    }

    #[allow(dead_code)]
    pub fn invalider_cache(&mut self) {
        self.cache_personnel = None;
        self.cache_personnel_id = None;
        self.cache_mois = None;
        self.cache_pointages.clear();
        self.cache_jours_pointes.clear();
    }
}

pub fn afficher(ui: &mut egui::Ui, db: &Database, state: &mut BilanState) {
    ui.label(
        egui::RichText::new("Bilan")
            .size(22.0)
            .color(theme::PRIMAIRE)
            .strong(),
    );
    ui.add_space(16.0);

    if state.cache_personnel.is_none() {
        state.cache_personnel = Some(db.get_personnel().unwrap_or_default());
    }
    let personnel = state.cache_personnel.clone().unwrap_or_default();

    ui.columns(2, |cols| {
        theme::carte(&mut cols[0], |ui| {
            ui.set_min_height(500.0);
            ui.label(
                egui::RichText::new("Personnel")
                    .size(15.0)
                    .color(theme::PRIMAIRE)
                    .strong(),
            );
            ui.add_space(8.0);
            theme::separateur(ui);

            for p in &personnel {
                let sel = state.personnel_selectionne == Some(p.id);
                let txt = if sel {
                    egui::RichText::new(p.nom_complet())
                        .size(13.5)
                        .color(theme::ACCENT)
                        .strong()
                } else {
                    egui::RichText::new(p.nom_complet())
                        .size(13.5)
                        .color(theme::TEXTE)
                };
                if ui.selectable_label(sel, txt).clicked() {
                    if state.personnel_selectionne != Some(p.id) {
                        state.personnel_selectionne = Some(p.id);
                        state.date_detaillee = None;
                        state.cache_personnel_id = None;
                    }
                }
            }
        });

        theme::carte(&mut cols[1], |ui| {
            let p_sel = personnel
                .iter()
                .find(|p| Some(p.id) == state.personnel_selectionne)
                .cloned();

            let Some(p_sel) = p_sel else {
                ui.label(
                    egui::RichText::new("Sélectionnez un personnel à gauche")
                        .size(13.0)
                        .color(theme::TEXTE_SECONDAIRE),
                );
                return;
            };

            ui.label(
                egui::RichText::new(p_sel.nom_complet())
                    .size(16.0)
                    .color(theme::PRIMAIRE)
                    .strong(),
            );
            ui.add_space(10.0);

            ui.horizontal(|ui| {
                if theme::bouton_image_seule(ui, "annee_gauche.png", 40.0, 22.0) {
                    state.mois_affiche = NaiveDate::from_ymd_opt(
                        state.mois_affiche.year() - 1,
                        state.mois_affiche.month(),
                        1,
                    )
                    .unwrap();
                    state.date_detaillee = None;
                    state.cache_mois = None;
                }
                ui.add_space(4.0);
                if theme::bouton_image_seule(ui, "mois_gauche.png", 40.0, 22.0) {
                    state.mois_affiche = mois_prec(state.mois_affiche);
                    state.date_detaillee = None;
                    state.cache_mois = None;
                }
                ui.add_space(4.0);
                if theme::bouton_image_seule(ui, "mois_droite.png", 40.0, 22.0) {
                    state.mois_affiche = mois_suiv(state.mois_affiche);
                    state.date_detaillee = None;
                    state.cache_mois = None;
                }
                ui.add_space(4.0);
                if theme::bouton_image_seule(ui, "annee_droite.png", 40.0, 22.0) {
                    state.mois_affiche = NaiveDate::from_ymd_opt(
                        state.mois_affiche.year() + 1,
                        state.mois_affiche.month(),
                        1,
                    )
                    .unwrap();
                    state.date_detaillee = None;
                    state.cache_mois = None;
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
                    .size(14.0)
                    .color(theme::PRIMAIRE)
                    .strong(),
                );
            });

            ui.add_space(10.0);

            let cache_valide = state.cache_personnel_id == Some(p_sel.id)
                && state.cache_mois == Some(state.mois_affiche);

            if !cache_valide {
                let nb = jours_dans_mois(state.mois_affiche.year(), state.mois_affiche.month());
                let jours_pointes = db
                    .get_jours_pointes_du_mois(
                        p_sel.id,
                        state.mois_affiche.year(),
                        state.mois_affiche.month(),
                    )
                    .unwrap_or_default();
                let pointages = db
                    .get_pointages_periode(
                        NaiveDate::from_ymd_opt(
                            state.mois_affiche.year(),
                            state.mois_affiche.month(),
                            1,
                        )
                        .unwrap(),
                        NaiveDate::from_ymd_opt(
                            state.mois_affiche.year(),
                            state.mois_affiche.month(),
                            nb,
                        )
                        .unwrap(),
                    )
                    .unwrap_or_default();

                state.cache_pointages = pointages
                    .into_iter()
                    .filter(|p| p.personnel_id == p_sel.id)
                    .map(|p| (p.date_pointage, p))
                    .collect();
                state.cache_jours_pointes = jours_pointes.into_iter().collect();
                state.cache_jours_pointes.sort();
                state.cache_personnel_id = Some(p_sel.id);
                state.cache_mois = Some(state.mois_affiche);
            }

            let nb = jours_dans_mois(state.mois_affiche.year(), state.mois_affiche.month());
            let dict = &state.cache_pointages;
            let jours_pointes_set: std::collections::HashSet<u32> =
                state.cache_jours_pointes.iter().copied().collect();

            ui.label(
                egui::RichText::new(format!(
                    "Jours pointés ce mois : {} / {}",
                    state.cache_jours_pointes.len(),
                    nb
                ))
                .size(12.5)
                .color(theme::TEXTE_SECONDAIRE),
            );

            ui.add_space(10.0);

            let mut nouvelle_detaillee: Option<NaiveDate> = None;

            ui.horizontal_wrapped(|ui| {
                ui.spacing_mut().item_spacing = egui::vec2(6.0, 6.0);
                for j in 1..=nb {
                    let date = NaiveDate::from_ymd_opt(
                        state.mois_affiche.year(),
                        state.mois_affiche.month(),
                        j,
                    )
                    .unwrap();
                    let pointe = jours_pointes_set.contains(&j);
                    let detail = state.date_detaillee == Some(date);

                    let note = dict.get(&date).and_then(calculer_moyenne_jour_20);

                    let (fond, texte) = if detail {
                        (theme::ACCENT, egui::Color32::WHITE)
                    } else if !pointe {
                        (theme::FOND, theme::PRIMAIRE)
                    } else {
                        match note {
                            Some(n) if n >= 16.0 => (theme::JOUR_BIEN_BG, theme::SUCCES),
                            Some(n) if n >= 10.0 => (theme::JOUR_MOYEN_BG, theme::WARNING),
                            Some(_) => (theme::JOUR_MAUVAIS_BG, theme::DANGER),
                            None => (theme::FOND, theme::PRIMAIRE),
                        }
                    };

                    let (rect, resp) = ui
                        .allocate_exact_size(egui::vec2(58.0, 52.0), egui::Sense::click());
                    if ui.is_rect_visible(rect) {
                        ui.painter().rect_filled(rect, theme::r_sm(), fond);
                        ui.painter().text(
                            rect.center(),
                            egui::Align2::CENTER_CENTER,
                            j.to_string(),
                            egui::FontId::proportional(17.0),
                            texte,
                        );
                    }
                    if resp.clicked() {
                        nouvelle_detaillee = Some(date);
                    }
                }
            });

            if let Some(d) = nouvelle_detaillee {
                state.date_detaillee = Some(d);
            }

            ui.add_space(14.0);

            // Comptage par tranche de note /20
            let mut bien = 0;
            let mut moyen = 0;
            let mut mauvais = 0;
            let mut absent = 0;
            for j in 1..=nb {
                let date = NaiveDate::from_ymd_opt(
                    state.mois_affiche.year(),
                    state.mois_affiche.month(),
                    j,
                )
                .unwrap();
                match dict.get(&date).and_then(calculer_moyenne_jour_20) {
                    Some(n) if n >= 16.0 => bien += 1,
                    Some(n) if n >= 10.0 => moyen += 1,
                    Some(_) => mauvais += 1,
                    None => absent += 1,
                }
            }

            ui.add_space(8.0);
            graphique_barres(ui, bien, moyen, mauvais, absent);

            // NOTE : la moyenne du mois n'est plus affichée (supprimée sur demande).

            if let Some(d) = state.date_detaillee {
                ui.add_space(14.0);
                ui.label(
                    egui::RichText::new(d.format("%d/%m/%Y").to_string())
                        .size(13.0)
                        .color(theme::PRIMAIRE)
                        .strong(),
                );
                match dict.get(&d) {
                    None => {
                        ui.label("Aucun pointage enregistré pour cette date.");
                    }
                    Some(pt) => {
                        let note_m = calculer_note_matin_20(pt)
                            .map(|n| format!("{:.1}/20", n))
                            .unwrap_or_else(|| "- /20".into());
                        let note_a = calculer_note_apres_midi_20(pt)
                            .map(|n| format!("{:.1}/20", n))
                            .unwrap_or_else(|| "- /20".into());
                        let moy_j = calculer_moyenne_jour_20(pt)
                            .map(|n| format!("{:.1}/20", n))
                            .unwrap_or_else(|| "- /20".into());

                        ui.label(format!("Matin : {}", note_m));
                        ui.label(format!("Après-midi : {}", note_a));
                        ui.label(format!("La moyenne du jour : {}", moy_j));

                        if let Some(o) = &pt.observation {
                            if !o.is_empty() {
                                ui.label(format!("Observation : {}", o));
                            }
                        }
                    }
                }
            }
        });
    });
}

/// Graphique en barres : les barres dont la valeur est 0 sont masquées.
fn graphique_barres(ui: &mut egui::Ui, bien: i32, moyen: i32, mauvais: i32, absent: i32) {
    let total = (bien + moyen + mauvais + absent).max(1);
    let entrees = [
        (bien, theme::SUCCES),
        (moyen, theme::WARNING),
        (mauvais, theme::DANGER),
        (absent, theme::TEXTE_SECONDAIRE),
    ];
    for (valeur, couleur) in entrees {
        if valeur == 0 {
            continue;
        }
        ui.horizontal(|ui| {
            let largeur_max = (ui.available_width() - 40.0).max(100.0);
            let ratio = valeur as f32 / total as f32;
            let (rect, _) =
                ui.allocate_exact_size(egui::vec2(largeur_max, 20.0), egui::Sense::hover());
            let painter = ui.painter();
            painter.rect_filled(rect, 3.0, egui::Color32::from_rgb(240, 242, 245));
            let barre =
                egui::Rect::from_min_size(rect.min, egui::vec2(largeur_max * ratio, 20.0));
            painter.rect_filled(barre, 3.0, couleur);
            painter.text(
                egui::pos2(rect.right() + 6.0, rect.center().y),
                egui::Align2::LEFT_CENTER,
                valeur.to_string(),
                egui::FontId::proportional(12.0),
                theme::TEXTE,
            );
        });
    }
}

fn calculer_note_matin_20(pt: &EnregistrementPointage) -> Option<f64> {
    let t = pt.matin_arrivee?;
    let limite = NaiveTime::from_hms_opt(8, 0, 0).unwrap();
    if t <= limite {
        Some(20.0)
    } else {
        let retard = (t - limite).num_minutes();
        if retard <= 30 {
            Some((20.0 - (retard as f64 * 10.0 / 30.0)).max(10.0))
        } else {
            Some((10.0 - ((retard - 30) as f64 * 10.0 / 60.0)).max(0.0))
        }
    }
}

fn calculer_note_apres_midi_20(pt: &EnregistrementPointage) -> Option<f64> {
    let t = pt.apres_midi_arrivee?;
    let limite = NaiveTime::from_hms_opt(14, 0, 0).unwrap();
    if t <= limite {
        Some(20.0)
    } else {
        let retard = (t - limite).num_minutes();
        if retard <= 30 {
            Some((20.0 - (retard as f64 * 10.0 / 30.0)).max(10.0))
        } else {
            Some((10.0 - ((retard - 30) as f64 * 10.0 / 60.0)).max(0.0))
        }
    }
}

/// Moyenne du jour :
/// - Si les DEUX sessions sont pointées → moyenne des deux.
/// - Si UNE SEULE est pointée → la note du jour EST celle de cette session
///   (pas de division par 2, pas de 0 artificiel).
/// - Si aucune → None.
fn calculer_moyenne_jour_20(pt: &EnregistrementPointage) -> Option<f64> {
    let m = calculer_note_matin_20(pt);
    let a = calculer_note_apres_midi_20(pt);
    match (m, a) {
        (Some(n1), Some(n2)) => Some((n1 + n2) / 2.0),
        (Some(n1), None) => Some(n1),
        (None, Some(n2)) => Some(n2),
        (None, None) => None,
    }
}

fn mois_prec(d: NaiveDate) -> NaiveDate {
    let (y, m) = if d.month() == 1 {
        (d.year() - 1, 12)
    } else {
        (d.year(), d.month() - 1)
    };
    NaiveDate::from_ymd_opt(y, m, 1).unwrap()
}

fn mois_suiv(d: NaiveDate) -> NaiveDate {
    let (y, m) = if d.month() == 12 {
        (d.year() + 1, 1)
    } else {
        (d.year(), d.month() + 1)
    };
    NaiveDate::from_ymd_opt(y, m, 1).unwrap()
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
