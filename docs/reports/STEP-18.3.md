# Étape 18.3 — Révocations signées et cache hors ligne

29/10/2026. État technique : **closed** sur le périmètre Windows/Chromium ; aucune approbation humaine ni production ni approbation humaine inventée.

Signature de racine, TTL 24 h, refus des anciennes listes et conservation des révocations ; gardes Web et Desktop, endpoint API adossé à D1.

Gate restante : 25 scénarios Web, signatures Rust et cache natif validés ; le renouvellement quotidien opérateur reste une obligation documentée.

Voir [le dossier de distribution](../DISTRIBUTION_CI.md), les tests de révocation Rust/Web/API, les tests de publication et les preuves distribution-* dans docs/evidence. Windows/Chromium uniquement ; production, DNS, mail, secrets et quotas conservés.
