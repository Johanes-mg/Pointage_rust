use crate::database::Database;
use crate::models::Personnel;
use crate::theme;
use chrono::{Datelike, Local, NaiveDate, NaiveTime, Timelike};
use std::collections::HashMap;

const CASE_LARGEUR: f32 = 46.0;
const CASE_HAUTEUR: f32 = 46.0;

#[derive(Clone, Debug, Default)]
pub struct PointageEnCours {
    pub coche_matin: bool,
    pub matin_arr: Option<NaiveTime>,
    pub matin_sort: Option<NaiveTime>,
    pub coche_aprem: bool,
    pub aprem_arr: Option<NaiveTime>,
    pub aprem_sort: Option<NaiveTime>,
    pub observation: String,
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum FiltreJour {
    Tous,
    Presents,
    Absents,
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum PeriodeConge {
    Jour,
    Matin,
    Aprem,
}

impl PeriodeConge {
    fn code(self) -> &'static str {
        match self {
            PeriodeConge::Jour => "jour",
            PeriodeConge::Matin => "matin",
            PeriodeConge::Aprem => "aprem",
        }
    }
}

pub struct CongeDialog {
    pub ouvert: bool,
    pub personnel_id: Option<i64>,
    pub nom_personnel: String,
    pub debut: NaiveDate,
    pub fin: NaiveDate,
    pub periode: PeriodeConge,
    pub message: String,
    pub message_ok: bool,
}

impl Default for CongeDialog {
    fn default() -> Self {
        let t = Local::now().date_naive();
        Self {
            ouvert: false,
            personnel_id: None,
            nom_personnel: String::new(),
            debut: t,
            fin: t,
            periode: PeriodeConge::Jour,
            message: String::new(),
            message_ok: true,
        }
    }
}

#[derive(Clone, Debug, Default)]
pub struct CongeJour {
    pub matin: bool,
    pub aprem: bool,
}

pub struct PointageState {
    pub mois_affiche: NaiveDate,
    pub date_selectionnee: NaiveDate,
    pub cache_personnel: Option<Vec<Personnel>>,
    pub cache_date_pointages: Option<NaiveDate>,
    pub pointages_jour: Vec<i64>,
    pub conges_jour: HashMap<i64, CongeJour>,
    pub cache_mois: Option<(i32, u32)>,
    pub stats_mois: HashMap<u32, i64>,
    pub feries_mois: HashMap<u32, Vec<String>>,
    pub ferie_jour: Option<String>,
    pub cache_ferie_date: Option<NaiveDate>,
    pub brouillons: HashMap<i64, PointageEnCours>,
    pub rafraichir: bool,
    pub recherche: String,
    pub filtre_jour: FiltreJour,
    pub dialog_conge: CongeDialog,
}

impl PointageState {
    pub fn new() -> Self {
        let t = Local::now().date_naive();
        Self {
            mois_affiche: NaiveDate::from_ymd_opt(t.year(), t.month(), 1).unwrap(),
            date_selectionnee: t,
            cache_personnel: None,
            cache_date_pointages: None,
            pointages_jour: Vec::new(),
            conges_jour: HashMap::new(),
            cache_mois: None,
            stats_mois: HashMap::new(),
            feries_mois: HashMap::new(),
            ferie_jour: None,
            cache_ferie_date: None,
            brouillons: HashMap::new(),
            rafraichir: true,
            recherche: String::new(),
            filtre_jour: FiltreJour::Tous,
            dialog_conge: CongeDialog::default(),
        }
    }

