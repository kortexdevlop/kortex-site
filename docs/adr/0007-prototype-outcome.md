# ADR-0007 — Résultats des prototypes

Statut : revue 3.4, NO-GO pour gel en attente de correction. Date : 23 octobre 2026. Owners : architecture, sécurité, propriétaire.

Les prototypes établissent une faisabilité locale du guest Core WASM unique sous Chromium/WebKit et Wasmtime Windows, du rendu Dioxus hôte et de l'isolation cross-origin. Les limites sont référencées dans evidence/ARCHITECTURE_GATE.md. Les dérogations Firefox et Linux/macOS ne sont pas des preuves de compatibilité.

L'audit codec/schéma reproduit trois divergences de validité protocolaire. La décision MUST rester NO-GO pour le gel et la phase 4 avant résolution de CHANGE_REQUESTS/CR-2026-001.md. Les exports/imports ne doivent pas être modifiés ; la correction proposée aligne l'implémentation sur les contrats candidats existants. Toute nouvelle empreinte guest sera réidentifiée et retestée dans les deux hôtes.

Les validations historiques restent traçables ; aucune réécriture du plan source ou succès fictif. Après correction et réexécution, une revue humaine propriétaire + technique doit approuver le GO et le périmètre de gel. Aucun FROZEN_V1 dans cet ADR.

Reprise CR-2026-001 du 23 octobre 2026 : défaut corrigé et retesté, recommandation révisée GO proposé sur Windows/Chromium/WebKit, en attente de revue humaine et définition du périmètre gelé. Le NO-GO initial ci-dessus constitue l’historique avant correction ; aucun gel automatique.
