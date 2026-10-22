#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod database;
mod excel_export;
mod models;
mod securite;
mod theme;
mod ui;

use database::Database;
use std::sync::Arc;
use ui::login::{LoginAction, LoginState};

enum AppState {
    Login(LoginState),
    Connecte(ui::main_window::MainWindowState),
}

struct App {
    db: Database,
    etat: AppState,
    utilisateur: Option<models::Utilisateur>,
}

impl App {
    fn new() -> Result<Self, String> {
        let db = Database::new()?;
        db.init_dossier()?;
        let login = LoginState::new(&db);
        Ok(Self {
            db,
            etat: AppState::Login(login),
            utilisateur: None,
        })
    }
}

impl eframe::App for App {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        theme::configurer_style(ctx);
        match &mut self.etat {
            AppState::Login(state) => match ui::login::afficher(ctx, &self.db, state) {
                LoginAction::Connecte(u) => {
                    if let Err(e) = self.db.ouvrir_session(&u.nom_utilisateur) {
                        eprintln!("Erreur ouverture session : {}", e);
                        return;
                    }
                    let mw = ui::main_window::MainWindowState::new(&self.db, &u);
                    self.utilisateur = Some(u);
                    self.etat = AppState::Connecte(mw);
                }
                LoginAction::Aucune => {}
            },
            AppState::Connecte(mw) => {
                if let Some(u) = &self.utilisateur {
                    match ui::main_window::afficher(ctx, &self.db, u, mw) {
                        ui::main_window::MainAction::Deconnexion => {
                            self.db.fermer_session();
                            self.utilisateur = None;
                            self.etat = AppState::Login(LoginState::new(&self.db));
                        }
                        ui::main_window::MainAction::Aucune => {}
                    }
                }
            }
        }
    }
}

fn main() -> Result<(), eframe::Error> {
    let app = match App::new() {
        Ok(a) => a,
        Err(e) => {
            eprintln!("Erreur fatale : {}", e);
            std::process::exit(1);
        }
    };

    let mut viewport = egui::ViewportBuilder::default()
        .with_inner_size([1200.0, 780.0])
        .with_min_inner_size([880.0, 560.0])
        .with_maximized(false)
        .with_resizable(true);

    match theme::charger_icone_fenetre("logo.ico") {
        Some(icone) => {
            viewport = viewport.with_icon(Arc::new(icone));
        }
        None => {
            eprintln!("Attention : impossible de charger Images/logo.ico, icone par defaut utilisee.");
        }
    }

    viewport = viewport.with_title("Pointage");

    let options = eframe::NativeOptions {
        viewport,
        follow_system_theme: false,
        default_theme: eframe::Theme::Light,
        ..Default::default()
    };

    eframe::run_native(
        "Pointage",
        options,
        Box::new(|cc| {
            theme::configurer_style(&cc.egui_ctx);
            ajuster_taille_fenetre(cc);
            Ok(Box::new(app))
        }),
    )
}

fn ajuster_taille_fenetre(cc: &eframe::CreationContext<'_>) {
    let ecran = cc
        .egui_ctx
        .input(|i| i.viewport().monitor_size)
        .map(|taille| {
            let ppp = cc.egui_ctx.pixels_per_point();
            if ppp > 0.0 {
                taille / ppp
            } else {
                taille
            }
        })
        .unwrap_or_else(|| egui::vec2(1280.0, 800.0));

    let cible = theme::taille_fenetre_recommandee(&cc.egui_ctx, ecran);
    cc.egui_ctx
        .send_viewport_cmd(egui::ViewportCommand::InnerSize(cible));
} 