    pub fn invalider_cache(&mut self) {
        self.cache_personnel = None;
        self.cache_date_pointages = None;
        self.pointages_jour.clear();
        self.conges_jour.clear();
        self.cache_mois = None;
        self.stats_mois.clear();
        self.feries_mois.clear();
        self.ferie_jour = None;
        self.cache_ferie_date = None;
        self.brouillons.clear();
        self.rafraichir = true;
    }
}

pub fn afficher(ui: &mut egui::Ui, db: &Database, state: &mut PointageState) {
    ui.label(
        egui::RichText::new("Pointage")
            .size(22.0)
            .color(theme::PRIMAIRE)
            .strong(),
    );
    ui.add_space(16.0);

    let dispo = ui.available_width();
    let hauteur_totale = ui.available_height().max(500.0);
    let ecart = 16.0_f32;
    let seuil = 900.0_f32;

    if dispo < seuil {
        calendrier_gauche(ui, db, state);
        ui.add_space(ecart);
        employes_droite(ui, db, state, dispo, hauteur_totale);
    } else {
        let largeur_gauche = (dispo * 0.40).clamp(360.0, dispo - 420.0);
        let largeur_droite = dispo - largeur_gauche - ecart;

        ui.horizontal_top(|ui| {
            ui.allocate_ui_with_layout(
                egui::vec2(largeur_gauche, hauteur_totale),
                egui::Layout::top_down(egui::Align::LEFT),
                |ui| {
                    ui.set_max_width(largeur_gauche);
                    calendrier_gauche(ui, db, state);
                },
            );

            ui.add_space(ecart);

            ui.allocate_ui_with_layout(
                egui::vec2(largeur_droite, hauteur_totale),
                egui::Layout::top_down(egui::Align::LEFT),
                |ui| {
                    ui.set_max_width(largeur_droite);
                    employes_droite(ui, db, state, largeur_droite, hauteur_totale);
                },
            );
        });
    }

    afficher_dialog_conge(ui.ctx(), db, state);
}

fn afficher_dialog_conge(ctx: &egui::Context, db: &Database, state: &mut PointageState) {
    if !state.dialog_conge.ouvert {
        return;
    }

    let mut fermer = false;
    let mut valider = false;

    egui::Window::new("Gestion de congé/Permission")
        .collapsible(false)
        .resizable(false)
        .anchor(egui::Align2::CENTER_CENTER, egui::vec2(0.0, 0.0))
        .show(ctx, |ui| {
            ui.set_min_width(420.0);

            let nom = state.dialog_conge.nom_personnel.clone();
            ui.label(
                egui::RichText::new(&nom)
                    .size(14.0)
                    .color(theme::PRIMAIRE)
                    .strong(),
            );
            ui.add_space(10.0);

            ui.label(
                egui::RichText::new("Période")
                    .size(12.0)
                    .color(theme::TEXTE_SECONDAIRE)
                    .strong(),
            );
            ui.add_space(4.0);
            ui.horizontal_wrapped(|ui| {
                let periode_actuelle = state.dialog_conge.periode;
                if periode_bouton(ui, "Jour entier", periode_actuelle == PeriodeConge::Jour) {
                    state.dialog_conge.periode = PeriodeConge::Jour;
                }
                ui.add_space(6.0);
                if periode_bouton(ui, "Matin", periode_actuelle == PeriodeConge::Matin) {
                    state.dialog_conge.periode = PeriodeConge::Matin;
                }
                ui.add_space(6.0);
                if periode_bouton(ui, "Après-midi", periode_actuelle == PeriodeConge::Aprem) {
                    state.dialog_conge.periode = PeriodeConge::Aprem;
                }
            });

            ui.add_space(12.0);

            ui.horizontal_wrapped(|ui| {
                ui.label(
                    egui::RichText::new("Du")
                        .size(12.0)
                        .color(theme::TEXTE_SECONDAIRE)
                        .strong(),
                );
                ui.add_space(4.0);
                champ_date_dialog(ui, &mut state.dialog_conge.debut, "conge_dlg_debut");
                ui.add_space(12.0);
                ui.label(
                    egui::RichText::new("Au")
                        .size(12.0)
                        .color(theme::TEXTE_SECONDAIRE)
                        .strong(),
                );
                ui.add_space(4.0);
                champ_date_dialog(ui, &mut state.dialog_conge.fin, "conge_dlg_fin");
            });

            ui.add_space(12.0);

            if let Some(pid) = state.dialog_conge.personnel_id {
                if let Ok(conges) = db.get_conges_personnel(pid) {
                    if !conges.is_empty() {
                        ui.label(
                            egui::RichText::new("Congés enregistrés :")
                                .size(12.0)
                                .color(theme::TEXTE_SECONDAIRE)
                                .strong(),
                        );
                        ui.add_space(4.0);
                        for c in conges.iter() {
                            ui.horizontal(|ui| {
                                let libelle_periode = match c.periode.as_str() {
                                    "matin" => "matin",
                                    "aprem" => "après-midi",
                                    _ => "jour entier",
                                };
                                ui.label(
                                    egui::RichText::new(format!(
                                        "{} -> {}  ({})",
                                        c.date_debut.format("%d/%m/%Y"),
                                        c.date_fin.format("%d/%m/%Y"),
                                        libelle_periode
                                    ))
                                    .size(12.0)
                                    .color(theme::TEXTE),
                                );
                                ui.with_layout(
                                    egui::Layout::right_to_left(egui::Align::Center),
                                    |ui| {
                                        if theme::bouton_image_seule(
                                            ui,
                                            "effacer.png",
                                            26.0,
                                            14.0,
                                        ) {
                                            let _ = db.supprimer_conge(c.id);
                                            state.cache_date_pointages = None;
                                            state.rafraichir = true;
                                        }
                                    },
                                );
                            });
                        }
                    }
                }
            }

            if !state.dialog_conge.message.is_empty() {
                ui.add_space(8.0);
                theme::message_inline(
                    ui,
                    &state.dialog_conge.message,
                    state.dialog_conge.message_ok,
                );
            }

            ui.add_space(12.0);
            ui.horizontal_wrapped(|ui| {
                if theme::bouton_image_texte(
                    ui,
                    "ajouter.png",
                    "Ajouter",
                    200.0,
                    40.0,
                ) {
                    valider = true;
                }
                ui.add_space(8.0);
                if theme::bouton_secondaire(ui, "Fermer", 100.0) {
                    fermer = true;
                }
            });
        });

    if valider {
        if let Some(pid) = state.dialog_conge.personnel_id {
            let debut = state.dialog_conge.debut;
            let fin = state.dialog_conge.fin;
            let periode = state.dialog_conge.periode.code();
            match db.ajouter_conge(pid, debut, fin, periode) {
                Ok(_) => {
                    state.dialog_conge.message = "Congé enregistré.".into();
                    state.dialog_conge.message_ok = true;
                    state.cache_date_pointages = None;
                    state.rafraichir = true;
                }
                Err(e) => {
                    state.dialog_conge.message = format!("Erreur : {}", e);
                    state.dialog_conge.message_ok = false;
                }
            }
        }
    }

    if fermer {
        state.dialog_conge.ouvert = false;
        state.dialog_conge.message.clear();
    }
}

fn periode_bouton(ui: &mut egui::Ui, texte: &str, actif: bool) -> bool {
    let texte_final = if actif {
        egui::RichText::new(texte)
            .size(12.5)
            .color(egui::Color32::WHITE)
            .strong()
    } else {
        egui::RichText::new(texte)
            .size(12.5)
            .color(theme::PRIMAIRE)
            .strong()
    };
    let fond = if actif { theme::ACCENT } else { egui::Color32::WHITE };
    let bordure = if actif { theme::ACCENT } else { theme::BORDURE };
    let bouton = egui::Button::new(texte_final)
        .fill(fond)
        .stroke(egui::Stroke::new(1.0_f32, bordure))
        .rounding(theme::r_sm())
        .min_size(egui::vec2(100.0, 30.0));
    ui.add(bouton).clicked()
}

fn champ_date_dialog(ui: &mut egui::Ui, date: &mut NaiveDate, id: &str) {
    let mut jour = date.day();
    let mut mois = date.month();
    let mut annee = date.year();
    let mut change = false;

    ui.push_id(id, |ui| {
        ui.horizontal(|ui| {
            ui.label(
                egui::RichText::new("JJ")
                    .size(10.0)
                    .color(theme::TEXTE_SECONDAIRE)
                    .strong(),
            );
            if ui
                .add(
                    egui::DragValue::new(&mut jour)
                        .range(1..=31)
                        .speed(1)
                        .custom_formatter(|n, _| format!("{:02}", n as i64)),
                )
                .changed()
            {
                change = true;
            }
            ui.label(
                egui::RichText::new("MM")
                    .size(10.0)
                    .color(theme::TEXTE_SECONDAIRE)
                    .strong(),
            );
            if ui
                .add(
                    egui::DragValue::new(&mut mois)
                        .range(1..=12)
                        .speed(1)
                        .custom_formatter(|n, _| format!("{:02}", n as i64)),
                )
                .changed()
            {
                change = true;
            }
            ui.label(
                egui::RichText::new("AAAA")
                    .size(10.0)
                    .color(theme::TEXTE_SECONDAIRE)
                    .strong(),
            );
            if ui
                .add(
                    egui::DragValue::new(&mut annee)
                        .range(1900..=2100)
                        .speed(1),
                )
                .changed()
            {
                change = true;
            }
        });
    });

    if change {
        if let Some(d) = NaiveDate::from_ymd_opt(annee, mois, jour) {
            *date = d;
        }
    }
}

fn charger_stats_mois(db: &Database, state: &mut PointageState) {
    let cle = (state.mois_affiche.year(), state.mois_affiche.month());
    if state.cache_mois != Some(cle) {
        state.stats_mois = db
            .get_statistiques_mois(cle.0, cle.1)
            .unwrap_or_default();
        state.feries_mois = db
            .get_jours_feries_du_mois(cle.0, cle.1)
            .unwrap_or_default();
        state.cache_mois = Some(cle);
    }
}

fn charger_ferie_jour(db: &Database, state: &mut PointageState) {
    if state.cache_ferie_date != Some(state.date_selectionnee) {
        state.ferie_jour = db.get_ferie_jour(state.date_selectionnee).unwrap_or(None);
        state.cache_ferie_date = Some(state.date_selectionnee);
    }
}

fn charger_pointages_jour(db: &Database, state: &mut PointageState) {
    if state.rafraichir {
        state.cache_date_pointages = None;
        state.rafraichir = false;
    }

    let date_changee = state.cache_date_pointages != Some(state.date_selectionnee);

    if !date_changee {
        if let Some(personnes) = state.cache_personnel.clone() {
            for p in &personnes {
                state
                    .brouillons
                    .entry(p.id)
                    .or_insert_with(PointageEnCours::default);
            }
        }
        return;
    }

    let pointages = db
        .get_pointages_du_jour(state.date_selectionnee)
        .unwrap_or_default();

    state.pointages_jour = pointages
        .iter()
        .filter(|pt| {
            pt.matin_arrivee.is_some()
                || pt.matin_sortie.is_some()
                || pt.apres_midi_arrivee.is_some()
                || pt.apres_midi_sortie.is_some()
        })
        .map(|p| p.personnel_id)
        .collect();

    state.conges_jour.clear();
    if let Some(personnes) = state.cache_personnel.clone() {
        for p in &personnes {
            if let Ok(periodes) = db.conges_du_jour_pour(p.id, state.date_selectionnee) {
                if !periodes.is_empty() {
                    let mut cj = CongeJour::default();
                    for per in periodes {
                        match per.as_str() {
                            "jour" => {
                                cj.matin = true;
                                cj.aprem = true;
                            }
                            "matin" => cj.matin = true,
                            "aprem" => cj.aprem = true,
                            _ => {}
                        }
                    }
                    state.conges_jour.insert(p.id, cj);
                }
            }
        }
    }

    state.brouillons.clear();
    if let Some(personnes) = state.cache_personnel.clone() {
        for p in &personnes {
            if let Some(e) = pointages.iter().find(|pt| pt.personnel_id == p.id) {
                let matin_ok = e.matin_arrivee.is_some() || e.matin_sortie.is_some();
                let aprem_ok = e.apres_midi_arrivee.is_some() || e.apres_midi_sortie.is_some();
                state.brouillons.insert(
                    p.id,
                    PointageEnCours {
                        coche_matin: matin_ok,
                        matin_arr: e.matin_arrivee,
                        matin_sort: e.matin_sortie,
                        coche_aprem: aprem_ok,
                        aprem_arr: e.apres_midi_arrivee,
                        aprem_sort: e.apres_midi_sortie,
                        observation: e.observation.clone().unwrap_or_default(),
                    },
                );
            } else {
                state.brouillons.insert(p.id, PointageEnCours::default());
            }
        }
    }
    state.cache_date_pointages = Some(state.date_selectionnee);
}

fn prochaine_periode(courante: Option<&str>) -> Option<&'static str> {
    match courante {
        None => Some("jour"),
        Some("jour") => Some("matin"),
        Some("matin") => Some("aprem"),
        Some("aprem") => None,
        _ => Some("jour"),
    }
}

