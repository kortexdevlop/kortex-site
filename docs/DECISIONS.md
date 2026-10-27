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
