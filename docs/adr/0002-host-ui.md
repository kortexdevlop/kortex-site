# UI declarative et rendu hote

Statut : décision transcrite du plan, contrat candidat au gel. Date : 23 octobre 2026. Owner : architecture ; sécurité pour identités, capacités et distribution.

## ADR-002

Décision MUST : Dioxus exclusivement dans les **hôtes** ; UI déclarative JSON versionnée issue du guest.

Motif : WASM sandboxé indépendant des DOM/WebView ; une bibliothèque de composants commune.

Preuves attendues : 3.2, 4.4. Règle : KTX-ADR-002 dans ../CONTRACTS.md.

## ADR-006

Décision MUST : Simple/Medium/Pro = densité/complexité visuelle ; **aucun** plan payant.

Motif : Fidélité à la capture, cohérence « outils gratuits ».

Preuves attendues : 10.2, 11.2. Règle : KTX-ADR-006 dans ../CONTRACTS.md.

Les noms, types et limites détaillés sont définis par les blocs KTX identifiés dans [CONTRACTS](../CONTRACTS.md). Une modification MUST suivre KTX-VERSION-001 et KTX-EXCEPTION-001. Aucun essai expérimental ni approbation humaine ne sont présumés par cette transcription.
