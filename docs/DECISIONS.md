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