fn libelle_bouton_ferie(courante: Option<&str>) -> &'static str {
    match courante {
        None => "Marquer férié (jour)",
        Some("jour") => "Férié -> Matin",
        Some("matin") => "Férié -> Après-midi",
        Some("aprem") => "Retirer férié",
        _ => "Marquer férié (jour)",
    }
}

fn appliquer_transition_ferie(
    db: &Database,
    date: NaiveDate,
    courante: Option<&str>,
) -> Option<String> {
    let _ = match courante {
        Some("jour") => {
            db.set_ferie(date, "jour", false).ok();
            db.set_ferie(date, "matin", true).ok();
        }
        Some("matin") => {
            db.set_ferie(date, "matin", false).ok();
            db.set_ferie(date, "aprem", true).ok();
        }
        Some("aprem") => {
            db.clear_ferie_jour(date).ok();
        }
        None => {
            db.set_ferie(date, "jour", true).ok();
        }
        _ => {}
    };
    prochaine_periode(courante).map(|s| s.to_string())
}

fn ajuster_date_selectionnee(state: &mut PointageState) {
    if state.mois_affiche.year() != state.date_selectionnee.year()
        || state.mois_affiche.month() != state.date_selectionnee.month()
    {
        state.date_selectionnee = state.mois_affiche;
        state.cache_ferie_date = None;
        state.rafraichir = true;
    }
}

fn ferie_bloque_periode(ferie_jour: Option<&str>, matin: bool) -> bool {
    match ferie_jour {
        Some("jour") => true,
        Some("matin") => matin,
        Some("aprem") => !matin,
        _ => false,
    }
}

fn est_week_end(d: NaiveDate) -> bool {
    matches!(d.weekday(), chrono::Weekday::Sat | chrono::Weekday::Sun)
}

fn est_avant_arrivee(p: &Personnel, date: NaiveDate) -> bool {
    if let Some(arrivee) = p.date_arrivee {
        date < arrivee
    } else {
        false
    }
}

