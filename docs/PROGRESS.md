# Progression

Étape courante : 4.2. Gel 3.4 approuvé le 23 octobre 2026. Les étapes absentes sont pending.
La dérogation 1.2 autorise la progression sans déclarer sa matrice techniquement validée.
Schéma de rapport : docs/reports/TEMPLATE.md.

```json
[
  {
    "id": "1.0",
    "status": "validated",
    "source_commit": null,
    "result_commit": "9c0d95f",
    "approved_by": "propriétaire : maintenant continue",
    "approved_at": "2026-10-23",
    "report_path": "docs/reports/STEP-1.0.md",
    "tests_executed": "voir rapport",
    "contract_changes": [],
    "evidence": [
      "docs/reports/STEP-1.0.md"
    ],
    "frozen": false
  },
  {
    "id": "1.1",
    "status": "validated",
    "source_commit": "9c0d95f",
    "result_commit": "fe3d120",
    "approved_by": "propriétaire : Continue",
    "approved_at": "2026-10-23",
    "report_path": "docs/reports/STEP-1.1.md",
    "tests_executed": "voir rapport",
    "contract_changes": [],
    "evidence": [
      "docs/reports/STEP-1.1.md"
    ],
    "frozen": false
  },
  {
    "id": "1.2",
    "status": "awaiting_approval",
    "source_commit": "fe3d120",
    "result_commit": "bdf2e3a",
    "approved_by": "propriétaire : Ignore les essais Linux/MacOs et continue",
    "approved_at": "2026-10-23",
    "waiver": {
      "reason": "Poursuite autorisée sans essais Linux/macOS ; Firefox reste non vérifié, pas de validation technique globale.",
      "evidence": "docs/DECISIONS.md"
    },
    "report_path": "docs/reports/STEP-1.2.md",
    "tests_executed": "voir rapport",
    "contract_changes": [],
    "evidence": [
      "docs/reports/STEP-1.2.md"
    ],
    "frozen": false
  },
  {
    "id": "1.3",
    "status": "validated",
    "source_commit": "bdf2e3a",
    "result_commit": "f9f8dce",
    "approved_by": "propriétaire : Oui continue",
    "approved_at": "2026-10-23",
    "report_path": "docs/reports/STEP-1.3.md",
    "tests_executed": "voir rapport",
    "contract_changes": [],
    "evidence": [
      "docs/reports/STEP-1.3.md"
    ],
    "frozen": false
  },
  {
    "id": "1.4",
    "status": "validated",
    "source_commit": "f9f8dce",
    "result_commit": "33dc4c5",
    "approved_by": "propriétaire : Oui, continue ; protection GitHub en attente explicitement acceptée",
    "approved_at": "2026-10-23",
    "report_path": "docs/reports/STEP-1.4.md",
    "tests_executed": "voir rapport",
    "contract_changes": [],
    "evidence": [
      "docs/reports/STEP-1.4.md"
    ],
    "frozen": false
  },
  {
    "id": "2.0",
    "status": "validated",
    "source_commit": "33dc4c5",
    "result_commit": "0624d4f",
    "report_path": "docs/reports/STEP-2.0.md",
    "tests_executed": "voir rapport",
    "approved_by": "propriétaire : Oui,continue",
    "approved_at": "2026-10-23",
    "contract_changes": [
      "Transcription candidate des contrats B.2, aucun gel"
    ],
    "evidence": [
      "docs/reports/STEP-2.0.md"
    ],
    "frozen": false
  },
  {
    "id": "2.1",
    "status": "validated",
    "source_commit": "0624d4f",
    "result_commit": "44b0780",
    "report_path": "docs/reports/STEP-2.1.md",
    "tests_executed": "voir rapport",
    "approved_by": "propriétaire : Oui continue",
    "approved_at": "2026-10-23",
    "contract_changes": [
      "Précisions candidates des bornes et du sous-ensemble de schémas ; aucun gel"
    ],
    "evidence": [
      "docs/reports/STEP-2.1.md"
    ],
    "frozen": false
  },
  {
    "id": "2.2",
    "status": "validated",
    "source_commit": "44b0780",
    "result_commit": "29f5544",
    "report_path": "docs/reports/STEP-2.2.md",
    "tests_executed": "voir rapport",
    "approved_by": "propriétaire : Oui continue",
    "approved_at": "2026-10-23",
    "contract_changes": [
      "Codec et constantes ABI candidats ; aucun gel"
    ],
    "evidence": [
      "docs/reports/STEP-2.2.md"
    ],
    "frozen": false
  },
  {
    "id": "2.3",
    "status": "validated",
    "source_commit": "29f5544",
    "result_commit": "728f57a",
    "report_path": "docs/reports/STEP-2.3.md",
    "tests_executed": "voir rapport",
    "approved_by": "propriétaire : Oui continue",
    "approved_at": "2026-10-23",
    "contract_changes": [
      "Props UI, workflow et capability.result candidats"
    ],
    "evidence": [
      "docs/reports/STEP-2.3.md"
    ],
    "frozen": false
  },
  {
    "id": "2.4",
    "status": "validated",
    "source_commit": "728f57a",
    "result_commit": "b046acc",
    "report_path": "docs/reports/STEP-2.4.md",
    "tests_executed": "voir rapport",
    "approved_by": "propriétaire : Oui continue",
    "approved_at": "2026-10-23",
    "contract_changes": [
      "API de validation partagée ; pas de changement des schémas"
    ],
    "evidence": [
      "docs/reports/STEP-2.4.md"
    ],
    "frozen": false
  },
  {
    "id": "2.5",
    "status": "validated",
    "source_commit": "b046acc",
    "result_commit": "d0f5e97",
    "report_path": "docs/reports/STEP-2.5.md",
    "tests_executed": "voir rapport",
    "approved_by": "propriétaire : Oui continue",
    "approved_at": "2026-10-23",
    "contract_changes": [
      "Corpus candidat de conformité, aucun gel de portabilité"
    ],
    "evidence": [
      "docs/reports/STEP-2.5.md"
    ],
    "frozen": false
  },
  {
    "id": "3.0",
    "status": "awaiting_approval",
    "source_commit": "d0f5e97",
    "result_commit": "da4dac8",
    "report_path": "docs/reports/STEP-3.0.md",
    "tests_executed": "voir rapport",
    "approved_by": "propriétaire : Oui continue, laisse firefox de coté",
    "approved_at": "2026-10-23",
    "contract_changes": [],
    "evidence": [
      "docs/evidence/spike-3.0.md"
    ],
    "frozen": false,
    "waiver": {
      "reason": "Poursuite autorisée sur Chromium/WebKit sans Firefox ; aucune compatibilité Firefox déclarée.",
      "evidence": "docs/DECISIONS.md"
    }
  },
  {
    "id": "3.1",
    "status": "validated",
    "source_commit": "da4dac8",
    "result_commit": "989428d",
    "report_path": "docs/reports/STEP-3.1.md",
    "tests_executed": "voir rapport",
    "approved_by": "propriétaire : Oui continue",
    "approved_at": "2026-10-23",
    "contract_changes": [],
    "evidence": [
      "docs/evidence/spike-3.1.md"
    ],
    "frozen": false
  },
  {
    "id": "3.2",
    "status": "validated",
    "source_commit": "989428d",
    "result_commit": "e94e0c3",
    "report_path": "docs/reports/STEP-3.2.md",
    "tests_executed": "voir rapport",
    "approved_by": "propriétaire : Oui continue",
    "approved_at": "2026-10-23",
    "contract_changes": [],
    "evidence": [
      "docs/evidence/spike-3.2.md",
      "docs/evidence/spike-3.2-chrome.png",
      "docs/evidence/spike-3.2-desktop.png"
    ],
    "frozen": false
  },
  {
    "id": "3.3",
    "status": "validated",
    "source_commit": "e94e0c3",
    "result_commit": "2a1b2f6",
    "report_path": "docs/reports/STEP-3.3.md",
    "tests_executed": "voir rapport",
    "approved_by": "propriétaire : Oui continue",
    "approved_at": "2026-10-23",
    "contract_changes": [
      "Politiques ZIP candidates et isolation iframe explicitées, plafonds B.2 inchangés"
    ],
    "evidence": [
      "docs/evidence/spike-3.3.md"
    ],
    "frozen": false
  },
  {
    "id": "3.4",
    "status": "validated",
    "source_commit": "7ad8e02",
    "result_commit": "8a57069",
    "report_path": "docs/reports/STEP-3.4.md",
    "tests_executed": "voir rapport",
    "approved_by": "propriétaire : Oui continue, après demande explicite de revue 3.4 et gel",
    "approved_at": "2026-10-23",
    "contract_changes": [
      "FROZEN_V1 approuvé par le propriétaire : ABI et cinq schémas existants, dérogations conservées"
    ],
    "evidence": [
      "docs/evidence/ARCHITECTURE_GATE.md",
      "docs/CHANGE_REQUESTS/CR-2026-001.md",
      "docs/evidence/cr-2026-001.md"
    ],
    "frozen": true
  },
  {
    "id": "4.0",
    "status": "validated",
    "source_commit": "8a57069",
    "result_commit": "bb0a4f7",
    "report_path": "docs/reports/STEP-4.0.md",
    "tests_executed": "voir rapport",
    "approved_by": "propriétaire : Oui continue",
    "approved_at": "2026-10-23",
    "contract_changes": [
      "Clarification ownership des buffers, ABI inchangée"
    ],
    "evidence": [
      "docs/evidence/sdk-4.0.md"
    ],
    "frozen": false
  },
  {
    "id": "4.1",
    "status": "validated",
    "source_commit": "bb0a4f7",
    "result_commit": "2696d7f",
    "report_path": "docs/reports/STEP-4.1.md",
    "tests_executed": "voir rapport",
    "approved_by": "propriétaire : Oui continue",
    "approved_at": "2026-10-23",
    "contract_changes": [
      "Aucun contrat gelé modifié ; interface moteur et horloge injectées"
    ],
    "evidence": [
      "docs/evidence/runtime-4.1.md"
    ],
    "frozen": false
  },
  {
    "id": "4.2",
    "status": "validated",
    "source_commit": "2696d7f",
    "result_commit": "46a7528",
    "report_path": "docs/reports/STEP-4.2.md",
    "tests_executed": "voir rapport",
    "approved_by": "propriétaire : Oui continue",
    "approved_at": "2026-10-23",
    "contract_changes": [
      "Politique capabilities et services hôtes abstraits ; aucun schéma/ABI gelé modifié"
    ],
    "evidence": [
      "docs/evidence/capabilities-4.2.md"
    ],
    "frozen": false
  },
  {
    "id": "4.3",
    "status": "validated",
    "source_commit": "46a7528",
    "result_commit": "9d86c17",
    "report_path": "docs/reports/STEP-4.3.md",
    "tests_executed": "voir rapport",
    "approved_by": "propriétaire : Oui continue",
    "approved_at": "2026-10-23",
    "contract_changes": [
      "Stockage abstrait et snapshot local ; aucun contrat gelé modifié"
    ],
    "evidence": [
      "docs/evidence/storage-4.3.md"
    ],
    "frozen": false
  },
  {
    "id": "4.4",
    "status": "validated",
    "source_commit": "9d86c17",
    "result_commit": "56dd7c8",
    "report_path": "docs/reports/STEP-4.4.md",
    "tests_executed": "voir rapport",
    "approved_by": "propriétaire : accord explicite pour enchaîner les quatre étapes 4.5 à 5.2",
    "approved_at": "2026-10-23",
    "contract_changes": [
      "Renderer couvrant les 17 types du schéma gelé (16 annoncés dans le texte du plan), aucun contrat gelé modifié"
    ],
    "evidence": [
      "docs/evidence/ui-4.4.md",
      "docs/evidence/ui-4.4-browser.json",
      "docs/evidence/ui-4.4-chromium.png",
      "docs/evidence/ui-4.4-webkit.png"
    ],
    "frozen": false
  },
  {
    "id": "4.5",
    "status": "validated",
    "source_commit": "56dd7c8",
    "result_commit": "820a130",
    "report_path": "docs/reports/STEP-4.5.md",
    "tests_executed": "voir rapport",
    "approved_by": "propriétaire : accord explicite pour enchaîner les quatre étapes 4.5 à 5.2",
    "approved_at": "2026-10-23",
    "contract_changes": [
      "Aucun contrat gelé modifié"
    ],
    "evidence": [
      "docs/reports/PHASE4_GATE.md"
    ],
    "frozen": false
  },
  {
    "id": "5.0",
    "status": "validated",
    "source_commit": "820a130",
    "result_commit": "commit contenant ce registre",
    "report_path": "docs/reports/STEP-5.0.md",
    "tests_executed": "voir rapport",
    "approved_by": "propriétaire : autorisation anticipée des quatre étapes 4.5 à 5.2",
    "approved_at": "2026-10-23",
    "contract_changes": [
      "Aucun contrat gelé modifié"
    ],
    "evidence": [
      "docs/evidence/web-5.0.md"
    ],
    "frozen": false
  }
]
```
