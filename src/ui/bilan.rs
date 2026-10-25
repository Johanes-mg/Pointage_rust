use crate::database::Database;
use crate::models::{EnregistrementPointage, Personnel};
use crate::theme;
use chrono::{Datelike, Local, NaiveDate, NaiveTime, Weekday};
use std::collections::{HashMap, HashSet};

const COL_RANG: f32 = 50.0;
const COL_NOTE: f32 = 140.0;
const COL_JOURS: f32 = 100.0;
const COL_GAP: f32 = 16.0;
const HAUTEUR_LIGNE: f32 = 32.0;
const MARGE_LATERALE: f32 = 12.0;
const HAUTEUR_LIGNE_PERSONNEL: f32 = 24.0;
const TAILLE_CASE_JOUR: f32 = 58.0;
const HAUTEUR_CASE_JOUR: f32 = 52.0;

#[derive(Clone, Debug, Default)]
pub struct ClassementEntry {
    pub personnel_id: i64,
    pub nom: String,
    pub note_moyenne: f64,
    pub nb_jours_pointes: usize,
}

struct Colonnes {
    x_rang: f32,
    x_nom: f32,
    x_note: f32,
    x_jours: f32,
    l_nom: f32,
}

impl Colonnes {
    fn calculer(largeur_totale: f32) -> Self {
        let x_rang = MARGE_LATERALE;
        let x_nom = x_rang + COL_RANG + COL_GAP;
        let x_jours = largeur_totale - MARGE_LATERALE - COL_JOURS;
        let x_note = x_jours - COL_GAP - COL_NOTE;
        let l_nom = (x_note - COL_GAP - x_nom).max(80.0);
        Self {
            x_rang,
            x_nom,
            x_note,
            x_jours,
            l_nom,
        }
    }
}

pub struct BilanState {
    pub mois_affiche: NaiveDate,
    pub personnel_selectionne: Option<i64>,
    pub date_detaillee: Option<NaiveDate>,
    pub cache_personnel: Option<Vec<Personnel>>,
    pub cache_personnel_id: Option<i64>,
    pub cache_mois: Option<NaiveDate>,
    pub cache_pointages: HashMap<NaiveDate, EnregistrementPointage>,
    pub cache_jours_pointes: Vec<u32>,
    pub cache_feries: HashMap<u32, Vec<String>>,
    pub cache_classement: Option<Vec<ClassementEntry>>,
    pub recherche: String,
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
            cache_feries: HashMap::new(),
            cache_classement: None,
            recherche: String::new(),
        }
    }

    pub fn invalider_cache(&mut self) {
        self.cache_personnel = None;
        self.cache_personnel_id = None;
        self.cache_mois = None;
        self.cache_pointages.clear();
        self.cache_jours_pointes.clear();
        self.cache_feries.clear();
        self.cache_classement = None;
    }

    fn invalider_mois(&mut self) {
        self.cache_personnel_id = None;
        self.cache_mois = None;
        self.cache_pointages.clear();
        self.cache_jours_pointes.clear();
        self.cache_feries.clear();
        self.cache_classement = None;
        self.date_detaillee = None;
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

    let hauteur_dispo = ui.available_height().max(400.0);
    let dispo = ui.available_width();
    let ecart = 16.0_f32;
    let seuil = 860.0_f32;

    if dispo < seuil {
        theme::carte(ui, |ui| {
            ui.set_min_height(hauteur_dispo / 2.0);
            colonne_gauche(ui, &personnel, state, hauteur_dispo / 2.0);
        });
        ui.add_space(16.0);
        theme::carte(ui, |ui| {
            ui.set_min_height(hauteur_dispo / 2.0);
            colonne_droite(ui, db, &personnel, state, hauteur_dispo / 2.0);
        });
    } else {
        let largeur_gauche = (dispo * 0.32).max(280.0);
        let largeur_droite = dispo - largeur_gauche - ecart;

        ui.horizontal_top(|ui| {
            ui.allocate_ui_with_layout(
                egui::vec2(largeur_gauche, hauteur_dispo),
                egui::Layout::top_down(egui::Align::LEFT),
                |ui| {
                    theme::carte(ui, |ui| {
                        colonne_gauche(ui, &personnel, state, hauteur_dispo);
                    });
                },
            );

            ui.add_space(ecart);

            ui.allocate_ui_with_layout(
                egui::vec2(largeur_droite, hauteur_dispo),
                egui::Layout::top_down(egui::Align::LEFT),
                |ui| {
                    theme::carte(ui, |ui| {
                        colonne_droite(ui, db, &personnel, state, hauteur_dispo);
                    });
                },
            );
        });
    }
}

