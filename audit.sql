.mode column
.headers on
.nullvalue '∅'
.echo off

.print ''
.print '=========================================='
.print ' 1. Liste des tables'
.print '=========================================='
.tables

.print ''
.print '=========================================='
.print ' 2. Schema complet'
.print '=========================================='
.schema

.print ''
.print '=========================================='
.print ' 3. Comptage par table'
.print '=========================================='
SELECT 'Utilisateur' AS Table_, COUNT(*) AS Lignes FROM Utilisateur
UNION ALL SELECT 'Personnel', COUNT(*) FROM Personnel
UNION ALL SELECT 'EnregistrementPointage', COUNT(*) FROM EnregistrementPointage
UNION ALL SELECT 'JourFerie', COUNT(*) FROM JourFerie
UNION ALL SELECT 'Parametre', COUNT(*) FROM Parametre;

.print ''
.print '=========================================='
.print ' 4. Pointages partiels (une seule borne)'
.print '=========================================='
SELECT ep.Id, p.Nom || ' ' || p.Prenom AS Personne, ep.DatePointage,
       ep.MatinArrivee, ep.MatinSortie,
       ep.ApresMidiArrivee, ep.ApresMidiSortie
FROM EnregistrementPointage ep
JOIN Personnel p ON p.Id = ep.PersonnelId
WHERE (ep.MatinArrivee IS NULL) <> (ep.MatinSortie IS NULL)
   OR (ep.ApresMidiArrivee IS NULL) <> (ep.ApresMidiSortie IS NULL);

.print ''
.print '=========================================='
.print ' 5. Pointages avec sortie < arrivee'
.print '=========================================='
SELECT ep.Id, p.Nom || ' ' || p.Prenom AS Personne, ep.DatePointage,
       ep.MatinArrivee, ep.MatinSortie,
       ep.ApresMidiArrivee, ep.ApresMidiSortie
FROM EnregistrementPointage ep
JOIN Personnel p ON p.Id = ep.PersonnelId
WHERE (ep.MatinArrivee IS NOT NULL AND ep.MatinSortie IS NOT NULL
       AND ep.MatinSortie < ep.MatinArrivee)
   OR (ep.ApresMidiArrivee IS NOT NULL AND ep.ApresMidiSortie IS NOT NULL
       AND ep.ApresMidiSortie < ep.ApresMidiArrivee);

.print ''
.print '=========================================='
.print ' 6. Pointages entierement vides'
.print '=========================================='
SELECT ep.Id, p.Nom || ' ' || p.Prenom AS Personne, ep.DatePointage
FROM EnregistrementPointage ep
JOIN Personnel p ON p.Id = ep.PersonnelId
WHERE ep.MatinArrivee IS NULL AND ep.MatinSortie IS NULL
  AND ep.ApresMidiArrivee IS NULL AND ep.ApresMidiSortie IS NULL
  AND (ep.Observation IS NULL OR TRIM(ep.Observation) = '');

.print ''
.print '=========================================='
.print ' 7. Jours feries avec pointages incompatibles'
.print '=========================================='
SELECT jf.DatePointage, jf.Periode,
       (SELECT COUNT(*) FROM EnregistrementPointage ep
        WHERE ep.DatePointage = jf.DatePointage
          AND (ep.MatinArrivee IS NOT NULL OR ep.MatinSortie IS NOT NULL
               OR ep.ApresMidiArrivee IS NOT NULL OR ep.ApresMidiSortie IS NOT NULL)) AS NbPointages
FROM JourFerie jf
ORDER BY jf.DatePointage, jf.Periode;

.print ''
.print '=========================================='
.print ' 8. Doublons pointages (PersonnelId, Date)'
.print '=========================================='
SELECT PersonnelId, DatePointage, COUNT(*) AS Nb
FROM EnregistrementPointage
GROUP BY PersonnelId, DatePointage
HAVING COUNT(*) > 1;

.print ''
.print '=========================================='
.print ' 9. Doublons jours feries'
.print '=========================================='
SELECT DatePointage, Periode, COUNT(*) AS Nb
FROM JourFerie
GROUP BY DatePointage, Periode
HAVING COUNT(*) > 1;

.print ''
.print '=========================================='
.print ' 10. Pointages orphelins'
.print '=========================================='
SELECT ep.Id, ep.PersonnelId, ep.DatePointage
FROM EnregistrementPointage ep
LEFT JOIN Personnel p ON p.Id = ep.PersonnelId
WHERE p.Id IS NULL;

.print ''
.print '=========================================='
.print ' 11. Liste complete des jours feries'
.print '=========================================='
SELECT DatePointage, Periode FROM JourFerie ORDER BY DatePointage;

.print ''
.print '=========================================='
.print ' 12. Dates avec pointages ET jour ferie'
.print '=========================================='
SELECT DISTINCT ep.DatePointage
FROM EnregistrementPointage ep
INNER JOIN JourFerie jf ON jf.DatePointage = ep.DatePointage
WHERE ep.MatinArrivee IS NOT NULL OR ep.MatinSortie IS NOT NULL
   OR ep.ApresMidiArrivee IS NOT NULL OR ep.ApresMidiSortie IS NOT NULL
ORDER BY ep.DatePointage;

.print ''
.print '=========================================='
.print ' 13. Integrite SQLite'
.print '=========================================='
PRAGMA integrity_check;
PRAGMA foreign_key_check;

.print ''
.print '=== FIN AUDIT ==='
