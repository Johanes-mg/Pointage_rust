use crate::database::Database;
use crate::models::Personnel;
use crate::theme;

pub struct PersonnelState {
    pub nom: String,
    pub prenom: String,
    pub matricule: String,
    pub telephone: String,
    pub adresse: String,
    pub email: String,
    pub statut: String,
    pub grade: String,
    pub situation: String,
    pub selection_id: Option<i64>,
    pub message: String,
    pub message_ok: bool,
    pub cache: Option<Vec<Personnel>>,
    pub premier_affichage: bool,
}

impl PersonnelState {
    pub fn new() -> Self {
        Self {
            nom: String::new(),
            prenom: String::new(),
            matricule: String::new(),
            telephone: String::new(),
            adresse: String::new(),
            email: String::new(),
            statut: String::new(),
            grade: String::new(),
            situation: String::new(),
            selection_id: None,
            message: String::new(),
            message_ok: true,
            cache: None,
            premier_affichage: true,
        }
    }

    pub fn invalider_cache(&mut self) {
        self.cache = None;
    }

    fn vider(&mut self) {
        self.nom.clear();
        self.prenom.clear();
        self.matricule.clear();
        self.telephone.clear();
        self.adresse.clear();
        self.email.clear();
        self.statut.clear();
        self.grade.clear();
        self.situation.clear();
        self.selection_id = None;
    }

    fn remplir(&mut self, p: &Personnel) {
        self.nom = p.nom.clone();
        self.prenom = p.prenom.clone();
        self.matricule = p.matricule.clone().unwrap_or_default();
        self.telephone = p.telephone.clone().unwrap_or_default();
        self.adresse = p.adresse.clone().unwrap_or_default();
        self.email = p.email.clone().unwrap_or_default();
        self.statut = p.statut.clone().unwrap_or_default();
        self.grade = p.grade.clone().unwrap_or_default();
        self.situation = p.situation_familiale.clone().unwrap_or_default();
        self.selection_id = Some(p.id);
    }
}

pub fn afficher(ui: &mut egui::Ui, db: &Database, state: &mut PersonnelState) {
    ui.add_space(8.0);

    let largeur_totale = ui.available_width();
    let largeur_gauche = (largeur_totale * 0.42).max(400.0);
    let largeur_droite = largeur_totale - largeur_gauche - 16.0;

    ui.horizontal_top(|ui| {
        ui.allocate_ui_with_layout(
            egui::vec2(largeur_gauche, ui.available_height()),
            egui::Layout::top_down(egui::Align::LEFT),
            |ui| {
                formulaire_gauche(ui, db, state);
            },
        );

        ui.add_space(16.0);

        ui.allocate_ui_with_layout(
            egui::vec2(largeur_droite, ui.available_height()),
            egui::Layout::top_down(egui::Align::LEFT),
            |ui| {
                table_droite(ui, db, state);
            },
        );
    });

    state.premier_affichage = false;
}