fn colonne_gauche(
    ui: &mut egui::Ui,
    personnel: &[Personnel],
    state: &mut BilanState,
    hauteur: f32,
) {
    ui.set_min_height(hauteur);
    ui.set_max_height(hauteur);

    ui.label(
        egui::RichText::new("Personnel")
            .size(15.0)
            .color(theme::PRIMAIRE)
            .strong(),
    );
    ui.add_space(8.0);

    let largeur_recherche = ui.available_width();
    let recherche_changee = theme::champ_recherche(ui, &mut state.recherche, largeur_recherche);

    ui.add_space(8.0);
    theme::separateur(ui);
    ui.add_space(4.0);

    let filtre = state.recherche.to_lowercase();
    let liste: Vec<&Personnel> = personnel
        .iter()
        .filter(|p| {
            if filtre.trim().is_empty() {
                true
            } else {
                let haystack = format!(
                    "{} {} {}",
                    p.nom,
                    p.prenom,
                    p.matricule.clone().unwrap_or_default()
                )
                .to_lowercase();
                haystack.contains(&filtre)
            }
        })
        .collect();

    if recherche_changee {
        if let Some(id) = state.personnel_selectionne {
            let present = liste.iter().any(|p| p.id == id);
            if !present {
                state.personnel_selectionne = None;
                state.date_detaillee = None;
                state.cache_personnel_id = None;
            }
        }
    }

    if liste.is_empty() {
        ui.vertical_centered(|ui| {
            ui.add_space(30.0);
            ui.label(
                egui::RichText::new(if personnel.is_empty() {
                    "Aucun personnel enregistré"
                } else {
                    "Aucun résultat"
                })
                .size(13.0)
                .color(theme::TEXTE_SECONDAIRE),
            );
            ui.add_space(30.0);
        });
        return;
    }

    let hauteur_liste = (hauteur - 110.0).max(120.0);

    egui::ScrollArea::vertical()
        .id_source("bilan_personnel_scroll")
        .max_height(hauteur_liste)
        .auto_shrink([false, false])
        .show_rows(
            ui,
            HAUTEUR_LIGNE_PERSONNEL,
            liste.len(),
            |ui, range| {
                for i in range {
                    let p = liste[i];
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
                        if sel {
                            state.personnel_selectionne = None;
                            state.date_detaillee = None;
                            state.cache_personnel_id = None;
                        } else {
                            state.personnel_selectionne = Some(p.id);
                            state.date_detaillee = None;
                            state.cache_personnel_id = None;
                        }
                    }
                }
            },
        );
}

fn colonne_droite(
    ui: &mut egui::Ui,
    db: &Database,
    personnel: &[Personnel],
    state: &mut BilanState,
    hauteur: f32,
) {
    ui.set_min_height(hauteur);
    ui.set_max_height(hauteur);

    egui::ScrollArea::vertical()
        .id_source("bilan_droite_scroll")
        .max_height(hauteur)
        .auto_shrink([false, false])
        .show(ui, |ui| match state.personnel_selectionne {
            None => afficher_classement(ui, db, personnel, state),
            Some(id) => {
                if let Some(p) = personnel.iter().find(|p| p.id == id).cloned() {
                    afficher_bilan_perso(ui, db, &p, state);
                } else {
                    state.personnel_selectionne = None;
                    afficher_classement(ui, db, personnel, state);
                }
            }
        });
}

