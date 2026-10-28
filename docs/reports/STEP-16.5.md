# Étape 16.5 — Comparaison entre outils

28/10/2026. Base : `16de346`.

Scénarios prouvés avec les outils signés réels :

- **Positif typé** : `timestamp-utc.to_unix` → `calculatrice.add`, 1200 secondes puis 1210, mêmes résultats Web/Windows. Aucun code produit ne connaît ces deux IDs.
- **Refus typés** : calculatrice produit un `number`, timestamp attend un `integer` ; le sens demandé initialement est donc refusé sans coercition ajoutée au contrat gelé. Une action de statistiques de texte inexistante est également refusée. Ces limites sont verrouillées en Rust et en Chromium.
- **Fichier** : octets `abc`, consentement séparé, SHA-256 `ba7816bf8f01cfea414140de5dae2223b00361a396177a9cb410ff61f20015ad` identique sur WASM Web et Wasmtime Windows. Le Web mesure zéro lecture avant consentement puis une lecture, exige un geste de téléchargement et ne persiste ni nom ni octets du fichier. Le test Windows exécute le vrai adaptateur produit, NativeFiles et le paquet signé ; la définition originale reste vide et inchangée.

Les fixtures et IDs propres à ces cas restent dans les tests. L’ordonnanceur utilise exclusivement le contrat et l’invocation universelle.

**Clôture universelle non prononcée** : la partie de 16.3 concernant le dialogue des capacités déclarées dans le lanceur Desktop manque encore. Les trois scénarios du catalogue actuel passent, mais ils ne suffisent pas à annoncer la prise en charge universelle de futurs outils sensibles. Windows/Chromium uniquement selon les exclusions demandées.
