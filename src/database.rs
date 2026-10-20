use crate::models::*;
use crate::securite;
use chrono::{NaiveDate, NaiveTime};
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
    pub nom_entreprise: String,
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

    #[allow(dead_code)]
    pub fn dossier_donnees(&self) -> &PathBuf {
        &self.dossier
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

    #[allow(dead_code)]
    pub fn avec_transaction<F, R>(&self, f: F) -> Result<R, String>
    where
        F: FnOnce(&Connection) -> Result<R, String>,
    {
        self.with_conn(|conn| {
            conn.execute_batch("BEGIN").map_err(|e| e.to_string())?;
            let r = f(conn);
            match r {
                Ok(v) => {
                    conn.execute_batch("COMMIT").map_err(|e| e.to_string())?;
                    Ok(v)
                }
                Err(e) => {
                    let _ = conn.execute_batch("ROLLBACK");
                    Err(e)
                }
            }
        })
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
                SituationFamiliale TEXT
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
            CREATE TABLE IF NOT EXISTS Parametre (
                Cle TEXT PRIMARY KEY,
                Valeur TEXT
            );
            CREATE INDEX IF NOT EXISTS idx_pointage_date ON EnregistrementPointage(DatePointage);
            CREATE INDEX IF NOT EXISTS idx_pointage_personnel ON EnregistrementPointage(PersonnelId);
            CREATE INDEX IF NOT EXISTS idx_pointage_perso_date ON EnregistrementPointage(PersonnelId, DatePointage);
            CREATE INDEX IF NOT EXISTS idx_personnel_nom ON Personnel(Nom, Prenom);
            CREATE INDEX IF NOT EXISTS idx_ferie_date ON JourFerie(DatePointage);
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
            return Err("Le nouveau mot de passe doit contenir au moins 4 caracteres.".into());
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
                "SELECT Id, Nom, Prenom, Matricule, Telephone, Adresse, Email, Statut, Grade, SituationFamiliale
                 FROM Personnel ORDER BY Nom, Prenom",
            )
            .map_err(|e| e.to_string())?;
        let iter = stmt
            .query_map([], |r| {
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
                })
            })
            .map_err(|e| e.to_string())?;
        iter.collect::<Result<Vec<_>, _>>().map_err(|e| e.to_string())
    }

    pub fn ajouter_personnel(&self, p: &Personnel) -> Result<i64, String> {
        self.with_conn(|conn| {
            conn.execute(
                "INSERT INTO Personnel (Nom, Prenom, Matricule, Telephone, Adresse, Email, Statut, Grade, SituationFamiliale)
                 VALUES (?1,?2,?3,?4,?5,?6,?7,?8,?9)",
                params![
                    p.nom, p.prenom, p.matricule, p.telephone, p.adresse,
                    p.email, p.statut, p.grade, p.situation_familiale
                ],
            )
            .map_err(|e| e.to_string())?;
            Ok(conn.last_insert_rowid())
        })
    }

    pub fn modifier_personnel(&self, p: &Personnel) -> Result<(), String> {
        self.with_conn(|conn| {
            conn.execute(
                "UPDATE Personnel SET Nom=?1, Prenom=?2, Matricule=?3, Telephone=?4, Adresse=?5,
                 Email=?6, Statut=?7, Grade=?8, SituationFamiliale=?9 WHERE Id=?10",
                params![
                    p.nom, p.prenom, p.matricule, p.telephone, p.adresse,
                    p.email, p.statut, p.grade, p.situation_familiale, p.id
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

    pub fn importer_base(&self, source: &str) -> Result<(), String> {
        let chemin = self
            .chemin_actif
            .borrow()
            .clone()
            .ok_or_else(|| "Aucune session ouverte".to_string())?;

        let utilisateur = self.utilisateur_courant.borrow().clone();

        let source_path = PathBuf::from(source);
        if !source_path.exists() {
            return Err(format!("Fichier source introuvable : {}", source));
        }

        let source_canon = std::fs::canonicalize(&source_path).map_err(|e| e.to_string())?;
        let actif_canon = if chemin.exists() {
            std::fs::canonicalize(&chemin).map_err(|e| e.to_string())?
        } else {
            chemin.clone()
        };

        if source_canon == actif_canon {
            return Err("Impossible d'importer la base active sur elle-même.".into());
        }

        let verif = Connection::open(&source_canon).map_err(|e| {
            format!(
                "Impossible d'ouvrir le fichier a importer (corrompu ?) : {}",
                e
            )
        })?;

        let tables_ok: i64 = verif
            .query_row(
                "SELECT COUNT(*) FROM sqlite_master
                 WHERE type='table'
                   AND name IN ('Personnel','EnregistrementPointage','Utilisateur','JourFerie')",
                [],
                |r| r.get(0),
            )
            .unwrap_or(0);
        drop(verif);

        if tables_ok < 4 {
            return Err(
                "Le fichier selectionne n'est pas une base Pointage valide (tables manquantes)."
                    .into(),
            );
        }

        let horodatage = chrono::Local::now().format("%Y%m%d_%H%M%S").to_string();
        let nom_backup = chemin
            .file_name()
            .and_then(|s| s.to_str())
            .map(|s| {
                let base = s.trim_end_matches(".db");
                format!("{}.backup_{}.db", base, horodatage)
            })
            .unwrap_or_else(|| format!("pointage.backup_{}.db", horodatage));
        let backup_path = self.dossier.join(nom_backup);

        let actuel_existe = chemin.exists();

        self.fermer_session();

        if actuel_existe {
            let _ = std::fs::copy(&chemin, &backup_path);
        }

        for suffix in ["", "-wal", "-shm"] {
            let p = PathBuf::from(format!("{}{}", chemin.to_string_lossy(), suffix));
            if p.exists() {
                std::fs::remove_file(&p).map_err(|e| {
                    format!(
                        "Impossible de supprimer {} (fichier verrouille ?) : {}",
                        p.display(),
                        e
                    )
                })?;
            }
        }

        std::fs::copy(&source_canon, &chemin).map_err(|e| e.to_string())?;

        if let Some(u) = utilisateur {
            self.ouvrir_session(&u)?;
        }
        Ok(())
    }
}
