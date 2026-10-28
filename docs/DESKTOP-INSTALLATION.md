# Installation et cycle de vie Desktop — 13.2 à 13.4

Le Store installe désormais les vrais paquets signés. L’application Windows propose Installer, une bibliothèque versionnée, la désinstallation, les mises à jour et le retour à la version précédente. Le mode hors connexion peut être choisi dans la barre latérale ; les versions déjà connues comme retirées ou révoquées restent bloquées.

## Vérifications et installation

Le consentement porte sur une fiche et une version précises. Fiche/actions et métadonnées de sécurité sont relues sans cache avant les opérations critiques ; toute différence impose de consulter la fiche de nouveau. Les permissions affichées doivent être confirmées, mais elles ne sont jamais accordées automatiquement au moteur.

Le téléchargement HTTPS reste sur l’origine API autorisée, sans redirection, avec connexion bornée, délai de 60 secondes, limite de 25 Mio et blocs de 64 Kio. Les transferts interrompus ou annulés sont nettoyés et un nouvel essai repart proprement. L’espace libre est contrôlé et les erreurs d’écriture sont propagées.

Le paquet est vérifié via les composants Rust existants : SHA-256, signature Ed25519, racine et snapshot de confiance, archive, manifeste/contrat, ABI et version minimale de Kortex. Aucun fichier ZIP n’est extrait sur disque. La version complète est déplacée dans installed/<id>/<version> sur le même volume ; elle ne devient active qu’après écriture atomique de library.json. Les répertoires de version ne sont pas écrasés.

Les clés par défaut du canal staging sont des bundles publics de confiance épinglés par l’hôte, conservés avec leur expiration et signature. Aucune clé privée n’est livrée. Une configuration hôte explicite peut fournir le bundle ou registre public ; le canal production n’accepte aucune confiance de test par défaut. Un outil ne peut jamais fournir sa propre racine. Le bundle transmis au moteur est scellé par son empreinte pour empêcher une substitution entre vérification et chargement.

## Bibliothèque, données et retour arrière

L’index JSON atomique garde version courante, précédente, versions archivées, blocages et transaction de premier démarrage. Un index géré manquant est refusé, sans réactivation de fichiers orphelins. Le stockage d’un profil reste dans tools/<id>/<empreinte-du-profil>.json. La désinstallation demande confirmation et conserve ces données par défaut. L’effacement explicite ne vise que le profil local ; les autres profils et outils sont conservés.

Verrou d’index, baux de lancement et verrou du profil empêchent une suppression pendant l’utilisation. Les chemins et points de réanalyse Windows sont contrôlés. Les suppressions récursives sont résolues et limitées au dossier géré attendu. Les fichiers retirés sont d’abord déplacés dans une transaction de récupération ; si la modification de l’index échoue, ils sont remis en place.

Une mise à jour compare la précédence SemVer (sans confondre les métadonnées de build avec une nouvelle version), la compatibilité et les permissions supplémentaires. Elle conserve l’ancienne archive et sauvegarde les données du profil avant le contrôle du nouveau moteur. Échec d’initialisation ou de contrôle : ancienne version conservée et données restaurées. Le premier vrai démarrage confirme la transaction ; un échec ou une version candidate corrompue restaure l’ancienne version. Le retour arrière manuel reste disponible ensuite.

Le protocole V1 n’expose pas de migration arbitraire de stockage : le format de stockage reste V1. Une version qui réclame une capacité sensible au démarrage pendant le contrôle est refusée, sans accord automatique. Le point de contrôle transactionnel et la restauration après échec de migration sont éprouvés par injection d’une migration partielle dans les tests ; aucune nouvelle commande publique de migration n’est inventée.

## Politique de lancement

En ligne : vérification locale complète et consultation de la disponibilité distante ; un retrait/révocation confirmé est enregistré localement. Hors connexion explicite : seules les archives localement vérifiées, non bloquées et couvertes par une confiance encore valide sont lancées. Le délai de validité du snapshot signé est respecté ; aucun délai de grâce non signé n’est ajouté. La politique complète de révocation hors ligne prévue en 18.3 reste une étape ultérieure.

## Reproduction

cargo test -p kortex-desktop -p kortex-xtask --locked --offline -- --test-threads=1

cargo build -p kortex-desktop --locked --offline

node scripts/test-desktop-lifecycle.mjs

Le test Windows prépare trois versions temporaires signées, utilise le vrai catalogue Cloudflare et les vrais téléchargements, vérifie les confirmations, le lancement, le mode hors connexion, la mise à jour, le retour arrière, une initialisation réellement défectueuse et les deux choix de désinstallation. Il révoque ensuite les versions et la clé, supprime les objets temporaires et restaure la politique opérateur. Les audits append-only sont conservés.

Correction du constructeur : les workspaces externes ont un target Cargo isolé par chemin canonique. Cela évite de réutiliser le fichier de sortie d’un autre workspace portant le même nom de crate.

Périmètre : Windows, Chromium/WebView2. Linux/macOS et Firefox exclus ; pas d’installateur Windows/Authenticode à cette étape. Tests et empreintes : docs/evidence/desktop-13.2-13.4-validation.json. Aucune modification DNS, production ou messagerie. 13.5 non engagée.
