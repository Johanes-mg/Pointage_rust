use crate::database::Database;
use crate::models::Utilisateur;
use crate::theme;

pub struct LoginState {
    pub nom_complet: String,
    pub utilisateur: String,
    pub mot_de_passe: String,
    pub confirmation: String,
    pub mode_creation: bool,
    pub message: String,
    pub message_ok: bool,
    pub afficher_mdp: bool,
    pub afficher_confirm: bool,
    pub premier_affichage: bool,
}

impl LoginState {
    pub fn new(db: &Database) -> Self {
        let premier = db.aucun_utilisateur().unwrap_or(true);
        Self {
            nom_complet: String::new(),
            utilisateur: String::new(),
            mot_de_passe: String::new(),
            confirmation: String::new(),
            mode_creation: premier,
            message: String::new(),
            message_ok: false,
            afficher_mdp: false,
            afficher_confirm: false,
            premier_affichage: true,
        }
    }
}

pub enum LoginAction {
    Aucune,
    Connecte(Utilisateur),
}

pub fn afficher(ctx: &egui::Context, db: &Database, state: &mut LoginState) -> LoginAction {
    let mut action = LoginAction::Aucune;

    egui::CentralPanel::default()
        .frame(egui::Frame::none().fill(theme::FOND_CARTE))
        .show(ctx, |ui| {
            let taille = ui.available_size();
            let largeur_gauche = (taille.x * 0.20).max(220.0);
            let largeur_droite = taille.x - largeur_gauche;

            ui.horizontal(|ui| {
                ui.spacing_mut().item_spacing.x = 0.0;

                panneau_gauche(ui, largeur_gauche, taille.y);
                panneau_droit(ui, db, state, &mut action, largeur_droite, taille.y);
            });
        });

    state.premier_affichage = false;
    action
}

fn panneau_gauche(ui: &mut egui::Ui, largeur: f32, hauteur: f32) {
    egui::Frame::none()
        .fill(theme::PRIMAIRE)
        .show(ui, |ui| {
            ui.allocate_ui_with_layout(
                egui::vec2(largeur, hauteur),
                egui::Layout::top_down(egui::Align::Center),
                |ui| {
                    ui.set_min_size(egui::vec2(largeur, hauteur));

                    let largeur_utile = (largeur - 30.0).max(120.0);

                    ui.add_space(30.0);
                    theme::image_ui(ui, "logo.png", 64.0, 64.0);
                    ui.add_space(10.0);
                    ui.label(
                        egui::RichText::new("Pointage")
                            .size(22.0)
                            .color(egui::Color32::WHITE)
                            .strong(),
                    );
                    ui.label(
                        egui::RichText::new("Gestion des présences")
                            .size(11.5)
                            .color(egui::Color32::from_rgb(180, 200, 225)),
                    );

                    ui.add_space(20.0);
                    barre_decorative(ui, 32.0);
                    ui.add_space(20.0);
                    puces_centre(ui, largeur_utile);

                    // Espace flexible pour pousser l'illustration tout en bas
                    let hauteur_restante = (hauteur - 340.0).max(20.0);
                    ui.add_space(hauteur_restante);

                    // Illustration rétablie en bas du panneau gauche
                    theme::image_ui(ui, "illustration.png", 140.0, 90.0);
                    ui.add_space(20.0);
                },
            );
        });
}

fn barre_decorative(ui: &mut egui::Ui, largeur: f32) {
    let (rect, _) = ui.allocate_exact_size(egui::vec2(largeur, 3.0), egui::Sense::hover());
    ui.painter().rect_filled(rect, 1.5, theme::ACCENT);
}

fn puces_centre(ui: &mut egui::Ui, largeur_dispo: f32) {
    let largeur_bloc = largeur_dispo.min(180.0);

    ui.allocate_ui_with_layout(
        egui::vec2(largeur_bloc, 0.0),
        egui::Layout::top_down(egui::Align::LEFT),
        |ui| {
            puce(ui, "Suivi du personnel");
            ui.add_space(6.0);
            puce(ui, "Pointage quotidien");
            ui.add_space(6.0);
            puce(ui, "Bilan mensuel");
            ui.add_space(6.0);
            puce(ui, "Exportation Excel");
        },
    );
}

fn puce(ui: &mut egui::Ui, texte: &str) {
    ui.horizontal(|ui| {
        let (rect, _) = ui.allocate_exact_size(egui::vec2(6.0, 6.0), egui::Sense::hover());
        ui.painter()
            .circle_filled(rect.center(), 3.0, theme::ACCENT);
        ui.add_space(8.0);
        ui.label(
            egui::RichText::new(texte)
                .size(11.0)
                .color(egui::Color32::from_rgb(200, 215, 235)),
        );
    });
}

