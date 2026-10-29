# Plan QA V1 — périmètre autorisé

Référence : plan directeur, étapes 20.0–20.5. Ce plan organise les preuves ; il ne déclare pas la V1 livrée. Windows x64 et Chromium/WebView2 sont les plateformes retenues par le propriétaire. Linux, macOS, Firefox et WebKit sont exclus des nouveaux gates. Les résultats historiques sur d’autres plateformes ne constituent pas un engagement de support.

## Matrice d’exécution

| Axe | Cas obligatoires | Vérification et preuve |
|---|---|---|
| Composants | API D1/R2, Astro SSR, Runtime iframe, Wasmtime, installateur | CI `contracts/native/web/security`, rapports 19.0–19.4 et charge isolée |
| Environnement | Windows x64 local ; Windows Server 2022/runner Windows ; Chromium épinglé et WebView2 Microsoft | Versions dans QA_BASELINE et preuves CI, aucune émulation Linux |
| Outils | Cinq outils réels, sixième Base64 ; archives valides, altérées et révoquées | `tests/e2e`, `tests/security`, `api/tests/publication.test.ts` ; fixtures 10k uniquement metadata |
| Permissions | Aucune, refus, consentement explicite, expiration/retrait, scope fichier | `tests/web/workflow-permissions.spec.ts`, `tests/security/network-files.spec.ts`, tests Rust policy |
| Réseau | Normal, hors ligne, timeout, interruption avant/après commit | `tests/release/failure-injection.test.mjs`, `scripts/qa-isolated.mjs`, tests sécurité |
| Identité | Anonyme, email/mot de passe, session expirée/révoquée, Google configuré/non configuré | `tests/e2e/account.spec.ts`, API auth ; Google réel demeure bloqué sans clients OAuth |
| Interface | 390px et desktop, clavier, focus, contraste, persistance sans fichiers | `tests/visual`, `tests/security`, phase 20.3 encore à exécuter ; lecteur d’écran manuel non attesté |

## Mesures et seuils

La charge catalogue utilise une D1, un bucket et deux Workers temporaires privés. Aucun changement de forfait ; maximum 1 000 lectures HTTP pour une exécution normale, 10 req/s pour les deux séries chronométrées. Le nombre de requêtes, la taille et les métriques D1 disponibles sont consignés ; ils ne valent pas une facture mensuelle extrapolée.

Chaque p95 utilise **100 observations minimum**, hors cinq échauffements. Percentile par rang le plus proche `ceil(n×0,95)`. Latence = aller-retour HTTPS client, corps entièrement lu ; inclut réseau et serveur, ne prétend pas mesurer le seul SQL. Deux séries distinctes de 100 requêtes à 10 req/s : liste p95 ≤500 ms, détail p95 ≤300 ms. Publier p50, p95, maximum et observations brutes. Un dépassement exige un ADR performance, jamais une réduction silencieuse de l’échantillon.

10 000 IDs uniques, pagination complète, catégories 9 990/9/1, recherche, tag, curseurs liés aux filtres, refus de requêtes invalides ; sitemap sans doublon et ajout 10 001 visible en moins de 15 minutes sans nouveau build ni déploiement Web. Les 10 000 fiches ne constituent pas 10 000 binaires signés : aucun fichier R2 synthétique n’est créé.

Les autres seuils Runtime/ressources, accessibilité, résilience et sécurité finale seront mesurés aux étapes 20.2–20.5. Ils ne sont pas couverts par le seul benchmark catalogue.

## Traçabilité des exigences finales

« Planifié » signifie qu’une preuve finale reste requise, même si des tests intermédiaires existent. Les recettes manuelles ne sont jamais signées au nom d’un humain.

