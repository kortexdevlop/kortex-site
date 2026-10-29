# Résilience 20.4 — 29/10/2026

Implémentation a2ec09e8d50def8a4801915c1a78daa63b3753fb. Périmètre Windows x64 / Chromium-WebView2. 100 tests API réussis, dont cinq nouveaux scénarios de panne et quatre de télémétrie. Sept scénarios navigateur réels réussis, sans skip. Suite Rust complète et preuves Windows dans resilience-validation.json et tests/resilience/native.md.

| Incident | Injection et vérification |
|---|---|
| D1 indisponible | Binding de test défaillant dans workerd : erreur 503 bornée, aucun détail privé ; health disponible ; même empreinte après reprise |
| R2 indisponible | get de test en erreur : téléchargement jamais 200, catalogue disponible ; octets [1,2,3,4] identiques après reprise |
| Réseau/API catalogue | Interception uniquement dans le test Chromium : message visible, calculatrice chargée intacte, résultat 1200 puis 5, reprise de la recherche |
| Téléchargement | Réponse 503 interceptée : fenêtre en erreur ; fermeture puis vrai téléchargement signé réussi |
| Hors connexion | Cinq vrais outils signés déjà chargés, cinq actions comparées aux vecteurs partagés ; ouverture supplémentaire refusée ; aucun input/résultat dans layout/historique |
| IA/session | Erreur fournisseur et sortie invalide refusées côté API ; réseau d’identité interrompu côté navigateur et proposition expirée : zéro démarrage guest ; accès manuel maintenu |
| Auth/synchronisation | Session expirée 401, compte non modifié ; suite API complète : CAS, tombstones, séparation des comptes, révocation et purge atomique |
| Installateur Windows | Coupures, altération, taille abusive, redirection, annulation, migration ratée et rollback : dix tests dédiés réussis, données utilisateur comparées |
| Parcours | Annulation avant effet et fermeture d’une future instance : aucune étape tardive, résultat de l’étape déjà terminée conservé |

Les fautes sont confinées au harness/local ou aux fixtures temporaires. Aucun défaut injecté en production, aucune consommation IA réelle ni charge D1 du compte partagé. Le catalogue API de tests possède des métadonnées synthétiques ; les cinq exécutions navigateur et Wasmtime utilisent de vraies archives et signatures. Aucun mock permanent ajouté.

Calibration catalogue corrigée : seuil p95 inchangé à 500 ms, désormais 100 observations après cinq échauffements (au lieu de 25 incluant le démarrage), fichiers Vitest sérialisés pour éviter une concurrence non contrôlée pendant la mesure. Tous les autres tests restent exécutés.

Limite : la campagne locale n’est pas une clôture du gate staging. CI complète de la révision finale, disponibilité réelle du catalogue/frame après quota D1 et retest de sécurité distant requis. Voir D1_QUOTA_INCIDENT.md. Aucun passage à 21.1.