fn formulaire_gauche(ui: &mut egui::Ui, db: &Database, state: &mut PersonnelState) {
    ui.add_space(10.0);
    theme::separateur(ui);
    ui.add_space(10.0);

    let mut enter_final = false;
    let premier = state.premier_affichage;

    ui.horizontal(|ui| {
        let largeur = (ui.available_width() - 12.0) / 2.0;

        if champ_mi_largeur(ui, "Nom *", &mut state.nom, largeur, "nom_field", premier) {
            ui.memory_mut(|m| m.request_focus(egui::Id::new("prenom_field")));
        }
        ui.add_space(12.0);
        if champ_mi_largeur(
            ui,
            "Prénom *",
            &mut state.prenom,
            largeur,
            "prenom_field",
            false,
        ) {
            ui.memory_mut(|m| m.request_focus(egui::Id::new("matricule_field")));
        }
    });
    ui.add_space(10.0);

    ui.horizontal(|ui| {
        let largeur = (ui.available_width() - 12.0) / 2.0;

        if champ_mi_largeur(
            ui,
            "N° Matricule (IM)",
            &mut state.matricule,
            largeur,
            "matricule_field",
            false,
        ) {
            ui.memory_mut(|m| m.request_focus(egui::Id::new("adresse_field")));
        }
        ui.add_space(12.0);
        if champ_mi_largeur(
            ui,
            "Adresse",
            &mut state.adresse,
            largeur,
            "adresse_field",
            false,
        ) {
            ui.memory_mut(|m| m.request_focus(egui::Id::new("telephone_field")));
        }
    });
    ui.add_space(10.0);

    ui.horizontal(|ui| {
        let largeur = (ui.available_width() - 12.0) / 2.0;

        if champ_mi_largeur(
            ui,
            "Téléphone",
            &mut state.telephone,
            largeur,
            "telephone_field",
            false,
        ) {
            ui.memory_mut(|m| m.request_focus(egui::Id::new("email_field")));
        }
        ui.add_space(12.0);
        if champ_mi_largeur(
            ui,
            "Email",
            &mut state.email,
            largeur,
            "email_field",
            false,
        ) {
            ui.memory_mut(|m| m.request_focus(egui::Id::new("statut_field")));
        }
    });
    ui.add_space(10.0);

    ui.horizontal(|ui| {
        let largeur = (ui.available_width() - 12.0) / 2.0;

        if champ_mi_largeur(
            ui,
            "Statut",
            &mut state.statut,
            largeur,
            "statut_field",
            false,
        ) {
            ui.memory_mut(|m| m.request_focus(egui::Id::new("grade_field")));
        }
        ui.add_space(12.0);
        if champ_mi_largeur(
            ui,
            "Grade",
            &mut state.grade,
            largeur,
            "grade_field",
            false,
        ) {
            enter_final = true;
        }
    });
    ui.add_space(10.0);

    ui.label(
        egui::RichText::new("Situation familiale")
            .color(theme::TEXTE_SECONDAIRE)
            .size(12.0)
            .strong(),
    );
    ui.add_space(4.0);
    egui::ComboBox::from_id_source("situation")
        .selected_text(if state.situation.is_empty() {
            "-".to_string()
        } else {
            state.situation.clone()
        })
        .width(ui.available_width())
        .show_ui(ui, |ui| {
            ui.selectable_value(&mut state.situation, String::new(), "-");
            ui.selectable_value(&mut state.situation, "Célibataire".into(), "Célibataire");
            ui.selectable_value(&mut state.situation, "Marié(e)".into(), "Marié(e)");
            ui.selectable_value(&mut state.situation, "Divorcé(e)".into(), "Divorcé(e)");
            ui.selectable_value(&mut state.situation, "Veuf(ve)".into(), "Veuf(ve)");
        });

    ui.add_space(16.0);

    if enter_final {
        if state.selection_id.is_some() {
            enregistrer(db, state, true);
        } else {
            enregistrer(db, state, false);
        }
    }

    ui.horizontal(|ui| {
        let largeur_bouton = (ui.available_width() - 16.0) / 3.0;

        if theme::bouton_image_texte(ui, "ajouter.png", "Ajouter", largeur_bouton, 42.0) {
            enregistrer(db, state, false);
        }
        ui.add_space(8.0);
        if state.selection_id.is_some() {
            if theme::bouton_image_texte(ui, "modifier.png", "Modifier", largeur_bouton, 42.0) {
                enregistrer(db, state, true);
            }
        } else {
            bouton_desactive(ui, "modifier.png", "Modifier", largeur_bouton, 42.0);
        }
        ui.add_space(8.0);
        if state.selection_id.is_some() {
            if theme::bouton_image_texte(ui, "effacer.png", "Supprimer", largeur_bouton, 42.0) {
                if let Some(id) = state.selection_id {
                    let _ = db.supprimer_personnel(id);
                    state.vider();
                    state.invalider_cache();
                    state.message = "Personnel supprimé.".into();
                    state.message_ok = true;
                }
            }
        } else {
            bouton_desactive(ui, "effacer.png", "Supprimer", largeur_bouton, 42.0);
        }
    });

    if !state.message.is_empty() {
        ui.add_space(12.0);
        theme::message_inline(ui, &state.message, state.message_ok);
    }
}

