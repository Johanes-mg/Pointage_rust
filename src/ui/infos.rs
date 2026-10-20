use crate::theme;

pub struct InfosState {
    _prive: (),
}

impl InfosState {
    pub fn new() -> Self {
        Self { _prive: () }
    }
}

pub fn afficher(ui: &mut egui::Ui, _state: &mut InfosState) {
    ui.label(
        egui::RichText::new("Instructions & Barème de Notation")
            .size(26.0)
            .color(theme::PRIMAIRE)
            .strong(),
    );
    ui.add_space(4.0);
    ui.label(
        egui::RichText::new(
            "Comment chaque journée de pointage est évaluée sur 20 points.",
        )
        .size(13.5)
        .color(theme::TEXTE_SECONDAIRE),
    );

    ui.add_space(28.0);

    titre_section(ui, "Règle générale");
    ui.add_space(10.0);
    paragraphe(
        ui,
        "Chaque journée est notée sur 20 points : 10 points pour la matinée, \
         10 points pour l'après-midi. La note du jour est la somme des deux.",
    );
    ui.add_space(6.0);
    paragraphe_doux(
        ui,
        "Les jours fériés (matin, après-midi ou jour entier) ne sont pas \
         comptabilisés dans la moyenne mensuelle.",
    );

    ui.add_space(28.0);

    titre_section(ui, "Matinée — 10 points");
    ui.add_space(10.0);
    ligne_regle(
        ui,
        "+ 5 points",
        theme::SUCCES,
        "Arrivée à 08h15 ou avant.",
    );
    ui.add_space(6.0);
    ligne_regle(
        ui,
        "+ 5 points",
        theme::SUCCES,
        "Départ à 11h45 ou après.",
    );
    ui.add_space(10.0);
    paragraphe_doux(
        ui,
        "Une seule condition remplie : 5/10. Aucune : 0/10.",
    );

    ui.add_space(28.0);

    titre_section(ui, "Après-midi — 10 points");
    ui.add_space(10.0);
    ligne_regle(
        ui,
        "+ 5 points",
        theme::SUCCES,
        "Arrivée à 14h15 ou avant.",
    );
    ui.add_space(6.0);
    ligne_regle(
        ui,
        "+ 5 points",
        theme::SUCCES,
        "Départ à 15h45 ou après.",
    );
    ui.add_space(10.0);
    paragraphe_doux(
        ui,
        "Même logique : une seule condition remplie : 5/10. Aucune : 0/10.",
    );

    ui.add_space(28.0);

    titre_section(ui, "Total du jour — sur 20");
    ui.add_space(10.0);
    paragraphe(
        ui,
        "Note du jour = (note matin /10) + (note après-midi /10).",
    );

    ui.add_space(16.0);

    ligne_seuil(
        ui,
        "16 / 20 et plus",
        theme::SUCCES,
        "Bien — affiché en vert dans le bilan.",
    );
    ui.add_space(8.0);
    ligne_seuil(
        ui,
        "10 à 15,9 / 20",
        theme::WARNING,
        "Moyen — affiché en orange dans le bilan.",
    );
    ui.add_space(8.0);
    ligne_seuil(
        ui,
        "Moins de 10 / 20",
        theme::DANGER,
        "Mauvais — affiché en rouge dans le bilan.",
    );

    ui.add_space(40.0);

    ui.with_layout(egui::Layout::right_to_left(egui::Align::TOP), |ui| {
        ui.label(
            egui::RichText::new("© Johanes-mg		")
                .size(11.0)
                .color(theme::TEXTE_SECONDAIRE),
        );
    });
}

fn titre_section(ui: &mut egui::Ui, texte: &str) {
    ui.horizontal(|ui| {
        ui.label(
            egui::RichText::new(texte)
                .size(16.0)
                .color(theme::PRIMAIRE)
                .strong(),
        );
        let largeur_dispo = ui.available_width();
        if largeur_dispo > 8.0 {
            ui.add_space(10.0);
            let (rect, _) = ui.allocate_exact_size(
                egui::vec2(largeur_dispo - 10.0, 1.0),
                egui::Sense::hover(),
            );
            ui.painter().rect_filled(rect, 0.0, theme::BORDURE);
        }
    });
}

fn paragraphe(ui: &mut egui::Ui, texte: &str) {
    ui.label(
        egui::RichText::new(texte)
            .size(13.5)
            .color(theme::TEXTE),
    );
}

fn paragraphe_doux(ui: &mut egui::Ui, texte: &str) {
    ui.label(
        egui::RichText::new(texte)
            .size(12.5)
            .color(theme::TEXTE_SECONDAIRE),
    );
}

fn ligne_regle(
    ui: &mut egui::Ui,
    gain: &str,
    couleur: egui::Color32,
    texte: &str,
) {
    ui.horizontal(|ui| {
        let (rect, _) = ui.allocate_exact_size(
            egui::vec2(10.0, 10.0),
            egui::Sense::hover(),
        );
        ui.painter().circle_filled(rect.center(), 5.0, couleur);
        ui.add_space(10.0);

        ui.add_sized(
            [90.0, 22.0],
            egui::Label::new(
                egui::RichText::new(gain)
                    .size(13.5)
                    .color(couleur)
                    .strong(),
            ),
        );
        ui.add_space(6.0);

        ui.add(
            egui::Label::new(
                egui::RichText::new(texte)
                    .size(13.5)
                    .color(theme::TEXTE),
            )
            .wrap(),
        );
    });
}

fn ligne_seuil(
    ui: &mut egui::Ui,
    seuil: &str,
    couleur: egui::Color32,
    texte: &str,
) {
    ui.horizontal(|ui| {
        let (rect, _) = ui.allocate_exact_size(
            egui::vec2(10.0, 10.0),
            egui::Sense::hover(),
        );
        ui.painter().circle_filled(rect.center(), 5.0, couleur);
        ui.add_space(10.0);

        ui.add_sized(
            [140.0, 22.0],
            egui::Label::new(
                egui::RichText::new(seuil)
                    .size(13.0)
                    .color(couleur)
                    .strong(),
            ),
        );
        ui.add_space(6.0);

        ui.add(
            egui::Label::new(
                egui::RichText::new(texte)
                    .size(13.0)
                    .color(theme::TEXTE),
            )
            .wrap(),
        );
    });
}
