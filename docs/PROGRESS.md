# Progression

Étape courante : 1.4. Aucun gel. Les étapes absentes sont pending.
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
    "status": "awaiting_approval",
    "source_commit": "f9f8dce",
    "result_commit": "commit contenant ce registre",
    "approved_by": null,
    "approved_at": null,
    "report_path": "docs/reports/STEP-1.4.md",
    "tests_executed": "voir rapport",
    "contract_changes": [],
    "evidence": [
      "docs/reports/STEP-1.4.md"
    ],
    "frozen": false
  }
]
```
