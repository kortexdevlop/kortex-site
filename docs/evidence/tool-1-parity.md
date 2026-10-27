# Parité de l’outil 1 — 2026-10-27

Référence : calculatrice 1.0.0, ABI/UI 1.0. Les empreintes exactes sont aussi vérifiées dans tests/golden/calculatrice-reference.json.

- Guest WASM : f095744f63b9594af61c02a6a515f2ad7ddab0ca3d5e7ccbcc9120a78f850c50
- Même archive .tool Web et Windows : fac58c7af499a828516f7d5ab04d7d0541faa0e6af8ad3aad3f8727aebca35d4
- Corpus de 13 échanges : 5c73355801ae1d56c1ad589999b0bdce1682ab8f88122b962ec7c09472af3b22

| Observation réelle | Chromium | WebKit | Windows/WebView2 + Wasmtime |
|---|---|---|---|
| Admission ZIP et signature de test | Réussie | Réussie | Réussie |
| Pavé 25 × 48, bouton activé avec Entrée | 1200 | 1200 | 1200 |
| action multiply(25,48) | 1200 | 1200 | 1200 |
| UI/action divide(25,0) | DIVIDE_BY_ZERO | DIVIDE_BY_ZERO | DIVIDE_BY_ZERO |
| Fermeture/réouverture sans ancien état | Réussie | Réussie | Réussie |
| Touches numériques ≥44 × 44 px | Réussie | Réussie | Réussie |
| Requêtes réseau externes pendant calcul | 0 | 0 | 0 |

Le transport virtuel local de Dioxus/WebView2 vers http://dioxus.index.html//__events est identifié et observé séparément dans le test Windows ; ce sont les événements hôte, pas des appels réseau du guest. Le module n’a aucun import Wasmtime et le manifeste déclare zéro permission. Cela ne prétend pas mesurer le trafic global du système Windows.

Le même fichier target/calculatrice-fixture/calculatrice.tool, sa signature détachée et sa clé publique éphémère servent aux parcours Web et natif. Le helper est réservé aux tests et ne persiste aucune clé privée. Les octets du paquet ne changent pas entre les deux parcours. Le test Node en vérifie aussi la signature et les hashes. Goldens complets exécutés sur SDK, Worker et moteur natif ; ZIP altéré refusé avant exécution native, protection Web hostile déjà couverte par la suite existante.

Captures inspectées : calculatrice-chromium.png, calculatrice-webkit.png et calculatrice-windows.png. Commandes et observation structurée : validation-7.1-7.3.json. Linux/macOS et Firefox exclus sur instruction du propriétaire. Les raccourcis globaux Entrée/Échap ne sont pas inclus ; voir le design et le gate.