fn afficher_classement(
    ui: &mut egui::Ui,
    db: &Database,
    personnel: &[Personnel],
    state: &mut BilanState,
) {
    ui.label(
        egui::RichText::new("Classement du mois")
            .size(16.0)
            .color(theme::PRIMAIRE)
            .strong(),
    );
    ui.add_space(10.0);

    barre_mois(ui, state);

    ui.add_space(12.0);
    theme::separateur(ui);
    ui.add_space(10.0);

    if state.cache_classement.is_none() {
        let annee = state.mois_affiche.year();
        let mois = state.mois_affiche.month();
        let feries = db.get_jours_feries_du_mois(annee, mois).unwrap_or_default();
        state.cache_feries = feries.clone();
        state.cache_classement = Some(calculer_classement(db, personnel, annee, mois, &feries));
    }

    let classement = state.cache_classement.clone().unwrap_or_default();

    if classement.is_empty() {
        ui.vertical_centered(|ui| {
            ui.add_space(40.0);
            ui.label(
                egui::RichText::new("Aucun personnel enregistré")
                    .size(13.0)
                    .color(theme::TEXTE_SECONDAIRE),
            );
            ui.add_space(40.0);
        });
        return;
    }

    let largeur_totale = ui.available_width();
    let cols = Colonnes::calculer(largeur_totale);

    entete_classement(ui, &cols);

    ui.add_space(4.0);

    let mut clic: Option<i64> = None;

    egui::ScrollArea::vertical()
        .id_source("bilan_classement_scroll")
        .auto_shrink([false, false])
        .show_rows(
            ui,
            HAUTEUR_LIGNE,
            classement.len(),
            |ui, range| {
                for i in range {
                    let e = &classement[i];
                    let row_bg = if i % 2 == 0 {
                        theme::FOND_CARTE
                    } else {
                        theme::FOND
                    };

                    let (rect_range, reponse_range) = ui.allocate_exact_size(
                        egui::vec2(largeur_totale, HAUTEUR_LIGNE),
                        egui::Sense::click(),
                    );

                    if ui.is_rect_visible(rect_range) {
                        let painter = ui.painter();
                        painter.rect_filled(rect_range, theme::r_sm(), row_bg);

                        let (note_txt, note_bg, note_fg) = badge_note(e.note_moyenne);

                        dessiner_cellule_texte(
                            painter,
                            rect_range,
                            cols.x_rang,
                            COL_RANG,
                            &format!("{}", i + 1),
                            egui::Align2::LEFT_CENTER,
                            egui::FontId::proportional(12.0),
                            theme::TEXTE_SECONDAIRE,
                        );

                        dessiner_cellule_texte(
                            painter,
                            rect_range,
                            cols.x_nom,
                            cols.l_nom,
                            &e.nom,
                            egui::Align2::LEFT_CENTER,
                            egui::FontId::proportional(13.0),
                            theme::TEXTE,
                        );

                        dessiner_cellule_note(
                            painter,
                            rect_range,
                            cols.x_note,
                            COL_NOTE,
                            &note_txt,
                            note_bg,
                            note_fg,
                        );

                        dessiner_cellule_texte(
                            painter,
                            rect_range,
                            cols.x_jours,
                            COL_JOURS,
                            &format!("{} j", e.nb_jours_pointes),
                            egui::Align2::RIGHT_CENTER,
                            egui::FontId::proportional(11.5),
                            theme::TEXTE_SECONDAIRE,
                        );
                    }

                    if reponse_range.hovered() {
                        ui.ctx().set_cursor_icon(egui::CursorIcon::PointingHand);
                    }
                    if reponse_range.clicked() {
                        clic = Some(e.personnel_id);
                    }

                    ui.add_space(2.0);
                }
            },
        );

    if let Some(id) = clic {
        state.personnel_selectionne = Some(id);
        state.date_detaillee = None;
        state.cache_personnel_id = None;
    }
}

fn entete_classement(ui: &mut egui::Ui, cols: &Colonnes) {
    let largeur_totale = ui.available_width();
    let (rect, _) =
        ui.allocate_exact_size(egui::vec2(largeur_totale, HAUTEUR_LIGNE), egui::Sense::hover());

    if ui.is_rect_visible(rect) {
        let painter = ui.painter();
        painter.rect_filled(rect, theme::r_sm(), theme::FOND);

        dessiner_cellule_texte(
            painter,
            rect,
            cols.x_rang,
            COL_RANG,
            "#",
            egui::Align2::LEFT_CENTER,
            egui::FontId::proportional(11.0),
            theme::TEXTE_SECONDAIRE,
        );

        dessiner_cellule_texte(
            painter,
            rect,
            cols.x_nom,
            cols.l_nom,
            "PERSONNE",
            egui::Align2::LEFT_CENTER,
            egui::FontId::proportional(11.0),
            theme::TEXTE_SECONDAIRE,
        );

        dessiner_cellule_texte(
            painter,
            rect,
            cols.x_note,
            COL_NOTE,
            "NOTE MOYENNE",
            egui::Align2::RIGHT_CENTER,
            egui::FontId::proportional(11.0),
            theme::TEXTE_SECONDAIRE,
        );

        dessiner_cellule_texte(
            painter,
            rect,
            cols.x_jours,
            COL_JOURS,
            "JOURS",
            egui::Align2::RIGHT_CENTER,
            egui::FontId::proportional(11.0),
            theme::TEXTE_SECONDAIRE,
        );
    }
}

