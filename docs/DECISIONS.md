# Décisions

ADR déclarés approuvés dans le plan du 23 octobre 2026 ; signataire et dates individuelles non fournis, non vérifiés indépendamment. Aucun gel expérimental effectué.

| ADR-001 | WebAssembly **Core**, cible `wasm32-unknown-unknown`, même binaire Web/Desktop | Évite l’exécution native non standard du Component Model dans le navigateur ; évite deux pipelines guest | 3.0, 3.1 puis 2.5 |
| ADR-002 | Dioxus exclusivement dans les **hôtes** ; UI déclarative JSON versionnée issue du guest | WASM sandboxé indépendant des DOM/WebView ; une bibliothèque de composants commune | 3.2, 4.4 |
| ADR-003 | API Worker source unique des règles métier catalogue | Astro consulte l’API via Service Binding ; Desktop via HTTPS | 9.4, 11.1 |
| ADR-004 | IA et workflows agissent sur le client actif ; aucune exécution cloud de `.tool` | Contrôle local des fichiers, autorisations, secrets | 15.2, 16.3 |
| ADR-005 | Capacités hôte médiées ; **pas** de GPU guest direct en V1 | Permet audits et portabilité | 4.2, 18.1 |
| ADR-006 | Simple/Medium/Pro = densité/complexité visuelle ; **aucun** plan payant | Fidélité à la capture, cohérence « outils gratuits » | 10.2, 11.2 |
| ADR-007 | ZIP `.tool` signé via descripteur **détaché** Ed25519, hachage SHA-256 | Évite le cycle « signer un ZIP contenant sa signature » | 8.2, 8.3 |
| ADR-008 | Versions immuables, alias `current_version` transactionnel en D1 | Publication/rollback atomiques ; pas de rebuild de clients | 9.5, 19.5 |
| ADR-009 | OIDC Authorization Code + PKCE pour clients ; session Web HttpOnly ; aucune clé longue durée en JS | Identité externe configurable sans fournisseur inventé | 14.0, 14.1 |
| ADR-010 | Un outil = un namespace d’état, des demandes hôte et une liste de capacités déclarées | Isolation entre outils, fichiers hors sandbox via handles et consentement | 4.2, 18.2 |

Mises à jour des cinq registres : citer étape et preuves ; conserver provenance du MASTER_PLAN, responsabilités dans ARCHITECTURE, arbitrages dans DECISIONS, contrats dans CONTRACTS (création 2.0), états et approbations dans PROGRESS. Aucun statut validated sans approbation explicite.

## Dérogation de progression — 23 octobre 2026

Instruction directe du propriétaire : « Ignore les essais Linux/MacOs et continue ». Les essais Linux/macOS de la baseline 1.2 sont omis sur décision du propriétaire ; aucune compatibilité de ces systèmes n'est déclarée. Le passage à 1.3 est autorisé par cette instruction malgré la matrice incomplète. Firefox conserve son échec de lancement documenté ; cette progression n'en vaut pas validation technique. Le plan source reste inchangé.

## Transcription 2.0 — 23 octobre 2026

ADR-001 : adr/0001-wasm-core.md ; ADR-002/006 : adr/0002-host-ui.md ; ADR-003/008/009 : adr/0003-service-catalog.md ; ADR-004/005/010 : adr/0004-local-execution.md ; ADR-007 : adr/0005-distribution-trust.md. Contrats : CONTRACTS.md. Statut candidat au gel, revue humaine des noms/types/limites attendue.

Le propriétaire a autorisé la suite après 1.4 avec activation de la protection GitHub laissée en attente ; le workflow reste local, sans preuve de protection distante.

## Dérogation Firefox — 23 octobre 2026

Instruction explicite du propriétaire : « Oui continue, laisse firefox de coté ». Poursuite après 3.0 autorisée avec Chromium/WebKit. Firefox est omis, non déclaré compatible ; son erreur de lancement reste documentée. Aucun gel de portabilité pour les plateformes non essayées.

## Lot 6.2–7.0 — 27 octobre 2026

Le propriétaire demande « Enchaîne les 3 prochaine étape ». Les étapes sont 6.2, 6.3 et 7.0. Ses exclusions antérieures Linux/macOS et Firefox restent applicables. La vérification native Windows est réalisée ; la matrice complète de 6.3 reste sous la dérogation déjà autorisée, sans statut validated global. La poursuite vers la déclaration 7.0 est autorisée dans ce périmètre. Le gate multi-OS de phase 20 reste non satisfait pour les plateformes exclues. Preuves : evidence/native-platforms.md et reports/STEP-6.3.md.

