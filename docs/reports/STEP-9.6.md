# Étape 9.6 — Gate staging réel

Statut : réalisée et testée, résultat en attente de revue après exécution du lot autorisé. Date : 2026-10-27. Base API : 56947c7. Résultat : commit contenant ce rapport, à identifier dans PROGRESS.md.

La création, le déploiement et la simulation CI opérateur utilisent des ressources de staging distinctes et des IDs réels, sans supposer une intégration OIDC Cloudflare/GitHub inexistante. Les jetons éphémères ne sont pas persistés. Le bucket est privé, les écritures conditionnelles testées réellement, les trois migrations appliquées à la base dédiée. Aucun DNS/messagerie ou élément de production touché. Le site web n'est pas déployé par anticipation.

Le pipeline construit et signe une calculatrice canonique, le Worker vérifie et publie réellement son paquet depuis R2. Une seconde version de contrôle est publiée, le pointeur revient à 1.0.0, puis 1.1.0 est révoquée et son téléchargement renvoie 410. Les octets téléchargés de la version saine correspondent au SHA et au ZIP figé. Le même fichier démarre après admission habituelle sur Chromium, WebKit et Windows/Wasmtime, avec multiply=1200 et les contrôles natifs de fermeture/erreur. Aucune réponse simulée à l'API de staging ; le relais local des octets vers le shell Web est explicitement décrit dans la preuve.

| Commande | Environnement | Sortie | Observation |
|---|---|---:|---|
| node scripts/staging-publication.mjs | Windows, cache Cargo D:, Cloudflare réel | 0 | Création/reprise, migrations, déploiement, PUT conditionnel, publication, rollback, révocation, download identique |
| pnpm --filter @kortex/api test | vrai workerd/D1/R2 local | 0 | 3 tests DTO + 26 tests Worker, dont les 10 000 entrées dans les deux tris |
| pnpm --filter @kortex/api typecheck | TypeScript 7.0.2 | 0 | Contrôle strict final |
| pnpm --filter @kortex/api build | Wrangler 4.152.0 | 0 | Build dry-run local, distinct du vrai déploiement staging |
| pnpm --filter @kortex/web-adapter typecheck / build | Windows | 0 | Adaptateur partagé avec admission/MIME courants |
| pnpm exec playwright test -c tests/integration/playwright.staging.config.ts | Chromium et WebKit | 0 | 2 tests de roundtrip HTTPS et démarrage Web |
| tests/integration/staging-desktop.ps1 | Windows réel, Wasmtime, WebView2 | 0 | Paquet téléchargé, calcul/UI/actions=1200, DIVIDE_BY_ZERO, fermeture/réouverture |
| node --test tests/publication/golden.test.mjs | Node 24.15.0 | 0 | Six schémas gelés et référence PACKAGE_FORMAT_V1 inchangés |

Logs et empreintes : docs/evidence/validation-9.4-9.6.json. Détails opératoires et incidents intermédiaires corrigés : docs/evidence/cloud-publish.md et infra/staging-checklist.md. Aucun échec n'est présenté comme réussi avant correction. Les captures des trois hôtes ont été examinées et sont lisibles. Le compte était déjà R2 actif ; aucun plan n'a été acheté lors du contournement UI du refus API.

Limites : confiance de test à durée limitée, identité temporaire d'opérateur plutôt qu'un vrai workflow GitHub, UI Web servie par le banc local, pas de production, pas d'essais Linux/macOS/Firefox. L'approbation du résultat après revue n'est pas inventée. Prochaine étape prévue : 10.0 (audit de la référence visuelle), non engagée.
