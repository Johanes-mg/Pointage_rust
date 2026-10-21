# Pointage

Application desktop Windows de gestion de pointage du personnel, écrite en **Rust** avec **egui / eframe**.
Elle permet de gérer le personnel, de saisir les pointages quotidiens, de visualiser un bilan mensuel avec notation sur 20, d'exporter en Excel et de marquer des jours fériés.

---

## Sommaire

- [Aperçu](#aperçu)
- [Fonctionnalités](#fonctionnalités)
- [Stack technique](#stack-technique)
- [Prérequis](#prérequis)
- [Compilation](#compilation)
- [Création de l'installateur](#création-de-linstallateur)
- [Structure du projet](#structure-du-projet)
- [Données utilisateur](#données-utilisateur)
- [Images et ressources](#images-et-ressources)
- [Notes de version](#notes-de-version)
- [Dépannage](#dépannage)
- [Licence](#licence)

---

## Aperçu

| Aspect | Valeur |
|---|---|
| Langage | Rust (edition 2021) |
| Interface | egui / eframe 0.28 |
| Base de données | SQLite (via `rusqlite` bundled) |
| Export | Excel (via `rust_xlsxwriter`) |
| Cible de compilation | `x86_64-pc-windows-gnullvm` |
| Version | 2.2.0 |
| Auteur | Johanes-mg |

---

## Fonctionnalités

### Authentification
- Création de compte au premier lancement.
- Connexion / déconnexion.
- Un fichier de base de données **par utilisateur** (`pointage_<nom>.db`).
- Mot de passe haché (PBKDF2-HMAC-SHA256, 100 000 itérations).

### Personnel
- Ajout, modification, suppression de membres.
- Champs : nom, prénom, matricule, adresse, téléphone, email, statut, grade, situation familiale.
- Tableau récapitulatif avec sélection.

### Pointage
- Calendrier mensuel avec sélection de date.
- Saisie matin / après-midi (heures sur 2 chiffres : `08:00`, `14:00`).
- Observation libre par membre.
- Bilan du jour : présents / absents / total / taux.
- **Marquage de jour férié** : cycle `jour entier → matin → après-midi → aucun` via un bouton dédié.
- Les jours fériés apparaissent en couleur orange dans le calendrier.

### Bilan
- Sélection d'un membre et d'un mois.
- Calendrier coloré selon la **note du jour sur 20** :
  - Vert : 16/20 et plus — Bien
  - Jaune : 10 à 15,9/20 — Moyen
  - Rouge : moins de 10/20 — Mauvais
- Détail du jour : note matin / note après-midi / moyenne du jour.
- Graphique par tranche (barres à 0 masquées).

### Compte
- Modification du nom complet.
- Changement de mot de passe.
- Export / import de la base de données (`.db`).
- Affichage du chemin de la base active.

### Accueil
- Horloge en temps réel.
- Bouton **Exporter en Excel** (fichier `.xlsx` dans `Documents/Pointage Exports/`).
- Export rapide du jour et export par plage de dates.

---

## Stack technique

| Crate | Usage |
|---|---|
| `eframe` / `egui` / `egui_extras` | Fenêtre native + widgets |
| `image` | Décodage PNG / ICO |
| `rusqlite` (bundled) | SQLite statique (aucune DLL externe) |
| `chrono` | Dates et heures |
| `sha2` / `hmac` / `base64` / `rand` | PBKDF2-HMAC-SHA256 |
| `rust_xlsxwriter` | Génération `.xlsx` |
| `rfd` | Dialogues fichier natifs |
| `directories` | Chemins `%APPDATA%\Johanes\Pointage\data\` |
| `winres` (build) | Ressources Windows (icône + version) |

---

## Prérequis

- **Rust** avec la toolchain `gnullvm` :

```powershell
rustup toolchain install stable-x86_64-pc-windows-gnullvm
rustup target add x86_64-pc-windows-gnullvm