fn panneau_droit(
    ui: &mut egui::Ui,
    db: &Database,
    state: &mut LoginState,
    action: &mut LoginAction,
    largeur: f32,
    hauteur: f32,
) {
    let largeur_formulaire = 380.0;

    egui::Frame::none().show(ui, |ui| {
        ui.allocate_ui_with_layout(
            egui::vec2(largeur, hauteur),
            egui::Layout::top_down(egui::Align::Center),
            |ui| {
                ui.set_min_size(egui::vec2(largeur, hauteur));

                egui::ScrollArea::vertical()
                    .id_source("login_droit_scroll")
                    .auto_shrink([false, false])
                    .show(ui, |ui| {
                        ui.vertical_centered(|ui| {
                            let hauteur_estimee = if state.mode_creation { 480.0 } else { 360.0 };
                            ui.add_space((hauteur - hauteur_estimee).max(30.0) / 2.0);

                            ui.allocate_ui_with_layout(
                                egui::vec2(largeur_formulaire, 0.0),
                                egui::Layout::top_down(egui::Align::Center),
                                |ui| {
                                    formulaire(ui, db, state, action, largeur_formulaire);
                                },
                            );
                        });
                    });
            },
        );
    });
}

fn formulaire(
    ui: &mut egui::Ui,
    db: &Database,
    state: &mut LoginState,
    action: &mut LoginAction,
    largeur_formulaire: f32,
) {
    let titre = if state.mode_creation {
        "Créer un compte"
    } else {
        "Connexion"
    };
    ui.label(
        egui::RichText::new(titre)
            .size(26.0)
            .color(theme::TEXTE)
            .strong(),
    );
    ui.add_space(6.0);
    ui.label(
        egui::RichText::new(if state.mode_creation {
            "Remplissez les informations ci-dessous"
        } else {
            "Entrez vos identifiants pour accéder à l'application"
        })
        .size(13.0)
        .color(theme::TEXTE_SECONDAIRE),
    );

    ui.add_space(24.0);

    let mut valider_formulaire = false;

    if state.mode_creation {
        let enter = champ_formulaire(
            ui,
            "Nom complet",
            &mut state.nom_complet,
            false,
            None,
            "nom_complet_field",
            largeur_formulaire,
            state.premier_affichage,
        );
        if enter {
            ui.memory_mut(|m| m.request_focus(egui::Id::new("username_field")));
        }
        ui.add_space(12.0);
    }

    let focus_initial_username = state.premier_affichage && !state.mode_creation;
    let enter_user = champ_formulaire(
        ui,
        "Nom d'utilisateur",
        &mut state.utilisateur,
        false,
        None,
        "username_field",
        largeur_formulaire,
        focus_initial_username || (state.premier_affichage && state.mode_creation),
    );
    if enter_user {
        ui.memory_mut(|m| m.request_focus(egui::Id::new("password_field")));
    }
    ui.add_space(12.0);

    let enter_pass = champ_formulaire(
        ui,
        "Mot de passe",
        &mut state.mot_de_passe,
        true,
        Some(&mut state.afficher_mdp),
        "password_field",
        largeur_formulaire,
        false,
    );

    if enter_pass {
        if state.mode_creation {
            ui.memory_mut(|m| m.request_focus(egui::Id::new("confirm_field")));
        } else {
            valider_formulaire = true;
        }
    }
    ui.add_space(12.0);

    if state.mode_creation {
        let enter_confirm = champ_formulaire(
            ui,
            "Confirmer le mot de passe",
            &mut state.confirmation,
            true,
            Some(&mut state.afficher_confirm),
            "confirm_field",
            largeur_formulaire,
            false,
        );
        if enter_confirm {
            valider_formulaire = true;
        }
        ui.add_space(12.0);
    }

    if valider_formulaire {
        *action = valider(db, state);
    }

    if !state.message.is_empty() {
        let couleur = if state.message_ok {
            theme::SUCCES
        } else {
            theme::DANGER
        };
        ui.label(
            egui::RichText::new(&state.message)
                .size(12.0)
                .color(couleur),
        );
        ui.add_space(8.0);
    }

    ui.add_space(8.0);

    let libelle = if state.mode_creation {
        "Créer le compte"
    } else {
        "Se connecter"
    };
    if bouton_primaire(ui, libelle, largeur_formulaire) {
        *action = valider(db, state);
    }

    ui.add_space(12.0);

    let libelle2 = if state.mode_creation {
        "J'ai déjà un compte"
    } else {
        "Créer un nouveau compte"
    };
    if bouton_secondaire(ui, libelle2, largeur_formulaire) {
        state.mode_creation = !state.mode_creation;
        state.message.clear();
        state.mot_de_passe.clear();
        state.confirmation.clear();
    }
}

