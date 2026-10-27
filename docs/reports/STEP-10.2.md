# Rapport 10.2 — navigation et primitives

> Actualisation : une nouvelle référence utilisateur 2706×1764 a été reçue puis reproduite le 27/10/2026. Les mentions de capture manquante et du concept initial ci-dessous sont historiques. État courant : docs/reports/DESIGN-WORKSPACE-2026-10-27.md et docs/evidence/workspace-validation.json. Aucune approbation de fidélité après résultat n’est inventée.

Date : 2026-10-27. Base : 9b0db4c. Résultat : commit contenant ce rapport, identifié dans PROGRESS.md.

Statut : **implémentation préparée ; validation formelle pending derrière 10.0**. Autorisation : lot explicite du propriétaire. Navigation et primitives fonctionnelles ; fidélité au Header d’origine non vérifiable sans l’image.

Livré : Header, Footer, Button/Pill/SearchField, MainLayout/Shell, préférences persistantes et atelier de composants. Navigation explicite Outils/Applications/Télécharger/Compte, saisie globale, densité gratuite, thème, Aa, FR/EN du shell. L’atelier explique ses destinations et la recherche soumet réellement un texte encodé à la route locale, sans présenter de faux catalogue ou compte. Contrôles natifs, lien d’évitement et focus, menu mobile à divulgation et fermeture Échap/retour du focus, stockage indisponible et absence de JavaScript gérés. Le menu reste non modal.

Écart volontaire au manifeste du plan : source de prévisualisation `website/src/previews/design.astro` au lieu de `src/pages/__dev/design.astro`, afin que la route soit injectée uniquement en dev et absente du build. L’injection est dynamique uniquement en dev pour permettre l’essai GET local. Aucun routing produit final ni publication.

| Commande | Date | Plateforme | Code | Résultat |
|---|---|---|---|---|
| pnpm exec playwright test tests/visual/header.spec.ts | 2026-10-27 | Windows Chromium/WebKit | 0 | 8 tests, captures desktop/mobile/sombre, clavier, reflow 320 px avec texte agrandi, persistance, axe clair/sombre, soumission échappée et repli stockage/JS |
| pnpm --filter @kortex/website typecheck | 2026-10-27 | Windows | 0 | aucun diagnostic |
| pnpm --filter @kortex/website build | 2026-10-27 | Windows | 0 | atelier exclu de la sortie |
| pnpm --filter @kortex/api test | 2026-10-27 | Worker local Windows | 0 | 3 DTO + 26 Worker, régression après changement de peers optionnelles Vite |
| node --test tests/config/secrets.test.mjs | 2026-10-27 | Windows | 0 | 6 tests |

Échecs initiaux corrigés : import.meta dans le test CommonJS ; contexte baseURL perdu lors de la réévaluation de configuration par les workers ; label ARIA sur un élément sans rôle ; liens sautés par Tab sur WebKit corrigés par tabindex=0 dans l’ordre natif ; query indisponible sur la route statique de dev corrigée par une injection non pré-rendue. Une première commande de contrôle de secrets visait un chemin incorrect, puis le bon test a réussi. Aucun de ces premiers essais n’est compté comme succès.

Preuves : `docs/evidence/design10-playwright.json`, captures `design10-*`, logs de typecheck/build et manifeste SHA-256 consolidé `validation-10.0-10.2.json`. Les captures sont des rendus de la proposition, sans baseline de fidélité ni nouveau screenshot de référence fabriqué. Revue des captures effectuée pour le débordement et la lisibilité, sans prétendre à une revue humaine de conformité.

Restant : retrouver/annoter l’original, comparer positions/tailles/couleurs du Header et faire revoir le résultat. 10.3 non engagée ; Linux/macOS et Firefox exclus. Domaine et messagerie inchangés. Aucune approbation indépendante ou après résultat inventée.
