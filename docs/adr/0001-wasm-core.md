# Core WASM et deux hotes

Statut : décision transcrite du plan, contrat candidat au gel. Date : 23 octobre 2026. Owner : architecture ; sécurité pour identités, capacités et distribution.

## ADR-001

Décision MUST : WebAssembly **Core**, cible `wasm32-unknown-unknown`, même binaire Web/Desktop.

Motif : Évite l’exécution native non standard du Component Model dans le navigateur ; évite deux pipelines guest.

Preuves attendues : 3.0, 3.1 puis 2.5. Règle : KTX-ADR-001 dans ../CONTRACTS.md.

Les noms, types et limites détaillés sont définis par les blocs KTX identifiés dans [CONTRACTS](../CONTRACTS.md). Une modification MUST suivre KTX-VERSION-001 et KTX-EXCEPTION-001. Aucun essai expérimental ni approbation humaine ne sont présumés par cette transcription.