fn dessiner_cellule_texte(
    painter: &egui::Painter,
    rect_ligne: egui::Rect,
    x_colonne: f32,
    largeur_colonne: f32,
    texte: &str,
    align2: egui::Align2,
    font: egui::FontId,
    couleur: egui::Color32,
) {
    let pos = match align2 {
        egui::Align2::LEFT_CENTER => {
            egui::pos2(rect_ligne.left() + x_colonne, rect_ligne.center().y)
        }
        egui::Align2::RIGHT_CENTER => egui::pos2(
            rect_ligne.left() + x_colonne + largeur_colonne,
            rect_ligne.center().y,
        ),
        _ => egui::pos2(
            rect_ligne.left() + x_colonne + largeur_colonne / 2.0,
            rect_ligne.center().y,
        ),
    };
    painter.text(pos, align2, texte, font, couleur);
}

fn dessiner_cellule_note(
    painter: &egui::Painter,
    rect_ligne: egui::Rect,
    x_colonne: f32,
    largeur_colonne: f32,
    texte: &str,
    fond: egui::Color32,
    couleur_texte: egui::Color32,
) {
    let font = egui::FontId::proportional(12.5);
    let galerie = painter.layout_no_wrap(texte.to_string(), font.clone(), couleur_texte);
    let largeur_texte = galerie.size().x;

    let x_fin = rect_ligne.left() + x_colonne + largeur_colonne;
    let x_debut_texte = x_fin - largeur_texte;

    let taille_carre = 14.0;
    let marge = 6.0;
    let x_carre = x_debut_texte - marge - taille_carre;

    let rect_carre = egui::Rect::from_center_size(
        egui::pos2(x_carre + taille_carre / 2.0, rect_ligne.center().y),
        egui::vec2(taille_carre, taille_carre),
    );
    painter.rect_filled(rect_carre, 3.0, fond);

    painter.text(
        egui::pos2(x_fin, rect_ligne.center().y),
        egui::Align2::RIGHT_CENTER,
        texte,
        font,
        couleur_texte,
    );
}

fn badge_note(note: f64) -> (String, egui::Color32, egui::Color32) {
    if note >= 16.0 {
        (format!("{:.1}/20", note), theme::JOUR_BIEN_BG, theme::SUCCES)
    } else if note >= 10.0 {
        (format!("{:.1}/20", note), theme::JOUR_MOYEN_BG, theme::WARNING)
    } else {
        (format!("{:.1}/20", note), theme::JOUR_MAUVAIS_BG, theme::DANGER)
    }
}

fn est_week_end(d: NaiveDate) -> bool {
    matches!(d.weekday(), Weekday::Sat | Weekday::Sun)
}

fn ferie_jour_entier(feries: &HashMap<u32, Vec<String>>, jour: u32) -> bool {
    feries
        .get(&jour)
        .map(|v| v.iter().any(|p| p == "jour"))
        .unwrap_or(false)
}

fn ferie_periode(feries: &HashMap<u32, Vec<String>>, jour: u32, periode: &str) -> bool {
    feries
        .get(&jour)
        .map(|v| v.iter().any(|p| p == periode))
        .unwrap_or(false)
}

fn calculer_note_matin_10(pt: &EnregistrementPointage) -> Option<f64> {
    let arrivee = pt.matin_arrivee;
    let depart = pt.matin_sortie;
    if arrivee.is_none() && depart.is_none() {
        return None;
    }
    let limite_arrivee = NaiveTime::from_hms_opt(8, 15, 0).unwrap();
    let limite_depart = NaiveTime::from_hms_opt(11, 45, 0).unwrap();
    let mut points = 0.0;
    if let Some(a) = arrivee {
        if a <= limite_arrivee {
            points += 5.0;
        }
    }
    if let Some(d) = depart {
        if d >= limite_depart {
            points += 5.0;
        }
    }
    Some(points)
}

