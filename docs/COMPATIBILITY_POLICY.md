# Compatibilité et versions

Le contenu normatif et les gels sont dans [CONTRACTS](CONTRACTS.md). Cette politique s'ajoute à l'historique ; elle ne réécrit ni schémas ni goldens. Plateformes prises en charge dans ce lot : Windows x64, Chromium et WebView2. Linux/macOS/Firefox/WebKit restent hors de la validation demandée.

| Frontière | Contrat courant | Règle |
|---|---|---|
| Guest/hôte | ABI Core WASM 1.0 | Pointeurs, signatures, mémoire et import sont vérifiés avant admission ; pas de WASI implicite |
| Messages | Enveloppe `v:1` | Corrélation obligatoire, tailles bornées, refus des réponses invalides |
| UI déclarative | UI schema 1.0 | Composants/props autorisés, aucun HTML/script guest |
| Outil | Manifest/contract 1.0 | Identités cohérentes, actions/schémas/permissions explicites |
| Archive | PACKAGE_FORMAT_V1 | Octets immuables par version, SHA et signature vérifiés |
| Confiance | Snapshots/canaux/révocations versionnés | Signatures, expiration et compteurs monotones ; les tombstones ne s'effacent pas |

Une rupture d'ABI exige une nouvelle version majeure et une migration ou un adaptateur explicite, accompagné des tests ancien/nouveau hôte et guest. Ne pas attribuer silencieusement un sens nouveau à un champ figé. Une capacité inconnue ne devient pas autorisée par défaut. Un minimum de Runtime non satisfait produit un refus/mise à jour explicite, pas un contournement.

SemVer distingue correctif compatible, ajout compatible et rupture. Chaque composant versionne ses propres artefacts. Changer d'octets impose une nouvelle version d'archive ou de bundle immuable ; un rollback doit rester autorisé par la confiance courante. Une baisse de compteur ou l'activation automatique d'un ancien Desktop n'est pas une stratégie de migration.

Compatibilité SQL : préférer les migrations additives et conserver le schéma compatible avec le Worker précédent pendant la bascule. Le rollback de code ne restaure pas D1. Une migration de données exige sauvegarde, vérification d'intégrité, scénario d'échec et restauration démontrée. Revalider sessions, retraits et suppressions après toute récupération.

Un correctif critique suit [CONTRIBUTING](CONTRIBUTING.md) et la partie G.2 du plan. Il doit documenter l'exception, les autorisations effectives, les tests sélectifs et globaux, la migration, le retour arrière et les revérifications aval. Une réécriture d'historique de gel ne vaut pas amendement.

Support et fin de vie : avant retrait d'une version encore diffusée, publier une décision datée avec versions affectées, migration, délai retenu, responsable et canal de communication. Aucun délai commercial, support 24/7 ou SLA contractuel n'est inventé. Une révocation de sécurité peut imposer un retrait immédiat, documenté et annoncé après revue ; elle ne promet pas de stopper un calcul déjà commencé hors ligne.

La RC et V1 ne sont pas encore gelées. Les objectifs de disponibilité/performance sont des cibles à vérifier et non une garantie de service. Toute annonce de support ou de fin de vie devra refléter les configurations et preuves effectivement livrées.
