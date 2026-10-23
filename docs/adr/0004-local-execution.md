# Execution locale et mediation des capacites

Statut : décision transcrite du plan, contrat candidat au gel. Date : 23 octobre 2026. Owner : architecture ; sécurité pour identités, capacités et distribution.

## ADR-004

Décision MUST : IA et workflows agissent sur le client actif ; aucune exécution cloud de `.tool`.

Motif : Contrôle local des fichiers, autorisations, secrets.

Preuves attendues : 15.2, 16.3. Règle : KTX-ADR-004 dans ../CONTRACTS.md.

## ADR-005

Décision MUST : Capacités hôte médiées ; **pas** de GPU guest direct en V1.

Motif : Permet audits et portabilité.

Preuves attendues : 4.2, 18.1. Règle : KTX-ADR-005 dans ../CONTRACTS.md.

## ADR-010

Décision MUST : Un outil = un namespace d’état, des demandes hôte et une liste de capacités déclarées.

Motif : Isolation entre outils, fichiers hors sandbox via handles et consentement.

Preuves attendues : 4.2, 18.2. Règle : KTX-ADR-010 dans ../CONTRACTS.md.

Les noms, types et limites détaillés sont définis par les blocs KTX identifiés dans [CONTRACTS](../CONTRACTS.md). Une modification MUST suivre KTX-VERSION-001 et KTX-EXCEPTION-001. Aucun essai expérimental ni approbation humaine ne sont présumés par cette transcription.