fn calendrier_gauche(ui: &mut egui::Ui, db: &Database, state: &mut PointageState) {
    charger_stats_mois(db, state);
    charger_ferie_jour(db, state);

    theme::carte(ui, |ui| {
        ui.set_width(ui.available_width());
        ui.set_max_width(ui.available_width());

        ui.horizontal_wrapped(|ui| {
            if theme::bouton_image_seule(ui, "annee_gauche.png", 40.0, 22.0) {
                state.mois_affiche = NaiveDate::from_ymd_opt(
                    state.mois_affiche.year() - 1,
                    state.mois_affiche.month(),
                    1,
                )
                .unwrap();
                ajuster_date_selectionnee(state);
            }
            ui.add_space(4.0);
            if theme::bouton_image_seule(ui, "mois_gauche.png", 40.0, 22.0) {
                let (y, m) = precedente(state.mois_affiche);
                state.mois_affiche = NaiveDate::from_ymd_opt(y, m, 1).unwrap();
                ajuster_date_selectionnee(state);
            }
            ui.add_space(4.0);
            if theme::bouton_image_seule(ui, "mois_droite.png", 40.0, 22.0) {
                let (y, m) = suivante(state.mois_affiche);
                state.mois_affiche = NaiveDate::from_ymd_opt(y, m, 1).unwrap();
                ajuster_date_selectionnee(state);
            }
            ui.add_space(4.0);
            if theme::bouton_image_seule(ui, "annee_droite.png", 40.0, 22.0) {
                state.mois_affiche = NaiveDate::from_ymd_opt(
                    state.mois_affiche.year() + 1,
                    state.mois_affiche.month(),
                    1,
                )
                .unwrap();
                ajuster_date_selectionnee(state);
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
            if state.date_selectionnee != t {
                state.date_selectionnee = t;
                state.cache_ferie_date = None;
                state.rafraichir = true;
            }
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

        let stats = state.stats_mois.clone();
        let feries = state.feries_mois.clone();
        let aujourdhui = Local::now().date_naive();
        let date_sel = state.date_selectionnee;

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
                    let week_end = est_week_end(date);
                    let pointe = stats.get(&jour).copied().unwrap_or(0) > 0;
                    let est_aujourdhui = date == aujourdhui;
                    let selectionne = date == date_sel;

                    let est_ferie = feries
                        .get(&jour)
                        .map(|v| v.iter().any(|p| p == "jour"))
                        .unwrap_or(false);
                    let est_ferie_matin = feries
                        .get(&jour)
                        .map(|v| v.iter().any(|p| p == "matin"))
                        .unwrap_or(false);
                    let est_ferie_aprem = feries
                        .get(&jour)
                        .map(|v| v.iter().any(|p| p == "aprem"))
                        .unwrap_or(false);

                    let (fond, texte, bordure) = if week_end {
                        (
                            egui::Color32::from_rgb(235, 235, 235),
                            egui::Color32::from_rgb(150, 150, 150),
                            egui::Color32::from_rgb(210, 210, 210),
                        )
                    } else if selectionne {
                        (theme::ACCENT, egui::Color32::WHITE, theme::PRIMAIRE)
                    } else if est_ferie {
                        (theme::WARNING, egui::Color32::WHITE, theme::WARNING)
                    } else if est_ferie_matin || est_ferie_aprem {
                        (theme::FERIE_DEMI_BG, theme::WARNING, theme::WARNING)
                    } else if pointe {
                        (theme::JOUR_BIEN_BG, theme::SUCCES, theme::SUCCES)
                    } else if est_aujourdhui {
                        (theme::SELECTION_JOUR, theme::ACCENT, theme::ACCENT)
                    } else {
                        (theme::FOND, theme::PRIMAIRE, theme::BORDURE)
                    };

                    let sens = if week_end {
                        egui::Sense::hover()
                    } else {
                        egui::Sense::click()
                    };
                    let (rect, resp) = ui
                        .allocate_exact_size(egui::vec2(CASE_LARGEUR, CASE_HAUTEUR), sens);

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
                        if !week_end && (est_ferie_matin || est_ferie_aprem) {
                            painter.text(
                                egui::pos2(rect.right() - 6.0, rect.top() + 4.0),
                                egui::Align2::RIGHT_TOP,
                                ".",
                                egui::FontId::proportional(11.0),
                                theme::WARNING,
                            );
                        }
                    }

                    if !week_end && resp.clicked() && !selectionne {
                        nouvelle_date = Some(date);
                    }
                    if !week_end && resp.hovered() {
                        ui.ctx().set_cursor_icon(egui::CursorIcon::PointingHand);
                    }
                }
            });
            ligne += 1;
        }

        if let Some(d) = nouvelle_date {
            state.date_selectionnee = d;
            state.cache_ferie_date = None;
            state.rafraichir = true;
            state.filtre_jour = FiltreJour::Tous;
        }

        ui.add_space(12.0);
        theme::separateur(ui);
        ui.add_space(12.0);

        egui::Frame::none()
            .fill(theme::FOND)
            .rounding(theme::r_md())
            .stroke(egui::Stroke::new(1.0_f32, theme::BORDURE))
            .inner_margin(egui::Margin::symmetric(14.0, 12.0))
            .show(ui, |ui| {
                ui.set_width(ui.available_width());
                ui.set_max_width(ui.available_width());

                ui.horizontal_wrapped(|ui| {
                    ui.label(
                        egui::RichText::new("Jour férié :")
                            .size(12.0)
                            .color(theme::TEXTE_SECONDAIRE)
                            .strong(),
                    );
                    let statut = match state.ferie_jour.as_deref() {
                        Some("jour") => "Jour entier",
                        Some("matin") => "Matin",
                        Some("aprem") => "Après-midi",
                        _ => "Aucun",
                    };
                    ui.label(
                        egui::RichText::new(statut)
                            .size(12.0)
                            .color(if state.ferie_jour.is_some() {
                                theme::WARNING
                            } else {
                                theme::TEXTE_SECONDAIRE
                            })
                            .strong(),
                    );
                });

                ui.add_space(8.0);

                let libelle = libelle_bouton_ferie(state.ferie_jour.as_deref());

                let bouton = if theme::image(ui.ctx(), "ferier.png").is_some() {
                    theme::bouton_image_texte(ui, "ferier.png", libelle, 240.0, 38.0)
                } else {
                    theme::bouton_secondaire(ui, libelle, 240.0)
                };

                if bouton {
                    let courante = state.ferie_jour.as_deref();
                    let nouvelle =
                        appliquer_transition_ferie(db, state.date_selectionnee, courante);
                    state.ferie_jour = nouvelle;
                    state.cache_mois = None;
                    state.cache_ferie_date = Some(state.date_selectionnee);
                    state.cache_date_pointages = None;
                    state.brouillons.clear();
                    state.rafraichir = true;
                }
            });
    });
}

fn conge_bloque_periode(cj: Option<&CongeJour>, matin: bool) -> bool {
    match cj {
        Some(c) => {
            if matin {
                c.matin
            } else {
                c.aprem
            }
        }
        None => false,
    }
}

