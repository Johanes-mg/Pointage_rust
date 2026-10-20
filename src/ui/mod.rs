pub mod accueil;
pub mod bilan;
pub mod compte;
pub mod infos;
pub mod login;
pub mod main_window;
pub mod personnel;
pub mod pointage;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Page {
    Accueil,
    Personnel,
    Pointage,
    Bilan,
    Compte,
    Infos,
}
