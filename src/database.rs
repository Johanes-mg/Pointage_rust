use crate::models::*;
use crate::securite;
use chrono::{Datelike, NaiveDate, NaiveTime, Weekday};
use rusqlite::{params, Connection};
use std::cell::RefCell;
use std::collections::{HashMap, HashSet};
use std::path::PathBuf;

pub struct Database {
    dossier: PathBuf,
    utilisateur_courant: RefCell<Option<String>>,
    conn: RefCell<Option<Connection>>,
    chemin_actif: RefCell<Option<PathBuf>>,
}

pub struct ExportSnapshot {
    pub personnel: Vec<Personnel>,
    pub pointages: Vec<EnregistrementPointage>,
    pub bilan: Vec<BilanPersonnel>,
    pub feries: HashMap<String, Vec<String>>,
    pub conges: Vec<Conge>,
    pub nom_entreprise: String,
}

pub fn note_matin_arrivee_5(t: NaiveTime) -> f64 {
    let limite = NaiveTime::from_hms_opt(8, 15, 0).unwrap();
    if t <= limite {
        5.0
    } else {
        0.0
    }
}

pub fn note_matin_sortie_5(t: NaiveTime) -> f64 {
    let limite = NaiveTime::from_hms_opt(11, 45, 0).unwrap();
    if t >= limite {
        5.0
    } else {
        0.0
    }
}

pub fn note_aprem_arrivee_5(t: NaiveTime) -> f64 {
    let limite = NaiveTime::from_hms_opt(14, 15, 0).unwrap();
    if t <= limite {
        5.0
    } else {
        0.0
    }
}

pub fn note_aprem_sortie_5(t: NaiveTime) -> f64 {
    let limite = NaiveTime::from_hms_opt(15, 45, 0).unwrap();
    if t >= limite {
        5.0
    } else {
        0.0
    }
}

impl Database {
    pub fn new() -> Result<Self, String> {
        let base = directories::ProjectDirs::from("mg", "Johanes", "Pointage")
            .ok_or("Impossible de determiner le dossier de donnees")?;
        let dossier = base.data_dir().to_path_buf();
        Ok(Self {
            dossier,
            utilisateur_courant: RefCell::new(None),
            conn: RefCell::new(None),
            chemin_actif: RefCell::new(None),
        })
    }

    pub fn init_dossier(&self) -> Result<(), String> {
        std::fs::create_dir_all(&self.dossier).map_err(|e| e.to_string())
    }

    pub fn normaliser_utilisateur(nom: &str) -> String {
        nom.chars()
            .filter(|c| c.is_alphanumeric())
            .collect::<String>()
            .to_lowercase()
    }

    pub fn nom_base_pour(nom_utilisateur: &str) -> String {
        let normalise = Self::normaliser_utilisateur(nom_utilisateur);
        let normalise = if normalise.is_empty() {
            "utilisateur".to_string()
        } else {
            normalise
        };
        format!("pointage_{}.db", normalise)
    }

    pub fn chemin_pour(&self, nom_utilisateur: &str) -> PathBuf {
        self.dossier.join(Self::nom_base_pour(nom_utilisateur))
    }

    pub fn lister_bases(&self) -> Result<Vec<String>, String> {
        let mut resultats = Vec::new();
        if !self.dossier.exists() {
            return Ok(resultats);
        }
        let entries = std::fs::read_dir(&self.dossier).map_err(|e| e.to_string())?;
        for entry in entries.flatten() {
            let path = entry.path();
            if let Some(nom) = path.file_name().and_then(|s| s.to_str()) {
                if nom.starts_with("pointage_") && nom.ends_with(".db") {
                    if let Some(stem) = nom
                        .strip_prefix("pointage_")
                        .and_then(|s| s.strip_suffix(".db"))
                    {
                        resultats.push(stem.to_string());
                    }
                }
            }
        }
        Ok(resultats)
    }

    pub fn ouvrir_session(&self, nom_utilisateur: &str) -> Result<(), String> {
        let normalise = Self::normaliser_utilisateur(nom_utilisateur);
        if normalise.is_empty() {
            return Err("Nom d'utilisateur invalide".into());
        }
        let chemin = self.chemin_pour(nom_utilisateur);

        let conn = Connection::open(&chemin).map_err(|e| e.to_string())?;
        conn.execute_batch(
            "PRAGMA foreign_keys = ON;
             PRAGMA journal_mode = WAL;
             PRAGMA synchronous = NORMAL;
             PRAGMA temp_store = MEMORY;
             PRAGMA cache_size = -8000;
             PRAGMA mmap_size = 268435456;",
        )
        .map_err(|e| e.to_string())?;

        Self::creer_tables(&conn)?;
        Self::migrer_tables(&conn)?;
        Self::migrer_utilisateurs(&conn)?;

        *self.utilisateur_courant.borrow_mut() = Some(normalise);
        *self.conn.borrow_mut() = Some(conn);
        *self.chemin_actif.borrow_mut() = Some(chemin);
        Ok(())
    }

    pub fn fermer_session(&self) {
        *self.utilisateur_courant.borrow_mut() = None;
        *self.conn.borrow_mut() = None;
        *self.chemin_actif.borrow_mut() = None;
    }

    pub fn chemin_base_actuel(&self) -> String {
        self.chemin_actif
            .borrow()
            .as_ref()
            .map(|p| p.to_string_lossy().to_string())
            .unwrap_or_default()
    }

    fn with_conn<F, R>(&self, f: F) -> Result<R, String>
    where
        F: FnOnce(&Connection) -> Result<R, String>,
    {
        let emprunt = self.conn.borrow();
        let conn = emprunt
            .as_ref()
            .ok_or_else(|| "Aucune session ouverte".to_string())?;
        f(conn)
    }

