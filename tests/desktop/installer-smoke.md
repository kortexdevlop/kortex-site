# Smoke installateur Windows

Exécuter `scripts/desktop-installer-smoke.ps1` avec l'installateur produit et un dossier de destination neuf sous target. Le script vérifie qu'aucune installation/raccourci Kortex existant ne sera écrasé ; installation `/S`, démarrage masqué, processus toujours vivant après quatre secondes, arrêt de ce processus uniquement, désinstallation silencieuse et absence du binaire installé.

Preuve réelle locale : `docs/evidence/pipeline-desktop-install.json`. Descripteur et checksum : `pipeline-desktop-release.json`. Le même scénario figure dans le workflow Windows. Les valeurs `visual_interactions:false` et `production_signed:false` sont intentionnelles : ne pas transformer un test de démarrage en preuve d'interface ou de signature de production.

Linux/macOS/Firefox exclus par le propriétaire ; pas de commandes de packaging supposées pour ces plateformes. Une livraison stable Windows nécessite un certificat Authenticode fourni et vérifié.