fn employes_droite(
    ui: &mut egui::Ui,
    db: &Database,
    state: &mut PointageState,
    _largeur_panneau: f32,
    hauteur_panneau: f32,
) {
    if state.cache_personnel.is_none() {
        state.cache_personnel = Some(db.get_personnel().unwrap_or_default());
    }

    charger_pointages_jour(db, state);

    let toutes = state.cache_personnel.clone().unwrap_or_default();

    let eligibles: Vec<Personnel> = toutes
        .iter()
        .filter(|p| !est_avant_arrivee(p, state.date_selectionnee))
        .cloned()
        .collect();

    let filtre_texte = state.recherche.to_lowercase();
    let personnes: Vec<Personnel> = eligibles
        .iter()
        .filter(|p| {
            if !filtre_texte.trim().is_empty() {
                let haystack = format!(
                    "{} {} {} {} {}",
                    p.nom,
                    p.prenom,
                    p.matricule.clone().unwrap_or_default(),
                    p.telephone.clone().unwrap_or_default(),
                    p.email.clone().unwrap_or_default()
                )
                .to_lowercase();
                if !haystack.contains(&filtre_texte) {
                    return false;
                }
            }
            match state.filtre_jour {
                FiltreJour::Tous => true,
                FiltreJour::Presents => state.pointages_jour.contains(&p.id),
                FiltreJour::Absents => {
                    !state.pointages_jour.contains(&p.id) && !en_conge_entier(state, p.id)
                }
            }
        })
        .cloned()
        .collect();

    let total = eligibles.len();
    let presents = eligibles
        .iter()
        .filter(|p| state.pointages_jour.contains(&p.id))
        .count();
    let en_conge = eligibles
        .iter()
        .filter(|p| en_conge_entier(state, p.id))
        .count();
    let absents = total.saturating_sub(presents).saturating_sub(en_conge);
    let taux = if total > 0 {
        (presents as f64 * 100.0) / total as f64
    } else {
        0.0
    };

    let ferie_jour = state.ferie_jour.clone();
    let jour_ferie_entier = matches!(ferie_jour.as_deref(), Some("jour"));
    let jour_week_end = est_week_end(state.date_selectionnee);

    egui::Frame::none()
        .fill(theme::FOND_BILAN)
        .rounding(theme::r_md())
        .inner_margin(egui::Margin::symmetric(20.0, 14.0))
        .show(ui, |ui| {
            ui.set_width(ui.available_width());
            ui.set_max_width(ui.available_width());

            let date_str = state.date_selectionnee.format("%d/%m/%Y").to_string();
            ui.label(
                egui::RichText::new(date_str)
                    .size(15.0)
                    .color(theme::PRIMAIRE)
                    .strong(),
            );
            ui.add_space(6.0);

            ui.horizontal_wrapped(|ui| {
                ui.label(
                    egui::RichText::new("BILAN DU JOUR")
                        .size(11.5)
                        .color(theme::TEXTE_SECONDAIRE)
                        .strong(),
                );
                ui.add_space(15.0);

                if bouton_filtre(
                    ui,
                    &format!("Présents : {}", presents),
                    theme::SUCCES,
                    state.filtre_jour == FiltreJour::Presents,
                ) {
                    state.filtre_jour = if state.filtre_jour == FiltreJour::Presents {
                        FiltreJour::Tous
                    } else {
                        FiltreJour::Presents
                    };
                }
                ui.add_space(10.0);

                if bouton_filtre(
                    ui,
                    &format!("Absents : {}", absents),
                    theme::DANGER,
                    state.filtre_jour == FiltreJour::Absents,
                ) {
                    state.filtre_jour = if state.filtre_jour == FiltreJour::Absents {
                        FiltreJour::Tous
                    } else {
                        FiltreJour::Absents
                    };
                }
                ui.add_space(10.0);

                if en_conge > 0 {
                    ui.label(
                        egui::RichText::new(format!("Congés : {}", en_conge))
                            .size(12.5)
                            .color(theme::WARNING)
                            .strong(),
                    );
                    ui.add_space(10.0);
                }

                ui.label(
                    egui::RichText::new(format!("Total : {}", total))
                        .size(12.5)
                        .color(theme::TEXTE)
                        .strong(),
                );
                ui.add_space(15.0);
                ui.label(
                    egui::RichText::new(format!("Taux : {:.0} %", taux))
                        .size(12.5)
                        .color(theme::PRIMAIRE)
                        .strong(),
                );

                if state.filtre_jour != FiltreJour::Tous {
                    ui.add_space(10.0);
                    ui.label(
                        egui::RichText::new(match state.filtre_jour {
                            FiltreJour::Presents => "Filtre : présents",
                            FiltreJour::Absents => "Filtre : absents",
                            FiltreJour::Tous => "",
                        })
                        .size(11.5)
                        .color(theme::ACCENT)
                        .strong()
                        .italics(),
                    );
                }
            });

            if jour_week_end {
                ui.add_space(6.0);
                ui.label(
                    egui::RichText::new("Week-end : pointage désactivé")
                        .size(12.0)
                        .color(theme::TEXTE_SECONDAIRE)
                        .strong(),
                );
            }

            if let Some(f) = &ferie_jour {
                ui.add_space(6.0);
                let txt = match f.as_str() {
                    "jour" => "Jour férié (jour entier) - pointage bloqué",
                    "matin" => "Matin férié - pointage matin bloqué",
                    "aprem" => "Après-midi férié - pointage après-midi bloqué",
                    _ => "",
                };
                ui.label(
                    egui::RichText::new(txt)
                        .size(12.0)
                        .color(theme::WARNING)
                        .strong(),
                );
            }
        });

    ui.add_space(12.0);

    let largeur_recherche = ui.available_width();
    let _recherche_changee = theme::champ_recherche(ui, &mut state.recherche, largeur_recherche);

    ui.add_space(12.0);

    if total == 0 {
        theme::carte(ui, |ui| {
            ui.set_width(ui.available_width());
            ui.vertical_centered(|ui| {
                ui.add_space(30.0);
                ui.label(
                    egui::RichText::new("Aucun personnel actif à cette date")
                        .size(13.0)
                        .color(theme::TEXTE_SECONDAIRE),
                );
                ui.add_space(30.0);
            });
        });
        return;
    }

    if jour_week_end {
        ui.add_space(8.0);
        theme::message_inline(
            ui,
            "Week-end : le pointage est désactivé pour cette date.",
            false,
        );
        return;
    }

    if jour_ferie_entier {
        ui.add_space(8.0);
        theme::message_inline(
            ui,
            "Jour férié : aucun pointage n'est autorisé pour cette date.",
            false,
        );
        return;
    }

    if personnes.is_empty() {
        theme::carte(ui, |ui| {
            ui.set_width(ui.available_width());
            ui.vertical_centered(|ui| {
                ui.add_space(30.0);
                ui.label(
                    egui::RichText::new("Aucun résultat")
                        .size(13.0)
                        .color(theme::TEXTE_SECONDAIRE),
                );
                ui.add_space(30.0);
            });
        });
        return;
    }

    let mut action_supprimer: Option<i64> = None;
    let mut action_enregistrer: Option<i64> = None;
    let mut action_ouvrir_conge: Option<(i64, String)> = None;

    let hauteur_liste = (hauteur_panneau - 200.0).max(240.0);

    egui::ScrollArea::vertical()
        .id_source("pointage_employes_scroll")
        .auto_shrink([false, false])
        .max_height(hauteur_liste)
        .show(ui, |ui| {
            ui.set_max_width(ui.available_width());
            ui.set_width(ui.available_width());

            for p in &personnes {
                let est_pointe = state.pointages_jour.contains(&p.id);
                let cj = state.conges_jour.get(&p.id).cloned();
                let conge_total = cj
                    .as_ref()
                    .map(|c| c.matin && c.aprem)
                    .unwrap_or(false);

                let fond_carte = if conge_total {
                    theme::FERIE_DEMI_BG
                } else if est_pointe {
                    theme::FOND_POINTAGE
                } else {
                    theme::FOND
                };

                egui::Frame::none()
                    .fill(fond_carte)
                    .rounding(theme::r_md())
                    .stroke(egui::Stroke::new(1.0_f32, theme::BORDURE))
                    .inner_margin(egui::Margin::symmetric(18.0, 14.0))
                    .show(ui, |ui| {
                        ui.set_width(ui.available_width());
                        ui.set_max_width(ui.available_width());

                        ui.horizontal_wrapped(|ui| {
                            ui.label(
                                egui::RichText::new(p.nom_complet())
                                    .size(14.5)
                                    .color(theme::TEXTE)
                                    .strong(),
                            );
                            ui.add_space(8.0);
                            if let Some(c) = &cj {
                                let badge_txt = if c.matin && c.aprem {
                                    "EN CONGÉ"
                                } else if c.matin {
                                    "CONGÉ MATIN"
                                } else if c.aprem {
                                    "CONGÉ APRÈS-MIDI"
                                } else {
                                    ""
                                };
                                if !badge_txt.is_empty() {
                                    theme::badge(
                                        ui,
                                        badge_txt,
                                        theme::WARNING,
                                        egui::Color32::WHITE,
                                    );
                                }
                            } else if est_pointe {
                                theme::badge(
                                    ui,
                                    "POINTÉ",
                                    theme::SUCCES,
                                    egui::Color32::WHITE,
                                );
                            }
                        });

                        ui.add_space(12.0);

                        if conge_total {
                            ui.horizontal_wrapped(|ui| {
                                ui.label(
                                    egui::RichText::new(
                                        "Personnel en congé toute la journée - pointage bloqué.",
                                    )
                                    .size(12.0)
                                    .color(theme::TEXTE_SECONDAIRE)
                                    .italics(),
                                );
                            });
                            ui.add_space(10.0);
                            ui.horizontal(|ui| {
                                ui.with_layout(
                                    egui::Layout::right_to_left(egui::Align::Center),
                                    |ui| {
                                        if theme::bouton_image_seule(ui, "conge.png", 40.0, 22.0)
                                        {
                                            action_ouvrir_conge =
                                                Some((p.id, p.nom_complet()));
                                        }
                                    },
                                );
                            });
                            return;
                        }

                        if let Some(b) = state.brouillons.get_mut(&p.id) {
                            let id_matin = format!("matin_{}", p.id);
                            let id_aprem = format!("aprem_{}", p.id);

                            let ferie_matin = ferie_bloque_periode(ferie_jour.as_deref(), true);
                            let ferie_aprem = ferie_bloque_periode(ferie_jour.as_deref(), false);
                            let conge_matin = conge_bloque_periode(cj.as_ref(), true);
                            let conge_aprem = conge_bloque_periode(cj.as_ref(), false);

                            let matin_bloque = ferie_matin || conge_matin;
                            let aprem_bloque = ferie_aprem || conge_aprem;
                            let matin_raison_ferie = ferie_matin;
                            let aprem_raison_ferie = ferie_aprem;

                            let largeur = ui.available_width();
                            let bascule_deux_colonnes = largeur >= 420.0;

                            if bascule_deux_colonnes {
                                ui.columns(2, |cols| {
                                    bloc_plage(
                                        &mut cols[0],
                                        &id_matin,
                                        "MATIN",
                                        &mut b.coche_matin,
                                        &mut b.matin_arr,
                                        &mut b.matin_sort,
                                        8,
                                        12,
                                        matin_bloque,
                                        matin_raison_ferie,
                                    );
                                    bloc_plage(
                                        &mut cols[1],
                                        &id_aprem,
                                        "APRÈS-MIDI",
                                        &mut b.coche_aprem,
                                        &mut b.aprem_arr,
                                        &mut b.aprem_sort,
                                        14,
                                        16,
                                        aprem_bloque,
                                        aprem_raison_ferie,
                                    );
                                });
                            } else {
                                bloc_plage(
                                    ui,
                                    &id_matin,
                                    "MATIN",
                                    &mut b.coche_matin,
                                    &mut b.matin_arr,
                                    &mut b.matin_sort,
                                    8,
                                    12,
                                    matin_bloque,
                                    matin_raison_ferie,
                                );
                                ui.add_space(8.0);
                                bloc_plage(
                                    ui,
                                    &id_aprem,
                                    "APRÈS-MIDI",
                                    &mut b.coche_aprem,
                                    &mut b.aprem_arr,
                                    &mut b.aprem_sort,
                                    14,
                                    16,
                                    aprem_bloque,
                                    aprem_raison_ferie,
                                );
                            }

                            ui.add_space(12.0);
                            theme::separateur(ui);
                            ui.add_space(10.0);

                            let largeur_totale = ui.available_width();
                            let largeur_boutons = 44.0 * 2.0 + 10.0 + 16.0;
                            let largeur_min_obs = 140.0;
                            let place_pour_ligne_obs_et_boutons =
                                largeur_totale >= (largeur_min_obs + largeur_boutons);

                            if place_pour_ligne_obs_et_boutons {
                                ui.horizontal(|ui| {
                                    ui.vertical(|ui| {
                                        ui.label(
                                            egui::RichText::new("Observation")
                                                .size(11.5)
                                                .color(theme::TEXTE_SECONDAIRE)
                                                .strong(),
                                        );
                                        ui.add_space(4.0);
                                        let largeur_champ =
                                            (ui.available_width() - largeur_boutons - 8.0)
                                                .max(largeur_min_obs);
                                        let id_obs =
                                            egui::Id::new(format!("obs_{}", p.id));
                                        ui.add(
                                            egui::TextEdit::singleline(&mut b.observation)
                                                .id(id_obs)
                                                .desired_width(largeur_champ)
                                                .margin(egui::vec2(10.0, 8.0)),
                                        );
                                    });

                                    ui.add_space(8.0);

                                    ui.with_layout(
                                        egui::Layout::right_to_left(egui::Align::Center),
                                        |ui| {
                                            if theme::bouton_image_seule(
                                                ui,
                                                "conge.png",
                                                44.0,
                                                22.0,
                                            ) {
                                                action_ouvrir_conge =
                                                    Some((p.id, p.nom_complet()));
                                            }

                                            ui.add_space(10.0);

                                            let rien_a_sauver = !b.coche_matin
                                                && !b.coche_aprem
                                                && b.observation.trim().is_empty();
                                            let bouton_actif = est_pointe || !rien_a_sauver;
                                            let img = if est_pointe {
                                                "non-enregistrer.png"
                                            } else {
                                                "enregistrer.png"
                                            };
                                            let clic = if bouton_actif {
                                                theme::bouton_image_seule(
                                                    ui,
                                                    img,
                                                    44.0,
                                                    22.0,
                                                )
                                            } else {
                                                bouton_image_seule_desactive(
                                                    ui, img, 44.0, 22.0,
                                                );
                                                false
                                            };
                                            if clic {
                                                if est_pointe {
                                                    action_supprimer = Some(p.id);
                                                } else {
                                                    action_enregistrer = Some(p.id);
                                                }
                                            }
                                        },
                                    );
                                });
                            } else {
                                ui.vertical(|ui| {
                                    ui.label(
                                        egui::RichText::new("Observation")
                                            .size(11.5)
                                            .color(theme::TEXTE_SECONDAIRE)
                                            .strong(),
                                    );
                                    ui.add_space(4.0);
                                    let id_obs = egui::Id::new(format!("obs_{}", p.id));
                                    ui.add(
                                        egui::TextEdit::singleline(&mut b.observation)
                                            .id(id_obs)
                                            .desired_width(ui.available_width())
                                            .margin(egui::vec2(10.0, 8.0)),
                                    );
                                    ui.add_space(8.0);
                                    ui.horizontal(|ui| {
                                        ui.with_layout(
                                            egui::Layout::right_to_left(egui::Align::Center),
                                            |ui| {
                                                if theme::bouton_image_seule(
                                                    ui,
                                                    "conge.png",
                                                    44.0,
                                                    22.0,
                                                ) {
                                                    action_ouvrir_conge =
                                                        Some((p.id, p.nom_complet()));
                                                }
                                                ui.add_space(10.0);
                                                let rien_a_sauver = !b.coche_matin
                                                    && !b.coche_aprem
                                                    && b.observation.trim().is_empty();
                                                let bouton_actif =
                                                    est_pointe || !rien_a_sauver;
                                                let img = if est_pointe {
                                                    "non-enregistrer.png"
                                                } else {
                                                    "enregistrer.png"
                                                };
                                                let clic = if bouton_actif {
                                                    theme::bouton_image_seule(
                                                        ui, img, 44.0, 22.0,
                                                    )
                                                } else {
                                                    bouton_image_seule_desactive(
                                                        ui, img, 44.0, 22.0,
                                                    );
                                                    false
                                                };
                                                if clic {
                                                    if est_pointe {
                                                        action_supprimer = Some(p.id);
                                                    } else {
                                                        action_enregistrer = Some(p.id);
                                                    }
                                                }
                                            },
                                        );
                                    });
                                });
                            }
                        }
                    });

                ui.add_space(12.0);
            }
        });

    if let Some((id, nom)) = action_ouvrir_conge {
        let t = state.date_selectionnee;
        let mut dialog = CongeDialog::default();
        dialog.ouvert = true;
        dialog.personnel_id = Some(id);
        dialog.nom_personnel = nom;
        dialog.debut = t;
        dialog.fin = t;
        state.dialog_conge = dialog;
    }

    if let Some(id) = action_supprimer {
        let _ = db.supprimer_pointage(id, state.date_selectionnee);
        state.pointages_jour.retain(|x| *x != id);
        state.cache_mois = None;
        state.cache_date_pointages = None;
        state.brouillons.remove(&id);
        state.rafraichir = true;
    }

    if let Some(id) = action_enregistrer {
        if let Some(b) = state.brouillons.get(&id).cloned() {
            let ferie_matin = ferie_bloque_periode(ferie_jour.as_deref(), true);
            let ferie_aprem = ferie_bloque_periode(ferie_jour.as_deref(), false);
            let cj = state.conges_jour.get(&id).cloned();
            let conge_matin = conge_bloque_periode(cj.as_ref(), true);
            let conge_aprem = conge_bloque_periode(cj.as_ref(), false);

            let matin_bloque = ferie_matin || conge_matin;
            let aprem_bloque = ferie_aprem || conge_aprem;

            let ma = if b.coche_matin && !matin_bloque {
                b.matin_arr
            } else {
                None
            };
            let ms = if b.coche_matin && !matin_bloque {
                b.matin_sort
            } else {
                None
            };
            let aa = if b.coche_aprem && !aprem_bloque {
                b.aprem_arr
            } else {
                None
            };
            let as_ = if b.coche_aprem && !aprem_bloque {
                b.aprem_sort
            } else {
                None
            };
            let obs = if b.observation.trim().is_empty() {
                None
            } else {
                Some(b.observation.trim().to_string())
            };

            let vide =
                ma.is_none() && ms.is_none() && aa.is_none() && as_.is_none() && obs.is_none();

            if vide {
                let _ = db.supprimer_pointage(id, state.date_selectionnee);
                state.pointages_jour.retain(|x| *x != id);
            } else {
                let _ = db.enregistrer_pointage(id, state.date_selectionnee, ma, ms, aa, as_, obs);
                if !state.pointages_jour.contains(&id) {
                    state.pointages_jour.push(id);
                }
            }
            state.cache_mois = None;
            state.cache_date_pointages = None;
            state.brouillons.remove(&id);
            state.rafraichir = true;
        }
    }
}

