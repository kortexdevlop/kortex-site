# Incident quota D1 et prévention

Le 29/10/2026, la préparation de charge 20.1 a épuisé le quota D1 gratuit partagé du compte : environ 457 millions puis 101 millions de lectures historiques. Une base distincte et sa suppression ne remboursent pas le quota du compte. Catalogue 503 et frame 404 intermittents en conséquence. La réussite du SQL d’administration ne prouve pas le bon fonctionnement du binding Worker.

Diagnostic réel read-only dans runtime-catalog-diagnostic.json : D1_ERROR free tier daily row read limit. Worker privé de diagnostic supprimé, nettoyage attesté. Aucun forfait modifié, quota contourné, donnée produit/DNS/mail supprimé. Reset annoncé par D1 : 30/10 à 00:00 UTC, soit 01:00 Paris ; disponibilité à revérifier réellement ensuite. Source : https://developers.cloudflare.com/d1/platform/limits/.

Prévention : migration 0012, index publication_jobs(tool_id,version,status), vérifiée localement et non appliquée à distance. Charge 10k désormais interdite sur le compte staging : KORTEX_QA_ACCOUNT_ID distinct obligatoire et KORTEX_QA_D1_READ_BUDGET / KORTEX_QA_D1_WRITE_BUDGET explicites. Comptage après chaque requête administrative, sans prétendre plafonner le coût de la requête déjà exécutée. Deux tests prouvent le refus avant création réseau. Gates Runtime/interface désormais reproductibles entièrement en local avec vrais API/D1/R2/Runtime, sans secrets Cloudflare hérités.

CI initiale 40885459456 : contrats et dépendances sécurité réussis ; trois tests Web staging échouent pendant le blocage D1. Natifs : un test HTTP local d’installation échoue aussi, sans attribution à D1 ; nouveau passage requis. Les tests distants restent obligatoires et n’ont pas été supprimés pour obtenir une CI verte.

Reprise après reset : vérifier catalogue/fiche/frame réels ; appliquer migration staging 0012, publier Runtime immuable 1.0.2 et site, puis repasser CI et smoke staging. Aucun replay 10k sur le compte produit. PR15 en brouillon ; aucune fusion ni publication tant que gates non passés. Les rapports 20.1 restent des résultats historiques, avec cet effet ultérieur explicitement signalé.