## Lot 7.1–7.3 — 27 octobre 2026

Le propriétaire demande « Enchaîne les 3 prochaines étape » : calculatrice réelle, parité Web/Windows et référence de compatibilité. Cette instruction autorise l’exécution du lot et son gel local documenté ; elle ne vaut ni revue de sécurité indépendante, ni preuve de compatibilité des plateformes exclues. Le plan source reste inchangé et 8.0 n’est pas démarrée.

Les quatre calculs sont du code guest commun aux clics/actions. Les hôtes ignorent le nom de l’outil. La correction du pont Web objet/Map et la largeur minimale commune des boutons corrigent des défauts génériques mis en évidence par les interactions réelles ; elles n’étendent aucun contrat. Le gate décrit explicitement ces corrections, plutôt que prétendre qu’aucun fichier hôte n’a changé.

Le design 7.0 envisageait Entrée globale et Échap/focus. Le schéma UI figé ne fournit ni keydown ni commande de focus guest ; ces raccourcis ne sont pas ajoutés silencieusement. Les contrôles natifs sont tabulables, Entrée/Espace sur bouton et Tout effacer restent disponibles. Les empreintes sont celles de la référence construite sur la chaîne Windows figée ; la reproductibilité générale et le packager de publication relèvent de la phase 8. Les signatures des fixtures utilisent une clé éphémère de test, sans bypass unsigned.

## Lot 8.0–8.2 — 27 octobre 2026

Le propriétaire demande « Enchaîne les 3 prochaines étape » après 7.3. Périmètre exécuté : bibliothèque ZIP canonique, CLI générique, SHA/signature et trust store versionné. Les exclusions Linux/macOS et Firefox restent applicables. Le plan source n’est pas modifié ; 8.3 n’est pas démarrée et le format n’est pas gelé avant son gate 8.5.

Décisions de canonicalisation : ordre lexical ASCII, DOS epoch 1980-01-01, Unix 0644, stored pour éviter une variation de compression, JSON trié récursivement. Contrôles de publication selon 8.0 : ZIP et cumul extrait 25 Mio, fichier 20 Mio, ratio 20:1. Les outils actuels ne référencent qu’une icône sous assets/ ; aucun fichier source ou asset non référencé n’entre dans le paquet.

Le descripteur reprend strictement B.2 ; schema_version=1 est désormais vérifié explicitement côté Web. L’URL de téléchargement appartient au chargeur et est retirée avant la validation du descripteur signé. Les commandes et bytes d’admission Web sont capturés avant les attentes asynchrones. Un manifeste de clés signé par racine publique épinglée fournit une version minimale à contrôler côté hôte ; le CLI ne prétend pas persister cette version ni déployer une rotation réseau.

Aucun secret de production n’est créé. Les fixtures publiques sont signées par des clés TEST éphémères détruites à la sortie du générateur ; les tests d’injection CI utilisent seulement leurs vecteurs de test en mémoire. La graine n’est acceptée ni dans la configuration applicative, ni dans le navigateur. Aucun envoi distant.

## Lot 8.3–9.0 — 27 octobre 2026

Le propriétaire demande « Enchaîne les 4 prochaines étapes ». Périmètre : corpus hostile/inspection ABI, préparation locale de publication, gate et gel PACKAGE_FORMAT_V1, migration catalogue. Cette autorisation permet de terminer le lot et consigner son gel local ; aucune approbation indépendante n'est inventée. Linux/macOS et Firefox restent exclus. Le plan source est conservé. Prochaine étape : 9.1.

ADR-007 est validé localement : ZIP canonique immuable, preuve B.2 détachée, racine hôte et snapshot de clés signé. Le même artefact du pipeline est admis et exécuté par Web/Windows sans exception par outil. Les admissions ont été alignées sur les contrôles de publication ; les anciens paquets sûrs restent lisibles. Les tests unitaires bas niveau du moteur conservent leur construction Core pour tester les guests invalides ; ce chemin n'est pas utilisé par l'application ni le gate.

La requête de publication est locale, validée par les trois schémas et les identités croisées, réutilise UUID/preuve à la répétition et refuse l'écrasement de version. Les clés TEST éphémères ne sont pas des clés de production. Le gel inclut artifact.schema et les règles de paquet dans CONTRACTS.md ; il n'élargit ni ABI/UI ni les cinq schémas déjà gelés.