fn table_droite(ui: &mut egui::Ui, db: &Database, state: &mut PersonnelState) {
    if state.cache.is_none() {
        state.cache = Some(db.get_personnel().unwrap_or_default());
    }
    let liste = state.cache.clone().unwrap_or_default();

    ui.horizontal(|ui| {
        ui.label(
            egui::RichText::new("Liste du personnel")
                .size(15.0)
                .color(theme::PRIMAIRE)
                .strong(),
        );
        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
            theme::badge(
                ui,
                &format!("{} MEMBRE(S)", liste.len()),
                theme::FOND_BILAN,
                theme::PRIMAIRE,
            );
        });
    });

    ui.add_space(10.0);
    theme::separateur(ui);
    ui.add_space(6.0);

    if liste.is_empty() {
        ui.vertical_centered(|ui| {
            ui.add_space(40.0);
            ui.label(
                egui::RichText::new("Aucun membre enregistré")
                    .size(13.0)
                    .color(theme::TEXTE_SECONDAIRE),
            );
            ui.add_space(40.0);
        });
        return;
    }

    egui::ScrollArea::vertical()
        .auto_shrink([false, false])
        .show(ui, |ui| {
            egui::Frame::none()
                .fill(theme::FOND)
                .rounding(theme::r_sm())
                .inner_margin(egui::Margin::symmetric(12.0, 10.0))
                .show(ui, |ui| {
                    ui.set_width(ui.available_width());
                    ui.horizontal(|ui| {
                        let cols = ["MATRICULE", "NOM", "PRÉNOM", "TÉLÉPHONE", "EMAIL", "GRADE"];
                        let largs = [90.0, 110.0, 110.0, 130.0, 180.0, 100.0];
                        for (col, l) in cols.iter().zip(largs.iter()) {
                            ui.add_sized(
                                [*l, 20.0],
                                egui::Label::new(
                                    egui::RichText::new(*col)
                                        .size(10.5)
                                        .color(theme::TEXTE_SECONDAIRE)
                                        .strong(),
                                ),
                            );
                        }
                    });
                });

            ui.add_space(4.0);

            let mut action: Option<ActionTable> = None;

            for p in &liste {
                let est_sel = state.selection_id == Some(p.id);
                let bg = if est_sel { theme::FOND_BILAN } else { theme::FOND_CARTE };

                egui::Frame::none()
                    .fill(bg)
                    .rounding(theme::r_sm())
                    .inner_margin(egui::Margin::symmetric(12.0, 10.0))
                    .show(ui, |ui| {
                        ui.set_width(ui.available_width());
                        let resp = ui.horizontal(|ui| {
                            let vals = [
                                p.matricule.as_deref().unwrap_or("-"),
                                &p.nom,
                                &p.prenom,
                                p.telephone.as_deref().unwrap_or("-"),
                                p.email.as_deref().unwrap_or("-"),
                                p.grade.as_deref().unwrap_or("-"),
                            ];
                            let largs = [90.0, 110.0, 110.0, 130.0, 180.0, 100.0];
                            for (val, l) in vals.iter().zip(largs.iter()) {
                                ui.add_sized(
                                    [*l, 20.0],
                                    egui::Label::new(
                                        egui::RichText::new(*val).size(12.0).color(theme::TEXTE),
                                    ),
                                );
                            }
                        });

                        let rect = resp.response.rect;
                        let interact = ui.interact(
                            rect,
                            egui::Id::new(format!("row_{}", p.id)),
                            egui::Sense::click(),
                        );

                        if interact.clicked() {
                            if est_sel {
                                action = Some(ActionTable::Deselectionner);
                            } else {
                                action = Some(ActionTable::Selectionner(p.clone()));
                            }
                        }
                        if interact.hovered() {
                            ui.ctx().set_cursor_icon(egui::CursorIcon::PointingHand);
                        }
                    });

                ui.add_space(2.0);
            }

            match action {
                Some(ActionTable::Selectionner(p)) => {
                    state.remplir(&p);
                    state.message.clear();
                }
                Some(ActionTable::Deselectionner) => {
                    state.vider();
                    state.message.clear();
                }
                None => {}
            }
        });
}

