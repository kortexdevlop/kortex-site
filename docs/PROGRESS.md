# Progression

Étape 14.1 (28/10/2026) : implémentation locale terminée sur demande explicite du propriétaire. Connexion PKCE, callback anti-rejeu, signatures/claims, sessions/cookies sécurisés, CSRF/logout, relais Astro privé et identité Desktop en lecture seule. 47 tests API, 13 tests proxy, 3 contrats réussis ; types, analyse Astro, builds et six migrations locales vérifiés. Validation réelle Google et clôture staging en attente des clients OAuth toujours inexistants ; connexion désactivée, aucun déploiement ni login réel revendiqué. Preuves : docs/evidence/identity-14.1-validation.json. 14.2 non engagée.

Lot 13.5–14.0 (28/10/2026) : 13.5 clos techniquement dans le périmètre Windows demandé ; cinq outils publics et sixième publié à chaud, aucun rebuild pendant publication, nettoyage confirmé. Linux/macOS non testés conformément à leur exclusion. 14.0 : Google choisi, schéma et contrôles préparés, 36 tests API réussis, types et migrations locales vérifiés, discovery/JWKS/S256 Google réels. Clôture 14.0 en attente des clients OAuth non encore créés selon le propriétaire ; connexion désactivée, aucun login réel revendiqué. Guide docs/AUTH.md. 14.1 non engagée. Statuts formels historiques conservés.

Lot 13.2–13.4 techniquement terminé et clos le 28/10/2026 : téléchargement/installation signée et atomique, bibliothèque/désinstallation avec données conservées par défaut, mises à jour avec permissions explicites et retour arrière. 24 tests Rust et parcours réel Windows/Cloudflare réussis, ouverture hors connexion incluse. Ressources temporaires nettoyées et politique opérateur restaurée. Preuves : docs/evidence/desktop-13.2-13.4-validation.json. 13.5 non engagée ; statuts formels de revue humaine conservés. Les paragraphes suivants décrivent les lots historiques.

Lot 12.6–13.1 techniquement réalisé et clos le 28/10/2026 sur demande du propriétaire : extension temporaire publiée/testée sans modification ou redéploiement des clients pendant publication, puis révoquée/nettoyée ; application Windows produit et Store relié à l’API réelle. Dix tests Rust ciblés, pipeline de signature, test navigateur Cloudflare et deux parcours Windows (connecté/hors connexion) réussis. Bibliothèque signée et paquet altéré refusé ; pagination testée sur 10 000 entrées synthétiques. Preuves : docs/evidence/desktop-phase13-validation.json. Pas de téléchargement/installation Store (13.2 non engagée). Les statuts formels historiques restent distincts de l’exécution technique, sans approbation visuelle inventée.

Lot 12.3–12.5 techniquement terminé et clos le 28/10/2026 sur demande du propriétaire. Explorateur CSV publié ; export réel corrigé et vérifié, stockage IndexedDB réel relu après rechargement avec permission navigateur, refus et absence de persistance couverts. Cinq paquets signés démarrent sur le vrai site Cloudflare ; neuf vecteurs partagés Web/Wasmtime. Build/validate des cinq outils et suite Rust complète réussis en série (125 tests, 1 ignorés explicitement), 3 essais Cloudflare de clôture réussis. Preuves : docs/evidence/phase12-closure.json. Le quota D1 et le disque plein sont des incidents historiques résolus. Les statuts de revue humaine historique restent distincts ; aucune approbation visuelle inventée. 12.6 non engagée. Production, DNS et messagerie intacts.

