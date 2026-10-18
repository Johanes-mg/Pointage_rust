use crate::database::Database;
use crate::models::Utilisateur;
use crate::theme;
use rfd::FileDialog;

pub struct CompteState {
    pub nom_complet: String,
    pub ancien_mdp: String,
    pub nouveau_mdp: String,
    pub confirmation: String,
    pub message_profil: String,
    pub message_profil_ok: bool,
    pub message_mdp: String,
    pub message_mdp_ok: bool,
    pub message_bdd: String,
    pub message_bdd_ok: bool,
}

impl CompteState {
    pub fn new(u: &Utilisateur) -> Self {
        Self {
            nom_complet: u.nom_complet.clone(),
            ancien_mdp: String::new(),
            nouveau_mdp: String::new(),
            confirmation: String::new(),
            message_profil: String::new(),
            message_profil_ok: true,
            message_mdp: String::new(),
            message_mdp_ok: true,
            message_bdd: String::new(),
            message_bdd_ok: true,
        }
    }
}

pub fn afficher(
    ui: &mut egui::Ui,
    db: &Database,
    utilisateur: &Utilisateur,
    state: &mut CompteState,
) {
    ui.add_space(8.0);

    let mut enter_profil = false;

    ui.horizontal(|ui| {
        ui.vertical(|ui| {
            ui.label(
                egui::RichText::new("Nom complet")
                    .size(12.0)
                    .color(theme::TEXTE_SECONDAIRE)
                    .strong(),
            );
            ui.add_space(4.0);
            let reponse = ui.add(
                egui::TextEdit::singleline(&mut state.nom_complet)
                    .id(egui::Id::new("compte_nom_complet"))
                    .desired_width(400.0)
                    .margin(egui::vec2(12.0, 9.0)),
            );
            if reponse.lost_focus() && ui.input(|i| i.key_pressed(egui::Key::Enter)) {
                enter_profil = true;
            }
        });

        ui.add_space(16.0);

        ui.vertical(|ui| {
            ui.add_space(22.0);
            if theme::bouton_image_texte(
                ui,
                "enregistrer_compte.png",
                "Enregistrer",
                180.0,
                42.0,
            ) {
                enter_profil = true;
            }
        });
    });

    if enter_profil {
        if state.nom_complet.trim().is_empty() {
            state.message_profil = "Le nom complet est obligatoire.".into();
            state.message_profil_ok = false;
        } else {
            match db.mettre_a_jour_nom_complet(utilisateur.id, state.nom_complet.trim()) {
                Ok(_) => {
                    state.message_profil = "Profil mis à jour.".into();
                    state.message_profil_ok = true;
                }
                Err(e) => {
                    state.message_profil = format!("Erreur : {}", e);
                    state.message_profil_ok = false;
                }
            }
        }
    }

    if !state.message_profil.is_empty() {
        ui.add_space(10.0);
        theme::message_inline(ui, &state.message_profil, state.message_profil_ok);
    }

    ui.add_space(28.0);

    ui.label(
        egui::RichText::new("Changer le mot de passe")
            .size(15.0)
            .color(theme::PRIMAIRE)
            .strong(),
    );
    ui.add_space(10.0);
    theme::separateur(ui);
    ui.add_space(10.0);

    ui.label(
        egui::RichText::new("Ancien mot de passe")
            .size(12.0)
            .color(theme::TEXTE_SECONDAIRE)
            .strong(),
    );
    ui.add_space(4.0);
    let ancien_reponse = ui.add(
        egui::TextEdit::singleline(&mut state.ancien_mdp)
            .id(egui::Id::new("compte_ancien_mdp"))
            .password(true)
            .desired_width(400.0)
            .margin(egui::vec2(12.0, 9.0)),
    );
    if ancien_reponse.lost_focus() && ui.input(|i| i.key_pressed(egui::Key::Enter)) {
        ui.memory_mut(|m| m.request_focus(egui::Id::new("compte_nouveau_mdp")));
    }
    ui.add_space(12.0);

    ui.horizontal(|ui| {
        let largeur = (ui.available_width().min(820.0) - 12.0) / 2.0;

        ui.vertical(|ui| {
            ui.label(
                egui::RichText::new("Nouveau mot de passe")
                    .size(12.0)
                    .color(theme::TEXTE_SECONDAIRE)
                    .strong(),
            );
            ui.add_space(4.0);
            let reponse = ui.add(
                egui::TextEdit::singleline(&mut state.nouveau_mdp)
                    .id(egui::Id::new("compte_nouveau_mdp"))
                    .password(true)
                    .desired_width(largeur)
                    .margin(egui::vec2(12.0, 9.0)),
            );
            if reponse.lost_focus() && ui.input(|i| i.key_pressed(egui::Key::Enter)) {
                ui.memory_mut(|m| m.request_focus(egui::Id::new("compte_confirmation")));
            }
        });

        ui.add_space(12.0);

        ui.vertical(|ui| {
            ui.label(
                egui::RichText::new("Confirmer le nouveau mot de passe")
                    .size(12.0)
                    .color(theme::TEXTE_SECONDAIRE)
                    .strong(),
            );
            ui.add_space(4.0);
            let reponse = ui.add(
                egui::TextEdit::singleline(&mut state.confirmation)
                    .id(egui::Id::new("compte_confirmation"))
                    .password(true)
                    .desired_width(largeur)
                    .margin(egui::vec2(12.0, 9.0)),
            );
            if reponse.lost_focus() && ui.input(|i| i.key_pressed(egui::Key::Enter)) {
                valider_mdp(db, utilisateur, state);
            }
        });
    });

    ui.add_space(16.0);

    if theme::bouton_image_texte(ui, "pass.png", "Changer le mot de passe", 280.0, 42.0) {
        valider_mdp(db, utilisateur, state);
    }

    if !state.message_mdp.is_empty() {
        ui.add_space(10.0);
        theme::message_inline(ui, &state.message_mdp, state.message_mdp_ok);
    }

    ui.add_space(28.0);

    theme::separateur(ui);
    ui.add_space(10.0);

    ui.label(
        egui::RichText::new("Base de données de ce compte :")
            .size(12.0)
            .color(theme::TEXTE_SECONDAIRE)
            .strong(),
    );
    ui.add_space(4.0);
    ui.label(
        egui::RichText::new(db.chemin_base_actuel())
            .size(11.5)
            .color(theme::TEXTE),
    );
    ui.add_space(14.0);

    ui.horizontal(|ui| {
        if theme::bouton_image_texte(ui, "exporter.png", "Exporter la base (.db)", 280.0, 42.0) {
            if let Some(chemin) = FileDialog::new()
                .set_title("Exporter la base de données")
                .add_filter("Base SQLite", &["db"])
                .set_file_name(format!(
                    "pointage_{}_{}.db",
                    utilisateur.nom_utilisateur,
                    chrono::Local::now().format("%Y-%m-%d")
                ))
                .save_file()
            {
                match db.exporter_base(&chemin.to_string_lossy()) {
                    Ok(_) => {
                        state.message_bdd = format!("Base exportée : {}", chemin.display());
                        state.message_bdd_ok = true;
                    }
                    Err(e) => {
                        state.message_bdd = format!("Erreur : {}", e);
                        state.message_bdd_ok = false;
                    }
                }
            }
        }
        ui.add_space(12.0);
        if theme::bouton_image_texte(ui, "ajouter.png", "Importer une base (.db)", 280.0, 42.0) {
            if let Some(chemin) = FileDialog::new()
                .set_title("Sélectionner une base à importer")
                .add_filter("Base SQLite", &["db"])
                .pick_file()
            {
                match db.importer_base(&chemin.to_string_lossy()) {
                    Ok(_) => {
                        state.message_bdd = "Base importée. Redémarrage nécessaire.".into();
                        state.message_bdd_ok = true;
                    }
                    Err(e) => {
                        state.message_bdd = format!("Erreur : {}", e);
                        state.message_bdd_ok = false;
                    }
                }
            }
        }
    });

    if !state.message_bdd.is_empty() {
        ui.add_space(10.0);
        theme::message_inline(ui, &state.message_bdd, state.message_bdd_ok);
    }

    ui.add_space(12.0);
    ui.label(
        egui::RichText::new(
            "Attention : l'import remplace TOUTES les données actuelles. Un redémarrage est nécessaire.",
        )
        .size(11.5)
        .color(theme::DANGER)
        .strong(),
    );
}

