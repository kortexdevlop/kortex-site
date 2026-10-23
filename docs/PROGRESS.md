# Progression

Étape courante : 2.4. Aucun gel. Les étapes absentes sont pending.
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
    "status": "awaiting_approval",
    "source_commit": "728f57a",
    "result_commit": "commit contenant ce registre",
    "report_path": "docs/reports/STEP-2.4.md",
    "tests_executed": "voir rapport",
    "approved_by": null,
    "approved_at": null,
    "contract_changes": [
      "API de validation partagée ; pas de changement des schémas"
    ],
    "evidence": [
      "docs/reports/STEP-2.4.md"
    ],
    "frozen": false
  }
]
```
