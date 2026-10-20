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
        .with_inner_size([1280.0, 800.0])
        .with_min_inner_size([900.0, 600.0]);

    match theme::charger_icone_fenetre("logo.ico") {
        Some(icone) => {
            let w = icone.width;
            let h = icone.height;
            viewport = viewport.with_icon(Arc::new(icone));
            eprintln!("Icone de fenetre chargee : {}x{}", w, h);
        }
        None => {
            eprintln!("Attention : impossible de charger Images/logo.ico, icone par defaut utilisee.");
        }
    }

    viewport = viewport.with_title("Pointage");

    let options = eframe::NativeOptions {
        viewport,
        ..Default::default()
    };

    eframe::run_native(
        "Pointage",
        options,
        Box::new(|_cc| Ok(Box::new(app))),
    )
}