fn calculer_note_apres_midi_10(pt: &EnregistrementPointage) -> Option<f64> {
    let arrivee = pt.apres_midi_arrivee;
    let depart = pt.apres_midi_sortie;
    if arrivee.is_none() && depart.is_none() {
        return None;
    }
    let limite_arrivee = NaiveTime::from_hms_opt(14, 15, 0).unwrap();
    let limite_depart = NaiveTime::from_hms_opt(16, 0, 0).unwrap();
    let mut points = 0.0;
    if let Some(a) = arrivee {
        if a <= limite_arrivee {
            points += 5.0;
        }
    }
    if let Some(d) = depart {
        if d >= limite_depart {
            points += 5.0;
        }
    }
    Some(points)
}

fn note_effective(
    pt: &EnregistrementPointage,
    feries: &HashMap<u32, Vec<String>>,
    jour: u32,
) -> Option<f64> {
    if ferie_jour_entier(feries, jour) {
        return None;
    }
    let matin_ferie = ferie_periode(feries, jour, "matin");
    let aprem_ferie = ferie_periode(feries, jour, "aprem");

    let note_m = if matin_ferie {
        None
    } else {
        calculer_note_matin_10(pt)
    };
    let note_a = if aprem_ferie {
        None
    } else {
        calculer_note_apres_midi_10(pt)
    };

    let mut max = 0.0;
    if !matin_ferie {
        max += 10.0;
    }
    if !aprem_ferie {
        max += 10.0;
    }
    if max <= 0.0 {
        return None;
    }

    match (note_m, note_a) {
        (None, None) => None,
        (Some(m), None) => Some((m / max) * 20.0),
        (None, Some(a)) => Some((a / max) * 20.0),
        (Some(m), Some(a)) => Some(((m + a) / max) * 20.0),
    }
}

fn jours_ouvrables_du_mois(
    annee: i32,
    mois: u32,
    feries: &HashMap<u32, Vec<String>>,
) -> u32 {
    let nb = jours_dans_mois(annee, mois);
    let mut total = 0u32;
    for j in 1..=nb {
        let d = NaiveDate::from_ymd_opt(annee, mois, j).unwrap();
        if est_week_end(d) {
            continue;
        }
        if ferie_jour_entier(feries, j) {
            continue;
        }
        total += 1;
    }
    total.max(1)
}

fn calculer_classement(
    db: &Database,
    personnel: &[Personnel],
    annee: i32,
    mois: u32,
    feries: &HashMap<u32, Vec<String>>,
) -> Vec<ClassementEntry> {
    let nb = jours_dans_mois(annee, mois);
    let debut = NaiveDate::from_ymd_opt(annee, mois, 1).unwrap();
    let fin = NaiveDate::from_ymd_opt(annee, mois, nb).unwrap();
    let pointages = db.get_pointages_periode(debut, fin).unwrap_or_default();
    let jours_ouvrables = jours_ouvrables_du_mois(annee, mois, feries) as f64;

    let mut resultats: Vec<ClassementEntry> = personnel
        .iter()
        .map(|p| {
            let mut somme = 0.0_f64;
            let mut compte = 0_usize;
            for pt in pointages.iter().filter(|pt| pt.personnel_id == p.id) {
                if est_week_end(pt.date_pointage) {
                    continue;
                }
                let jour = pt.date_pointage.day();
                if let Some(n) = note_effective(pt, feries, jour) {
                    somme += n;
                    compte += 1;
                }
            }
            let note = if jours_ouvrables > 0.0 {
                somme / jours_ouvrables
            } else {
                0.0
            };
            ClassementEntry {
                personnel_id: p.id,
                nom: p.nom_complet(),
                note_moyenne: note,
                nb_jours_pointes: compte,
            }
        })
        .collect();

    resultats.sort_by(|a, b| {
        b.note_moyenne
            .partial_cmp(&a.note_moyenne)
            .unwrap_or(std::cmp::Ordering::Equal)
            .then(b.nb_jours_pointes.cmp(&a.nb_jours_pointes))
            .then(a.nom.cmp(&b.nom))
    });
    resultats
}