Le catalogue D1 sépare métadonnées et artefacts. Le pointeur published_version est FK composite différé et contrôlé par triggers, avec basculement transactionnel vers une version publiée. Les contenus de versions sont immuables ; les actions cessent d'être modifiables après publication et les statuts de versions ne reviennent pas en draft. Les index partiels de pagination ont été vérifiés sur 10 000 outils dans SQLite et D1 local. La CLI sqlite3 absente est remplacée pour la prévalidation par le vrai SQLite intégré à Node ; le test Wrangler local prouve aussi l'application/rejeu de migration. Aucun provisionnement, upload ou déploiement distant. R2 et l'état de publication restent à 9.1.

## 9.1 puis préparation 9.2 — arrêt demandé au contrôle Cloudflare

Le propriétaire autorise 9.1 et la préparation de 9.2, puis demande un arrêt au besoin de connexion Cloudflare avec état et procédure de reprise. 9.1 est satisfaite par la migration additive et les tests SQLite/D1 locaux. Les preuves R2/crypto restent des attestations à produire par le futur service ; aucun test SQL n'est présenté comme une vérification réseau réelle. Les transitions sont auditées sans UUID dérivés susceptibles de collision. Le rollback du pointeur est distinct d'un retour interdit des états de publication.

La préparation API 9.2 fonctionne localement. Le nom DB du plan 9.2 est adopté comme binding effectif ; KORTEX_DB reste celui du banc 9.0. Le plugin Vitest actuel est choisi dans une version compatible avec les moteurs verrouillés et la date de compatibilité supportée. Le nouveau schéma API est candidat, aucune évolution des six schémas gelés.

La vérification en lecture seule contredit l'hypothèse d'absence totale de connexion : un OAuth Cloudflare existe déjà, limité notamment à Pages. Les inventaires D1/R2 sont refusés, et les déploiements ne peuvent être vérifiés. Ne pas inférer une absence de ressources ni lancer une reconnexion, une permission, une activation R2 ou un déploiement sans intervention du propriétaire. Arrêt à ce point ; 9.2 reste en attente, 9.3 non démarrée. La procédure exacte est dans infra/cloudflare-setup.md. Aucun compte/ID ou secret n'est exposé dans les preuves.

## Retrait ancien projet et implémentation 9.3 — 27 octobre 2026

La demande ultérieure du propriétaire autorise explicitement la suppression des ressources de l'ancien projet kortex.site et le démarrage de l'étape suivante. Les ressources ont été rattachées au domaine avant suppression. Le propriétaire précise ensuite « Conserver la messagerie » : cette exception conserve ses DNS et le routage, l'enregistrement du domaine et les protections par défaut. Le nouveau dépôt et les autres domaines ne sont pas concernés. Le rapport CLOUDFLARE-RESET-2026-10-27.md distingue les suppressions confirmées et les éléments conservés ; la rétention historique du fournisseur n'est pas présentée comme purgée.

9.3 utilise les deux ordres et index existants, des valeurs SQL préparées, un curseur HMAC-SHA256 strict lié aux paramètres normalisés et une clé serveur dédiée. Le filtre de tag se nomme tag et sélectionne une valeur exacte ; les paramètres répétés sont refusés. Les DTO candidats de 9.2 et les six schémas gelés sont inchangés. Les actions sont lues dans le contrat immuable de la version publiée courante, leur source canonique, sans charger de version draft. Les validateurs AJV sont générés au build pour éviter eval dans le Worker. Un ETag est calculé sur la représentation après vérification de publication ; un outil retiré n'obtient pas de 304 depuis son ancienne représentation.

Les tests s'exécutent dans le vrai D1/workerd local avec 10 000 outils publiés et des attestations d'état TEST synthétiques. Ils ne prétendent pas vérifier la signature/R2 de ces données artificielles. Aucun seed de démonstration, clé de test, appel distant ou exception par outil dans l'application déployée. Le périmètre reste 9.3 ; 9.4 et le provisionnement 9.6 ne sont pas engagés.

## Lot 9.4–9.6 — 27 octobre 2026

