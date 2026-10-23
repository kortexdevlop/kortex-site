# Distribution et confiance

Statut : décision transcrite du plan, contrat candidat au gel. Date : 23 octobre 2026. Owner : architecture ; sécurité pour identités, capacités et distribution.

## ADR-007

Décision MUST : ZIP `.tool` signé via descripteur **détaché** Ed25519, hachage SHA-256.

Motif : Évite le cycle « signer un ZIP contenant sa signature ».

Preuves attendues : 8.2, 8.3. Règle : KTX-ADR-007 dans ../CONTRACTS.md.

Les noms, types et limites détaillés sont définis par les blocs KTX identifiés dans [CONTRACTS](../CONTRACTS.md). Une modification MUST suivre KTX-VERSION-001 et KTX-EXCEPTION-001. Aucun essai expérimental ni approbation humaine ne sont présumés par cette transcription.
