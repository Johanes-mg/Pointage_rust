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
        egui::RichText::new("Barème de notation")
            .size(26.0)
            .color(theme::PRIMAIRE)
            .strong(),
    );
    ui.add_space(4.0);
    ui.label(
        egui::RichText::new("Règles appliquées au calcul du bilan mensuel.")
            .size(13.5)
            .color(theme::TEXTE_SECONDAIRE),
    );

    ui.add_space(28.0);
    titre_section(ui, "Principe général");
    ui.add_space(10.0);
    paragraphe(ui, "Chaque journée est notée sur 20 points.");
    ui.add_space(4.0);
    paragraphe(ui, "10 points pour la matinée, 10 points pour l'après-midi.");
    ui.add_space(4.0);
    paragraphe(ui, "La note du jour est la somme des deux.");

    ui.add_space(28.0);
    titre_section(ui, "Matinée (10 points)");
    ui.add_space(10.0);
    ligne_regle(ui, "+ 5 points", theme::SUCCES, "Arrivée à 08h15 ou avant.");
    ui.add_space(6.0);
    ligne_regle(ui, "+ 5 points", theme::SUCCES, "Départ à 11h45 ou après.");
    ui.add_space(10.0);
    paragraphe_doux(
        ui,
        "Une seule condition remplie : 5/10. Aucune condition : 0/10.",
    );

    ui.add_space(28.0);
    titre_section(ui, "Après-midi (10 points)");
    ui.add_space(10.0);
    ligne_regle(ui, "+ 5 points", theme::SUCCES, "Arrivée à 14h15 ou avant.");
    ui.add_space(6.0);
    ligne_regle(ui, "+ 5 points", theme::SUCCES, "Départ à 15h45 ou après.");
    ui.add_space(10.0);
    paragraphe_doux(
        ui,
        "Une seule condition remplie : 5/10. Aucune condition : 0/10.",
    );

    ui.add_space(28.0);
    titre_section(ui, "Jours fériés");
    ui.add_space(10.0);
    paragraphe(ui, "Un jour férié retire la ou les périodes concernées du calcul.");
    ui.add_space(4.0);
    paragraphe_doux(
        ui,
        "Matin : seule la matinée est retirée. La note du jour reste sur 10, ramenée sur 20.",
    );
    ui.add_space(4.0);
    paragraphe_doux(
        ui,
        "Après-midi : seul l'après-midi est retiré. Même principe.",
    );
    ui.add_space(4.0);
    paragraphe_doux(
        ui,
        "Jour entier : la journée entière est exclue du calcul du bilan et du classement.",
    );
    ui.add_space(4.0);
    paragraphe_doux(
        ui,
        "Le pointage est bloqué sur une période fériée dans la page Pointage.",
    );

    ui.add_space(28.0);
    titre_section(ui, "Congés");
    ui.add_space(10.0);
    paragraphe(
        ui,
        "Un congé couvre une plage de dates pour un membre, avec une période précise.",
    );
    ui.add_space(4.0);
    paragraphe_doux(
        ui,
        "Trois périodes disponibles : jour entier, matin, après-midi.",
    );
    ui.add_space(4.0);
    paragraphe_doux(
        ui,
        "Le bouton congé (icône) sur la page Pointage ouvre une fenêtre pour saisir la plage et la période.",
    );
    ui.add_space(4.0);
    paragraphe_doux(
        ui,
        "Pendant un congé, le pointage est bloqué pour la période concernée uniquement.",
    );
    ui.add_space(4.0);
    paragraphe_doux(
        ui,
        "Jour entier : la journée est entièrement exclue du bilan et du classement.",
    );
    ui.add_space(4.0);
    paragraphe_doux(
        ui,
        "Matin : seule la matinée est exclue ; l'après-midi reste pointé et noté sur 10 ramené sur 20.",
    );
    ui.add_space(4.0);
    paragraphe_doux(
        ui,
        "Après-midi : symétriquement, la matinée reste pointée et notée sur 10 ramenée sur 20.",
    );
    ui.add_space(4.0);
    paragraphe_doux(
        ui,
        "Les jours de congé apparaissent en orange dans le bilan personnel.",
    );

    ui.add_space(28.0);
    titre_section(ui, "Note du jour (sur 20)");
    ui.add_space(10.0);
    paragraphe(ui, "Note du jour = note matin + note après-midi, hors périodes fériées et congés.");

    ui.add_space(16.0);
    ligne_seuil(ui, "16 / 20 et plus", theme::SUCCES, "Bien");
    ui.add_space(8.0);
    ligne_seuil(ui, "10 à 15,9 / 20", theme::WARNING, "Moyen");
    ui.add_space(8.0);
    ligne_seuil(ui, "Moins de 10 / 20", theme::DANGER, "Mauvais");

    ui.add_space(40.0);
    ui.with_layout(egui::Layout::right_to_left(egui::Align::TOP), |ui| {
        ui.label(
            egui::RichText::new("© Johanes-mg")
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
    ui.label(egui::RichText::new(texte).size(13.5).color(theme::TEXTE));
}

fn paragraphe_doux(ui: &mut egui::Ui, texte: &str) {
    ui.label(
        egui::RichText::new(texte)
            .size(12.5)
            .color(theme::TEXTE_SECONDAIRE),
    );
}

fn ligne_regle(ui: &mut egui::Ui, gain: &str, couleur: egui::Color32, texte: &str) {
    ui.horizontal(|ui| {
        let (rect, _) = ui.allocate_exact_size(egui::vec2(10.0, 10.0), egui::Sense::hover());
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
            egui::Label::new(egui::RichText::new(texte).size(13.5).color(theme::TEXTE)).wrap(),
        );
    });
}

fn ligne_seuil(ui: &mut egui::Ui, seuil: &str, couleur: egui::Color32, texte: &str) {
    ui.horizontal(|ui| {
        let (rect, _) = ui.allocate_exact_size(egui::vec2(10.0, 10.0), egui::Sense::hover());
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
            egui::Label::new(egui::RichText::new(texte).size(13.0).color(theme::TEXTE)).wrap(),
        );
    });
}
