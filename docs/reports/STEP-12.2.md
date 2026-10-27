# Rapport 12.2 — Dates UTC et Unix

27/10/2026, Windows. Base 0547b00. Commit dans PROGRESS.md ; réalisation du lot autorisée, sans approbation humaine formelle fabriquée.

Nouvelle crate tool-timestamp-utc, manifeste/contrat/icône et module SDK WASM. Actions déterministes to_unix/from_unix et interface réactive, utilisant la même conversion. Entrée UTC explicite terminée par Z, unité seconds ou milliseconds obligatoire. Sortie {iso_utc,unix_seconds,unix_millis}. Pas d’horloge système, timezone OS, DST ou permission ; précision sous-milliseconde et secondes intercalaires refusées. Années 0001–9999, arithmétique contrôlée et division euclidienne pour les dates négatives.

Quatre tests Rust : epoch, -1 ms, année bissextile, limites d’années, aller-retour, erreur de date/fuseau/précision, flottants/champs inconnus et overflow. Comparaison UI/action avec schéma UI validé. Les mêmes résultats sont obtenus dans Wasmtime Windows et dans les interfaces Dioxus Web/WebView2. Essais locaux et distants Chromium/WebKit avec le véritable paquet signé, sans runtime spécial pour cet outil.

Pipeline canonique réussi, aucune dépendance ajoutée hors des versions déjà verrouillées (time 0.3.55), aucune nouvelle importation WASI ni accès système admis par la validation du paquet. Publication réelle API/D1/R2 staging et ouverture depuis le site sans rebuild. Registre de confiance du Runtime enrichi de clés publiques de staging, opérateur temporaire restauré ; production/DNS/messagerie intacts. Preuves : docs/evidence/phase12-validation.json et rapports associés. Phase 12.3 non engagée.
