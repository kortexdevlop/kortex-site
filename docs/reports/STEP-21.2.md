# Étape 21.2 — Restauration locale et distante démontrée

État technique : **local_and_cloud_recovery_verified**. Implémentation 040c18c3ca0914e2ab5c9a820641028cc3558341.

Le 30 octobre 2026, drill privé D1 Time Travel/R2 réussi : trois restaurations incluant une annulation vers l’incident puis retour sain ; même hash de paquet, API/site/téléchargement 200, aucune ligne saine connue perdue ; RTO 4 499 ms et âge de sauvegarde 31 347 ms. Toutes les ressources temporaires sont supprimées et leur absence revérifiée.

Trois essais bornés ont mesuré au total 1 915 lectures et 619 écritures SQL administratives, migrations incluses. Ce total exclut les requêtes internes Worker, les requêtes en échec et les autres usages du compte. Aucun test de charge ni import 10k effectué. La fixture expirée et le wrapper Astro erroné ont été corrigés sans relâcher les signatures.

Voir [rapport détaillé](DISASTER_DRILL.md), [preuve distante](../evidence/disaster-drill.json), [relevé et garde-fous](../evidence/d1-guarded-recovery.json). Trois tests locaux des budgets réussis. Trois tests Web staging ciblés réussis après renouvellement signé des révocations et index 0012.

Cette recette minimale ne constitue pas une preuve de cadence permanente des sauvegardes ou une approbation humaine du gel. Périmètre Windows x64 / Chromium-WebView2 ; production, DNS et messagerie conservés.