enum ActionTable {
    Selectionner(Personnel),
    Deselectionner,
}

fn champ_mi_largeur(
    ui: &mut egui::Ui,
    label: &str,
    valeur: &mut String,
    largeur: f32,
    id: &str,
    focus_initial: bool,
) -> bool {
    let mut enter = false;
    ui.vertical(|ui| {
        ui.label(
            egui::RichText::new(label)
                .color(theme::TEXTE_SECONDAIRE)
                .size(12.0)
                .strong(),
        );
        ui.add_space(4.0);
        let reponse = ui.add(
            egui::TextEdit::singleline(valeur)
                .id(egui::Id::new(id))
                .desired_width(largeur)
                .margin(egui::vec2(12.0, 9.0)),
        );
        if focus_initial && !reponse.has_focus() {
            reponse.request_focus();
        }
        if reponse.lost_focus() && ui.input(|i| i.key_pressed(egui::Key::Enter)) {
            enter = true;
        }
    });
    enter
}

fn bouton_desactive(ui: &mut egui::Ui, fichier: &str, texte: &str, largeur: f32, hauteur: f32) {
    let ctx = ui.ctx().clone();
    let (rect, _) = ui.allocate_exact_size(egui::vec2(largeur, hauteur), egui::Sense::hover());

    let painter = ui.painter();
    painter.rect_filled(rect, theme::r_md(), theme::FOND);
    painter.rect_stroke(
        rect,
        theme::r_md(),
        egui::Stroke::new(1.0_f32, theme::BORDURE),
    );

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

fn enregistrer(db: &Database, state: &mut PersonnelState, modifier: bool) {
    if state.nom.trim().is_empty() || state.prenom.trim().is_empty() {
        state.message = "Le nom et le prénom sont obligatoires.".into();
        state.message_ok = false;
        return;
    }

    let p = lire(state);

    if modifier {
        if let Some(id) = state.selection_id {
            let mut p_mod = p;
            p_mod.id = id;
            match db.modifier_personnel(&p_mod) {
                Ok(_) => {
                    state.message = "Personnel modifié.".into();
                    state.message_ok = true;
                    state.vider();
                    state.invalider_cache();
                }
                Err(e) => {
                    state.message = format!("Erreur : {}", e);
                    state.message_ok = false;
                }
            }
        }
    } else {
        match db.ajouter_personnel(&p) {
            Ok(_) => {
                state.message = "Personnel ajouté.".into();
                state.message_ok = true;
                state.vider();
                state.invalider_cache();
            }
            Err(e) => {
                state.message = format!("Erreur : {}", e);
                state.message_ok = false;
            }
        }
    }
}

fn lire(state: &PersonnelState) -> Personnel {
    let opt = |s: &String| -> Option<String> {
        if s.trim().is_empty() {
            None
        } else {
            Some(s.trim().to_string())
        }
    };
    Personnel {
        id: 0,
        nom: state.nom.trim().into(),
        prenom: state.prenom.trim().into(),
        matricule: opt(&state.matricule),
        telephone: opt(&state.telephone),
        adresse: opt(&state.adresse),
        email: opt(&state.email),
        statut: opt(&state.statut),
        grade: opt(&state.grade),
        situation_familiale: opt(&state.situation),
    }
}