Lot 12.0–12.2 réalisé sur instruction du propriétaire : convertisseur de casse Unicode, SHA-256 de fichier local et dates UTC/Unix. Trois paquets canoniques signés/vérifiés et publiés réellement sur le staging existant ; site non reconstruit, DNS/production/messagerie inchangés. 9 tests Rust, 2 tests Wasmtime Windows, 12 essais navigateur locaux/distants Chromium/WebKit et 3 interfaces Windows réussis. Vrais fichiers, handles/permissions, annulation, dates négatives/overflow et sorties UI/actions vérifiés ; clés privées en mémoire uniquement. Les mêmes SHA sont tracés localement, sur Cloudflare et dans WebView2. Le dialogue Windows de sélection positive n’est pas automatisé : sélection de fichier contrôlée à sa frontière, avec vraies lectures et permissions du backend. Statuts formels pending derrière la revue visuelle existante, sans validation humaine inventée. État historique au 27/10/2026 ; le lot suivant est clos ci-dessus. Linux/macOS et Firefox exclus.

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
    "result_commit": "8ffb37d",
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
    "result_commit": "8ffb37d",
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
    "result_commit": "8ffb37d",
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
    "result_commit": "8ffb37d",
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
  },
  {
    "id": "9.1",
    "status": "validated",
    "source_commit": "751a84f",
    "result_commit": "8975fd4",
    "report_path": "docs/reports/STEP-9.1.md",
    "tests_executed": "2026-10-27 Windows, SQLite : 30 refus + transaction rollback ; D1 local : 2 migrations, 12 refus, idempotence/revocation ; régression 10k catalogue ; sorties 0",
    "approved_by": "propriétaire : terminer la première étape puis préparer la deuxième et arrêt au besoin de connexion Cloudflare",
    "approved_at": "2026-10-27",
    "contract_changes": [
      "Migration additive publication ; aucun schéma gelé modifié"
    ],
    "evidence": [
      "docs/evidence/validation-9.1.json",
      "infra/r2-policy.md"
    ],
    "frozen": false
  },
  {
    "id": "9.2",
    "status": "validated",
    "source_commit": "8237f5e",
    "result_commit": "8975fd4",
    "report_path": "docs/reports/STEP-9.2.md",
    "tests_executed": "2026-10-27 Windows : 3 tests DTO Node et 6 tests workerd, typecheck, dry-run, HTTP local 200/404 et 2 migrations locales réussis. Lecture OAuth : session existante ; inventaires D1/R2 refusés code 10000. Aucun déploiement. Reprise : profil kortex confirmé, D1/R2 list autorisés, 2 bases de comptes existantes, aucun bucket retourné dans la juridiction par défaut ; Workers staging/production non trouvés code 10007. Provisionnement distant reporté à 9.6.",
    "approved_by": "propriétaire : préparation 9.2 et reprise du contrôle après Profil kortex activé ; aucune autorisation de création ou déploiement",
    "approved_at": "2026-10-27",
    "authorization": "Propriétaire : préparer la deuxième étape, vérifier la connexion Cloudflare et arrêter au besoin de connexion avec état et procédure de reprise",
    "contract_changes": [
      "Nouveau schéma API candidat ; six schémas gelés inchangés"
    ],
    "evidence": [
      "docs/evidence/validation-9.2-local.json",
      "docs/evidence/cloudflare-checkpoint-9.2.json",
      "infra/cloudflare-setup.md",
      "docs/evidence/cloudflare-reauthorization-9.2.json",
      "docs/evidence/cloudflare-kortex-profile-9.2.json"
    ],
    "frozen": false,
    "previous_block_reason": "Réautorisation utilisateur réussie et scopes Workers/D1 demandés, mais le contrôle suivant renouvelle un jeton limité à Pages ; D1/R2 refusés code 10000. Cause non établie. Profil kortex dédié proposé, sans login ni modification de droits par l’agent."
  },
  {
    "id": "9.3",
    "status": "validated",
    "source_commit": "088bceec414e0dcd7f4282402604abff549cc278",
    "result_commit": "0a21749",
    "approved_by": "Propriétaire : C'est parfait, enchaîne les trois prochaines étapes",
    "approved_at": "2026-10-27",
    "report_path": "docs/reports/STEP-9.3.md",
    "tests_executed": "3 DTO Node + 15 Worker ; 10 000 outils par tri, filtres, curseurs HMAC, cache et erreurs ; typecheck/build/smoke/frozen/secrets réussis",
    "contract_changes": [],
    "evidence": [
      "docs/evidence/validation-9.3-local.json",
      "docs/evidence/cloudflare-reset-2026-10-27.json",
      "docs/reports/CLOUDFLARE-RESET-2026-10-27.md"
    ],
    "frozen": false
  },
  {
    "id": "9.4",
    "status": "validated",
    "source_commit": "6c235f0",
    "result_commit": "d3345ce",
    "approved_by": "Propriétaire : lot explicite 9.4–9.6, poursuite après validation technique intermédiaire (pas revue indépendante)",
    "approved_at": "2026-10-27",
    "report_path": "docs/reports/STEP-9.4.md",
    "tests_executed": "3 DTO + 19 Worker ; streaming R2 exact/24 Mio, cache, absence, remplacement, révocation ; typecheck",
    "contract_changes": [],
    "evidence": [
      "docs/evidence/validation-9.4-9.6.json"
    ],
    "frozen": false
  },
  {
    "id": "9.5",
    "status": "validated",
    "source_commit": "d3345ce",
    "result_commit": "56947c7",
    "approved_by": "Propriétaire : lot explicite 9.4–9.6, poursuite après validation technique intermédiaire (pas revue indépendante)",
    "approved_at": "2026-10-27",
    "report_path": "docs/reports/STEP-9.5.md",
    "tests_executed": "3 DTO + 26 Worker ; 7 publication/auth/courses/transactions/rollback/révocation ; typecheck et build",
    "contract_changes": [],
    "evidence": [
      "docs/evidence/validation-9.4-9.6.json"
    ],
    "frozen": false
  },
  {
    "id": "9.6",
    "status": "validated",
    "source_commit": "56947c7",
    "result_commit": "3fbaada",
    "approved_by": "Propriétaire : Enchaîne les 3 prochaines étapes, poursuite explicite après le lot 9.4–9.6",
    "approved_at": "2026-10-27",
    "report_path": "docs/reports/STEP-9.6.md",
    "tests_executed": "Roundtrip Cloudflare réel ; PUT R2 conditionnel, publication, rollback et 410 ; Chromium/WebKit et Windows ouverts sur les mêmes octets ; API finale 3 DTO + 26 Worker",
    "contract_changes": [],
    "evidence": [
      "docs/evidence/validation-9.4-9.6.json",
      "docs/evidence/cloud-publish.md",
      "docs/evidence/staging9-chromium.png",
      "docs/evidence/staging9-webkit.png",
      "docs/evidence/staging9-windows.png"
    ],
    "frozen": false
  },
  {
    "id": "10.0",
    "status": "awaiting_approval",
    "source_commit": "b7ffda5",
    "result_commit": "aaaf960",
    "approved_by": null,
    "approved_at": null,
    "report_path": "docs/reports/STEP-10.0.md",
    "tests_executed": "Nouvelle référence utilisateur 2706x1764 reçue et mesurée ; SHA tracé ; géométrie ±4 px, CIEDE2000 sur 11 points ; revue visuelle propriétaire attendue",
    "contract_changes": [],
    "evidence": [
      "docs/evidence/validation-10.0-10.2.json",
      "docs/evidence/workspace-validation.json",
      "docs/reports/DESIGN-WORKSPACE-2026-10-27.md"
    ],
    "frozen": false
  },
  {
    "id": "10.1",
    "status": "pending",
    "source_commit": "b7ffda5",
    "result_commit": "aaaf960",
    "approved_by": null,
    "approved_at": null,
    "report_path": "docs/reports/STEP-10.1.md",
    "tests_executed": "Préparation réversible : 2 tests de contrastes/source unique réussis ; validation dépendante de 10.0 ; reprise selon nouvelle référence workspace : 14 tests, 20 compatibilité, 10 Node, build SSR et 10 HTTP ; contrastes du raster non déclarés AA",
    "contract_changes": [],
    "evidence": [
      "docs/evidence/validation-10.0-10.2.json",
      "docs/evidence/workspace-validation.json",
      "docs/reports/DESIGN-WORKSPACE-2026-10-27.md"
    ],
    "frozen": false
  },
  {
    "id": "10.2",
    "status": "pending",
    "source_commit": "b7ffda5",
    "result_commit": "aaaf960",
    "approved_by": null,
    "approved_at": null,
    "report_path": "docs/reports/STEP-10.2.md",
    "tests_executed": "Préparation réversible : 8 tests Chromium/WebKit, axe, clavier/mobile et build réussis ; fidélité visuelle non vérifiée ; reprise selon nouvelle référence workspace : 14 tests, 20 compatibilité, 10 Node, build SSR et 10 HTTP ; contrastes du raster non déclarés AA",
    "contract_changes": [],
    "evidence": [
      "docs/evidence/validation-10.0-10.2.json",
      "docs/evidence/workspace-validation.json",
      "docs/reports/DESIGN-WORKSPACE-2026-10-27.md"
    ],
    "frozen": false
  },
  {
    "id": "10.3",
    "status": "pending",
    "source_commit": "b7ffda5",
    "result_commit": "aaaf960",
    "approved_by": null,
    "approved_at": null,
    "report_path": "docs/reports/STEP-10.3.md",
    "tests_executed": "Implémentation préparée : cartes, états et recherche ; 20 tests navigateur consolidés ; reprise selon nouvelle référence workspace : 14 tests, 20 compatibilité, 10 Node, build SSR et 10 HTTP ; contrastes du raster non déclarés AA",
    "contract_changes": [],
    "evidence": [
      "docs/evidence/validation-10.3-11.0.json",
      "docs/evidence/workspace-validation.json",
      "docs/reports/DESIGN-WORKSPACE-2026-10-27.md"
    ],
    "frozen": false
  },
  {
    "id": "10.4",
    "status": "pending",
    "source_commit": "b7ffda5",
    "result_commit": "aaaf960",
    "approved_by": null,
    "approved_at": null,
    "report_path": "docs/reports/STEP-10.4.md",
    "tests_executed": "Captures desktop/tablette/mobile et axe ; comparaison à l’original non exécutée ; reprise selon nouvelle référence workspace : 14 tests, 20 compatibilité, 10 Node, build SSR et 10 HTTP ; contrastes du raster non déclarés AA",
    "contract_changes": [],
    "evidence": [
      "docs/evidence/validation-10.3-11.0.json",
      "docs/evidence/workspace-validation.json",
      "docs/reports/DESIGN-WORKSPACE-2026-10-27.md"
    ],
    "frozen": false
  },
  {
    "id": "11.0",
    "status": "pending",
    "source_commit": "b7ffda5",
    "result_commit": "aaaf960",
    "approved_by": null,
    "approved_at": null,
    "report_path": "docs/reports/STEP-11.0.md",
    "tests_executed": "SSR compilé ; 10 tests Node, 8 HTTP sur Worker compilé ; API 3 DTO + 26 Worker ; reprise selon nouvelle référence workspace : 14 tests, 20 compatibilité, 10 Node, build SSR et 10 HTTP ; contrastes du raster non déclarés AA",
    "contract_changes": [],
    "evidence": [
      "docs/evidence/validation-10.3-11.0.json",
      "docs/evidence/workspace-validation.json",
      "docs/reports/DESIGN-WORKSPACE-2026-10-27.md"
    ],
    "frozen": false
  },
  {
    "id": "11.1",
    "status": "pending",
    "source_commit": "aa59e70",
    "result_commit": "3a5d1fc",
    "approved_by": null,
    "approved_at": null,
    "report_path": "docs/reports/STEP-11.1.md",
    "tests_executed": "Implémentation locale du lot autorisé : 10 essais intégrés Worker SSR/D1/Runtime Chromium et WebKit, 14 régressions maquette, 14 Node ; zéro diagnostic strict, build réussi ; calcul signé 25×48=1200, archive altérée refusée. Statut formel pending derrière la revue visuelle, sans approbation inventée.",
    "contract_changes": [],
    "evidence": [
      "docs/evidence/website-11.1-11.3-validation.json",
      "docs/evidence/website-11.1-11.3-playwright.json",
      "docs/evidence/website-workspace-regression/playwright.json"
    ],
    "frozen": false
  },
  {
    "id": "11.2",
    "status": "pending",
    "source_commit": "aa59e70",
    "result_commit": "3a5d1fc",
    "approved_by": null,
    "approved_at": null,
    "report_path": "docs/reports/STEP-11.2.md",
    "tests_executed": "Implémentation locale du lot autorisé : 10 essais intégrés Worker SSR/D1/Runtime Chromium et WebKit, 14 régressions maquette, 14 Node ; zéro diagnostic strict, build réussi ; calcul signé 25×48=1200, archive altérée refusée. Statut formel pending derrière la revue visuelle, sans approbation inventée.",
    "contract_changes": [],
    "evidence": [
      "docs/evidence/website-11.1-11.3-validation.json",
      "docs/evidence/website-11.1-11.3-playwright.json",
      "docs/evidence/website-workspace-regression/playwright.json"
    ],
    "frozen": false
  },
  {
    "id": "11.3",
    "status": "pending",
    "source_commit": "aa59e70",
    "result_commit": "3a5d1fc",
    "approved_by": null,
    "approved_at": null,
    "report_path": "docs/reports/STEP-11.3.md",
    "tests_executed": "Implémentation locale du lot autorisé : 10 essais intégrés Worker SSR/D1/Runtime Chromium et WebKit, 14 régressions maquette, 14 Node ; zéro diagnostic strict, build réussi ; calcul signé 25×48=1200, archive altérée refusée. Statut formel pending derrière la revue visuelle, sans approbation inventée.",
    "contract_changes": [],
    "evidence": [
      "docs/evidence/website-11.1-11.3-validation.json",
      "docs/evidence/website-11.1-11.3-playwright.json",
      "docs/evidence/website-workspace-regression/playwright.json"
    ],
    "frozen": false
  },
  {
    "id": "11.4",
    "status": "pending",
    "source_commit": "524a6b0",
    "result_commit": "865b25b",
    "approved_by": null,
    "approved_at": null,
    "report_path": "docs/reports/STEP-11.4.md",
    "tests_executed": "Lot expressément autorisé et réalisé : API 3 DTO + 28 Worker ; site 51 fichiers sans diagnostic et 16 Node ; 8 essais locaux + 2 distants Chromium/WebKit ; sitemap 10000 IDs ; recherche D1 Cloudflare 10000 entrées p95 430,69 ms, ressources temporaires supprimées ; publication signée, calcul 1200, révocation, hash et déploiement Web inchangés. Statut formel pending derrière la revue visuelle existante, sans approbation inventée.",
    "contract_changes": [],
    "evidence": [
      "docs/evidence/website-11.4-11.6-validation.json",
      "docs/evidence/website-11.4-11.6-playwright.json",
      "docs/evidence/web-hot-publish.json",
      "docs/evidence/hot-publish-playwright.json",
      "docs/evidence/search-staging-benchmark.json"
    ],
    "frozen": false
  },
  {
    "id": "11.5",
    "status": "pending",
    "source_commit": "524a6b0",
    "result_commit": "865b25b",
    "approved_by": null,
    "approved_at": null,
    "report_path": "docs/reports/STEP-11.5.md",
    "tests_executed": "Lot expressément autorisé et réalisé : API 3 DTO + 28 Worker ; site 51 fichiers sans diagnostic et 16 Node ; 8 essais locaux + 2 distants Chromium/WebKit ; sitemap 10000 IDs ; recherche D1 Cloudflare 10000 entrées p95 430,69 ms, ressources temporaires supprimées ; publication signée, calcul 1200, révocation, hash et déploiement Web inchangés. Statut formel pending derrière la revue visuelle existante, sans approbation inventée.",
    "contract_changes": [],
    "evidence": [
      "docs/evidence/website-11.4-11.6-validation.json",
      "docs/evidence/website-11.4-11.6-playwright.json",
      "docs/evidence/web-hot-publish.json",
      "docs/evidence/hot-publish-playwright.json",
      "docs/evidence/search-staging-benchmark.json"
    ],
    "frozen": false
  },
  {
    "id": "11.6",
    "status": "pending",
    "source_commit": "524a6b0",
    "result_commit": "865b25b",
    "approved_by": null,
    "approved_at": null,
    "report_path": "docs/reports/STEP-11.6.md",
    "tests_executed": "Lot expressément autorisé et réalisé : API 3 DTO + 28 Worker ; site 51 fichiers sans diagnostic et 16 Node ; 8 essais locaux + 2 distants Chromium/WebKit ; sitemap 10000 IDs ; recherche D1 Cloudflare 10000 entrées p95 430,69 ms, ressources temporaires supprimées ; publication signée, calcul 1200, révocation, hash et déploiement Web inchangés. Statut formel pending derrière la revue visuelle existante, sans approbation inventée.",
    "contract_changes": [],
    "evidence": [
      "docs/evidence/website-11.4-11.6-validation.json",
      "docs/evidence/website-11.4-11.6-playwright.json",
      "docs/evidence/web-hot-publish.json",
      "docs/evidence/hot-publish-playwright.json",
      "docs/evidence/search-staging-benchmark.json"
    ],
    "frozen": false
  },
  {
    "id": "12.0",
    "status": "pending",
    "source_commit": "0547b00",
    "result_commit": "d53ff5d",
    "approved_by": null,
    "approved_at": null,
    "report_path": "docs/reports/STEP-12.0.md",
    "tests_executed": "Lot autorisé réalisé : 9 tests Rust des trois guests, 2 tests Wasmtime Windows avec vrais fichiers/permissions, 6 essais Chromium/WebKit locaux et 6 Cloudflare, 3 interfaces Windows WebView2. Build/validate/package/sign/verify canoniques et publication réelle de trois paquets ; même SHA local/Web/Desktop ; site non reconstruit. Statut formel pending derrière la revue visuelle existante, sans approbation inventée.",
    "contract_changes": [],
    "evidence": [
      "docs/evidence/phase12-validation.json",
      "docs/evidence/phase12-local-playwright.json",
      "docs/evidence/phase12-staging-playwright.json",
      "docs/evidence/phase12-staging.json",
      "docs/evidence/phase12-windows.json"
    ],
    "frozen": false
  },
  {
    "id": "12.1",
    "status": "pending",
    "source_commit": "0547b00",
    "result_commit": "d53ff5d",
    "approved_by": null,
    "approved_at": null,
    "report_path": "docs/reports/STEP-12.1.md",
    "tests_executed": "Lot autorisé réalisé : 9 tests Rust des trois guests, 2 tests Wasmtime Windows avec vrais fichiers/permissions, 6 essais Chromium/WebKit locaux et 6 Cloudflare, 3 interfaces Windows WebView2. Build/validate/package/sign/verify canoniques et publication réelle de trois paquets ; même SHA local/Web/Desktop ; site non reconstruit. Statut formel pending derrière la revue visuelle existante, sans approbation inventée.",
    "contract_changes": [],
    "evidence": [
      "docs/evidence/phase12-validation.json",
      "docs/evidence/phase12-local-playwright.json",
      "docs/evidence/phase12-staging-playwright.json",
      "docs/evidence/phase12-staging.json",
      "docs/evidence/phase12-windows.json"
    ],
    "frozen": false
  },
  {
    "id": "12.2",
    "status": "pending",
    "source_commit": "0547b00",
    "result_commit": "d53ff5d",
    "approved_by": null,
    "approved_at": null,
    "report_path": "docs/reports/STEP-12.2.md",
    "tests_executed": "Lot autorisé réalisé : 9 tests Rust des trois guests, 2 tests Wasmtime Windows avec vrais fichiers/permissions, 6 essais Chromium/WebKit locaux et 6 Cloudflare, 3 interfaces Windows WebView2. Build/validate/package/sign/verify canoniques et publication réelle de trois paquets ; même SHA local/Web/Desktop ; site non reconstruit. Statut formel pending derrière la revue visuelle existante, sans approbation inventée.",
    "contract_changes": [],
    "evidence": [
      "docs/evidence/phase12-validation.json",
      "docs/evidence/phase12-local-playwright.json",
      "docs/evidence/phase12-staging-playwright.json",
      "docs/evidence/phase12-staging.json",
      "docs/evidence/phase12-windows.json"
    ],
    "frozen": false
  },
  {
    "id": "12.3",
    "status": "pending",
    "source_commit": "d1450f0",
    "result_commit": "db5f3ec",
    "approved_by": null,
    "approved_at": null,
    "report_path": "docs/reports/STEP-12.3.md",
    "tests_executed": "125 tests Rust workspace réussis en série (1 ignorés explicites), 14 tests CSV, 4 tests Wasmtime phase12, build/validate cinq outils, 5 démarrages Web locaux, 9 vecteurs Web/Wasmtime, 3 tests Cloudflare (export réel, refus, stockage/rechargement et cinq outils). Clôture technique complète ; revue humaine historique conservée sans approbation inventée.",
    "contract_changes": [],
    "evidence": [
      "docs/evidence/phase12-closure.json",
      "docs/evidence/phase12-csv-chromium.png",
      "docs/evidence/phase12-csv-local-playwright-chromium.json",
      "docs/evidence/phase12-windows.json",
      "docs/evidence/phase12-windows-explorateur-csv.png",
      "docs/evidence/five-tools-web.json",
      "docs/evidence/five-tools-matrix.md",
      "docs/evidence/phase12-staging.json",
      "docs/evidence/phase12-csv-staging-playwright.json"
    ],
    "frozen": false,
    "technical_status": "closed",
    "closed_at": "2026-10-28",
    "closure_request": "propriétaire : Termines les correctement et clotures les avant de passer à la suite"
  },
  {
    "id": "12.4",
    "status": "pending",
    "source_commit": "d1450f0",
    "result_commit": "db5f3ec",
    "approved_by": null,
    "approved_at": null,
    "report_path": "docs/reports/STEP-12.4.md",
    "tests_executed": "125 tests Rust workspace réussis en série (1 ignorés explicites), 14 tests CSV, 4 tests Wasmtime phase12, build/validate cinq outils, 5 démarrages Web locaux, 9 vecteurs Web/Wasmtime, 3 tests Cloudflare (export réel, refus, stockage/rechargement et cinq outils). Clôture technique complète ; revue humaine historique conservée sans approbation inventée.",
    "contract_changes": [
      "KTX-WEB-ORIGIN-001 : allow-downloads ajouté au sandbox pour file.save après consentement, sur demande explicite de corriger et clôturer."
    ],
    "evidence": [
      "docs/evidence/phase12-closure.json",
      "docs/evidence/phase12-csv-chromium.png",
      "docs/evidence/phase12-csv-local-playwright-chromium.json",
      "docs/evidence/phase12-windows.json",
      "docs/evidence/phase12-windows-explorateur-csv.png",
      "docs/evidence/five-tools-web.json",
      "docs/evidence/five-tools-matrix.md",
      "docs/evidence/phase12-staging.json",
      "docs/evidence/phase12-csv-staging-playwright.json"
    ],
    "frozen": false,
    "technical_status": "closed",
    "closed_at": "2026-10-28",
    "closure_request": "propriétaire : Termines les correctement et clotures les avant de passer à la suite"
  },
  {
    "id": "12.5",
    "status": "pending",
    "source_commit": "d1450f0",
    "result_commit": "db5f3ec",
    "approved_by": null,
    "approved_at": null,
    "report_path": "docs/reports/STEP-12.5.md",
    "tests_executed": "125 tests Rust workspace réussis en série (1 ignorés explicites), 14 tests CSV, 4 tests Wasmtime phase12, build/validate cinq outils, 5 démarrages Web locaux, 9 vecteurs Web/Wasmtime, 3 tests Cloudflare (export réel, refus, stockage/rechargement et cinq outils). Clôture technique complète ; revue humaine historique conservée sans approbation inventée.",
    "contract_changes": [],
    "evidence": [
      "docs/evidence/phase12-closure.json",
      "docs/evidence/phase12-csv-chromium.png",
      "docs/evidence/phase12-csv-local-playwright-chromium.json",
      "docs/evidence/phase12-windows.json",
      "docs/evidence/phase12-windows-explorateur-csv.png",
      "docs/evidence/five-tools-web.json",
      "docs/evidence/five-tools-matrix.md",
      "docs/evidence/phase12-staging.json",
      "docs/evidence/phase12-csv-staging-playwright.json"
    ],
    "frozen": false,
    "technical_status": "closed",
    "closed_at": "2026-10-28",
    "closure_request": "propriétaire : Termines les correctement et clotures les avant de passer à la suite"
  },
  {
    "id": "12.6",
    "status": "pending",
    "source_commit": "6a47c5e",
    "result_commit": "06bf7b5",
    "approved_by": null,
    "approved_at": null,
    "report_path": "docs/reports/STEP-12.6.md",
    "tests_executed": "CLI externe build/validate/package/sign/verify, tests xtask et pipeline local, publication et action echo sur le staging réel ; hashes et déploiements clients identiques pendant publication ; fixture et clé révoquées, R2 nettoyé, réglages restaurés.",
    "contract_changes": [],
    "evidence": [
      "docs/evidence/desktop-phase13-validation.json",
      "docs/evidence/extension-staging.json",
      "docs/evidence/desktop-offline-windows.json"
    ],
    "frozen": false,
    "technical_status": "closed",
    "closed_at": "2026-10-28",
    "authorization": "propriétaire : Enchaines ces 3 étapes, travaille de façon hyper intelligente pour avancer rapidement"
  },
  {
    "id": "13.0",
    "status": "pending",
    "source_commit": "6a47c5e",
    "result_commit": "06bf7b5",
    "approved_by": null,
    "approved_at": null,
    "report_path": "docs/reports/STEP-13.0.md",
    "tests_executed": "Application produit Windows démarrée dans WebView2, bibliothèque signée sans compte, outil ouvert et conversion Unicode exécutée hors réseau sans dialogue de permission indu ; refus du paquet altéré.",
    "contract_changes": [],
    "evidence": [
      "docs/evidence/desktop-phase13-validation.json",
      "docs/evidence/desktop-online-windows.json",
      "docs/evidence/desktop-offline-windows.json"
    ],
    "frozen": false,
    "technical_status": "closed",
    "closed_at": "2026-10-28",
    "authorization": "propriétaire : Enchaines ces 3 étapes, travaille de façon hyper intelligente pour avancer rapidement"
  },
  {
    "id": "13.1",
    "status": "pending",
    "source_commit": "6a47c5e",
    "result_commit": "06bf7b5",
    "approved_by": null,
    "approved_at": null,
    "report_path": "docs/reports/STEP-13.1.md",
    "tests_executed": "Store réel Cloudflare : cinq fiches, recherche, catégorie, tri et actions ; contrôleur pagination sur 10000 synthétiques ; HTTP local réel, TTL, 404/503, offline et schémas partagés. Dix tests Rust ciblés au total pour desktop/xtask.",
    "contract_changes": [],
    "evidence": [
      "docs/evidence/desktop-phase13-validation.json",
      "docs/evidence/desktop-online-windows.json",
      "docs/evidence/desktop-offline-windows.json"
    ],
    "frozen": false,
    "technical_status": "closed",
    "closed_at": "2026-10-28",
    "authorization": "propriétaire : Enchaines ces 3 étapes, travaille de façon hyper intelligente pour avancer rapidement"
  },
  {
    "id": "13.2",
    "status": "pending",
    "source_commit": "aff61f9",
    "result_commit": "e654607",
    "approved_by": null,
    "approved_at": null,
    "report_path": "docs/reports/STEP-13.2.md",
    "tests_executed": "24 tests Rust ciblés réussis ; parcours réel Windows/Cloudflare : installation signée, ouverture hors connexion, confirmation des permissions, mise à jour, refus du guest défaillant, retour arrière, désinstallation avec conservation puis purge isolée. Fixtures, objets et clé temporaires nettoyés ; politique opérateur restaurée.",
    "contract_changes": [],
    "evidence": [
      "docs/evidence/desktop-13.2-13.4-validation.json",
      "docs/evidence/desktop-lifecycle-windows.json",
      "docs/evidence/desktop-lifecycle-staging.json"
    ],
    "frozen": false,
    "technical_status": "closed",
    "closed_at": "2026-10-28",
    "authorization": "propriétaire : Enchaines ces 3 étapes, travaille de façon hyper intelligente pour avancer rapidement"
  },
  {
    "id": "13.3",
    "status": "pending",
    "source_commit": "aff61f9",
    "result_commit": "e654607",
    "approved_by": null,
    "approved_at": null,
    "report_path": "docs/reports/STEP-13.3.md",
    "tests_executed": "24 tests Rust ciblés réussis ; parcours réel Windows/Cloudflare : installation signée, ouverture hors connexion, confirmation des permissions, mise à jour, refus du guest défaillant, retour arrière, désinstallation avec conservation puis purge isolée. Fixtures, objets et clé temporaires nettoyés ; politique opérateur restaurée.",
    "contract_changes": [],
    "evidence": [
      "docs/evidence/desktop-13.2-13.4-validation.json",
      "docs/evidence/desktop-lifecycle-windows.json",
      "docs/evidence/desktop-lifecycle-staging.json"
    ],
    "frozen": false,
    "technical_status": "closed",
    "closed_at": "2026-10-28",
    "authorization": "propriétaire : Enchaines ces 3 étapes, travaille de façon hyper intelligente pour avancer rapidement"
  },
  {
    "id": "13.4",
    "status": "pending",
    "source_commit": "aff61f9",
    "result_commit": "e654607",
    "approved_by": null,
    "approved_at": null,
    "report_path": "docs/reports/STEP-13.4.md",
    "tests_executed": "24 tests Rust ciblés réussis ; parcours réel Windows/Cloudflare : installation signée, ouverture hors connexion, confirmation des permissions, mise à jour, refus du guest défaillant, retour arrière, désinstallation avec conservation puis purge isolée. Fixtures, objets et clé temporaires nettoyés ; politique opérateur restaurée.",
    "contract_changes": [],
    "evidence": [
      "docs/evidence/desktop-13.2-13.4-validation.json",
      "docs/evidence/desktop-lifecycle-windows.json",
      "docs/evidence/desktop-lifecycle-staging.json"
    ],
    "frozen": false,
    "technical_status": "closed",
    "closed_at": "2026-10-28",
    "authorization": "propriétaire : Enchaines ces 3 étapes, travaille de façon hyper intelligente pour avancer rapidement"
  },
  {
    "id": "13.5",
    "status": "pending",
    "source_commit": "ad00aa5",
    "result_commit": "009e677",
    "approved_by": null,
    "approved_at": null,
    "report_path": "docs/reports/STEP-13.5.md",
    "tests_executed": "Windows réel : cinq outils installés/lancés/rendus/désinstallés, sixième publié pendant app ouverte sans rebuild ; conversion hors connexion, update/rollback/purge et nettoyage staging vérifiés. Linux/macOS exclus, non testés.",
    "contract_changes": [],
    "evidence": [
      "docs/evidence/desktop-store135-validation.json"
    ],
    "frozen": false,
    "technical_status": "closed",
    "closed_at": "2026-10-28",
    "authorization": "propriétaire : Enchaines ces 2 étapes rapidement"
  },
  {
    "id": "14.0",
    "status": "pending",
    "source_commit": "ad00aa5",
    "result_commit": "009e677",
    "approved_by": null,
    "approved_at": null,
    "report_path": "docs/reports/STEP-14.0.md",
    "tests_executed": "36 tests API dont 8 identité, types et cinq migrations locales réussis ; Google discovery/JWKS/S256 réels vérifiés. Clients OAuth inexistants selon propriétaire ; aucune connexion testée.",
    "contract_changes": [],
    "evidence": [
      "docs/evidence/identity-14.0-validation.json"
    ],
    "frozen": false,
    "technical_status": "awaiting_external_configuration",
    "remaining": [
      "Clients OAuth Google Web et Desktop, consentement et configuration staging"
    ],
    "authorization": "propriétaire : Enchaines ces 2 étapes rapidement"
  },
  {
    "id": "14.1",
    "status": "pending",
    "source_commit": "53a0c02",
    "result_commit": null,
    "approved_by": null,
    "approved_at": null,
    "report_path": "docs/reports/STEP-14.1.md",
    "tests_executed": "47 tests API dont 11 nouveaux scénarios auth, 13 tests proxy, 3 contrats ; types, analyse Astro (54 fichiers sans erreur), builds API/Web et six migrations locales réussis. RSA et D1 réels, transport IdP contrôlé. Pas de login Google réel ni déploiement.",
    "contract_changes": [],
    "evidence": [
      "docs/evidence/identity-14.1-validation.json"
    ],
    "frozen": false,
    "technical_status": "awaiting_real_google_validation",
    "implementation_status": "complete_local",
    "remaining": [
      "Clients OAuth Google Web/Desktop et consentement",
      "Secrets serveur, déploiement staging et login réel navigateur/cookies/logout"
    ],
    "authorization": "propriétaire : Enchaine la prochaine étape rapidement ; Termine"
  }
]
```
