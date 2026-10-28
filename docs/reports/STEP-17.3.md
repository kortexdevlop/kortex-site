# Étape 17.3 — Confidentialité et ergonomie du bureau

28/10/2026, base 10da685. Terminée et close techniquement dans le périmètre Windows/Chromium demandé.

Huit scénarios réels staging passent, plus 37 tests Web de régression. Huit fenêtres et huit moteurs actifs ; neuvième ouverture désactivée ; toutes fermables au dock ; zéro moteur restant après fermeture. Compteurs DOM avant/pendant/après et collecte mémoire consignés, sans assimiler une observation à une preuve générale d’absence de fuite.

Deux outils de hachage signés lisent simultanément les vrais fichiers abc/def après deux consentements distincts et produisent leurs vrais SHA-256. Le test retarde uniquement la lecture Blob puis appelle l’API native originale. Un handle d’A présenté au courtier B est rejeté INVALID_HANDLE, même avec accord de B. Une avance contrôlée de l’horloge de cinq minutes fait redemander le droit ; refus PERMISSION. Aucun contenu ou nom de ces fichiers dans le stockage parent.

Rechargement/reconnexion, outils déjà chargés hors connexion, fermeture pendant invocation et annulation du parcours couverts. Clavier, focus, titres/régions nommés, annonces et onglets ; axe WCAG A/AA avec zéro violation critique, violation sérieuse color-contrast exposée et conservée dans la preuve. Ceci ne certifie ni AA globale ni un essai manuel avec lecteur d’écran. Captures 1440×1100 et 390×844 inspectées ; débordement contrôlé à 320×740.

Voir docs/evidence/workspace-gate.md, desktop-isolation.json, desktop-files.json, desktop-router.json, desktop-validation.json et logs. Le mode hors connexion ne concerne que les moteurs déjà chargés : pas de PWA ni de démarrage Web à froid hors ligne. Les autorisations sont redemandées après reload. Différences Web/Windows documentées ; aucun essai Linux/macOS/Firefox.

Staging Web déployé ; calculatrice staging renouvelée avec nouvelle signature vérifiée, API et runtime staging redéployés pour sa publication en conservant les variables actives et en restaurant la politique opérateur. Pas de production, DNS, messagerie ou quota IA modifié. Réserve Google 14.5 inchangée. Phase 18 non engagée ; statut humain formel laissé en attente.