| ID | Test automatisé ou recette identifiée | État de la preuve finale |
|---|---|---|
| I01 | CI Validate, `scripts/check-progress.mjs`, recette A22-3 attestation propriétaire | CI intermédiaire ; A22-3 planifiée |
| I02 | `tests/contracts/cross-language.test.mjs`, `tests/e2e/calculatrice-web.spec.ts`, tests Rust ABI | Preuves intermédiaires 12.5/19.4 |
| I03 | Contrats croisés et validateurs générés ; gate 20.5 | Gate final planifié |
| I04 | `api/tests/publication.test.ts`, `tests/security/revocation.spec.ts`, tests Rust verify | Exécutés en CI ; smoke final 22.2 planifié |
| I05 | `tests/e2e/package-web.spec.ts`, `tests/e2e/package-desktop.ps1` | Preuves outils réels antérieures ; Windows seulement |
| I06 | Preuves 12.6/19.1 Base64 et hot publication ; `scripts/qa-isolated.mjs` | Ajout catalogue sans rebuild mesuré séparément |
| I07 | `tests/perf/catalog-load.mjs`, `api/tests/publication.test.ts`, `scripts/qa-isolated.mjs` | Rapport 20.1 et reprise 19.5 |
| I08 | `tests/perf/sitemap-load.mjs`, `tests/visual`, recette SEO A20-3 | Charge SSR ; revue finale 20.3 planifiée |
| I09 | `tests/security/headers.spec.ts`, `workspace-isolation.spec.ts`, `network-files.spec.ts` | Tests CI intermédiaires |
| I10 | Smoke installateur Windows 19.3, recette A22-2 Windows | Signature production absente ; autres OS exclus |
| I11 | `tests/e2e/account.spec.ts`, API auth/sync ; recette Google A14-5 | Email exécuté ; Google réel bloqué |
| I12 | `tests/security/ai-privacy.spec.ts`, `tests/e2e/ai.spec.ts` | Preuves 15.x ; revue humaine restante |
| I13 | `tests/e2e/workflow-run.spec.ts`, Rust workflows, tests permissions | Preuves intermédiaires ; résilience 20.4 planifiée |
| I14 | `tests/e2e/workspace.spec.ts`, `workspace-actions.spec.ts` | Preuves 17.x ; charge UI 20.3 planifiée |
| I15 | `tests/security/abi-limits.spec.ts`, `network-files.spec.ts`, Rust policy | CI sécurité ; revue finale restante |
| I16 | `tests/visual`, `tests/web/ui-focus.spec.ts`, recette lecteur écran A20-3 | Revue finale et recette manuelle planifiées |
| I17 | `scripts/qa-isolated.mjs`, futurs gates Runtime/résilience 20.2–20.5 | Catalogue mesuré ; autres gates planifiés |
| I18 | `tests/publication`, `tests/release/failure-injection.test.mjs`, drill distant isolé | Staging ; production refusée |
| I19 | Recettes A21-0 supervision, A21-1 backup, A21-3 incident | Planifiées, aucune signature inventée |
| I20 | Recette A22-0 provisionnement et A22-2 smoke production | Bloquée, production non configurée |
| I21 | `scripts/security-gate.mjs`, gates 20.5/22.2 RELEASE_CANDIDATE | Résultats intermédiaires seulement |
| I22 | Recette A22-3 approbations nominatives + attestation/tag | Planifiée, humaine obligatoire |

## Règles de clôture

Un échec conserve sa preuve datée. Un test ignoré doit avoir une raison et ne peut satisfaire un gate critique. Le smoke HTTPS externe facultatif des tests Rust reste explicitement exclu de la validation locale ; les essais Cloudflare réels apportent une preuve distincte. Une exécution sur fixtures ou un double de test n’est jamais présentée comme une exécution distante. Les identifiants de ressources temporaires sont audités, les secrets exclus des rapports, et le nettoyage fait partie du résultat.

Exécution reproductible et versions : [QA_BASELINE](reports/QA_BASELINE.md), [recettes E2E](../tests/e2e/README.md). Les états formels du registre restent soumis aux approbations existantes ; les statuts techniques sont séparés.
