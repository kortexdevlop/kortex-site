# Préparation RC — 20.5

État : **préparée, non gelée et non publiable**. Implémentation a2ec09e8d50def8a4801915c1a78daa63b3753fb, périmètre Windows x64 / Chromium-WebView2. Évaluation réelle par node scripts/release-candidate.mjs docs/evidence/rc-20.5/input.json docs/evidence/v1-signed-matrix.json : refus attendu (code de sortie 2), release_candidate=false et production_ready=false.

L’inventaire contient les cinq archives déjà vérifiées cryptographiquement dans les moteurs, leurs SHA256, le bundle API local dry-run, un manifeste complet des fichiers du site, le Runtime 1.0.2 et les deux SBOM CycloneDX. L’installateur Windows référencé est l’ancienne préversion publiée : il n’a pas été reconstruit pour ce lot et reste non signé. Les révisions de fabrication sont explicitement distinctes ; aucun ancien hash n’est présenté comme un nouveau build de cette RC.

Blocages au gel : CI complète de la révision finale non encore réussie ; retest staging après quota D1 ; reconstruction/attestation de tous les artefacts depuis la révision retenue ; certificat de signature Windows absent ; approbation finale absente. Les réserves préproduction Google OAuth réel, politique de rétention, plafonds financiers et revue humaine sécurité/licences restent visibles. Aucun tag v1.0.0 ou RC, aucune fusion de PR, aucune promotion ou publication en production.

Le script est un évaluateur de cohérence d’un inventaire opérateur, pas une autorité de signatures ni un remplaçant des gates CI/OIDC de publication existants. Deux tests prouvent le refus des jobs failed/skipped, hashes/révisions manquants, cinq outils non distincts, installateur non signé et approbation absente. Les données positives synthétiques restent exclusivement dans ces tests.

Sécurité locale : scanner dépendances/SAST sans finding bloquant ; avertissements Rust et licences à examiner conservés. Preuves docs/evidence/rc-20.5/ et v1-signed-matrix.json. La dernière validation observée 40904514247 concerne c026f2d8a64229a16d68e191aa5e48b4687eeef3 ; elle n’est pas déclarée verte sur a2ec09e8d50def8a4801915c1a78daa63b3753fb. La PR15 affiche les nouveaux runs après chaque push.
