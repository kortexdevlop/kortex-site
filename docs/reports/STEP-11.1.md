# Rapport 11.1 — Catalogue dynamique

27/10/2026, Windows. Base aa59e70. Implémentation locale sur instruction d’enchaîner les trois étapes ; statut formel pending, sans approbation visuelle inventée. Commit résultat dans PROGRESS.md.

`/tools` et `/tools/[id]` sont SSR, sans génération de fichier par outil. Source unique : Service Binding API_WORKER et DTO validés. La liste appelle l’API une fois, limite 24, conserve les filtres et le curseur signé dans le lien suivant. Recherche bornée à 100 caractères, catégorie et tri dans l’URL. Catégories observées dans les données API de la page, sans inventaire global fictif ; le compteur décrit les éléments chargés, avec + si une autre page existe. Les favoris restent locaux et limités aux cartes chargées.

Fiche unique, description/actions/version réelles, lien Ouvrir, title/description Astro échappés, canonical sur l’origine configurée. 404 noindex sans canonical pour absent/non publié ; 503 explicite si le service manque. Réponses publiques réussies : max-age=60 ; erreurs no-store. Aucun compte ou cookie dans le rendu public. Vérification supplémentaire de cohérence ID/version/SHA entre résumé, manifeste, contrat et descripteur. Un descripteur reçu n’est jamais considéré comme vérifié cryptographiquement par le site : le Runtime conserve cette responsabilité.

Validation : `pnpm --filter @kortex/website check` : 42 fichiers, zéro diagnostic, 14 tests Node. `pnpm --filter @kortex/website build` réussi, atelier/fixtures absents. `pnpm exec playwright test --config tests/website.config.ts` : 10 tests Chromium/WebKit pour le lot, dont pagination sans doublons entre pages, filtre, ajout D1 suivi d’une nouvelle URL sans reconstruction, masqué 404, indisponibilité 503 et texte hostile échappé.

Banc isolé : migrations réelles et API réelle dans workerd/D1 local, attestations de publication TEST pour les enregistrements de catalogue. Ce n’est pas un test du pipeline de publication R2 distant. Aucun mock dans le produit. État SQLite généré sous target/website-integration, jamais connecté à staging/production. La suite utilise le Worker SSR compilé et les vrais Service Bindings dans un même processus Wrangler ; les processus séparés avaient exposé une collision de registre local, corrigée dans le banc.

Preuves : docs/evidence/website-11.1-11.3-playwright.json et docs/evidence/website-11.1-11.3-validation.json. Linux/macOS/Firefox exclus. Aucun déploiement, changement DNS ou messagerie.