fn afficher_bilan_perso(
    ui: &mut egui::Ui,
    db: &Database,
    p_sel: &Personnel,
    state: &mut BilanState,
) {
    ui.label(
        egui::RichText::new(p_sel.nom_complet())
            .size(16.0)
            .color(theme::PRIMAIRE)
            .strong(),
    );
    ui.add_space(10.0);

    barre_mois(ui, state);

    ui.add_space(10.0);

    let cache_valide = state.cache_personnel_id == Some(p_sel.id)
        && state.cache_mois == Some(state.mois_affiche);

    if !cache_valide {
        let annee = state.mois_affiche.year();
        let mois = state.mois_affiche.month();
        let nb = jours_dans_mois(annee, mois);

        let jours_pointes = db
            .get_jours_pointes_du_mois(p_sel.id, annee, mois)
            .unwrap_or_default();

        let pointages = db
            .get_pointages_periode(
                NaiveDate::from_ymd_opt(annee, mois, 1).unwrap(),
                NaiveDate::from_ymd_opt(annee, mois, nb).unwrap(),
            )
            .unwrap_or_default();

        state.cache_feries = db.get_jours_feries_du_mois(annee, mois).unwrap_or_default();

        state.cache_pointages = pointages
            .into_iter()
            .filter(|pt| pt.personnel_id == p_sel.id)
            .map(|pt| (pt.date_pointage, pt))
            .collect();
        state.cache_jours_pointes = jours_pointes.into_iter().collect();
        state.cache_jours_pointes.sort();
        state.cache_personnel_id = Some(p_sel.id);
        state.cache_mois = Some(state.mois_affiche);
    }

    let nb = jours_dans_mois(state.mois_affiche.year(), state.mois_affiche.month());
    let dict = state.cache_pointages.clone();
    let jours_pointes_set: HashSet<u32> = state.cache_jours_pointes.iter().copied().collect();
    let feries = state.cache_feries.clone();

    let jours_ouvrables = jours_ouvrables_du_mois(
        state.mois_affiche.year(),
        state.mois_affiche.month(),
        &feries,
    );

    ui.label(
        egui::RichText::new(format!(
            "Jours pointés ce mois : {} / {} ouvrables",
            state.cache_jours_pointes.len(),
            jours_ouvrables
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
            let week_end = est_week_end(date);
            let pointe = jours_pointes_set.contains(&j);
            let detail = state.date_detaillee == Some(date);

            let ferie_j = ferie_jour_entier(&feries, j);
            let ferie_m = ferie_periode(&feries, j, "matin");
            let ferie_a = ferie_periode(&feries, j, "aprem");

            let note = dict
                .get(&date)
                .and_then(|pt| note_effective(pt, &feries, j));

            let (fond, texte) = if week_end {
                (
                    egui::Color32::from_rgb(235, 235, 235),
                    egui::Color32::from_rgb(150, 150, 150),
                )
            } else if detail {
                (theme::ACCENT, egui::Color32::WHITE)
            } else if ferie_j {
                (theme::WARNING, egui::Color32::WHITE)
            } else if ferie_m || ferie_a {
                (theme::FERIE_DEMI_BG, theme::WARNING)
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

            let sens = if week_end {
                egui::Sense::hover()
            } else {
                egui::Sense::click()
            };
            let (rect, resp) = ui
                .allocate_exact_size(egui::vec2(TAILLE_CASE_JOUR, HAUTEUR_CASE_JOUR), sens);
            if ui.is_rect_visible(rect) {
                ui.painter().rect_filled(rect, theme::r_sm(), fond);
                ui.painter().text(
                    rect.center(),
                    egui::Align2::CENTER_CENTER,
                    j.to_string(),
                    egui::FontId::proportional(17.0),
                    texte,
                );
                if !week_end && (ferie_m || ferie_a) && !ferie_j {
                    ui.painter().text(
                        egui::pos2(rect.right() - 5.0, rect.top() + 3.0),
                        egui::Align2::RIGHT_TOP,
                        ".",
                        egui::FontId::proportional(11.0),
                        theme::WARNING,
                    );
                }
            }
            if !week_end && resp.clicked() {
                nouvelle_detaillee = Some(date);
            }
        }
    });

    if let Some(d) = nouvelle_detaillee {
        state.date_detaillee = Some(d);
    }

    ui.add_space(14.0);

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

        if est_week_end(date) {
            continue;
        }

        if ferie_jour_entier(&feries, j) {
            continue;
        }

        match dict.get(&date).and_then(|pt| note_effective(pt, &feries, j)) {
            Some(n) if n >= 16.0 => bien += 1,
            Some(n) if n >= 10.0 => moyen += 1,
            Some(_) => mauvais += 1,
            None => {
                absent += 1;
            }
        }
    }

    ui.add_space(8.0);
    graphique_barres(ui, bien, moyen, mauvais, absent);

    if let Some(d) = state.date_detaillee {
        ui.add_space(14.0);
        ui.label(
            egui::RichText::new(d.format("%d/%m/%Y").to_string())
                .size(13.0)
                .color(theme::PRIMAIRE)
                .strong(),
        );

        if est_week_end(d) {
            ui.label(
                egui::RichText::new("Week-end : non comptabilisé")
                    .size(12.5)
                    .color(theme::TEXTE_SECONDAIRE)
                    .strong(),
            );
            return;
        }

        let j = d.day();
        if ferie_jour_entier(&feries, j) {
            ui.label(
                egui::RichText::new("Jour férié (jour entier)")
                    .size(12.5)
                    .color(theme::WARNING)
                    .strong(),
            );
        } else if ferie_periode(&feries, j, "matin") {
            ui.label(
                egui::RichText::new("Matin férié")
                    .size(12.5)
                    .color(theme::WARNING)
                    .strong(),
            );
        } else if ferie_periode(&feries, j, "aprem") {
            ui.label(
                egui::RichText::new("Après-midi férié")
                    .size(12.5)
                    .color(theme::WARNING)
                    .strong(),
            );
        }

        match dict.get(&d) {
            None => {
                if !ferie_jour_entier(&feries, j) {
                    ui.label("Aucun pointage enregistré pour cette date.");
                }
            }
            Some(pt) => {
                let matin_ferie = ferie_periode(&feries, j, "matin");
                let aprem_ferie = ferie_periode(&feries, j, "aprem");
                let note_m = if matin_ferie {
                    None
                } else {
                    calculer_note_matin_10(pt)
                };
                let note_a = if aprem_ferie {
                    None
                } else {
                    calculer_note_apres_midi_10(pt)
                };
                let total_j = note_effective(pt, &feries, j);

                let note_m_txt = note_m
                    .map(|n| format!("{:.0}/10", n))
                    .unwrap_or_else(|| "- /10".into());
                let note_a_txt = note_a
                    .map(|n| format!("{:.0}/10", n))
                    .unwrap_or_else(|| "- /10".into());
                let total_txt = total_j
                    .map(|n| format!("{:.0}/20", n))
                    .unwrap_or_else(|| "- /20".into());

                ui.label(format!("Matin : {}", note_m_txt));
                ui.label(format!("Après-midi : {}", note_a_txt));
                ui.label(format!("Total du jour : {}", total_txt));

                if let Some(o) = &pt.observation {
                    if !o.is_empty() {
                        ui.label(format!("Observation : {}", o));
                    }
                }
            }
        }
    }
}

