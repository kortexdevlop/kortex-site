# Cinq outils sur l’hôte bureau Windows

Vérification automatisée et manuelle de l’étape 12.5. Hôte : `kortex-runtime-desktop` (Dioxus + Wasmtime, WebView2). Linux, macOS et Firefox sont exclus du périmètre.

## Automatisé

```powershell
node scripts/prepare-phase12.mjs
cargo test -p kortex-runtime-native --test phase12 --locked --offline
cargo build -p kortex-runtime-desktop --locked --offline
pwsh -File tests/tools/phase12-desktop.ps1
```

- `five_tools_produce_the_shared_web_vectors_on_the_native_wasmtime_host` charge les cinq paquets signés (calculatrice depuis `target/staging/calculatrice`, les quatre autres depuis `target/phase12`) et compare les 9 vecteurs de `tests/integration/five-tools-vectors.json` aux sorties du guest dans Wasmtime. Ces vecteurs sont ceux du test Web `tests/integration/five-tools.spec.ts`.
- `real_windows_csv_stream_*` lit un vrai fichier CSV de plus de 800 Kio en blocs de 256 Kio avec handles et permissions réels, vérifie l’aperçu paginé, refuse `file.save`, écrit puis relit `prefs/recent` après redémarrage de l’outil et vérifie le refus de lecture du stockage.
- `phase12-desktop.ps1` lance chaque paquet dans WebView2, exécute une action depuis le panneau d’actions et teste le refus de permission. Pour l’explorateur CSV : `summarize_csv` puis refus de `file.pick`.

## Manuel (dialogues natifs)

Les dialogues Windows de sélection et d’enregistrement ne sont pas automatisés.
1. Ouvrir le paquet `explorateur-csv` ; choisir un CSV UTF-8 valide : l’aperçu affiche 50 lignes, la recherche filtre, l’onglet Graphique montre les barres.
2. Choisir un CSV avec un guillemet non fermé : le message `UNTERMINATED_QUOTE` s’affiche et aucune table ne reste.
3. « Exporter le rapport » : accepter, vérifier `rapport-csv.csv` ; recommencer en refusant : « Autorisation refusée » et aucun fichier.
4. « Mémoriser ce fichier », fermer l’outil, le rouvrir, « Afficher les fichiers récents » : le nom revient.
5. Pour la calculatrice, le convertisseur de casse et les dates UTC : aucune fenêtre de consentement ne doit apparaître.
