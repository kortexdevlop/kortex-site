# Matrice des cinq outils — étape 12.5

Mis à jour le 28/10/2026 : publications réelles et tests Cloudflare rejoués, voir phase12-closure.json.

| Outil | Version | SHA paquet | SHA guest (module.wasm) | Permissions déclarées | Actions vérifiées | Web (Chromium, paquet signé) | Bureau Windows (WebView2) | Publié staging |
|---|---|---|---|---|---|---|---|---|
| calculatrice | 1.0.0 | `1d4c1a0967b5` | `f095744f63b9` | aucune | add, divide | oui | calculatrice : preuve 7.x/9.x | oui |
| convertisseur-casse | 1.0.0 | `d4dfadd3c099` | `7c0345eb1d2e` | aucune | upper, title | oui | oui | oui |
| calculateur-hash | 1.0.0 | `aee5fc4a0e71` | `701b38100ea3` | file.pick, file.read | sha256_bytes | oui | oui | oui |
| timestamp-utc | 1.0.0 | `7ca4588ad879` | `dbd029d4ea26` | aucune | from_unix, to_unix | oui | oui | oui |
| explorateur-csv | 1.0.0 | `183f838f5548` | `0b8561e2b1da` | file.pick, file.read, file.save, storage.read, storage.write | summarize_csv, export_report | oui | oui | oui |

## Lecture

- **Mêmes valeurs** : les 9 vecteurs de tests/integration/five-tools-vectors.json sont exécutés par le test Web (Playwright, `five-tools.spec.ts`, Chromium via Chrome système) et par le test Wasmtime Windows (`five_tools_produce_the_shared_web_vectors_on_the_native_wasmtime_host`). Les deux comparent les sorties exactes ; 9/9 réussis des deux côtés.
- **Même binaire guest** : les deux hôtes chargent la même archive signée (même SHA paquet vérifié par le descripteur) ; le SHA du `module.wasm` est calculé depuis cette archive. Le bureau ne recompile pas l'outil.
- **Permissions** : les trois outils sans permission (calculatrice, convertisseur de casse, dates UTC) ne déclenchent aucun dialogue de consentement après exécution de toutes leurs actions (contrôle Web). Le calculateur de hash et l'explorateur CSV ne demandent leurs permissions que sur clic.
- **Aucun contournement par identifiant d'outil** : recherche des cinq identifiants dans crates/kortex-core, kortex-runtime-native, kortex-ui, kortex-capabilities, kortex-abi, kortex-sdk, kortex-verify, kortex-pack, runtime, web-adapter/src, api/src, database/migrations et shared : aucun résultat.

## Clôture du 28/10/2026

- Les cinq paquets sont publiés et démarrent depuis le site de staging réel ; les neuf vecteurs partagés passent aussi sur Cloudflare.
- Le rapport CSV est réellement téléchargé dans le sandbox après consentement, et son contenu est identique à l’action export_report. Le site a été redéployé uniquement pour autoriser cette capacité générique.
- Un profil Chrome temporaire avec permission navigateur persistent-storage accordée par CDP utilise le vrai IndexedDB : mémorisation, rechargement et relecture du nom du fichier. Aucune API de stockage n’est simulée. Le refus utilisateur ne révèle aucun nom ; un contexte sans persistance navigateur est refusé sans fausse confirmation.
- La garantie du stockage reste conditionnelle à la permission du navigateur ; ni mode privé ni données évictables ne sont présentés comme durables. Il n’existe pas de détecteur fiable de navigation privée.
- WebKit n’a pas été rejoué dans ce lot. Les dialogues natifs Windows positifs de sauvegarde/sélection restent hors automatisation ; le backend réel, les refus et le stockage après redémarrage sont couverts dans Wasmtime. La calculatrice dispose des preuves WebView2 des phases 7 à 9.
- Les identifiants des outils sont absents du code hôte/catalogue/API ; la fixture contractuelle de publication de la calculatrice reste intentionnellement un exemple.
- Aucun DNS, domaine, environnement de production ou routage de messagerie modifié. Phase 12.6 non engagée.
