# Rapport 12.5 — Cinq outils sur deux hôtes

27/10/2026, Windows. Base d1450f0. Lot 12.3–12.5 autorisé ; **étape non satisfaite** : l’outil 5 n’est pas publié sur le staging.

Réalisé : `build --all` et `validate --all` canoniques réussis pour les cinq outils ; paquet de l’outil 5 construit, validé, signé et vérifié (clé en mémoire). Matrice 5×2 : docs/evidence/five-tools-matrix.md. Test Web `tests/integration/five-tools.spec.ts` (Chromium via Chrome système) : 5/5 paquets signés démarrent et produisent les 9 vecteurs partagés ; test Wasmtime Windows : mêmes 9 vecteurs, 9/9 ; WebView2 : quatre outils de la phase 12. Les outils sans permission ne déclenchent aucun dialogue. Même archive signée sur les deux hôtes, donc même `module.wasm`. Aucun des cinq identifiants dans l’hôte, le catalogue, l’API, les migrations ou le schéma partagé.

Non réalisé : publication de l’outil 5. `scripts/publish-phase12-staging.mjs` (restreint à cet outil) s’arrête sur `D1_ERROR: exceeded D1's free tier daily row read limit` avant toute écriture ; la politique d’administration temporaire est restaurée et le site n’a pas été redéployé (docs/evidence/phase12-csv-staging.json). Le catalogue staging liste quatre outils. Une relance après réinitialisation du quota (minuit UTC) ou un changement de plan, décision du propriétaire du compte, est nécessaire pour : publier, ouvrir depuis le site, rejouer le test Cloudflare du CSV avec origine d’exécution dédiée.

Autres limites : WebKit non exécuté ; `cargo test --workspace` voir PROGRESS.md ; calculatrice non rejouée dans WebView2 (preuve des phases 7 à 9). Aucune validation humaine formelle inventée.

## Mise à jour 2026-10-28

Après réinitialisation du quota D1, `scripts/publish-phase12-staging.mjs` (restreint à `explorateur-csv`) a publié l’outil 5 sur le staging (sha256 183f838f…, `passed: true`, site non redéployé, politique restaurée ; docs/evidence/phase12-staging.json). Non rejoué : test Cloudflare du CSV avec origine d’exécution dédiée et ouverture depuis le site. Le statut formel reste pending.