fn en_conge_entier(state: &PointageState, pid: i64) -> bool {
    state
        .conges_jour
        .get(&pid)
        .map(|c| c.matin && c.aprem)
        .unwrap_or(false)
}

fn bouton_filtre(
    ui: &mut egui::Ui,
    texte: &str,
    couleur: egui::Color32,
    actif: bool,
) -> bool {
    let texte_final = if actif {
        egui::RichText::new(texte)
            .size(12.5)
            .color(egui::Color32::WHITE)
            .strong()
    } else {
        egui::RichText::new(texte)
            .size(12.5)
            .color(couleur)
            .strong()
    };

    let fond = if actif {
        couleur
    } else {
        egui::Color32::WHITE
    };
    let bordure = if actif {
        couleur
    } else {
        theme::BORDURE
    };

    let bouton = egui::Button::new(texte_final)
        .fill(fond)
        .stroke(egui::Stroke::new(1.0_f32, bordure))
        .rounding(theme::r_sm())
        .min_size(egui::vec2(0.0, 26.0));

    ui.add(bouton).clicked()
}

fn bouton_image_seule_desactive(
    ui: &mut egui::Ui,
    fichier_image: &str,
    taille: f32,
    taille_icone: f32,
) {
    let ctx = ui.ctx().clone();
    let (rect, _) = ui.allocate_exact_size(egui::vec2(taille, taille), egui::Sense::hover());

    let painter = ui.painter();
    painter.rect_filled(rect, theme::r_md(), theme::FOND);
    painter.rect_stroke(rect, theme::r_md(), egui::Stroke::new(1.0_f32, theme::BORDURE));

    if let Some(tex) = theme::image(&ctx, fichier_image) {
        let img_rect =
            egui::Rect::from_center_size(rect.center(), egui::vec2(taille_icone, taille_icone));
        painter.image(
            tex.id(),
            img_rect,
            egui::Rect::from_min_max(egui::pos2(0.0, 0.0), egui::pos2(1.0, 1.0)),
            egui::Color32::from_white_alpha(80),
        );
    }
}