Le propriétaire approuve le résultat 9.3 (« C'est parfait ») et demande les trois étapes suivantes en une fois après présentation. Périmètre présenté : téléchargement privé, publication/révocation, démonstration staging réelle. Cela autorise les créations et le déploiement du staging prévu, sans toucher à la messagerie ni à la production. Les validations techniques intermédiaires permettent de poursuivre ce lot ; aucune revue indépendante n'est revendiquée.

La décision de téléchargement doit consulter D1 à chaque requête : aucun cache public/CDN à long TTL immutable qui rendrait une révocation inopérante. Les octets restent immuables et adressés par SHA. Le détail existant fournit déjà descripteur et digest ; le header Link ajoute le chemin de téléchargement sans modifier le DTO candidat. Le chargeur Web accepte le MIME Kortex prévu par le plan.

L'authentification GitHub OIDC contrôle les claims exacts, dont l'identifiant numérique du dépôt pour éviter une confiance fondée seulement sur son nom. L'émetteur/JWKS GitHub sont fixes. Pour 9.6, aucun remote GitHub n'est attaché au dossier : on ne fabrique pas de jeton GitHub ni d'intégration Cloudflare OIDC supposée. L'alternative admise est une identité d'opérateur staging signée Ed25519, clé publique épinglée par le déploiement effectué avec la session Cloudflare du propriétaire. Le jeton dure au maximum dix minutes, son audience/subject/ref/workflow et ses actions sont épinglés, sa clé privée reste en mémoire du processus opérateur puis est détruite à sa sortie. Cette confiance provient de l'opérateur déjà authentifié auprès de Cloudflare ; elle n'est pas un login GitHub réussi. Le mode est interdit par le serveur en production. L'activation d'un workflow réel GitHub restera à configurer avec ses claims exacts.

La publication inspecte réellement le paquet signé et ne fait pas confiance à des attestations fournies par le demandeur. Les opérations SQL deviennent visibles ensemble ; R2 et D1 ne sont pas une transaction distribuée. Un objet changé après vérification ne peut être servi silencieusement : l'ETag attesté est vérifié, puis le client hôte vérifie SHA/signature. Les mutations portent une empreinte canonique et un audit append-only, avec rollback compare-and-swap et répétition sans effet supplémentaire. Une réponse idempotente relate l'opération initiale, sans réactiver une version depuis retirée.

Le gate 9.6 a ensuite été réellement exécuté : staging distinct créé, migrations appliquées et API déployée sur workers.dev ; les DNS et la messagerie restent conservés. Le refus API/CLI de création du bucket a été résolu par création UI du même bucket privé, sans achat. Publication, téléchargement exact, rollback et révocation 410 réussissent ; les octets cloud démarrent après vérification sur Chromium, WebKit et Windows/Wasmtime. Les hôtes de test épinglent une racine TEST opérateur et non une racine fournie par le paquet. Les fichiers cloud-publish.md, validation-9.4-9.6.json et les captures consignent les preuves et les limites, sans revendiquer un workflow GitHub réel ou un site de production livré. La suite est 10.0, non engagée.

## 2026-10-27 — Lot 12.3 à 12.5 : outil CSV et matrice des cinq outils

Le propriétaire demande « Enchaîne les 3 prochaines étapes » après 12.2. Périmètre : parseur et statistiques bornés, interface et actions de l’explorateur CSV, matrice des cinq outils. Choix techniques : parseur maison en flux plutôt que la crate `csv` (absente du verrou hors ligne ; le flux est imposé par le budget de 100 M d’instructions par appel) ; suppression de la statistique « distincts » (coût de hachage) ; vecteurs partagés Web/Wasmtime. Aucun contrat public gelé n’est modifié. La publication staging de l’outil 5 est bloquée par le quota quotidien de lectures D1 du compte (limite du plan gratuit) : aucune ressource n’a été écrite, la politique temporaire a été restaurée. Un passage à un plan payant n’est pas décidé ici. Linux/macOS et Firefox restent exclus ; WebKit n’a pas pu s’exécuter faute de navigateur Playwright installé.

## Décisions du propriétaire — 29 octobre 2026

Licence propriétaire choisie explicitement ; plafond mensuel 0 €, aucune montée automatique vers une offre payante. Contact sécurité : admin@kortex.site. La messagerie existante reste conservée. Ces décisions ne constituent pas une attestation de certificat Windows, de configuration Google ou de validation de production.

## Google OAuth reporté — décision du propriétaire

Google OAuth est écarté de la version actuelle ; la connexion/inscription email et mot de passe est conservée. Les clients Google restent absents. Le contrôleur accepte ce report explicite avec le mode email/mot de passe ; les autres gates restent applicables.

## Reports explicites du propriétaire — Google et signature Windows

Le propriétaire demande de mettre de côté temporairement Google OAuth et les signatures Windows. La connexion email/mot de passe reste le mode retenu. Le contrôle de préparation accepte ces deux reports explicites sans déclarer Google configuré ni un certificat Authenticode valide. Les signatures cryptographiques des paquets, canaux et mises à jour restent exigées : la décision concerne la signature de code Windows.
