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
        egui::RichText::new("Bareme de notation")
            .size(26.0)
            .color(theme::PRIMAIRE)
            .strong(),
    );
    ui.add_space(4.0);
    ui.label(
        egui::RichText::new("Regles appliquees au calcul du bilan mensuel.")
            .size(13.5)
            .color(theme::TEXTE_SECONDAIRE),
    );

    ui.add_space(28.0);
    titre_section(ui, "Principe general");
    ui.add_space(10.0);
    paragraphe(ui, "Chaque journee est notee sur 20 points.");
    ui.add_space(4.0);
    paragraphe(ui, "10 points pour la matinee, 10 points pour l'apres-midi.");
    ui.add_space(4.0);
    paragraphe(ui, "La note du jour est la somme des deux.");

    ui.add_space(28.0);
    titre_section(ui, "Matinee (10 points)");
    ui.add_space(10.0);
    ligne_regle(ui, "+ 5 points", theme::SUCCES, "Arrivee a 08h15 ou avant.");
    ui.add_space(6.0);
    ligne_regle(ui, "+ 5 points", theme::SUCCES, "Depart a 11h45 ou apres.");
    ui.add_space(10.0);
    paragraphe_doux(
        ui,
        "Une seule condition remplie : 5/10. Aucune condition : 0/10.",
    );

    ui.add_space(28.0);
    titre_section(ui, "Apres-midi (10 points)");
    ui.add_space(10.0);
    ligne_regle(ui, "+ 5 points", theme::SUCCES, "Arrivee a 14h15 ou avant.");
    ui.add_space(6.0);
    ligne_regle(ui, "+ 5 points", theme::SUCCES, "Depart a 15h45 ou apres.");
    ui.add_space(10.0);
    paragraphe_doux(
        ui,
        "Une seule condition remplie : 5/10. Aucune condition : 0/10.",
    );

    ui.add_space(28.0);
    titre_section(ui, "Jours feries");
    ui.add_space(10.0);
    paragraphe(ui, "Un jour ferie retire la ou les periodes concernees du calcul.");
    ui.add_space(4.0);
    paragraphe_doux(
        ui,
        "Matin : seule la matinee est retiree. La note du jour reste sur 10, ramenee sur 20.",
    );
    ui.add_space(4.0);
    paragraphe_doux(
        ui,
        "Apres-midi : seul l'apres-midi est retire. Meme principe.",
    );
    ui.add_space(4.0);
    paragraphe_doux(
        ui,
        "Jour entier : la journee entiere est exclue du calcul du bilan et du classement.",
    );
    ui.add_space(4.0);
    paragraphe_doux(
        ui,
        "Le pointage est bloque sur une periode feriee dans la page Pointage.",
    );

    ui.add_space(28.0);
    titre_section(ui, "Note du jour (sur 20)");
    ui.add_space(10.0);
    paragraphe(ui, "Note du jour = note matin + note apres-midi, hors periodes feriees.");

    ui.add_space(16.0);
    ligne_seuil(ui, "16 / 20 et plus", theme::SUCCES, "Bien");
    ui.add_space(8.0);
    ligne_seuil(ui, "10 a 15,9 / 20", theme::WARNING, "Moyen");
    ui.add_space(8.0);
    ligne_seuil(ui, "Moins de 10 / 20", theme::DANGER, "Mauvais");

    ui.add_space(40.0);
    ui.with_layout(egui::Layout::right_to_left(egui::Align::TOP), |ui| {
        ui.label(
            egui::RichText::new("(c) Johanes-mg")
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
