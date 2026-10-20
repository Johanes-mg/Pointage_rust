use crate::database::Database;
use crate::models::Utilisateur;
use crate::theme;
use crate::ui::{accueil, bilan, compte, infos, personnel, pointage, Page};

pub struct MainWindowState {
    pub page: Page,
    pub page_precedente: Option<Page>,
    pub accueil: accueil::AccueilState,
    pub personnel: personnel::PersonnelState,
    pub pointage: pointage::PointageState,
    pub bilan: bilan::BilanState,
    pub compte: compte::CompteState,
    pub infos: infos::InfosState,
}

impl MainWindowState {
    pub fn new(_db: &Database, utilisateur: &Utilisateur) -> Self {
        Self {
            page: Page::Accueil,
            page_precedente: None,
            accueil: accueil::AccueilState::new(),
            personnel: personnel::PersonnelState::new(),
            pointage: pointage::PointageState::new(),
            bilan: bilan::BilanState::new(),
            compte: compte::CompteState::new(utilisateur),
            infos: infos::InfosState::new(),
        }
    }

    fn rafraichir_caches(&mut self) {
        let ancienne = self.page_precedente;
        let actuelle = self.page;

        if self.personnel.mutation_effectuee {
            self.pointage.invalider_cache();
            self.bilan.invalider_cache();
            self.personnel.mutation_effectuee = false;
        }

        if actuelle == Page::Pointage && ancienne != Some(Page::Pointage) {
            self.pointage.invalider_cache();
        }

        if actuelle == Page::Bilan && ancienne != Some(Page::Bilan) {
            self.bilan.invalider_cache();
        }

        if ancienne == Some(Page::Pointage) && actuelle != Page::Pointage {
            self.bilan.invalider_cache();
        }

        self.page_precedente = Some(actuelle);
    }
}

pub enum MainAction {
    Aucune,
    Deconnexion,
}

pub fn afficher(
    ctx: &egui::Context,
    db: &Database,
    utilisateur: &Utilisateur,
    state: &mut MainWindowState,
) -> MainAction {
    let mut action = MainAction::Aucune;

    egui::TopBottomPanel::top("topbar")
        .exact_height(68.0)
        .frame(egui::Frame::none().fill(theme::PRIMAIRE))
        .show(ctx, |ui| {
            ui.horizontal_centered(|ui| {
                ui.add_space(16.0);
                theme::image_ui(ui, "logo.png", 40.0, 40.0);
                ui.add_space(12.0);

                ui.vertical(|ui| {
                    ui.add_space(16.0);
                    ui.label(
                        egui::RichText::new(&utilisateur.nom_complet)
                            .size(12.0)
                            .color(egui::Color32::from_rgb(200, 215, 235)),
                    );
                });

                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    ui.add_space(16.0);

                    if theme::bouton_image_seule(ui, "deconnecter.png", 40.0, 22.0) {
                        action = MainAction::Deconnexion;
                    }

                    ui.add_space(16.0);

                    let pages = [
                        (Page::Infos, "infos.png", "Infos"),
                        (Page::Compte, "compte.png", "Compte"),
                        (Page::Bilan, "bilan.png", "Bilan"),
                        (Page::Pointage, "pointage.png", "Pointage"),
                        (Page::Personnel, "personnel.png", "Personnel"),
                        (Page::Accueil, "accueil.png", "Accueil"),
                    ];

                    for (page, img, label) in pages {
                        let actif = state.page == page;
                        let fond = if actif { theme::ACCENT } else { theme::PRIMAIRE };
                        let hover_fond = if actif {
                            theme::ACCENT
                        } else {
                            theme::PRIMAIRE_CLAIR
                        };

                        if bouton_nav_image(ui, img, label, fond, hover_fond) {
                            if state.page != page {
                                state.page = page;
                            }
                        }
                    }
                });
            });
        });

    state.rafraichir_caches();

    egui::CentralPanel::default()
        .frame(egui::Frame::none().fill(theme::FOND))
        .show(ctx, |ui| {
            let page = state.page;
            let marge = egui::Margin::symmetric(30.0, 20.0);

            egui::Frame::none().inner_margin(marge).show(ui, |ui| {
                if page == Page::Bilan {
                    bilan::afficher(ui, db, &mut state.bilan);
                } else {
                    egui::ScrollArea::vertical()
                        .auto_shrink([false, false])
                        .show(ui, |ui| match page {
                            Page::Accueil => accueil::afficher(ui, db, &mut state.accueil),
                            Page::Personnel => personnel::afficher(ui, db, &mut state.personnel),
                            Page::Pointage => pointage::afficher(ui, db, &mut state.pointage),
                            Page::Bilan => {}
                            Page::Compte => {
                                compte::afficher(ui, db, utilisateur, &mut state.compte)
                            }
                            Page::Infos => infos::afficher(ui, &mut state.infos),
                        });
                }
            });
        });

    action
}

fn bouton_nav_image(
    ui: &mut egui::Ui,
    fichier: &str,
    texte: &str,
    fond: egui::Color32,
    hover_fond: egui::Color32,
) -> bool {
    let ctx = ui.ctx().clone();

    let galerie = ui.fonts(|f| {
        f.layout_no_wrap(
            texte.to_string(),
            egui::FontId::proportional(12.5),
            egui::Color32::WHITE,
        )
    });
    let largeur_texte = galerie.size().x;
    let largeur = 20.0 + 6.0 + largeur_texte + 20.0;

    let (rect, resp) = ui.allocate_exact_size(egui::vec2(largeur, 40.0), egui::Sense::click());

    let bg = if resp.hovered() { hover_fond } else { fond };

    let painter = ui.painter();
    painter.rect_filled(rect, theme::r_md(), bg);

    let mut x = rect.left() + 10.0;
    if let Some(tex) = theme::image(&ctx, fichier) {
        let icon_rect = egui::Rect::from_min_size(
            egui::pos2(x, rect.center().y - 9.0),
            egui::vec2(18.0, 18.0),
        );
        painter.image(
            tex.id(),
            icon_rect,
            egui::Rect::from_min_max(egui::pos2(0.0, 0.0), egui::pos2(1.0, 1.0)),
            egui::Color32::WHITE,
        );
        x += 18.0 + 6.0;
    }

    painter.text(
        egui::pos2(x, rect.center().y),
        egui::Align2::LEFT_CENTER,
        texte,
        egui::FontId::proportional(12.5),
        egui::Color32::WHITE,
    );

    if resp.hovered() {
        ui.ctx().set_cursor_icon(egui::CursorIcon::PointingHand);
    }
    resp.clicked()
}
