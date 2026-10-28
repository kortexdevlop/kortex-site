# Bureau Web V1 — gate 17.1 à 17.3

28 octobre 2026. Résultat technique : réussi sur Windows/Chromium, huit scénarios staging et 37 tests Web. Site : https://kortex-web-staging.userpgl.workers.dev/workspace .

- Huit iframes et moteurs signés indépendants ; la neuvième fenêtre est refusée. Fenêtre réduite/onglet caché reste chargé ; fermeture décharge son moteur et retire son canal. Aucun moteur restant après les huit fermetures.
- Souris et clavier, ordre z et focus, dock, mobile par onglets, contrôles nommés et focus visible. La restauration garde les identifiants logiques, recrée les moteurs et ne restaure aucun droit ou contenu.
- Contrôle central uniquement : une fenêtre ne peut pas commander sa voisine ; source, origine, nonce, session et requête liés. L’outil WebAssembly tourne dans son Worker ; le renderer et les courtiers hôte de confiance ne font pas partie du code guest. Les iframes utilisent la même origine runtime dédiée, distincte du site ; elles ne prétendent pas fournir une origine navigateur différente pour chaque outil.
- Deux lectures réelles concurrentes, permissions séparées, handle étranger refusé, consentement expiré redemandé. Contrôle de temps/retard utilisé seulement par le test et aucune réponse métier fabriquée.
- Résultats et données fichiers restent en mémoire locale. Historique : date/outil/action/état seulement, cent entrées. Layout strict sans contenu ; pas de sync cloud du bureau.
- Annulation explicite avant consentement, fenêtre fermée durant une requête et prochaine fenêtre fermée : aucune continuation silencieuse. Effets déjà réalisés ne sont pas annulés rétroactivement.
- Hors connexion : une fenêtre déjà chargée et son contrat en cache mémoire restent utilisables. L’ouverture d’un nouvel outil demande une connexion. Un reload hors ligne n’est pas supporté ; aucune PWA ou cache permanent des paquets n’est revendiqué.
- Axe : zéro critique ; écart sérieux color-contrast toujours visible dans desktop-isolation.json. Palette imposée de l’accueil conservée. Essais automatisés de noms/focus/clavier et inspection des captures ; pas de certification AA ni de lecteur d’écran humain revendiqué.

## Différences navigateur / application Windows

Le bureau Web V1 reste une page à fenêtres, remplacées par des onglets sur mobile, et ne simule pas un système d’exploitation. Les fichiers sont des handles temporaires du navigateur, le stockage est IndexedDB, les accès réseau exigent HTTPS/allowlist/CORS et les gestes sensibles restent frais. La présence de l’application locale n’est jamais inventée. Le lanceur Windows existant utilise ses services natifs/Wasmtime et ses dialogues ; cette étape ne lui ajoute pas un bureau natif ni une fonction App Store. Linux/macOS/Firefox exclus suivant la demande du propriétaire.

## Publication et preuves

Le premier lancement de calculatrice 1.0.0 a correctement refusé sa clé expirée. Version additive 1.0.1 reconstruite/signée/vérifiée puis publiée dans le staging propriétaire ; anciennes versions immuables préservées. Nouvelle racine/clé publique staging valable sept jours, seed privée seulement en mémoire. Documents publics desktop-calculator-trust.json et desktop-calculator-descriptor.json, publication et restauration de la politique dans desktop-calculator-staging.json. Les clés de test doivent être renouvelées avant leur expiration ; aucune clé de production n’est promise par ce gate.

Build SSR et contrôle Astro 92 fichiers sans erreur/avertissement. Captures desktop-windows.png / desktop-mobile.png, contrôles desktop-ui.json / desktop-router.json / desktop-files.json / desktop-isolation.json, logs desktop-playwright.log / desktop-web-unit.log et empreintes desktop-validation.json. Contrôles de sources : SHA-256 UTF-8 LF ; captures : octets binaires.

Production, DNS, messagerie, secrets et quotas IA préservés ; fournisseur Google 14.5 toujours en attente de ses clients. Phase 18 non commencée. Clôture technique seulement, sans inventer une approbation humaine visuelle.
