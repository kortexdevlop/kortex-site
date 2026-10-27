# Rapport 12.0 — Convertisseur de casse

27/10/2026, Windows. Base 0547b00. Implémentation du lot 12.0–12.2 expressément autorisée ; résultat Git dans PROGRESS.md. Statut formel pending derrière la revue visuelle existante, sans approbation inventée.

Nouvelle crate tool-convertisseur-casse, manifeste/contrat fermés, icône SVG locale et export SDK Core WASM. Actions upper/lower/title et saisie réactive utilisent une seule fonction. Unicode, chaîne vide, caractères combinés, emoji, expansions et limites d’octets testés. Aucune permission. Normalisation et règle française explicite documentées dans TOOLS-PHASE12.md ; aucune promesse de titlecase universel.

Trois tests Rust, véritable rendu Dioxus dans Chromium/WebKit local et Cloudflare, mêmes actions dans Wasmtime et interface WebView2 Windows. Le pipeline canonique build/validate/package/sign/verify produit un paquet ABI vérifié puis signé Ed25519. Entrée publiée réellement via API admin de staging, avec artefact R2 immuable et vérification dans le Runtime isolé. Aucun rebuild ni redéploiement du site Web.

Le registre Cargo et le lockfile ajoutent uniquement les trois crates du lot, sans changement de version des dépendances existantes. Aucun contrat public gelé, renderer ou capacité spécifique modifié. Le premier test de rendu a détecté une clé UI dupliquée entre titre et bouton title ; corrigée avant signature/publication. Preuves consolidées : docs/evidence/phase12-validation.json, rapports Playwright, publication et Windows. Domaine/production/messagerie inchangés ; Linux/macOS et Firefox exclus.