fn bloc_plage(
    ui: &mut egui::Ui,
    id_unique: &str,
    titre: &str,
    coche: &mut bool,
    arrivee: &mut Option<NaiveTime>,
    sortie: &mut Option<NaiveTime>,
    h_defaut_arr: u32,
    h_defaut_sort: u32,
    bloque: bool,
    bloque_par_ferie: bool,
) {
    ui.push_id(id_unique, |ui| {
        let fond = if bloque {
            egui::Color32::from_rgb(245, 245, 245)
        } else if titre == "MATIN" {
            egui::Color32::from_rgb(245, 250, 255)
        } else {
            egui::Color32::from_rgb(255, 250, 245)
        };

        egui::Frame::none()
            .fill(fond)
            .rounding(theme::r_md())
            .inner_margin(egui::Margin::same(12.0))
            .show(ui, |ui| {
                ui.set_width(ui.available_width());
                ui.set_max_width(ui.available_width());

                if bloque {
                    if *coche {
                        *coche = false;
                        *arrivee = None;
                        *sortie = None;
                    }

                    ui.horizontal_wrapped(|ui| {
                        ui.label(
                            egui::RichText::new(titre)
                                .size(11.5)
                                .color(theme::TEXTE_SECONDAIRE)
                                .strong(),
                        );
                        let lbl = if bloque_par_ferie { "Férié" } else { "Congé" };
                        ui.label(
                            egui::RichText::new(lbl)
                                .size(11.0)
                                .color(theme::WARNING)
                                .strong(),
                        );
                    });
                    ui.add_space(8.0);
                    let txt = if bloque_par_ferie {
                        "Pointage bloqué (jour férié)"
                    } else {
                        "Pointage bloqué (congé)"
                    };
                    ui.label(
                        egui::RichText::new(txt)
                            .size(11.5)
                            .color(theme::TEXTE_SECONDAIRE)
                            .italics(),
                    );
                    return;
                }

                let coche_avant = *coche;

                ui.horizontal(|ui| {
                    ui.label(
                        egui::RichText::new(titre)
                            .size(11.5)
                            .color(theme::PRIMAIRE)
                            .strong(),
                    );
                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        ui.checkbox(coche, "Pointer");
                    });
                });

                let coche_apres = *coche;

                if coche_apres && !coche_avant {
                    if arrivee.is_none() {
                        *arrivee = NaiveTime::from_hms_opt(h_defaut_arr, 0, 0);
                    }
                    if sortie.is_none() {
                        *sortie = NaiveTime::from_hms_opt(h_defaut_sort, 0, 0);
                    }
                }

                if !coche_apres && coche_avant {
                    *arrivee = None;
                    *sortie = None;
                }

                ui.add_space(8.0);

                let h_init_arr = arrivee.map(|t| t.hour()).unwrap_or(h_defaut_arr);
                let m_init_arr = arrivee.map(|t| t.minute()).unwrap_or(0);
                let h_init_sor = sortie.map(|t| t.hour()).unwrap_or(h_defaut_sort);
                let m_init_sor = sortie.map(|t| t.minute()).unwrap_or(0);

                let mut h = h_init_arr;
                let mut m = m_init_arr;
                let mut h2 = h_init_sor;
                let mut m2 = m_init_sor;

                ui.horizontal_wrapped(|ui| {
                    ui.label(
                        egui::RichText::new("Arrivée")
                            .size(11.0)
                            .color(theme::TEXTE_SECONDAIRE)
                            .strong(),
                    );
                    ui.add_space(4.0);
                    let ch1 = ui
                        .add(
                            egui::DragValue::new(&mut h)
                                .range(0..=23)
                                .speed(1)
                                .custom_formatter(|n, _| format!("{:02}", n as i64)),
                        )
                        .changed();
                    let ch2 = ui
                        .add(
                            egui::DragValue::new(&mut m)
                                .range(0..=59)
                                .speed(1)
                                .custom_formatter(|n, _| format!("{:02}", n as i64)),
                        )
                        .changed();
                    if ch1 || ch2 {
                        if let Some(t) = NaiveTime::from_hms_opt(h, m, 0) {
                            *arrivee = Some(t);
                            *coche = true;
                        }
                    }

                    ui.add_space(10.0);

                    ui.label(
                        egui::RichText::new("Sortie")
                            .size(11.0)
                            .color(theme::TEXTE_SECONDAIRE)
                            .strong(),
                    );
                    ui.add_space(4.0);
                    let ch3 = ui
                        .add(
                            egui::DragValue::new(&mut h2)
                                .range(0..=23)
                                .speed(1)
                                .custom_formatter(|n, _| format!("{:02}", n as i64)),
                        )
                        .changed();
                    let ch4 = ui
                        .add(
                            egui::DragValue::new(&mut m2)
                                .range(0..=59)
                                .speed(1)
                                .custom_formatter(|n, _| format!("{:02}", n as i64)),
                        )
                        .changed();
                    if ch3 || ch4 {
                        if let Some(t) = NaiveTime::from_hms_opt(h2, m2, 0) {
                            *sortie = Some(t);
                            *coche = true;
                        }
                    }
                });
            });
    });
}

fn precedente(d: NaiveDate) -> (i32, u32) {
    if d.month() == 1 {
        (d.year() - 1, 12)
    } else {
        (d.year(), d.month() - 1)
    }
}

fn suivante(d: NaiveDate) -> (i32, u32) {
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