    fn creer_tables(conn: &Connection) -> Result<(), String> {
        conn.execute_batch(
            "CREATE TABLE IF NOT EXISTS Utilisateur (
                Id INTEGER PRIMARY KEY AUTOINCREMENT,
                NomUtilisateur TEXT NOT NULL UNIQUE,
                MotDePasseHash TEXT NOT NULL,
                Sel TEXT NOT NULL,
                NomComplet TEXT NOT NULL,
                Role TEXT NOT NULL DEFAULT 'operateur',
                DateCreation TEXT DEFAULT (datetime('now','localtime'))
            );
            CREATE TABLE IF NOT EXISTS Personnel (
                Id INTEGER PRIMARY KEY AUTOINCREMENT,
                Nom TEXT NOT NULL,
                Prenom TEXT NOT NULL,
                Matricule TEXT,
                Telephone TEXT,
                Adresse TEXT,
                Email TEXT,
                Statut TEXT,
                Grade TEXT,
                SituationFamiliale TEXT,
                DateArrivee TEXT
            );
            CREATE TABLE IF NOT EXISTS EnregistrementPointage (
                Id INTEGER PRIMARY KEY AUTOINCREMENT,
                PersonnelId INTEGER NOT NULL,
                DatePointage TEXT NOT NULL,
                MatinArrivee TEXT,
                MatinSortie TEXT,
                ApresMidiArrivee TEXT,
                ApresMidiSortie TEXT,
                Observation TEXT,
                FOREIGN KEY (PersonnelId) REFERENCES Personnel(Id) ON DELETE CASCADE,
                UNIQUE (PersonnelId, DatePointage)
            );
            CREATE TABLE IF NOT EXISTS JourFerie (
                Id INTEGER PRIMARY KEY AUTOINCREMENT,
                DatePointage TEXT NOT NULL,
                Periode TEXT NOT NULL CHECK (Periode IN ('jour','matin','aprem')),
                UNIQUE (DatePointage, Periode)
            );
            CREATE TABLE IF NOT EXISTS Conge (
                Id INTEGER PRIMARY KEY AUTOINCREMENT,
                PersonnelId INTEGER NOT NULL,
                DateDebut TEXT NOT NULL,
                DateFin TEXT NOT NULL,
                Periode TEXT NOT NULL DEFAULT 'jour' CHECK (Periode IN ('jour','matin','aprem')),
                FOREIGN KEY (PersonnelId) REFERENCES Personnel(Id) ON DELETE CASCADE
            );
            CREATE TABLE IF NOT EXISTS Parametre (
                Cle TEXT PRIMARY KEY,
                Valeur TEXT
            );
            CREATE INDEX IF NOT EXISTS idx_pointage_date ON EnregistrementPointage(DatePointage);
            CREATE INDEX IF NOT EXISTS idx_pointage_personnel ON EnregistrementPointage(PersonnelId);
            CREATE INDEX IF NOT EXISTS idx_pointage_perso_date ON EnregistrementPointage(PersonnelId, DatePointage);
            CREATE INDEX IF NOT EXISTS idx_personnel_nom ON Personnel(Nom, Prenom);
            CREATE INDEX IF NOT EXISTS idx_ferie_date ON JourFerie(DatePointage);
            CREATE INDEX IF NOT EXISTS idx_conge_personnel ON Conge(PersonnelId);
            CREATE INDEX IF NOT EXISTS idx_conge_dates ON Conge(DateDebut, DateFin);
            INSERT OR IGNORE INTO Parametre (Cle, Valeur) VALUES ('NomEntreprise','Pointage');",
        )
        .map_err(|e| e.to_string())?;
        Ok(())
    }

    fn migrer_tables(conn: &Connection) -> Result<(), String> {
        let index_existe: i64 = conn
            .query_row(
                "SELECT COUNT(*) FROM sqlite_master
                 WHERE type='index'
                   AND tbl_name='EnregistrementPointage'
                   AND (sql LIKE '%UNIQUE%' OR sql LIKE '%PersonnelId, DatePointage%' OR sql LIKE '%PersonnelId,DatePointage%')",
                [],
                |r| r.get(0),
            )
            .unwrap_or(0);

        if index_existe == 0 {
            let contrainte_ok: i64 = conn
                .query_row(
                    "SELECT COUNT(*) FROM sqlite_master
                     WHERE type='table'
                       AND name='EnregistrementPointage'
                       AND sql LIKE '%UNIQUE (PersonnelId, DatePointage)%'",
                    [],
                    |r| r.get(0),
                )
                .unwrap_or(0);

            if contrainte_ok == 0 {
                let _ = conn.execute_batch(
                    "CREATE UNIQUE INDEX IF NOT EXISTS idx_pointage_unique
                     ON EnregistrementPointage(PersonnelId, DatePointage);",
                );
            }
        }

        let ferie_existe: i64 = conn
            .query_row(
                "SELECT COUNT(*) FROM sqlite_master WHERE type='table' AND name='JourFerie'",
                [],
                |r| r.get(0),
            )
            .unwrap_or(0);
        if ferie_existe == 0 {
            let _ = conn.execute_batch(
                "CREATE TABLE IF NOT EXISTS JourFerie (
                    Id INTEGER PRIMARY KEY AUTOINCREMENT,
                    DatePointage TEXT NOT NULL,
                    Periode TEXT NOT NULL CHECK (Periode IN ('jour','matin','aprem')),
                    UNIQUE (DatePointage, Periode)
                );
                CREATE INDEX IF NOT EXISTS idx_ferie_date ON JourFerie(DatePointage);",
            );
        }

        let date_arrivee_existe: i64 = conn
            .query_row(
                "SELECT COUNT(*) FROM pragma_table_info('Personnel') WHERE name = 'DateArrivee'",
                [],
                |r| r.get(0),
            )
            .unwrap_or(0);
        if date_arrivee_existe == 0 {
            let _ = conn.execute_batch("ALTER TABLE Personnel ADD COLUMN DateArrivee TEXT;");
        }

        let conge_existe: i64 = conn
            .query_row(
                "SELECT COUNT(*) FROM sqlite_master WHERE type='table' AND name='Conge'",
                [],
                |r| r.get(0),
            )
            .unwrap_or(0);
        if conge_existe == 0 {
            let _ = conn.execute_batch(
                "CREATE TABLE IF NOT EXISTS Conge (
                    Id INTEGER PRIMARY KEY AUTOINCREMENT,
                    PersonnelId INTEGER NOT NULL,
                    DateDebut TEXT NOT NULL,
                    DateFin TEXT NOT NULL,
                    Periode TEXT NOT NULL DEFAULT 'jour' CHECK (Periode IN ('jour','matin','aprem')),
                    FOREIGN KEY (PersonnelId) REFERENCES Personnel(Id) ON DELETE CASCADE
                );
                CREATE INDEX IF NOT EXISTS idx_conge_personnel ON Conge(PersonnelId);
                CREATE INDEX IF NOT EXISTS idx_conge_dates ON Conge(DateDebut, DateFin);",
            );
        } else {
            let periode_existe: i64 = conn
                .query_row(
                    "SELECT COUNT(*) FROM pragma_table_info('Conge') WHERE name = 'Periode'",
                    [],
                    |r| r.get(0),
                )
                .unwrap_or(0);
            if periode_existe == 0 {
                let _ = conn.execute_batch(
                    "ALTER TABLE Conge ADD COLUMN Periode TEXT NOT NULL DEFAULT 'jour';",
                );
            }
        }

        Ok(())
    }

    fn migrer_utilisateurs(conn: &Connection) -> Result<(), String> {
        let mut stmt = conn
            .prepare("SELECT Id, NomUtilisateur FROM Utilisateur ORDER BY Id")
            .map_err(|e| e.to_string())?;
        let iter = stmt
            .query_map([], |r| Ok((r.get::<_, i64>(0)?, r.get::<_, String>(1)?)))
            .map_err(|e| e.to_string())?;
        let mut lignes: Vec<(i64, String)> = Vec::new();
        for item in iter {
            lignes.push(item.map_err(|e| e.to_string())?);
        }
        drop(stmt);

        let mut vus: HashMap<String, i64> = HashMap::new();
        let mut a_supprimer: Vec<i64> = Vec::new();
        let mut a_renommer: Vec<(i64, String)> = Vec::new();

        for (id, nom) in lignes {
            let normalise = Self::normaliser_utilisateur(&nom);
            if normalise.is_empty() {
                a_supprimer.push(id);
                continue;
            }
            if nom == normalise {
                vus.entry(normalise).or_insert(id);
            } else {
                if vus.contains_key(&normalise) {
                    a_supprimer.push(id);
                } else {
                    a_renommer.push((id, normalise.clone()));
                    vus.insert(normalise, id);
                }
            }
        }

        for (id, nouveau) in a_renommer {
            let _ = conn.execute(
                "UPDATE Utilisateur SET NomUtilisateur = ?1 WHERE Id = ?2",
                params![nouveau, id],
            );
        }

        for id in a_supprimer {
            let _ = conn.execute("DELETE FROM Utilisateur WHERE Id = ?1", params![id]);
        }

        Ok(())
    }

    pub fn aucun_utilisateur(&self) -> Result<bool, String> {
        let bases = self.lister_bases()?;
        if bases.is_empty() {
            return Ok(true);
        }
        for base in &bases {
            let chemin = self.dossier.join(format!("pointage_{}.db", base));
            if let Ok(conn) = Connection::open(&chemin) {
                let n: i64 = conn
                    .query_row(
                        "SELECT COUNT(*) FROM sqlite_master WHERE type='table' AND name='Utilisateur'",
                        [],
                        |r| r.get(0),
                    )
                    .unwrap_or(0);
                if n > 0 {
                    let count: i64 = conn
                        .query_row("SELECT COUNT(*) FROM Utilisateur", [], |r| r.get(0))
                        .unwrap_or(0);
                    if count > 0 {
                        return Ok(false);
                    }
                }
            }
        }
        Ok(true)
    }

    pub fn creer_utilisateur(
        &self,
        nom_utilisateur: &str,
        mot_de_passe: &str,
        nom_complet: &str,
    ) -> Result<Utilisateur, String> {
        let nom_utilisateur = nom_utilisateur.trim();
        if nom_utilisateur.is_empty() {
            return Err("Nom d'utilisateur obligatoire.".into());
        }
        if mot_de_passe.len() < 4 {
            return Err("Mot de passe trop court.".into());
        }

        let nom_normalise = Self::normaliser_utilisateur(nom_utilisateur);
        if nom_normalise.is_empty() {
            return Err("Nom d'utilisateur invalide.".into());
        }

        let chemin = self.chemin_pour(nom_utilisateur);
        let conn = Connection::open(&chemin).map_err(|e| e.to_string())?;
        conn.execute_batch(
            "PRAGMA foreign_keys = ON;
             PRAGMA journal_mode = WAL;
             PRAGMA synchronous = NORMAL;",
        )
        .map_err(|e| e.to_string())?;
        Self::creer_tables(&conn)?;
        Self::migrer_tables(&conn)?;
        Self::migrer_utilisateurs(&conn)?;

        let existe: i64 = conn
            .query_row(
                "SELECT COUNT(*) FROM Utilisateur WHERE NomUtilisateur = ?1",
                params![nom_normalise],
                |r| r.get(0),
            )
            .unwrap_or(0);

        if existe > 0 {
            return Err(format!(
                "Le compte '{}' existe deja dans cette base.",
                nom_utilisateur
            ));
        }

        let sel = securite::generer_sel();
        let hash = securite::hacher_mot_de_passe(mot_de_passe, &sel);

        conn.execute(
            "INSERT INTO Utilisateur (NomUtilisateur, MotDePasseHash, Sel, NomComplet, Role)
             VALUES (?1, ?2, ?3, ?4, 'operateur')",
            params![nom_normalise, hash, sel, nom_complet],
        )
        .map_err(|e| e.to_string())?;

        let id = conn.last_insert_rowid();

        Ok(Utilisateur {
            id,
            nom_utilisateur: nom_normalise,
            mot_de_passe_hash: hash,
            sel,
            nom_complet: nom_complet.to_string(),
            role: "operateur".into(),
            date_creation: None,
        })
    }

    pub fn authentifier(
        &self,
        nom_utilisateur: &str,
        mot_de_passe: &str,
    ) -> Result<Option<Utilisateur>, String> {
        let nom_normalise = Self::normaliser_utilisateur(nom_utilisateur.trim());
        if nom_normalise.is_empty() {
            return Ok(None);
        }
        let chemin = self.chemin_pour(nom_utilisateur);
        if !chemin.exists() {
            return Ok(None);
        }

        let conn = Connection::open(&chemin).map_err(|e| e.to_string())?;
        let table_existe: i64 = conn
            .query_row(
                "SELECT COUNT(*) FROM sqlite_master WHERE type='table' AND name='Utilisateur'",
                [],
                |r| r.get(0),
            )
            .unwrap_or(0);
        if table_existe == 0 {
            return Ok(None);
        }

        let _ = Self::migrer_utilisateurs(&conn);

        let row = conn
            .query_row(
                "SELECT Id, NomUtilisateur, MotDePasseHash, Sel, NomComplet, Role
                 FROM Utilisateur WHERE NomUtilisateur = ?1",
                params![nom_normalise],
                |r| {
                    Ok(Utilisateur {
                        id: r.get(0)?,
                        nom_utilisateur: r.get(1)?,
                        mot_de_passe_hash: r.get(2)?,
                        sel: r.get(3)?,
                        nom_complet: r.get(4)?,
                        role: r.get(5)?,
                        date_creation: None,
                    })
                },
            )
            .ok();

        if let Some(u) = row {
            if securite::verifier_mot_de_passe(mot_de_passe, &u.sel, &u.mot_de_passe_hash) {
                return Ok(Some(u));
            }
        }
        Ok(None)
    }

    pub fn changer_mot_de_passe(
        &self,
        utilisateur_id: i64,
        ancien: &str,
        nouveau: &str,
    ) -> Result<bool, String> {
        if nouveau.len() < 4 {
            return Err("Le nouveau mot de passe doit contenir au moins 4 caractères.".into());
        }
        self.with_conn(|conn| {
            let (hash, sel): (String, String) = conn
                .query_row(
                    "SELECT MotDePasseHash, Sel FROM Utilisateur WHERE Id = ?1",
                    params![utilisateur_id],
                    |r| Ok((r.get(0)?, r.get(1)?)),
                )
                .map_err(|e| e.to_string())?;

            if !securite::verifier_mot_de_passe(ancien, &sel, &hash) {
                return Ok(false);
            }
            let nsel = securite::generer_sel();
            let nhash = securite::hacher_mot_de_passe(nouveau, &nsel);
            conn.execute(
                "UPDATE Utilisateur SET MotDePasseHash = ?1, Sel = ?2 WHERE Id = ?3",
                params![nhash, nsel, utilisateur_id],
            )
            .map_err(|e| e.to_string())?;
            Ok(true)
        })
    }

    pub fn mettre_a_jour_nom_complet(&self, id: i64, nom: &str) -> Result<(), String> {
        self.with_conn(|conn| {
            conn.execute(
                "UPDATE Utilisateur SET NomComplet = ?1 WHERE Id = ?2",
                params![nom, id],
            )
            .map_err(|e| e.to_string())?;
            Ok(())
        })
    }

    pub fn get_personnel(&self) -> Result<Vec<Personnel>, String> {
        self.with_conn(|conn| Self::query_personnel(conn))
    }

    fn query_personnel(conn: &Connection) -> Result<Vec<Personnel>, String> {
        let mut stmt = conn
            .prepare(
                "SELECT Id, Nom, Prenom, Matricule, Telephone, Adresse, Email, Statut, Grade, SituationFamiliale, DateArrivee
                 FROM Personnel ORDER BY Nom, Prenom",
            )
            .map_err(|e| e.to_string())?;
        let iter = stmt
            .query_map([], |r| {
                let date_str: Option<String> = r.get(10)?;
                let date_arrivee =
                    date_str.and_then(|s| NaiveDate::parse_from_str(&s, "%Y-%m-%d").ok());
                Ok(Personnel {
                    id: r.get(0)?,
                    nom: r.get(1)?,
                    prenom: r.get(2)?,
                    matricule: r.get(3)?,
                    telephone: r.get(4)?,
                    adresse: r.get(5)?,
                    email: r.get(6)?,
                    statut: r.get(7)?,
                    grade: r.get(8)?,
                    situation_familiale: r.get(9)?,
                    date_arrivee,
                })
            })
            .map_err(|e| e.to_string())?;
        iter.collect::<Result<Vec<_>, _>>().map_err(|e| e.to_string())
    }

    pub fn ajouter_personnel(&self, p: &Personnel) -> Result<i64, String> {
        self.with_conn(|conn| {
            let date_str = p.date_arrivee.map(|d| d.format("%Y-%m-%d").to_string());
            conn.execute(
                "INSERT INTO Personnel (Nom, Prenom, Matricule, Telephone, Adresse, Email, Statut, Grade, SituationFamiliale, DateArrivee)
                 VALUES (?1,?2,?3,?4,?5,?6,?7,?8,?9,?10)",
                params![
                    p.nom, p.prenom, p.matricule, p.telephone, p.adresse,
                    p.email, p.statut, p.grade, p.situation_familiale, date_str
                ],
            )
            .map_err(|e| e.to_string())?;
            Ok(conn.last_insert_rowid())
        })
    }

    pub fn modifier_personnel(&self, p: &Personnel) -> Result<(), String> {
        self.with_conn(|conn| {
            let date_str = p.date_arrivee.map(|d| d.format("%Y-%m-%d").to_string());
            conn.execute(
                "UPDATE Personnel SET Nom=?1, Prenom=?2, Matricule=?3, Telephone=?4, Adresse=?5,
                 Email=?6, Statut=?7, Grade=?8, SituationFamiliale=?9, DateArrivee=?10 WHERE Id=?11",
                params![
                    p.nom, p.prenom, p.matricule, p.telephone, p.adresse,
                    p.email, p.statut, p.grade, p.situation_familiale, date_str, p.id
                ],
            )
            .map_err(|e| e.to_string())?;
            Ok(())
        })
    }

    pub fn supprimer_personnel(&self, id: i64) -> Result<(), String> {
        self.with_conn(|conn| {
            conn.execute("DELETE FROM Personnel WHERE Id = ?1", params![id])
                .map_err(|e| e.to_string())?;
            Ok(())
        })
    }

    fn lire_pointage(r: &rusqlite::Row) -> rusqlite::Result<EnregistrementPointage> {
        let date_str: String = r.get(2)?;
        let date = NaiveDate::parse_from_str(&date_str, "%Y-%m-%d")
            .unwrap_or_else(|_| chrono::Local::now().date_naive());

        let parse_time = |opt: Option<String>| -> Option<NaiveTime> {
            opt.and_then(|s| NaiveTime::parse_from_str(&s, "%H:%M:%S").ok())
        };

        Ok(EnregistrementPointage {
            id: r.get(0)?,
            personnel_id: r.get(1)?,
            date_pointage: date,
            matin_arrivee: parse_time(r.get(3)?),
            matin_sortie: parse_time(r.get(4)?),
            apres_midi_arrivee: parse_time(r.get(5)?),
            apres_midi_sortie: parse_time(r.get(6)?),
            observation: r.get(7)?,
            nom_complet: r.get(8).ok(),
        })
    }

    const SELECT_POINTAGE: &'static str =
        "SELECT ep.Id, ep.PersonnelId, ep.DatePointage,
                ep.MatinArrivee, ep.MatinSortie,
                ep.ApresMidiArrivee, ep.ApresMidiSortie,
                ep.Observation, (p.Nom || ' ' || p.Prenom) AS NomComplet
         FROM EnregistrementPointage ep
         INNER JOIN Personnel p ON p.Id = ep.PersonnelId ";

    pub fn get_pointages_du_jour(
        &self,
        date: NaiveDate,
    ) -> Result<Vec<EnregistrementPointage>, String> {
        self.with_conn(|conn| {
            let sql = format!(
                "{} WHERE ep.DatePointage = ?1 ORDER BY p.Nom, p.Prenom",
                Self::SELECT_POINTAGE
            );
            let mut stmt = conn.prepare(&sql).map_err(|e| e.to_string())?;
            let iter = stmt
                .query_map(
                    params![date.format("%Y-%m-%d").to_string()],
                    Self::lire_pointage,
                )
                .map_err(|e| e.to_string())?;
            iter.collect::<Result<Vec<_>, _>>().map_err(|e| e.to_string())
        })
    }

    pub fn get_pointages_periode(
        &self,
        debut: NaiveDate,
        fin: NaiveDate,
    ) -> Result<Vec<EnregistrementPointage>, String> {
        self.with_conn(|conn| Self::query_pointages_periode(conn, debut, fin))
    }

    fn query_pointages_periode(
        conn: &Connection,
        debut: NaiveDate,
        fin: NaiveDate,
    ) -> Result<Vec<EnregistrementPointage>, String> {
        let sql = format!(
            "{} WHERE ep.DatePointage BETWEEN ?1 AND ?2 ORDER BY ep.DatePointage DESC, p.Nom, p.Prenom",
            Self::SELECT_POINTAGE
        );
        let mut stmt = conn.prepare(&sql).map_err(|e| e.to_string())?;
        let iter = stmt
            .query_map(
                params![
                    debut.format("%Y-%m-%d").to_string(),
                    fin.format("%Y-%m-%d").to_string()
                ],
                Self::lire_pointage,
            )
            .map_err(|e| e.to_string())?;
        iter.collect::<Result<Vec<_>, _>>().map_err(|e| e.to_string())
    }

    pub fn enregistrer_pointage(
        &self,
        personnel_id: i64,
        date: NaiveDate,
        matin_arrivee: Option<NaiveTime>,
        matin_sortie: Option<NaiveTime>,
        aprem_arrivee: Option<NaiveTime>,
        aprem_sortie: Option<NaiveTime>,
        observation: Option<String>,
    ) -> Result<(), String> {
        self.with_conn(|conn| {
            let fmt = |t: Option<NaiveTime>| t.map(|x| x.format("%H:%M:%S").to_string());
            conn.execute(
                "INSERT INTO EnregistrementPointage
                 (PersonnelId, DatePointage, MatinArrivee, MatinSortie, ApresMidiArrivee, ApresMidiSortie, Observation)
                 VALUES (?1,?2,?3,?4,?5,?6,?7)
                 ON CONFLICT(PersonnelId, DatePointage) DO UPDATE SET
                    MatinArrivee = excluded.MatinArrivee,
                    MatinSortie = excluded.MatinSortie,
                    ApresMidiArrivee = excluded.ApresMidiArrivee,
                    ApresMidiSortie = excluded.ApresMidiSortie,
                    Observation = excluded.Observation",
                params![
                    personnel_id,
                    date.format("%Y-%m-%d").to_string(),
                    fmt(matin_arrivee),
                    fmt(matin_sortie),
                    fmt(aprem_arrivee),
                    fmt(aprem_sortie),
                    observation
                ],
            )
            .map_err(|e| e.to_string())?;
            Ok(())
        })
    }

    pub fn supprimer_pointage(&self, personnel_id: i64, date: NaiveDate) -> Result<(), String> {
        self.with_conn(|conn| {
            conn.execute(
                "DELETE FROM EnregistrementPointage WHERE PersonnelId = ?1 AND DatePointage = ?2",
                params![personnel_id, date.format("%Y-%m-%d").to_string()],
            )
            .map_err(|e| e.to_string())?;
            Ok(())
        })
    }

    pub fn get_statistiques_mois(
        &self,
        annee: i32,
        mois: u32,
    ) -> Result<HashMap<u32, i64>, String> {
        self.with_conn(|conn| {
            let mut stmt = conn
                .prepare(
                    "SELECT CAST(strftime('%d', DatePointage) AS INTEGER) AS Jour, COUNT(*)
                     FROM EnregistrementPointage
                     WHERE strftime('%Y', DatePointage) = ?1 AND strftime('%m', DatePointage) = ?2
                       AND (MatinArrivee IS NOT NULL OR MatinSortie IS NOT NULL
                            OR ApresMidiArrivee IS NOT NULL OR ApresMidiSortie IS NOT NULL)
                     GROUP BY Jour",
                )
                .map_err(|e| e.to_string())?;
            let iter = stmt
                .query_map(
                    params![format!("{:04}", annee), format!("{:02}", mois)],
                    |r| Ok((r.get::<_, u32>(0)?, r.get::<_, i64>(1)?)),
                )
                .map_err(|e| e.to_string())?;
            let mut map = HashMap::new();
            for item in iter {
                let (j, n) = item.map_err(|e| e.to_string())?;
                map.insert(j, n);
            }
            Ok(map)
        })
    }

    pub fn get_jours_pointes_du_mois(
        &self,
        personnel_id: i64,
        annee: i32,
        mois: u32,
    ) -> Result<HashSet<u32>, String> {
        self.with_conn(|conn| {
            let mut stmt = conn
                .prepare(
                    "SELECT DISTINCT CAST(strftime('%d', DatePointage) AS INTEGER)
                     FROM EnregistrementPointage
                     WHERE PersonnelId = ?1
                       AND strftime('%Y', DatePointage) = ?2
                       AND strftime('%m', DatePointage) = ?3
                       AND (MatinArrivee IS NOT NULL OR MatinSortie IS NOT NULL
                            OR ApresMidiArrivee IS NOT NULL OR ApresMidiSortie IS NOT NULL)",
                )
                .map_err(|e| e.to_string())?;
            let iter = stmt
                .query_map(
                    params![
                        personnel_id,
                        format!("{:04}", annee),
                        format!("{:02}", mois)
                    ],
                    |r| r.get::<_, u32>(0),
                )
                .map_err(|e| e.to_string())?;
            let mut set = HashSet::new();
            for item in iter {
                set.insert(item.map_err(|e| e.to_string())?);
            }
            Ok(set)
        })
    }

    pub fn get_jours_feries_du_mois(
        &self,
        annee: i32,
        mois: u32,
    ) -> Result<HashMap<u32, Vec<String>>, String> {
        self.with_conn(|conn| {
            let mut stmt = conn
                .prepare(
                    "SELECT CAST(strftime('%d', DatePointage) AS INTEGER) AS Jour, Periode
                     FROM JourFerie
                     WHERE strftime('%Y', DatePointage) = ?1
                       AND strftime('%m', DatePointage) = ?2
                     ORDER BY Jour",
                )
                .map_err(|e| e.to_string())?;
            let iter = stmt
                .query_map(
                    params![format!("{:04}", annee), format!("{:02}", mois)],
                    |r| Ok((r.get::<_, u32>(0)?, r.get::<_, String>(1)?)),
                )
                .map_err(|e| e.to_string())?;
            let mut map: HashMap<u32, Vec<String>> = HashMap::new();
            for item in iter {
                let (j, p) = item.map_err(|e| e.to_string())?;
                map.entry(j).or_default().push(p);
            }
            Ok(map)
        })
    }

    pub fn get_ferie_jour(&self, date: NaiveDate) -> Result<Option<String>, String> {
        self.with_conn(|conn| {
            let date_str = date.format("%Y-%m-%d").to_string();
            let mut stmt = conn
                .prepare("SELECT Periode FROM JourFerie WHERE DatePointage = ?1")
                .map_err(|e| e.to_string())?;
            let iter = stmt
                .query_map(params![date_str], |r| r.get::<_, String>(0))
                .map_err(|e| e.to_string())?;
            let mut periodes = Vec::new();
            for p in iter {
                periodes.push(p.map_err(|e| e.to_string())?);
            }
            if periodes.is_empty() {
                Ok(None)
            } else if periodes.iter().any(|p| p == "jour") {
                Ok(Some("jour".into()))
            } else {
                Ok(Some(periodes[0].clone()))
            }
        })
    }

    pub fn set_ferie(
        &self,
        date: NaiveDate,
        periode: &str,
        actif: bool,
    ) -> Result<(), String> {
        self.with_conn(|conn| {
            let date_str = date.format("%Y-%m-%d").to_string();
            if actif {
                conn.execute(
                    "INSERT OR IGNORE INTO JourFerie (DatePointage, Periode) VALUES (?1, ?2)",
                    params![date_str, periode],
                )
                .map_err(|e| e.to_string())?;
            } else {
                conn.execute(
                    "DELETE FROM JourFerie WHERE DatePointage = ?1 AND Periode = ?2",
                    params![date_str, periode],
                )
                .map_err(|e| e.to_string())?;
            }
            Ok(())
        })
    }

    pub fn clear_ferie_jour(&self, date: NaiveDate) -> Result<(), String> {
        self.with_conn(|conn| {
            conn.execute(
                "DELETE FROM JourFerie WHERE DatePointage = ?1",
                params![date.format("%Y-%m-%d").to_string()],
            )
            .map_err(|e| e.to_string())?;
            Ok(())
        })
    }

    pub fn get_conges_personnel(&self, personnel_id: i64) -> Result<Vec<Conge>, String> {
        self.with_conn(|conn| Self::query_conges_personnel(conn, personnel_id))
    }

    fn query_conges_personnel(
        conn: &Connection,
        personnel_id: i64,
    ) -> Result<Vec<Conge>, String> {
        let mut stmt = conn
            .prepare(
                "SELECT Id, PersonnelId, DateDebut, DateFin, Periode
                 FROM Conge WHERE PersonnelId = ?1 ORDER BY DateDebut",
            )
            .map_err(|e| e.to_string())?;
        let iter = stmt
            .query_map(params![personnel_id], |r| {
                let dd: String = r.get(2)?;
                let df: String = r.get(3)?;
                Ok(Conge {
                    id: r.get(0)?,
                    personnel_id: r.get(1)?,
                    date_debut: NaiveDate::parse_from_str(&dd, "%Y-%m-%d")
                        .unwrap_or_else(|_| chrono::Local::now().date_naive()),
                    date_fin: NaiveDate::parse_from_str(&df, "%Y-%m-%d")
                        .unwrap_or_else(|_| chrono::Local::now().date_naive()),
                    periode: r.get(4).unwrap_or_else(|_| "jour".to_string()),
                })
            })
            .map_err(|e| e.to_string())?;
        iter.collect::<Result<Vec<_>, _>>().map_err(|e| e.to_string())
    }

    pub fn get_conges_du_mois(
        &self,
        annee: i32,
        mois: u32,
    ) -> Result<Vec<Conge>, String> {
        self.with_conn(|conn| {
            let debut_mois = NaiveDate::from_ymd_opt(annee, mois, 1).unwrap();
            let nb = jours_dans_mois(annee, mois);
            let fin_mois = NaiveDate::from_ymd_opt(annee, mois, nb).unwrap();
            let mut stmt = conn
                .prepare(
                    "SELECT Id, PersonnelId, DateDebut, DateFin, Periode
                     FROM Conge
                     WHERE DateFin >= ?1 AND DateDebut <= ?2
                     ORDER BY PersonnelId, DateDebut",
                )
                .map_err(|e| e.to_string())?;
            let iter = stmt
                .query_map(
                    params![
                        debut_mois.format("%Y-%m-%d").to_string(),
                        fin_mois.format("%Y-%m-%d").to_string()
                    ],
                    |r| {
                        let dd: String = r.get(2)?;
                        let df: String = r.get(3)?;
                        Ok(Conge {
                            id: r.get(0)?,
                            personnel_id: r.get(1)?,
                            date_debut: NaiveDate::parse_from_str(&dd, "%Y-%m-%d")
                                .unwrap_or_else(|_| chrono::Local::now().date_naive()),
                            date_fin: NaiveDate::parse_from_str(&df, "%Y-%m-%d")
                                .unwrap_or_else(|_| chrono::Local::now().date_naive()),
                            periode: r.get(4).unwrap_or_else(|_| "jour".to_string()),
                        })
                    },
                )
                .map_err(|e| e.to_string())?;
            iter.collect::<Result<Vec<_>, _>>().map_err(|e| e.to_string())
        })
    }

    pub fn conges_du_jour_pour(
        &self,
        personnel_id: i64,
        date: NaiveDate,
    ) -> Result<Vec<String>, String> {
        self.with_conn(|conn| {
            let date_str = date.format("%Y-%m-%d").to_string();
            let mut stmt = conn
                .prepare(
                    "SELECT Periode FROM Conge
                     WHERE PersonnelId = ?1 AND DateDebut <= ?2 AND DateFin >= ?2",
                )
                .map_err(|e| e.to_string())?;
            let iter = stmt
                .query_map(params![personnel_id, date_str], |r| r.get::<_, String>(0))
                .map_err(|e| e.to_string())?;
            let mut out = Vec::new();
            for p in iter {
                out.push(p.map_err(|e| e.to_string())?);
            }
            Ok(out)
        })
    }

    pub fn ajouter_conge(
        &self,
        personnel_id: i64,
        debut: NaiveDate,
        fin: NaiveDate,
        periode: &str,
    ) -> Result<i64, String> {
        if fin < debut {
            return Err("La date de fin doit etre posterieure ou egale a la date de debut.".into());
        }
        if !matches!(periode, "jour" | "matin" | "aprem") {
            return Err("Periode invalide.".into());
        }
        self.with_conn(|conn| {
            conn.execute(
                "INSERT INTO Conge (PersonnelId, DateDebut, DateFin, Periode) VALUES (?1, ?2, ?3, ?4)",
                params![
                    personnel_id,
                    debut.format("%Y-%m-%d").to_string(),
                    fin.format("%Y-%m-%d").to_string(),
                    periode
                ],
            )
            .map_err(|e| e.to_string())?;
            Ok(conn.last_insert_rowid())
        })
    }

    pub fn supprimer_conge(&self, conge_id: i64) -> Result<(), String> {
        self.with_conn(|conn| {
            conn.execute("DELETE FROM Conge WHERE Id = ?1", params![conge_id])
                .map_err(|e| e.to_string())?;
            Ok(())
        })
    }

    fn query_bilan(
        conn: &Connection,
        debut: NaiveDate,
        fin: NaiveDate,
    ) -> Result<Vec<BilanPersonnel>, String> {
        let mut stmt = conn
            .prepare(
                "SELECT p.Nom, p.Prenom,
                        COUNT(ep.Id) AS JoursPresents,
                        SUM(CASE WHEN ep.MatinArrivee IS NOT NULL OR ep.MatinSortie IS NOT NULL THEN 1 ELSE 0 END),
                        SUM(CASE WHEN ep.ApresMidiArrivee IS NOT NULL OR ep.ApresMidiSortie IS NOT NULL THEN 1 ELSE 0 END),
                        SUM(
                          COALESCE((strftime('%s', ep.MatinSortie) - strftime('%s', ep.MatinArrivee)), 0)
                          + COALESCE((strftime('%s', ep.ApresMidiSortie) - strftime('%s', ep.ApresMidiArrivee)), 0)
                        ),
                        AVG(strftime('%s', ep.MatinArrivee))
                 FROM Personnel p
                 LEFT JOIN EnregistrementPointage ep
                     ON ep.PersonnelId = p.Id AND ep.DatePointage BETWEEN ?1 AND ?2
                 GROUP BY p.Id, p.Nom, p.Prenom
                 ORDER BY p.Nom, p.Prenom",
            )
            .map_err(|e| e.to_string())?;
        let iter = stmt
            .query_map(
                params![
                    debut.format("%Y-%m-%d").to_string(),
                    fin.format("%Y-%m-%d").to_string()
                ],
                |r| {
                    let sec_total: Option<f64> = r.get(5).ok();
                    let moy: Option<f64> = r.get(6).ok();
                    let total_heures = sec_total
                        .map(|s| {
                            let h = (s / 3600.0) as i64;
                            let m = ((s % 3600.0) / 60.0) as i64;
                            format!("{}h{:02}", h, m)
                        })
                        .unwrap_or_else(|| "-".into());
                    let moyenne_arrivee = moy
                        .map(|s| {
                            let t = s as i64 % 86400;
                            format!("{:02}:{:02}", t / 3600, (t % 3600) / 60)
                        })
                        .unwrap_or_else(|| "-".into());
                    Ok(BilanPersonnel {
                        nom: format!("{} {}", r.get::<_, String>(0)?, r.get::<_, String>(1)?)
                            .trim()
                            .to_string(),
                        jours_presents: r.get::<_, Option<i64>>(2)?.unwrap_or(0),
                        matins_presents: r.get::<_, Option<i64>>(3)?.unwrap_or(0),
                        apres_midis_presents: r.get::<_, Option<i64>>(4)?.unwrap_or(0),
                        total_heures,
                        moyenne_arrivee,
                    })
                },
            )
            .map_err(|e| e.to_string())?;
        iter.collect::<Result<Vec<_>, _>>().map_err(|e| e.to_string())
    }

    fn charger_feries_par_date(
        conn: &Connection,
        debut: NaiveDate,
        fin: NaiveDate,
    ) -> Result<HashMap<NaiveDate, Vec<String>>, String> {
        let mut map: HashMap<NaiveDate, Vec<String>> = HashMap::new();
        let mut stmt = conn
            .prepare(
                "SELECT DatePointage, Periode
                 FROM JourFerie
                 WHERE DatePointage BETWEEN ?1 AND ?2",
            )
            .map_err(|e| e.to_string())?;
        let iter = stmt
            .query_map(
                params![
                    debut.format("%Y-%m-%d").to_string(),
                    fin.format("%Y-%m-%d").to_string()
                ],
                |r| Ok((r.get::<_, String>(0)?, r.get::<_, String>(1)?)),
            )
            .map_err(|e| e.to_string())?;
        for item in iter {
            let (d, p) = item.map_err(|e| e.to_string())?;
            if let Ok(date) = NaiveDate::parse_from_str(&d, "%Y-%m-%d") {
                map.entry(date).or_default().push(p);
            }
        }
        Ok(map)
    }

    fn charger_conges_par_date(
        conn: &Connection,
        debut: NaiveDate,
        fin: NaiveDate,
    ) -> Result<HashMap<i64, Vec<(NaiveDate, NaiveDate, String)>>, String> {
        let mut map: HashMap<i64, Vec<(NaiveDate, NaiveDate, String)>> = HashMap::new();
        let mut stmt = conn
            .prepare(
                "SELECT PersonnelId, DateDebut, DateFin, Periode
                 FROM Conge
                 WHERE DateFin >= ?1 AND DateDebut <= ?2",
            )
            .map_err(|e| e.to_string())?;
        let iter = stmt
            .query_map(
                params![
                    debut.format("%Y-%m-%d").to_string(),
                    fin.format("%Y-%m-%d").to_string()
                ],
                |r| {
                    let dd: String = r.get(1)?;
                    let df: String = r.get(2)?;
                    Ok((
                        r.get::<_, i64>(0)?,
                        NaiveDate::parse_from_str(&dd, "%Y-%m-%d")
                            .unwrap_or_else(|_| chrono::Local::now().date_naive()),
                        NaiveDate::parse_from_str(&df, "%Y-%m-%d")
                            .unwrap_or_else(|_| chrono::Local::now().date_naive()),
                        r.get::<_, String>(3).unwrap_or_else(|_| "jour".to_string()),
                    ))
                },
            )
            .map_err(|e| e.to_string())?;
        for item in iter {
            let (pid, dd, df, per) = item.map_err(|e| e.to_string())?;
            map.entry(pid).or_default().push((dd, df, per));
        }
        Ok(map)
    }

    fn ferie_entier(feries: &HashMap<NaiveDate, Vec<String>>, date: NaiveDate) -> bool {
        feries
            .get(&date)
            .map(|v| v.iter().any(|p| p == "jour"))
            .unwrap_or(false)
    }

    fn periode_conges(
        conges: &HashMap<i64, Vec<(NaiveDate, NaiveDate, String)>>,
        personnel_id: i64,
        date: NaiveDate,
    ) -> (bool, bool) {
        let mut jour_entier = false;
        let mut matin = false;
        let mut aprem = false;
        if let Some(plages) = conges.get(&personnel_id) {
            for (d, f, p) in plages {
                if date >= *d && date <= *f {
                    match p.as_str() {
                        "jour" => jour_entier = true,
                        "matin" => matin = true,
                        "aprem" => aprem = true,
                        _ => {}
                    }
                }
            }
        }
        if jour_entier {
            (true, true)
        } else {
            (matin, aprem)
        }
    }

    pub fn get_classement_periode(
        &self,
        debut: NaiveDate,
        fin: NaiveDate,
    ) -> Result<Vec<(String, f64, usize)>, String> {
        self.with_conn(|conn| {
            let feries = Self::charger_feries_par_date(conn, debut, fin)?;
            let conges = Self::charger_conges_par_date(conn, debut, fin)?;

            let mut stmt = conn
                .prepare(
                    "SELECT p.Id, p.Nom || ' ' || p.Prenom AS Nom,
                            ep.DatePointage, ep.MatinArrivee, ep.MatinSortie,
                            ep.ApresMidiArrivee, ep.ApresMidiSortie
                     FROM EnregistrementPointage ep
                     INNER JOIN Personnel p ON p.Id = ep.PersonnelId
                     WHERE ep.DatePointage BETWEEN ?1 AND ?2
                     ORDER BY p.Nom, p.Prenom, ep.DatePointage",
                )
                .map_err(|e| e.to_string())?;
            let iter = stmt
                .query_map(
                    params![
                        debut.format("%Y-%m-%d").to_string(),
                        fin.format("%Y-%m-%d").to_string()
                    ],
                    |r| {
                        let date_str: String = r.get(2)?;
                        let date = NaiveDate::parse_from_str(&date_str, "%Y-%m-%d")
                            .unwrap_or_else(|_| chrono::Local::now().date_naive());
                        let parse_time = |opt: Option<String>| -> Option<NaiveTime> {
                            opt.and_then(|s| NaiveTime::parse_from_str(&s, "%H:%M:%S").ok())
                        };
                        Ok((
                            r.get::<_, i64>(0)?,
                            r.get::<_, String>(1)?,
                            date,
                            parse_time(r.get(3)?),
                            parse_time(r.get(4)?),
                            parse_time(r.get(5)?),
                            parse_time(r.get(6)?),
                        ))
                    },
                )
                .map_err(|e| e.to_string())?;

            let mut acc: HashMap<String, (f64, usize)> = HashMap::new();
            for item in iter {
                let (pid, nom, date, ma, ms, aa, as_) = item.map_err(|e| e.to_string())?;
                if matches!(date.weekday(), Weekday::Sat | Weekday::Sun) {
                    continue;
                }
                if Self::ferie_entier(&feries, date) {
                    continue;
                }
                let (conge_matin, conge_aprem) = Self::periode_conges(&conges, pid, date);
                if conge_matin && conge_aprem {
                    continue;
                }
                let matin_bloque = conge_matin
                    || feries
                        .get(&date)
                        .map(|v| v.iter().any(|p| p == "matin"))
                        .unwrap_or(false);
                let aprem_bloque = conge_aprem
                    || feries
                        .get(&date)
                        .map(|v| v.iter().any(|p| p == "aprem"))
                        .unwrap_or(false);
                let mut max_jour = 0.0;
                if !matin_bloque {
                    max_jour += 10.0;
                }
                if !aprem_bloque {
                    max_jour += 10.0;
                }
                if max_jour <= 0.0 {
                    continue;
                }

                let mut pts = 0.0;
                let mut a_matin = false;
                let mut a_aprem = false;

                if !matin_bloque && (ma.is_some() || ms.is_some()) {
                    a_matin = true;
                    if let Some(x) = ma {
                        pts += note_matin_arrivee_5(x);
                    }
                    if let Some(x) = ms {
                        pts += note_matin_sortie_5(x);
                    }
                }
                if !aprem_bloque && (aa.is_some() || as_.is_some()) {
                    a_aprem = true;
                    if let Some(x) = aa {
                        pts += note_aprem_arrivee_5(x);
                    }
                    if let Some(x) = as_ {
                        pts += note_aprem_sortie_5(x);
                    }
                }

                if !(a_matin || a_aprem) {
                    continue;
                }

                let note20 = (pts / max_jour) * 20.0;
                let e = acc.entry(nom).or_insert((0.0, 0));
                e.0 += note20;
                e.1 += 1;
            }

            let jours_ouvrables = {
                let mut total = 0u32;
                let mut d = debut;
                while d <= fin {
                    let weekend = matches!(d.weekday(), Weekday::Sat | Weekday::Sun);
                    if !weekend && !Self::ferie_entier(&feries, d) {
                        total += 1;
                    }
                    d = match d.succ_opt() {
                        Some(n) => n,
                        None => break,
                    };
                }
                total.max(1) as f64
            };

            let mut out: Vec<(String, f64, usize)> = acc
                .into_iter()
                .map(|(nom, (somme, compte))| {
                    (
                        nom,
                        if jours_ouvrables > 0.0 {
                            somme / jours_ouvrables
                        } else {
                            0.0
                        },
                        compte,
                    )
                })
                .collect();

            out.sort_by(|a, b| {
                b.1.partial_cmp(&a.1)
                    .unwrap_or(std::cmp::Ordering::Equal)
                    .then(b.2.cmp(&a.2))
                    .then(a.0.cmp(&b.0))
            });

            Ok(out)
        })
    }

    pub fn exporter_snapshot(
        &self,
        debut: NaiveDate,
        fin: NaiveDate,
    ) -> Result<ExportSnapshot, String> {
        self.with_conn(|conn| {
            conn.execute_batch("BEGIN").map_err(|e| e.to_string())?;
            let resultat = (|| -> Result<ExportSnapshot, String> {
                let personnel = Self::query_personnel(conn)?;
                let pointages = Self::query_pointages_periode(conn, debut, fin)?;
                let bilan = Self::query_bilan(conn, debut, fin)?;
                let feries = {
                    let mut map: HashMap<String, Vec<String>> = HashMap::new();
                    let mut stmt = conn
                        .prepare(
                            "SELECT DatePointage, Periode FROM JourFerie
                             WHERE DatePointage BETWEEN ?1 AND ?2
                             ORDER BY DatePointage, Periode",
                        )
                        .map_err(|e| e.to_string())?;
                    let iter = stmt
                        .query_map(
                            params![
                                debut.format("%Y-%m-%d").to_string(),
                                fin.format("%Y-%m-%d").to_string()
                            ],
                            |r| Ok((r.get::<_, String>(0)?, r.get::<_, String>(1)?)),
                        )
                        .map_err(|e| e.to_string())?;
                    for item in iter {
                        let (d, p) = item.map_err(|e| e.to_string())?;
                        map.entry(d).or_default().push(p);
                    }
                    map
                };
                let conges = {
                    let mut out = Vec::new();
                    let mut stmt = conn
                        .prepare(
                            "SELECT Id, PersonnelId, DateDebut, DateFin, Periode FROM Conge
                             WHERE DateFin >= ?1 AND DateDebut <= ?2
                             ORDER BY PersonnelId, DateDebut",
                        )
                        .map_err(|e| e.to_string())?;
                    let iter = stmt
                        .query_map(
                            params![
                                debut.format("%Y-%m-%d").to_string(),
                                fin.format("%Y-%m-%d").to_string()
                            ],
                            |r| {
                                let dd: String = r.get(2)?;
                                let df: String = r.get(3)?;
                                Ok(Conge {
                                    id: r.get(0)?,
                                    personnel_id: r.get(1)?,
                                    date_debut: NaiveDate::parse_from_str(&dd, "%Y-%m-%d")
                                        .unwrap_or_else(|_| {
                                            chrono::Local::now().date_naive()
                                        }),
                                    date_fin: NaiveDate::parse_from_str(&df, "%Y-%m-%d")
                                        .unwrap_or_else(|_| {
                                            chrono::Local::now().date_naive()
                                        }),
                                    periode: r.get(4).unwrap_or_else(|_| "jour".to_string()),
                                })
                            },
                        )
                        .map_err(|e| e.to_string())?;
                    for item in iter {
                        out.push(item.map_err(|e| e.to_string())?);
                    }
                    out
                };
                let nom_entreprise = conn
                    .query_row(
                        "SELECT Valeur FROM Parametre WHERE Cle='NomEntreprise'",
                        [],
                        |r| r.get::<_, String>(0),
                    )
                    .unwrap_or_else(|_| "Pointage".into());
                Ok(ExportSnapshot {
                    personnel,
                    pointages,
                    bilan,
                    feries,
                    conges,
                    nom_entreprise,
                })
            })();
            match resultat {
                Ok(snap) => {
                    conn.execute_batch("COMMIT").map_err(|e| e.to_string())?;
                    Ok(snap)
                }
                Err(e) => {
                    let _ = conn.execute_batch("ROLLBACK");
                    Err(e)
                }
            }
        })
    }

    pub fn exporter_base(&self, destination: &str) -> Result<(), String> {
        let dest_path = PathBuf::from(destination);
        if dest_path.exists() {
            std::fs::remove_file(&dest_path).map_err(|e| e.to_string())?;
        }
        self.with_conn(|conn| {
            let esc = destination.replace('\'', "''");
            conn.execute_batch(&format!("VACUUM INTO '{}';", esc))
                .map_err(|e| e.to_string())?;
            Ok(())
        })
    }

    fn sauvegarde_avant_import(&self) -> Result<PathBuf, String> {
        let actif = self
            .chemin_actif
            .borrow()
            .clone()
            .ok_or("Aucune base active a sauvegarder.".to_string())?;

        let dossier_backups = self.dossier.join("backups");
        std::fs::create_dir_all(&dossier_backups).map_err(|e| e.to_string())?;

        let nom_fichier = actif
            .file_stem()
            .and_then(|s| s.to_str())
            .unwrap_or("base")
            .to_string();

        let horodatage = chrono::Local::now().format("%Y%m%d_%H%M%S").to_string();
        let nom_backup = format!("{}_{}.db", nom_fichier, horodatage);
        let chemin_backup = dossier_backups.join(nom_backup);

        if chemin_backup.exists() {
            std::fs::remove_file(&chemin_backup).map_err(|e| e.to_string())?;
        }

        let dest_str = chemin_backup.to_string_lossy().to_string();
        let esc = dest_str.replace('\'', "''");
        self.with_conn(|conn| {
            conn.execute_batch(&format!("VACUUM INTO '{}';", esc))
                .map_err(|e| e.to_string())?;
            Ok(())
        })?;

        Ok(chemin_backup)
    }

    pub fn importer_base(&self, source: &str) -> Result<(), String> {
        let source_path = PathBuf::from(source);
        if !source_path.exists() {
            return Err(format!("Fichier source introuvable : {}", source));
        }

        let source_canon = std::fs::canonicalize(&source_path).map_err(|e| e.to_string())?;

        if let Some(actif) = self.chemin_actif.borrow().clone() {
            if let Ok(actif_canon) = std::fs::canonicalize(&actif) {
                if actif_canon == source_canon {
                    return Err(
                        "Impossible d'importer la base actuellement ouverte dans elle-meme."
                            .into(),
                    );
                }
            }
        }

        let _ = self.sauvegarde_avant_import()?;

        let verif = Connection::open_with_flags(
            &source_canon,
            rusqlite::OpenFlags::SQLITE_OPEN_READ_ONLY | rusqlite::OpenFlags::SQLITE_OPEN_URI,
        )
        .map_err(|e| {
            format!(
                "Impossible d'ouvrir le fichier a importer (corrompu ?) : {}",
                e
            )
        })?;

        let tables_ok: i64 = verif
            .query_row(
                "SELECT COUNT(*) FROM sqlite_master
                 WHERE type='table'
                   AND name IN ('Personnel','EnregistrementPointage','JourFerie')",
                [],
                |r| r.get(0),
            )
            .unwrap_or(0);
        if tables_ok < 3 {
            return Err(
                "Le fichier selectionne n'est pas une base Pointage valide (tables manquantes)."
                    .into(),
            );
        }

        let colonnes_personnel: i64 = verif
            .query_row(
                "SELECT COUNT(*) FROM pragma_table_info('Personnel')
                 WHERE name IN ('Id','Nom','Prenom','Matricule','Telephone','Adresse','Email','Statut','Grade','SituationFamiliale')",
                [],
                |r| r.get(0),
            )
            .unwrap_or(0);
        if colonnes_personnel < 10 {
            return Err("Le fichier source a un schema Personnel incompatible.".into());
        }

        let colonnes_pointage: i64 = verif
            .query_row(
                "SELECT COUNT(*) FROM pragma_table_info('EnregistrementPointage')
                 WHERE name IN ('Id','PersonnelId','DatePointage','MatinArrivee','MatinSortie','ApresMidiArrivee','ApresMidiSortie','Observation')",
                [],
                |r| r.get(0),
            )
            .unwrap_or(0);
        if colonnes_pointage < 8 {
            return Err("Le fichier source a un schema EnregistrementPointage incompatible.".into());
        }

        let a_date_arrivee: i64 = verif
            .query_row(
                "SELECT COUNT(*) FROM pragma_table_info('Personnel') WHERE name = 'DateArrivee'",
                [],
                |r| r.get(0),
            )
            .unwrap_or(0);

        let a_conge: i64 = verif
            .query_row(
                "SELECT COUNT(*) FROM sqlite_master WHERE type='table' AND name='Conge'",
                [],
                |r| r.get(0),
            )
            .unwrap_or(0);

        let a_conge_periode: i64 = if a_conge > 0 {
            verif
                .query_row(
                    "SELECT COUNT(*) FROM pragma_table_info('Conge') WHERE name = 'Periode'",
                    [],
                    |r| r.get(0),
                )
                .unwrap_or(0)
        } else {
            0
        };

        type LignePersonnel = (
            i64, String, String,
            Option<String>, Option<String>, Option<String>,
            Option<String>, Option<String>, Option<String>, Option<String>,
            Option<String>,
        );
        type LignePointage = (
            i64, i64, String,
            Option<String>, Option<String>, Option<String>, Option<String>,
            Option<String>,
        );
        type LigneConge = (i64, i64, String, String, String);

        let personnel: Vec<LignePersonnel> = {
            let sql = if a_date_arrivee > 0 {
                "SELECT Id, Nom, Prenom, Matricule, Telephone, Adresse, Email, Statut, Grade, SituationFamiliale, DateArrivee
                 FROM Personnel ORDER BY Id"
            } else {
                "SELECT Id, Nom, Prenom, Matricule, Telephone, Adresse, Email, Statut, Grade, SituationFamiliale, NULL
                 FROM Personnel ORDER BY Id"
            };
            let mut stmt = verif.prepare(sql).map_err(|e| e.to_string())?;
            let iter = stmt
                .query_map([], |r| {
                    Ok((
                        r.get(0)?, r.get(1)?, r.get(2)?, r.get(3)?, r.get(4)?,
                        r.get(5)?, r.get(6)?, r.get(7)?, r.get(8)?, r.get(9)?,
                        r.get(10)?,
                    ))
                })
                .map_err(|e| e.to_string())?;
            iter.collect::<Result<Vec<_>, _>>().map_err(|e| e.to_string())?
        };

        let pointages: Vec<LignePointage> = {
            let mut stmt = verif
                .prepare(
                    "SELECT Id, PersonnelId, DatePointage,
                            MatinArrivee, MatinSortie, ApresMidiArrivee, ApresMidiSortie, Observation
                     FROM EnregistrementPointage ORDER BY Id",
                )
                .map_err(|e| e.to_string())?;
            let iter = stmt
                .query_map([], |r| {
                    Ok((
                        r.get(0)?, r.get(1)?, r.get(2)?, r.get(3)?,
                        r.get(4)?, r.get(5)?, r.get(6)?, r.get(7)?,
                    ))
                })
                .map_err(|e| e.to_string())?;
            iter.collect::<Result<Vec<_>, _>>().map_err(|e| e.to_string())?
        };

        let feries: Vec<(i64, String, String)> = {
            let mut stmt = verif
                .prepare("SELECT Id, DatePointage, Periode FROM JourFerie ORDER BY Id")
                .map_err(|e| e.to_string())?;
            let iter = stmt
                .query_map([], |r| Ok((r.get(0)?, r.get(1)?, r.get(2)?)))
                .map_err(|e| e.to_string())?;
            iter.collect::<Result<Vec<_>, _>>().map_err(|e| e.to_string())?
        };

        let conges: Vec<LigneConge> = if a_conge > 0 {
            let sql = if a_conge_periode > 0 {
                "SELECT Id, PersonnelId, DateDebut, DateFin, Periode FROM Conge ORDER BY Id"
            } else {
                "SELECT Id, PersonnelId, DateDebut, DateFin, 'jour' FROM Conge ORDER BY Id"
            };
            let mut stmt = verif.prepare(sql).map_err(|e| e.to_string())?;
            let iter = stmt
                .query_map([], |r| {
                    Ok((r.get(0)?, r.get(1)?, r.get(2)?, r.get(3)?, r.get(4)?))
                })
                .map_err(|e| e.to_string())?;
            iter.collect::<Result<Vec<_>, _>>().map_err(|e| e.to_string())?
        } else {
            Vec::new()
        };

        let parametres: Vec<(String, Option<String>)> = {
            let existe: i64 = verif
                .query_row(
                    "SELECT COUNT(*) FROM sqlite_master WHERE type='table' AND name='Parametre'",
                    [],
                    |r| r.get(0),
                )
                .unwrap_or(0);
            if existe == 0 {
                Vec::new()
            } else {
                let mut stmt = verif
                    .prepare("SELECT Cle, Valeur FROM Parametre")
                    .map_err(|e| e.to_string())?;
                let iter = stmt
                    .query_map([], |r| Ok((r.get(0)?, r.get(1)?)))
                    .map_err(|e| e.to_string())?;
                iter.collect::<Result<Vec<_>, _>>().map_err(|e| e.to_string())?
            }
        };

        drop(verif);

        self.with_conn(|conn| {
            conn.execute_batch("BEGIN").map_err(|e| e.to_string())?;

            let resultat = (|| -> Result<(), String> {
                let ids_personnel_locaux: HashSet<i64> = {
                    let mut stmt = conn
                        .prepare("SELECT Id FROM Personnel")
                        .map_err(|e| e.to_string())?;
                    let iter = stmt
                        .query_map([], |r| r.get::<_, i64>(0))
                        .map_err(|e| e.to_string())?;
                    let mut set = HashSet::new();
                    for item in iter {
                        set.insert(item.map_err(|e| e.to_string())?);
                    }
                    set
                };

                let cles_pointage_locales: HashSet<(i64, String)> = {
                    let mut stmt = conn
                        .prepare("SELECT PersonnelId, DatePointage FROM EnregistrementPointage")
                        .map_err(|e| e.to_string())?;
                    let iter = stmt
                        .query_map([], |r| Ok((r.get::<_, i64>(0)?, r.get::<_, String>(1)?)))
                        .map_err(|e| e.to_string())?;
                    let mut set = HashSet::new();
                    for item in iter {
                        set.insert(item.map_err(|e| e.to_string())?);
                    }
                    set
                };

                let cles_ferie_locales: HashSet<(String, String)> = {
                    let mut stmt = conn
                        .prepare("SELECT DatePointage, Periode FROM JourFerie")
                        .map_err(|e| e.to_string())?;
                    let iter = stmt
                        .query_map([], |r| Ok((r.get::<_, String>(0)?, r.get::<_, String>(1)?)))
                        .map_err(|e| e.to_string())?;
                    let mut set = HashSet::new();
                    for item in iter {
                        set.insert(item.map_err(|e| e.to_string())?);
                    }
                    set
                };

                let ids_conge_locaux: HashSet<i64> = {
                    let mut stmt = conn
                        .prepare("SELECT Id FROM Conge")
                        .map_err(|e| e.to_string())?;
                    let iter = stmt
                        .query_map([], |r| r.get::<_, i64>(0))
                        .map_err(|e| e.to_string())?;
                    let mut set = HashSet::new();
                    for item in iter {
                        set.insert(item.map_err(|e| e.to_string())?);
                    }
                    set
                };

                let cles_param_locales: HashSet<String> = {
                    let mut stmt = conn
                        .prepare("SELECT Cle FROM Parametre")
                        .map_err(|e| e.to_string())?;
                    let iter = stmt
                        .query_map([], |r| r.get::<_, String>(0))
                        .map_err(|e| e.to_string())?;
                    let mut set = HashSet::new();
                    for item in iter {
                        set.insert(item.map_err(|e| e.to_string())?);
                    }
                    set
                };

                let mut ids_personnel_effectifs: HashSet<i64> = ids_personnel_locaux.clone();

                {
                    let mut stmt = conn
                        .prepare(
                            "INSERT OR IGNORE INTO Personnel
                             (Id, Nom, Prenom, Matricule, Telephone, Adresse, Email, Statut, Grade, SituationFamiliale, DateArrivee)
                             VALUES (?1,?2,?3,?4,?5,?6,?7,?8,?9,?10,?11)",
                        )
                        .map_err(|e| e.to_string())?;
                    for p in &personnel {
                        if ids_personnel_locaux.contains(&p.0) {
                            continue;
                        }
                        stmt.execute(params![
                            p.0, p.1, p.2, p.3, p.4, p.5, p.6, p.7, p.8, p.9, p.10
                        ])
                        .map_err(|e| e.to_string())?;
                        ids_personnel_effectifs.insert(p.0);
                    }
                }

                {
                    let mut stmt = conn
                        .prepare(
                            "INSERT OR IGNORE INTO EnregistrementPointage
                             (Id, PersonnelId, DatePointage,
                              MatinArrivee, MatinSortie, ApresMidiArrivee, ApresMidiSortie, Observation)
                             VALUES (?1,?2,?3,?4,?5,?6,?7,?8)",
                        )
                        .map_err(|e| e.to_string())?;
                    for pt in &pointages {
                        if !ids_personnel_effectifs.contains(&pt.1) {
                            continue;
                        }
                        let cle = (pt.1, pt.2.clone());
                        if cles_pointage_locales.contains(&cle) {
                            continue;
                        }
                        let _ = stmt.execute(params![
                            pt.0, pt.1, pt.2, pt.3, pt.4, pt.5, pt.6, pt.7
                        ]);
                    }
                }

                {
                    let mut stmt = conn
                        .prepare(
                            "INSERT OR IGNORE INTO JourFerie (Id, DatePointage, Periode) VALUES (?1,?2,?3)",
                        )
                        .map_err(|e| e.to_string())?;
                    for f in &feries {
                        let cle = (f.1.clone(), f.2.clone());
                        if cles_ferie_locales.contains(&cle) {
                            continue;
                        }
                        let _ = stmt.execute(params![f.0, f.1, f.2]);
                    }
                }

                {
                    let mut stmt = conn
                        .prepare(
                            "INSERT OR IGNORE INTO Conge (Id, PersonnelId, DateDebut, DateFin, Periode) VALUES (?1,?2,?3,?4,?5)",
                        )
                        .map_err(|e| e.to_string())?;
                    for c in &conges {
                        if ids_conge_locaux.contains(&c.0) {
                            continue;
                        }
                        if !ids_personnel_effectifs.contains(&c.1) {
                            continue;
                        }
                        let _ = stmt.execute(params![c.0, c.1, c.2, c.3, c.4]);
                    }
                }

                {
                    let mut stmt = conn
                        .prepare("INSERT OR IGNORE INTO Parametre (Cle, Valeur) VALUES (?1,?2)")
                        .map_err(|e| e.to_string())?;
                    for p in &parametres {
                        if cles_param_locales.contains(&p.0) {
                            continue;
                        }
                        let _ = stmt.execute(params![p.0, p.1]);
                    }
                    conn.execute(
                        "INSERT OR IGNORE INTO Parametre (Cle, Valeur) VALUES ('NomEntreprise','Pointage')",
                        [],
                    )
                    .map_err(|e| e.to_string())?;
                }

                let max_personnel: i64 = conn
                    .query_row(
                        "SELECT COALESCE(MAX(Id), 0) FROM Personnel",
                        [],
                        |r| r.get(0),
                    )
                    .unwrap_or(0);
                let max_pointage: i64 = conn
                    .query_row(
                        "SELECT COALESCE(MAX(Id), 0) FROM EnregistrementPointage",
                        [],
                        |r| r.get(0),
                    )
                    .unwrap_or(0);
                let max_ferie: i64 = conn
                    .query_row(
                        "SELECT COALESCE(MAX(Id), 0) FROM JourFerie",
                        [],
                        |r| r.get(0),
                    )
                    .unwrap_or(0);
                let max_conge: i64 = conn
                    .query_row(
                        "SELECT COALESCE(MAX(Id), 0) FROM Conge",
                        [],
                        |r| r.get(0),
                    )
                    .unwrap_or(0);

                for (table, maxid) in [
                    ("Personnel", max_personnel),
                    ("EnregistrementPointage", max_pointage),
                    ("JourFerie", max_ferie),
                    ("Conge", max_conge),
                ] {
                    let exists: i64 = conn
                        .query_row(
                            "SELECT COUNT(*) FROM sqlite_sequence WHERE name = ?1",
                            params![table],
                            |r| r.get(0),
                        )
                        .unwrap_or(0);
                    if exists > 0 {
                        let _ = conn.execute(
                            "UPDATE sqlite_sequence SET seq = MAX(seq, ?1) WHERE name = ?2",
                            params![maxid, table],
                        );
                    } else {
                        let _ = conn.execute(
                            "INSERT INTO sqlite_sequence (name, seq) VALUES (?1, ?2)",
                            params![table, maxid],
                        );
                    }
                }

                Ok(())
            })();

            match resultat {
                Ok(_) => {
                    conn.execute_batch("COMMIT").map_err(|e| e.to_string())?;
                    Ok(())
                }
                Err(e) => {
                    let _ = conn.execute_batch("ROLLBACK");
                    Err(e)
                }
            }
        })
    }
}

fn jours_dans_mois(annee: i32, mois: u32) -> u32 {
    match mois {
        1 | 3 | 5 | 7 | 8 | 10 | 12 => 31,
        4 | 6 | 9 | 11 => 30,
        2 => {
            if (annee % 4 == 0 && annee % 100 != 0) || annee % 400 == 0 {
                29
            } else {
                28
            }
        }
        _ => 30,
    }
}
