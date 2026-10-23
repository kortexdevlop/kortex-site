# Catalogue via API et publication atomique

Statut : décision transcrite du plan, contrat candidat au gel. Date : 23 octobre 2026. Owner : architecture ; sécurité pour identités, capacités et distribution.

## ADR-003

Décision MUST : API Worker source unique des règles métier catalogue.

Motif : Astro consulte l’API via Service Binding ; Desktop via HTTPS.

Preuves attendues : 9.4, 11.1. Règle : KTX-ADR-003 dans ../CONTRACTS.md.

## ADR-008

Décision MUST : Versions immuables, alias `current_version` transactionnel en D1.

Motif : Publication/rollback atomiques ; pas de rebuild de clients.

Preuves attendues : 9.5, 19.5. Règle : KTX-ADR-008 dans ../CONTRACTS.md.

## ADR-009

Décision MUST : OIDC Authorization Code + PKCE pour clients ; session Web HttpOnly ; aucune clé longue durée en JS.

Motif : Identité externe configurable sans fournisseur inventé.

Preuves attendues : 14.0, 14.1. Règle : KTX-ADR-009 dans ../CONTRACTS.md.

Les noms, types et limites détaillés sont définis par les blocs KTX identifiés dans [CONTRACTS](../CONTRACTS.md). Une modification MUST suivre KTX-VERSION-001 et KTX-EXCEPTION-001. Aucun essai expérimental ni approbation humaine ne sont présumés par cette transcription.
