# Étape 3.2 — Rendu Dioxus

23 octobre 2026. Base 989428d. Prérequis 3.1 approuvé par « Oui continue ».
Créations : tests/spikes/dioxus-render/Cargo.toml, Cargo.lock, src/main.rs, assets/style.css, check.cjs ; docs/evidence/spike-3.2.md et deux captures ; ce rapport. Le lock isolé et le contrôle reproductible sont des compléments nécessaires aux essais de cette étape.
Modifications : DEPENDENCY_BASELINE.md, PROGRESS.md. Aucun contrat ni fixture existant modifié.
Résultats : cargo check et cargo build Desktop : code 0 ; dx build Web : code 0 après activation explicite web-sys/Location. Tests Chrome et WebView Desktop : code 0, DOM/clic/saisie/focus et parité sémantique vérifiés. Captures inspectées. Voir preuve pour commandes et erreurs intermédiaires.
Critères du prototype satisfaits sur Windows : formulaire réactif, carte, table hôte et graphique SVG rendus par le même code. Limites : seul le sous-ensemble du fixture est interprété ; table/carte ajoutées côté hôte ; pas de dispatch des événements vers guest encore ; accessibilité complète et renderer des 18 nœuds ultérieurs.
Firefox et Linux/macOS omis selon autorisations antérieures. Aucun gel. Statut awaiting_approval. Commit résultat : commit contenant ce rapport.
