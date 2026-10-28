use chrono::NaiveDateTime;

#[derive(Debug, Clone, Default)]
#[allow(dead_code)]
pub struct Utilisateur {
    pub id: i64,
    pub nom_utilisateur: String,
    pub mot_de_passe_hash: String,
    pub sel: String,
    pub nom_complet: String,
    pub role: String,
    pub date_creation: Option<NaiveDateTime>,
}

#[derive(Debug, Clone, Default)]
pub struct Personnel {
    pub id: i64,
    pub nom: String,
    pub prenom: String,
    pub matricule: Option<String>,
    pub telephone: Option<String>,
    pub adresse: Option<String>,
    pub email: Option<String>,
    pub statut: Option<String>,
    pub grade: Option<String>,
    pub situation_familiale: Option<String>,
    pub date_arrivee: Option<chrono::NaiveDate>,
}

impl Personnel {
    pub fn nom_complet(&self) -> String {
        format!("{} {}", self.nom, self.prenom).trim().to_string()
    }
}

#[derive(Debug, Clone, Default)]
#[allow(dead_code)]
pub struct EnregistrementPointage {
    pub id: i64,
    pub personnel_id: i64,
    pub date_pointage: chrono::NaiveDate,
    pub matin_arrivee: Option<chrono::NaiveTime>,
    pub matin_sortie: Option<chrono::NaiveTime>,
    pub apres_midi_arrivee: Option<chrono::NaiveTime>,
    pub apres_midi_sortie: Option<chrono::NaiveTime>,
    pub observation: Option<String>,
    pub nom_complet: Option<String>,
}

#[derive(Debug, Clone, Default)]
pub struct BilanPersonnel {
    pub nom: String,
    pub jours_presents: i64,
    pub matins_presents: i64,
    pub apres_midis_presents: i64,
    pub total_heures: String,
    pub moyenne_arrivee: String,
}

#[derive(Debug, Clone, Default)]
pub struct Conge {
    pub id: i64,
    pub personnel_id: i64,
    pub date_debut: chrono::NaiveDate,
    pub date_fin: chrono::NaiveDate,
    pub periode: String,
}

#[allow(dead_code)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MentionPonctualite {
    Absent,
    Bien,
    Moyen,
    Mauvais,
}
