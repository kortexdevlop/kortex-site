# Étape 5.0 — Shell Runtime Web

Base 820a130 ; accord anticipé pour les quatre étapes. Création de l'application Dioxus 0.7.10, cinq états prévus et état indisponible explicite, modes standard/compact/contraste, styles hôtes locaux et frontière UiRenderer. RuntimeWebConfig vérifie les origines fixes publiques et identité/version ; aucun secret client. Le canal vers l'adaptateur est un hook hôte inactif tant que kortexHost n'existe pas. Aucun outil simulé présenté comme réel.

Validation : dx build --platform web --debug-symbols false réussi et dist produit ; cargo check -p kortex-runtime-web --locked réussi ; quatre tests Chromium/WebKit du shell réussis : indisponible, modes, Tab, offline, loading/error sans outil. Tests de transitions utilisent un double de canal uniquement dans Playwright ; pas de double en production.

Ajouts nécessaires : Cargo.lock et membre workspace, tests/web/harness.ts/config/shell.spec.ts et exclusions des sorties Playwright. Preuve et reproduction : docs/evidence/web-5.0.md. Étape validée avec autorisation anticipée ; passage à 5.1.