fn champ_formulaire(
    ui: &mut egui::Ui,
    label: &str,
    valeur: &mut String,
    password: bool,
    mut afficher: Option<&mut bool>,
    id: &str,
    largeur: f32,
    focus_initial: bool,
) -> bool {
    let mut enter_presse = false;

    ui.allocate_ui_with_layout(
        egui::vec2(largeur, 0.0),
        egui::Layout::top_down(egui::Align::LEFT),
        |ui| {
            ui.label(
                egui::RichText::new(label)
                    .color(theme::TEXTE_SECONDAIRE)
                    .size(12.0)
                    .strong(),
            );
            ui.add_space(4.0);

            if password {
                let afficher_ref = afficher.as_mut().unwrap();
                let largeur_saisie = largeur - 42.0;

                ui.horizontal(|ui| {
                    ui.spacing_mut().item_spacing.x = 6.0;

                    let reponse = ui.add(
                        egui::TextEdit::singleline(valeur)
                            .id(egui::Id::new(id))
                            .password(!**afficher_ref)
                            .desired_width(largeur_saisie)
                            .margin(egui::vec2(10.0, 9.0)),
                    );

                    if focus_initial && !reponse.has_focus() {
                        reponse.request_focus();
                    }

                    if reponse.has_focus() && ui.input(|i| i.key_pressed(egui::Key::Enter)) {
                        enter_presse = true;
                    }

                    let nom = if **afficher_ref {
                        "cacher.png"
                    } else {
                        "non-cacher.png"
                    };
                    if theme::bouton_image_seule(ui, nom, 36.0, 18.0) {
                        **afficher_ref = !**afficher_ref;
                    }
                });
            } else {
                let reponse = ui.add(
                    egui::TextEdit::singleline(valeur)
                        .id(egui::Id::new(id))
                        .desired_width(largeur)
                        .margin(egui::vec2(10.0, 9.0)),
                );

                if focus_initial && !reponse.has_focus() {
                    reponse.request_focus();
                }

                if reponse.has_focus() && ui.input(|i| i.key_pressed(egui::Key::Enter)) {
                    enter_presse = true;
                }
            }
        },
    );

    enter_presse
}

fn bouton_primaire(ui: &mut egui::Ui, texte: &str, largeur: f32) -> bool {
    let b = egui::Button::new(
        egui::RichText::new(texte)
            .color(egui::Color32::WHITE)
            .size(13.5)
            .strong(),
    )
    .fill(theme::ACCENT)
    .stroke(egui::Stroke::NONE)
    .rounding(theme::r_md())
    .min_size(egui::vec2(largeur, 42.0));

    ui.add(b).clicked()
}

fn bouton_secondaire(ui: &mut egui::Ui, texte: &str, largeur: f32) -> bool {
    let b = egui::Button::new(egui::RichText::new(texte).color(theme::TEXTE).size(13.0))
        .fill(egui::Color32::WHITE)
        .stroke(egui::Stroke::new(1.0_f32, theme::BORDURE))
        .rounding(theme::r_md())
        .min_size(egui::vec2(largeur, 42.0));

    ui.add(b).clicked()
}

fn valider(db: &Database, state: &mut LoginState) -> LoginAction {
    state.message.clear();

    if state.utilisateur.trim().is_empty() {
        state.message = "Le nom d'utilisateur est obligatoire.".into();
        state.message_ok = false;
        return LoginAction::Aucune;
    }
    if state.mot_de_passe.is_empty() {
        state.message = "Le mot de passe est obligatoire.".into();
        state.message_ok = false;
        return LoginAction::Aucune;
    }

    if state.mode_creation {
        if state.nom_complet.trim().is_empty() {
            state.message = "Le nom complet est obligatoire.".into();
            state.message_ok = false;
            return LoginAction::Aucune;
        }
        if state.mot_de_passe.len() < 4 {
            state.message = "Le mot de passe doit contenir au moins 4 caractères.".into();
            state.message_ok = false;
            return LoginAction::Aucune;
        }
        if state.mot_de_passe != state.confirmation {
            state.message = "Les deux mots de passe ne correspondent pas.".into();
            state.message_ok = false;
            return LoginAction::Aucune;
        }

        match db.creer_utilisateur(
            state.utilisateur.trim(),
            &state.mot_de_passe,
            state.nom_complet.trim(),
        ) {
            Ok(u) => LoginAction::Connecte(u),
            Err(e) => {
                state.message = format!("Erreur : {}", e);
                state.message_ok = false;
                LoginAction::Aucune
            }
        }
    } else {
        match db.authentifier(state.utilisateur.trim(), &state.mot_de_passe) {
            Ok(Some(u)) => LoginAction::Connecte(u),
            Ok(None) => {
                state.message = "Nom d'utilisateur ou mot de passe incorrect.".into();
                state.message_ok = false;
                LoginAction::Aucune
            }
            Err(e) => {
                state.message = format!("Erreur : {}", e);
                state.message_ok = false;
                LoginAction::Aucune
            }
        }
    }
}
