use crate::database::Database;
use crate::models::EnregistrementPointage;
use chrono::NaiveDate;
use rust_xlsxwriter::{Color, Format, FormatAlign, FormatBorder, Workbook};
use std::collections::HashMap;

pub fn exporter_tout(
    db: &Database,
    debut: NaiveDate,
    fin: NaiveDate,
) -> Result<String, String> {
    let dossier = directories::UserDirs::new()
        .and_then(|d| Some(d.document_dir()?.join("Pointage Exports")))
        .ok_or("Impossible de trouver Mes Documents")?;
    std::fs::create_dir_all(&dossier).map_err(|e| e.to_string())?;

    let periode = if debut == fin {
        debut.format("%Y-%m-%d").to_string()
    } else {
        format!("{}_{}", debut.format("%Y-%m-%d"), fin.format("%Y-%m-%d"))
    };
    let horodatage = chrono::Local::now().format("%H%M%S").to_string();
    let chemin = dossier.join(format!("pointage_{}_{}.xlsx", periode, horodatage));

    let snapshot = db.exporter_snapshot(debut, fin)?;

    let mut matricules: HashMap<String, String> = HashMap::new();
    for p in &snapshot.personnel {
        let cle = format!("{} {}", p.nom, p.prenom).trim().to_string();
        if let Some(m) = &p.matricule {
            if !m.trim().is_empty() {
                matricules.insert(cle, m.trim().to_string());
            }
        }
    }

    let mut wb = Workbook::new();

    let entete = Format::new()
        .set_bold()
        .set_font_color(Color::White)
        .set_background_color(Color::RGB(0x152B54))
        .set_align(FormatAlign::Center)
        .set_align(FormatAlign::VerticalCenter)
        .set_border(FormatBorder::Thin);

    let ws = wb.add_worksheet();
    ws.set_name("Personnel").map_err(|e| e.to_string())?;
    let headers = [
        "MATRICULE", "NOM", "PRENOM", "TELEPHONE", "ADRESSE", "EMAIL", "STATUT", "GRADE",
        "SITUATION FAMILIALE",
    ];
    for (i, h) in headers.iter().enumerate() {
        ws.write_with_format(0, i as u16, *h, &entete)
            .map_err(|e| e.to_string())?;
    }
    for (i, p) in snapshot.personnel.iter().enumerate() {
        let r = (i + 1) as u32;
        ws.write(r, 0, p.matricule.as_deref().unwrap_or("")).ok();
        ws.write(r, 1, &p.nom).ok();
        ws.write(r, 2, &p.prenom).ok();
        ws.write(r, 3, p.telephone.as_deref().unwrap_or("")).ok();
        ws.write(r, 4, p.adresse.as_deref().unwrap_or("")).ok();
        ws.write(r, 5, p.email.as_deref().unwrap_or("")).ok();
        ws.write(r, 6, p.statut.as_deref().unwrap_or("")).ok();
        ws.write(r, 7, p.grade.as_deref().unwrap_or("")).ok();
        ws.write(r, 8, p.situation_familiale.as_deref().unwrap_or("")).ok();
    }
    ws.autofit();

    let ws = wb.add_worksheet();
    ws.set_name("Pointages").map_err(|e| e.to_string())?;
    let headers = [
        "MATRICULE", "NOM", "PRENOM", "DATE", "STATUT", "MATIN ARRIVEE", "MATIN SORTIE",
        "APRES-MIDI ARRIVEE", "APRES-MIDI SORTIE", "TOTAL JOUR", "OBSERVATION",
    ];
    for (i, h) in headers.iter().enumerate() {
        ws.write_with_format(0, i as u16, *h, &entete)
            .map_err(|e| e.to_string())?;
    }
    for (i, pt) in snapshot.pointages.iter().enumerate() {
        let r = (i + 1) as u32;
        let (n, p) = split_nom(pt);
        let cle = format!("{} {}", n, p).trim().to_string();
        let matricule = matricules.get(&cle).cloned().unwrap_or_default();
        let date_key = pt.date_pointage.format("%Y-%m-%d").to_string();
        let statut = match snapshot.feries.get(&date_key) {
            Some(periodes) => {
                let jour_entier = periodes.iter().any(|x| x == "jour");
                let matin = periodes.iter().any(|x| x == "matin");
                let aprem = periodes.iter().any(|x| x == "aprem");
                if jour_entier {
                    "ferie".to_string()
                } else {
                    let mut parts: Vec<&str> = Vec::new();
                    if matin {
                        parts.push("matin");
                    }
                    if aprem {
                        parts.push("aprem");
                    }
                    format!("ferie {}", parts.join(" + "))
                }
            }
            None => String::new(),
        };

        ws.write(r, 0, matricule).ok();
        ws.write(r, 1, n).ok();
        ws.write(r, 2, p).ok();
        ws.write(r, 3, pt.date_pointage.format("%d/%m/%Y").to_string()).ok();
        ws.write(r, 4, statut).ok();
        ws.write(r, 5, fmt_h(pt.matin_arrivee)).ok();
        ws.write(r, 6, fmt_h(pt.matin_sortie)).ok();
        ws.write(r, 7, fmt_h(pt.apres_midi_arrivee)).ok();
        ws.write(r, 8, fmt_h(pt.apres_midi_sortie)).ok();
        ws.write(r, 9, total_jour(pt)).ok();
        ws.write(r, 10, pt.observation.as_deref().unwrap_or("")).ok();
    }
    ws.autofit();

    let ws = wb.add_worksheet();
    ws.set_name("Bilan").map_err(|e| e.to_string())?;
    let headers = [
        "PERSONNE", "JOURS PRESENT", "MATINS", "APRES-MIDIS", "TOTAL HEURES",
        "ARRIVEE MOYENNE",
    ];
    for (i, h) in headers.iter().enumerate() {
        ws.write_with_format(0, i as u16, *h, &entete)
            .map_err(|e| e.to_string())?;
    }
    for (i, b) in snapshot.bilan.iter().enumerate() {
        let r = (i + 1) as u32;
        ws.write(r, 0, &b.nom).ok();
        ws.write(r, 1, b.jours_presents).ok();
        ws.write(r, 2, b.matins_presents).ok();
        ws.write(r, 3, b.apres_midis_presents).ok();
        ws.write(r, 4, &b.total_heures).ok();
        ws.write(r, 5, &b.moyenne_arrivee).ok();
    }
    ws.autofit();

    let ws = wb.add_worksheet();
    ws.set_name("Notation").map_err(|e| e.to_string())?;
    let headers = ["PERSONNE", "MOYENNE /20", "JOURS EVALUES"];
    for (i, h) in headers.iter().enumerate() {
        ws.write_with_format(0, i as u16, *h, &entete)
            .map_err(|e| e.to_string())?;
    }
    let classement = db.get_classement_periode(debut, fin).unwrap_or_default();
    for (i, c) in classement.iter().enumerate() {
        let r = (i + 1) as u32;
        ws.write(r, 0, &c.0).ok();
        ws.write(r, 1, (c.1 * 10.0).round() / 10.0).ok();
        ws.write(r, 2, c.2 as u32).ok();
    }
    ws.autofit();

    let ws = wb.add_worksheet();
    ws.set_name("Periode").map_err(|e| e.to_string())?;
    ws.write(0, 0, "Service").ok();
    ws.write(0, 1, &snapshot.nom_entreprise).ok();
    ws.write(1, 0, "Du").ok();
    ws.write(1, 1, debut.format("%d/%m/%Y").to_string()).ok();
    ws.write(2, 0, "Au").ok();
    ws.write(2, 1, fin.format("%d/%m/%Y").to_string()).ok();
    ws.write(3, 0, "Exporte le").ok();
    ws.write(3, 1, chrono::Local::now().format("%d/%m/%Y %H:%M").to_string()).ok();
    ws.autofit();

    wb.save(&chemin).map_err(|e| e.to_string())?;
    Ok(chemin.to_string_lossy().to_string())
}

fn split_nom(pt: &EnregistrementPointage) -> (String, String) {
    let nom = pt.nom_complet.clone().unwrap_or_default();
    let mut it = nom.splitn(2, ' ');
    let n = it.next().unwrap_or("").to_string();
    let p = it.next().unwrap_or("").to_string();
    (n, p)
}

fn fmt_h(t: Option<chrono::NaiveTime>) -> String {
    t.map(|x| x.format("%H:%M").to_string()).unwrap_or_default()
}

fn total_jour(pt: &EnregistrementPointage) -> String {
    let mut secs: i64 = 0;
    if let (Some(a), Some(s)) = (pt.matin_arrivee, pt.matin_sortie) {
        secs += (s - a).num_seconds();
    }
    if let (Some(a), Some(s)) = (pt.apres_midi_arrivee, pt.apres_midi_sortie) {
        secs += (s - a).num_seconds();
    }
    if secs <= 0 {
        "".into()
    } else {
        format!("{}h{:02}", secs / 3600, (secs % 3600) / 60)
    }
}