fn barre_mois(ui: &mut egui::Ui, state: &mut BilanState) {
    ui.horizontal_wrapped(|ui| {
        if theme::bouton_image_seule(ui, "annee_gauche.png", 40.0, 22.0) {
            state.mois_affiche = NaiveDate::from_ymd_opt(
                state.mois_affiche.year() - 1,
                state.mois_affiche.month(),
                1,
            )
            .unwrap();
            state.invalider_mois();
        }
        ui.add_space(4.0);
        if theme::bouton_image_seule(ui, "mois_gauche.png", 40.0, 22.0) {
            state.mois_affiche = mois_prec(state.mois_affiche);
            state.invalider_mois();
        }
        ui.add_space(4.0);
        if theme::bouton_image_seule(ui, "mois_droite.png", 40.0, 22.0) {
            state.mois_affiche = mois_suiv(state.mois_affiche);
            state.invalider_mois();
        }
        ui.add_space(4.0);
        if theme::bouton_image_seule(ui, "annee_droite.png", 40.0, 22.0) {
            state.mois_affiche = NaiveDate::from_ymd_opt(
                state.mois_affiche.year() + 1,
                state.mois_affiche.month(),
                1,
            )
            .unwrap();
            state.invalider_mois();
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
}

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
            let barre = egui::Rect::from_min_size(rect.min, egui::vec2(largeur_max * ratio, 20.0));
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