fn valider_mdp(db: &Database, utilisateur: &Utilisateur, state: &mut CompteState) {
    state.message_mdp.clear();
    if state.ancien_mdp.is_empty()
        || state.nouveau_mdp.is_empty()
        || state.confirmation.is_empty()
    {
        state.message_mdp = "Tous les champs sont obligatoires.".into();
        state.message_mdp_ok = false;
    } else if state.nouveau_mdp.len() < 4 {
        state.message_mdp = "Le nouveau mot de passe doit contenir au moins 4 caractères.".into();
        state.message_mdp_ok = false;
    } else if state.nouveau_mdp != state.confirmation {
        state.message_mdp = "Les deux mots de passe ne correspondent pas.".into();
        state.message_mdp_ok = false;
    } else {
        match db.changer_mot_de_passe(utilisateur.id, &state.ancien_mdp, &state.nouveau_mdp) {
            Ok(true) => {
                state.message_mdp = "Mot de passe changé avec succès.".into();
                state.message_mdp_ok = true;
                state.ancien_mdp.clear();
                state.nouveau_mdp.clear();
                state.confirmation.clear();
            }
            Ok(false) => {
                state.message_mdp = "L'ancien mot de passe est incorrect.".into();
                state.message_mdp_ok = false;
            }
            Err(e) => {
                state.message_mdp = format!("Erreur : {}", e);
                state.message_mdp_ok = false;
            }
        }
    }
}
