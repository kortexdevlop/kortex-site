# Catalogue 10k — campagne réelle du 29 octobre 2026

**Gate réussi**, environnement privé `kortex-qa-e3732687`, source b841c05904f56ba059be8f61a10bcece4d610ae4. Preuve complète : [qa-isolated-e3732687.json](../evidence/qa-isolated-e3732687.json). Aucun outil synthétique publié sur le catalogue réel. Toutes les ressources de la campagne finale ont été supprimées, après vérification de leur propriétaire.

## Résultats

| Mesure HTTPS, corps entièrement lu | Observations | p50 | p95 | Maximum | Objectif |
|---|---:|---:|---:|---:|---:|
| Liste, recherche et tag | 100 à 10 req/s | 70,16 ms | **97,29 ms** | 108,74 ms | ≤500 ms |
| Fiche détaillée | 100 à 10 req/s | 72,35 ms | **91,67 ms** | 279,20 ms | ≤300 ms |

Cinq échauffements exclus par série. Rang le plus proche, aucune observation écartée. Zéro reprise transport pendant la campagne finale ; le client est capable de reprendre un GET interrompu en incluant l’attente dans sa latence. 467 requêtes HTTP via le client catalogue/sitemap, hors contrôle Cloudflare et opérations admin du drill release. Ces résultats mesurent le poste Windows vers Cloudflare, pas une latence SQL seule ni un engagement de production.

- 100 pages de 100 résultats : **10 000 IDs uniques**, tous les IDs attendus retrouvés, sans doublon ni omission dans la liste complète.
- Catégories extrêmes vérifiées intégralement : calculation 9 990, developer 9, data 1. Recherche, tag, tri par nom et refus d’un curseur réutilisé avec d’autres filtres ; limites, paramètres inconnus/dupliqués et curseurs malformés refusés.
- Sitemap réel Astro : **20 segments / 10 000 URLs outils uniques**, puis **21 segments / 10 001 URLs**.
- Fiches SSR réelles contrôlées pour l’outil de référence et le nouvel ID ; les 10 000 pages détaillées ne sont pas toutes téléchargées individuellement. Leur disponibilité universelle n’est pas assimilée à 10 000 tests de rendu distincts.
- Ajout de la 10 001e fiche : API, SSR et sitemap visibles en **4 432 ms**. Identifiant du déploiement Web et hash du bundle compilé inchangés.

Les fixtures sont des métadonnées de test uniquement, avec marqueur `SYNTHETIC_METADATA_NO_R2`, sans binaires R2. Les champs de vérification nécessaires aux contraintes sont synthétiques ; ils ne prouvent aucune signature de ces 10 000 fiches. Le drill release utilise séparément de vrais paquets WASM signés et vérifiés.

## Index et coûts observables

Les index existants suffisent aux objectifs HTTP. `ANALYZE` est exécuté après l’import en masse dans la base temporaire. Le diagnostic de la vraie projection liste montre `idx_tools_recent`, puis une recherche liée sur `idx_tool_versions_status` ; aucune nouvelle migration d’index ni modification de contrat API. Le rapport général inclut aussi un EXPLAIN d’une projection réduite sur l’ID : son plan avec tri temporaire est différent et n’est pas présenté comme celui de la projection complète de production.

[Diagnostic final D1](../evidence/qa-d1-query-cost-final.json), région WEUR/CDG, 10 000 fiches : liste 21 lignes retournées / **62 lignes lues**, SQL 1,1749 ms ; détail 1 ligne / **5 lignes lues**, SQL 0,3192 ms ; aucune écriture pour ces lectures. Stockage après ajout : **65 216 512 octets**. Le diagnostic de calibration précédent, sur environ 7 500 lignes avant statistiques, lisait 22 003 lignes pour la liste ; après import 10k et ANALYZE, 62. Les tailles de jeu différentes restent explicites : [calibration](../evidence/qa-d1-query-cost.json).

Le compteur direct de l’orchestrateur final couvre **236 appels D1**, **100 970 446 lignes lues**, **380 076 lignes écrites**, environ **17 528 ms SQL cumulées**. Il couvre surtout la préparation des fixtures et leurs triggers/audits ; il n’inclut pas toutes les requêtes internes aux Workers ni une facture globale du compte. L’import initial de calibration avait lu 457 722 747 lignes ; l’import par bornes évite les recherches globales inutiles mais les triggers continuent d’amplifier le coût d’une préparation 10k. Cette limite est conservée dans les preuves, sans la confondre avec le coût de 62/5 lignes des lectures produit. Pas de changement de forfait ni de suppression des audits pour accélérer le test. La future politique d’import massif et son budget doivent tenir compte de cette amplification avant usage récurrent.

Budget de campagne : une D1, un bucket et deux Workers privés temporaires, 1 000 requêtes catalogue/sitemap maximum, 10 req/s par série, bail d’accès de deux heures ; aucun service IA, aucun quota IA consommé. Les unités D1 sont des compteurs mesurés, pas une estimation monétaire ni une garantie d’absence de facturation du compte. Les essais de calibration échoués restent audités et ne sont pas comptés comme des succès.

## Reproductibilité et nettoyage

Windows 11 build 22621, Node 24.15.0, Intel i7-2600K 3,40 GHz / 8 processeurs logiques, 25 713 045 504 octets de RAM. Versions Wrangler/Playwright et instructions dans [QA_BASELINE](QA_BASELINE.md) et [QA_PLAN](../QA_PLAN.md).

Le site compilé est déployé une seule fois dans la campagne. L’API et les routes Astro sont les vraies sources de l’application. Le wrapper refuse tout accès HTTP sans secret et après expiration ; la liaison catalogue nommée reste interne. Secrets aléatoires et clés privées de test ne figurent pas dans les rapports.

Campagne finale : `passed:true`, `cleanup:true`, aucune erreur de nettoyage. L’essai b877b858 avait laissé son API liée après suppression du site ; la suppression finale du Worker propriétaire a été effectuée et attestée dans `qa-cleanup-b877b858.json`. Le nettoyage supprime désormais le site avant l’API. Tous les autres essais échoués ont leurs preuves et leur résultat de nettoyage explicites. L’inventaire distant final `qa-cleanup-inventory.json` confirme zéro Worker, D1 ou bucket `kortex-qa-` restant. Catalogue réel de six outils conservé : `release-staging-preserved.json`. Production, DNS et messagerie n’ont pas été modifiés.

Les objectifs liste/détail et visibilité étant atteints, aucun ADR de dérogation aux seuils n’est nécessaire. Les performances Runtime, l’accessibilité et la résilience finale des étapes 20.2–20.5 ne sont pas couvertes par ce résultat.
