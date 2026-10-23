# Étape 3.0 — Prototype ABI navigateur

23 octobre 2026, base d0f5e97. Prérequis 2.5 approuvé par « Oui continue ».
Créations : guest Cargo.toml/Cargo.lock/src/lib.rs, harness index.html/worker.ts/test.spec.ts, docs/evidence/spike-3.0.md, ce rapport. Modification administrative : PROGRESS.md. Baseline inchangée : aucune dépendance ajustée.
Guest Rust réel, ABI v1, Worker JS-compatible TypeScript, arithmétique entière contrôlée. Pas de runtime produit livré. Import probe refusé, sans permissions.
Build release code 0 ; Playwright deux tests réussis, un échoué avant lancement (Firefox), code global 1. Erreurs intermédiaires de compilation Map.pointer et chargement import.meta du test corrigées avant les essais finaux. Détails, versions, hash et mesures dans la preuve.
Critères satisfaits : même binaire et ABI dans Chromium/WebKit, allocations reprises, erreurs et redémarrage vérifiés. Critère non satisfait : essai Firefox explicitement demandé, son exécutable ne démarre pas même après réinstallation. Aucun gel de portabilité, aucun succès Firefox revendiqué.
Statut active : attente de résolution Firefox ou décision explicite sur ce navigateur. L'autorisation précédente d'omettre Linux/macOS ne s'étend pas automatiquement à Firefox. Étape 3.1 non commencée.
Commit résultat : commit contenant ce rapport.

Reprise autorisée CR-2026-001 : nouveau binaire 3385c7f64a333da67e884b44f1260582d7ae5b5dd2921d033af7a1dce7548e65, Chromium/WebKit et Wasmtime retestés avec 33 cas communs. Ancien hash conservé dans la preuve originale ; résultats consolidés dans docs/evidence/cr-2026-001.md.
