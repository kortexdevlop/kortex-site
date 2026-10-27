# Progression

Lot 8.3–9.0 terminé le 27 octobre 2026 : corpus hostile, pipeline local signé, gate PACKAGE_FORMAT_V1 et catalogue D1 indexé validés localement. Prochaine étape : 9.1, non démarrée. Linux/macOS et Firefox restent exclus ; aucun service cloud provisionné.
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
    "result_commit": "d0ffe9b",
    "report_path": "docs/reports/STEP-5.0.md",
    "tests_executed": "voir rapport",
    "approved_by": "propriétaire : accord explicite pour enchaîner les quatre étapes 4.5 à 5.2",
    "approved_at": "2026-10-23",
    "contract_changes": [
      "Aucun contrat gelé modifié"
    ],
    "evidence": [
      "docs/evidence/web-5.0.md"
    ],
    "frozen": false
  },
  {
    "id": "5.1",
    "status": "validated",
    "source_commit": "d0ffe9b",
    "result_commit": "2c17303",
    "report_path": "docs/reports/STEP-5.1.md",
    "tests_executed": "voir rapport",
    "approved_by": "propriétaire : accord explicite pour enchaîner les quatre étapes 4.5 à 5.2",
    "approved_at": "2026-10-23",
    "contract_changes": [
      "Aucun contrat gelé modifié"
    ],
    "evidence": [
      "docs/evidence/web-5.1.md"
    ],
    "frozen": false
  },
  {
    "id": "5.2",
    "status": "validated",
    "source_commit": "2c17303",
    "result_commit": "5ae8d6a",
    "report_path": "docs/reports/STEP-5.2.md",
    "tests_executed": "voir rapport",
    "approved_by": "propriétaire : autorisation anticipée des quatre étapes 4.5 à 5.2",
    "approved_at": "2026-10-23",
    "contract_changes": [
      "Aucun contrat gelé modifié"
    ],
    "evidence": [
      "docs/evidence/web-5.2.md"
    ],
    "frozen": false
  },
  {
    "id": "5.3",
    "status": "validated",
    "source_commit": "5ae8d6a",
    "result_commit": "ceac36a",
    "report_path": "docs/reports/STEP-5.3.md",
    "tests_executed": "26 tests adaptateur et 6 tests Web ; voir rapport",
    "approved_by": "propriétaire : Enchaîne les trois prochaines étapes",
    "approved_at": "2026-10-23",
    "contract_changes": [
      "Aucun contrat gelé modifié"
    ],
    "evidence": [
      "docs/evidence/web-5.3.md",
      "web-adapter/tests/loader.spec.ts"
    ],
    "frozen": false
  },
  {
    "id": "5.4",
    "status": "validated",
    "source_commit": "ceac36a",
    "result_commit": "02f6736",
    "report_path": "docs/reports/STEP-5.4.md",
    "tests_executed": "26 tests adaptateur ; 8 tests Web ; parité des contrats",
    "approved_by": "propriétaire : Enchaîne les trois prochaines étapes ; Termine ; Reprend ou tu en était et termine l’étape",
    "approved_at": "2026-10-27",
    "contract_changes": [
      "Aucun contrat gelé modifié"
    ],
    "evidence": [
      "docs/evidence/validation-5.4-6.1.json",
      "docs/evidence/web-host.md"
    ],
    "frozen": false
  },
  {
    "id": "6.0",
    "status": "validated",
    "source_commit": "ceac36a",
    "result_commit": "02f6736",
    "report_path": "docs/reports/STEP-6.0.md",
    "tests_executed": "2 tests natifs, 9 ABI, 36 régressions ; Clippy complet",
    "approved_by": "propriétaire : Enchaîne les trois prochaines étapes ; Termine ; Reprend ou tu en était et termine l’étape",
    "approved_at": "2026-10-27",
    "contract_changes": [
      "Aucun contrat gelé modifié"
    ],
    "evidence": [
      "docs/evidence/validation-5.4-6.1.json",
      "docs/evidence/native-engine.md"
    ],
    "frozen": false
  },
  {
    "id": "6.1",
    "status": "validated",
    "source_commit": "ceac36a",
    "result_commit": "02f6736",
    "report_path": "docs/reports/STEP-6.1.md",
    "tests_executed": "Parcours réel Windows WebView2 ; voir rapport",
    "approved_by": "propriétaire : Enchaîne les trois prochaines étapes ; Termine ; Reprend ou tu en était et termine l’étape",
    "approved_at": "2026-10-27",
    "contract_changes": [
      "Aucun contrat gelé modifié"
    ],
    "evidence": [
      "docs/evidence/validation-5.4-6.1.json",
      "docs/evidence/native-desktop.png"
    ],
    "frozen": false
  },
  {
    "id": "6.2",
    "status": "validated",
    "source_commit": "9c3e2d3",
    "result_commit": "269d7a2",
    "report_path": "docs/reports/STEP-6.2.md",
    "tests_executed": "48 tests Rust, HTTPS réel, consentement natif ; voir rapport",
    "approved_by": "propriétaire : Enchaîne les 3 prochaine étape ; exclusions Linux/macOS et Firefox maintenues",
    "approved_at": "2026-10-27",
    "contract_changes": [
      "Aucun schéma ou ABI gelé modifié"
    ],
    "evidence": [
      "docs/evidence/validation-6.2-7.0.json",
      "docs/evidence/native-capabilities.png"
    ],
    "frozen": false
  },
  {
    "id": "6.3",
    "status": "awaiting_approval",
    "source_commit": "9c3e2d3",
    "result_commit": "269d7a2",
    "report_path": "docs/reports/STEP-6.3.md",
    "tests_executed": "Windows réel, parité guest et récupération ; autres OS exclus",
    "approved_by": "propriétaire : Enchaîne les 3 prochaine étape ; exclusions Linux/macOS et Firefox maintenues",
    "approved_at": "2026-10-27",
    "contract_changes": [
      "Aucun schéma ou ABI gelé modifié"
    ],
    "evidence": [
      "docs/evidence/validation-6.2-7.0.json",
      "docs/evidence/native-platforms.md"
    ],
    "frozen": false,
    "waiver": {
      "reason": "Windows vérifié ; Linux/macOS exclus par instruction antérieure du propriétaire. Poursuite 7.0 autorisée sans valider la matrice complète ni le gate phase 20.",
      "evidence": "docs/evidence/native-platforms.md"
    }
  },
  {
    "id": "7.0",
    "status": "validated",
    "source_commit": "9c3e2d3",
    "result_commit": "269d7a2",
    "report_path": "docs/reports/STEP-7.0.md",
    "tests_executed": "3 tests déclaration, 46 cas contrats, cargo check wasm locked",
    "approved_by": "propriétaire : Enchaîne les 3 prochaine étape ; exclusions Linux/macOS et Firefox maintenues",
    "approved_at": "2026-10-27",
    "contract_changes": [
      "Aucun schéma ou ABI gelé modifié"
    ],
    "evidence": [
      "docs/evidence/validation-6.2-7.0.json",
      "tools/calculatrice/design.md"
    ],
    "frozen": false
  },
  {
    "id": "7.1",
    "status": "validated",
    "source_commit": "17eb70a",
    "result_commit": "84b0633",
    "report_path": "docs/reports/STEP-7.1.md",
    "tests_executed": "3 tests guest, 16 cas, 13 goldens, WASM et zéro import",
    "approved_by": "propriétaire : Enchaîne les 3 prochaines étape ; exclusions Linux/macOS et Firefox maintenues",
    "approved_at": "2026-10-27",
    "contract_changes": [
      "Aucun schéma ou ABI gelé modifié ; correction générique du pont objet Web"
    ],
    "evidence": [
      "docs/evidence/validation-7.1-7.3.json",
      "docs/evidence/tool-1-parity.md",
      "tests/golden/calculatrice-reference.json"
    ],
    "frozen": false
  },
  {
    "id": "7.2",
    "status": "validated",
    "source_commit": "17eb70a",
    "result_commit": "84b0633",
    "report_path": "docs/reports/STEP-7.2.md",
    "tests_executed": "Parcours calculatrice réels Chromium/WebKit/Windows ; même archive signée",
    "approved_by": "propriétaire : Enchaîne les 3 prochaines étape ; exclusions Linux/macOS et Firefox maintenues",
    "approved_at": "2026-10-27",
    "contract_changes": [
      "Aucun schéma ou ABI gelé modifié ; correction générique du pont objet Web"
    ],
    "evidence": [
      "docs/evidence/validation-7.1-7.3.json",
      "docs/evidence/tool-1-parity.md",
      "tests/golden/calculatrice-reference.json"
    ],
    "frozen": false
  },
  {
    "id": "7.3",
    "status": "validated",
    "source_commit": "17eb70a",
    "result_commit": "84b0633",
    "report_path": "docs/reports/STEP-7.3.md",
    "tests_executed": "80 tests Rust, 28 adaptateur Web, 10 shell/E2E, 4 Node, 46 cas contrats ; voir rapport",
    "approved_by": "propriétaire : Enchaîne les 3 prochaines étape ; exclusions Linux/macOS et Firefox maintenues",
    "approved_at": "2026-10-27",
    "contract_changes": [
      "Aucun schéma ou ABI gelé modifié ; correction générique du pont objet Web"
    ],
    "evidence": [
      "docs/evidence/validation-7.1-7.3.json",
      "docs/evidence/tool-1-parity.md",
      "tests/golden/calculatrice-reference.json"
    ],
    "frozen": true
  },
  {
    "id": "8.0",
    "status": "validated",
    "source_commit": "d6a70d8",
    "result_commit": "5e99096",
    "report_path": "docs/reports/STEP-8.0.md",
    "tests_executed": "6 tests archives, deux constructions déterministes et ABI statique",
    "approved_by": "propriétaire : Enchaîne les 3 prochaines étape ; exclusions Linux/macOS et Firefox maintenues",
    "approved_at": "2026-10-27",
    "contract_changes": [
      "Nouveau schéma artifact candidat ; aucun des cinq schémas gelés modifié",
      "ZIP canonique et SignedKeys candidats au gel 8.5"
    ],
    "evidence": [
      "docs/evidence/validation-8.0-8.2.json",
      "docs/SIGNING.md",
      "tests/fixtures/artifact-signed-test.json"
    ],
    "frozen": false
  },
  {
    "id": "8.1",
    "status": "validated",
    "source_commit": "d6a70d8",
    "result_commit": "5e99096",
    "report_path": "docs/reports/STEP-8.1.md",
    "tests_executed": "Discovery de deux outils arbitraires, doublons, CLI build/validate/package --all et --out",
    "approved_by": "propriétaire : Enchaîne les 3 prochaines étape ; exclusions Linux/macOS et Firefox maintenues",
    "approved_at": "2026-10-27",
    "contract_changes": [
      "Nouveau schéma artifact candidat ; aucun des cinq schémas gelés modifié",
      "ZIP canonique et SignedKeys candidats au gel 8.5"
    ],
    "evidence": [
      "docs/evidence/validation-8.0-8.2.json",
      "docs/SIGNING.md",
      "tests/fixtures/artifact-signed-test.json"
    ],
    "frozen": false
  },
  {
    "id": "8.2",
    "status": "validated",
    "source_commit": "d6a70d8",
    "result_commit": "5e99096",
    "report_path": "docs/reports/STEP-8.2.md",
    "tests_executed": "4 tests signature, injection CI TEST, 94 tests workspace et 40 tests Web ; voir rapport",
    "approved_by": "propriétaire : Enchaîne les 3 prochaines étape ; exclusions Linux/macOS et Firefox maintenues",
    "approved_at": "2026-10-27",
    "contract_changes": [
      "Nouveau schéma artifact candidat ; aucun des cinq schémas gelés modifié",
      "ZIP canonique et SignedKeys candidats au gel 8.5"
    ],
    "evidence": [
      "docs/evidence/validation-8.0-8.2.json",
      "docs/SIGNING.md",
      "tests/fixtures/artifact-signed-test.json"
    ],
    "frozen": false
  },
  {
    "id": "8.3",
    "status": "validated",
    "source_commit": "02b0f6d",
    "result_commit": "commit contenant ce rapport",
    "report_path": "docs/reports/STEP-8.3.md",
    "tests_executed": "98 tests Rust, 44 tests Web, Windows UI réel, 10k mutations ; sorties 0, 2026-10-27 Windows ; 1 HTTPS externe opt-in ignoré",
    "approved_by": "propriétaire : Enchaîne les 4 prochaines étapes ; autorisation du lot et du gel local, sans attestation indépendante ; exclusions Linux/macOS et Firefox maintenues",
    "approved_at": "2026-10-27",
    "contract_changes": [
      "Aucun contrat public gelé modifié"
    ],
    "evidence": [
      "docs/evidence/validation-8.3-9.0.json",
      "docs/evidence/pack-security.md"
    ],
    "frozen": false
  },
  {
    "id": "8.4",
    "status": "validated",
    "source_commit": "02b0f6d",
    "result_commit": "commit contenant ce rapport",
    "report_path": "docs/reports/STEP-8.4.md",
    "tests_executed": "release-tool dry-run et 3 tests Node pipeline/golden/catalogue : sorties 0, 2026-10-27 Windows",
    "approved_by": "propriétaire : Enchaîne les 4 prochaines étapes ; autorisation du lot et du gel local, sans attestation indépendante ; exclusions Linux/macOS et Firefox maintenues",
    "approved_at": "2026-10-27",
    "contract_changes": [
      "Aucun contrat public gelé modifié"
    ],
    "evidence": [
      "docs/evidence/validation-8.3-9.0.json",
      "docs/TOOL_PUBLISHING.md"
    ],
    "frozen": false
  },
  {
    "id": "8.5",
    "status": "validated",
    "source_commit": "02b0f6d",
    "result_commit": "commit contenant ce rapport",
    "report_path": "docs/reports/STEP-8.5.md",
    "tests_executed": "98 tests Rust, 44 tests Web, Windows UI réel, 10k mutations ; sorties 0, 2026-10-27 Windows ; 1 HTTPS externe opt-in ignoré",
    "approved_by": "propriétaire : Enchaîne les 4 prochaines étapes ; autorisation du lot et du gel local, sans attestation indépendante ; exclusions Linux/macOS et Firefox maintenues",
    "approved_at": "2026-10-27",
    "contract_changes": [
      "Gel PACKAGE_FORMAT_V1, artifact.schema et immutabilité identité/version ; cinq contrats antérieurs inchangés"
    ],
    "evidence": [
      "docs/evidence/validation-8.3-9.0.json",
      "docs/reports/PHASE8_GATE.md"
    ],
    "frozen": true
  },
  {
    "id": "9.0",
    "status": "validated",
    "source_commit": "02b0f6d",
    "result_commit": "commit contenant ce rapport",
    "report_path": "docs/reports/STEP-9.0.md",
    "tests_executed": "SQLite 3.51.3 et Wrangler 4.152.0 D1 local : 10 000 outils, 4 plans indexés, 13/5 refus ; sorties 0, 2026-10-27 Windows",
    "approved_by": "propriétaire : Enchaîne les 4 prochaines étapes ; autorisation du lot et du gel local, sans attestation indépendante ; exclusions Linux/macOS et Firefox maintenues",
    "approved_at": "2026-10-27",
    "contract_changes": [
      "Aucun contrat public gelé modifié"
    ],
    "evidence": [
      "docs/evidence/validation-8.3-9.0.json",
      "docs/DATABASE.md"
    ],
    "frozen": false
  }
]
```
