# Kortex Desktop — étapes 13.0 et 13.1

**Mise à jour 13.2–13.4 :** installation depuis le Store, bibliothèque versionnée, désinstallation, mises à jour et retour arrière sont implémentés. Voir DESKTOP-INSTALLATION.md et les preuves associées. Les descriptions ci-dessous documentent l’état initial de 13.0–13.1.

L’application produit est desktop/, distincte du shell expérimental runtime/desktop. Elle réutilise son composant ToolSession et les moteurs kortex-runtime-native / kortex-ui : pas de second moteur ni de branche d’identifiant d’outil. Les paramètres de paquet, les clés publiques et le panneau JSON du shell expérimental sont masqués en mode produit. Un clic d’ouverture dans la bibliothèque démarre l’outil sans consentement global inutile ; le courtier demande séparément chaque capacité sensible.

## Lancer et vérifier

- cargo build -p kortex-desktop --locked --offline
- cargo test -p kortex-desktop -p kortex-xtask --locked --offline
- pwsh -NoProfile -File tests/desktop/product-smoke.ps1

Binaire Windows de développement : target/debug/kortex-desktop.exe. Le canal livré ici est le staging déjà configuré. Le Store lit le catalogue réel ; aucun outil de démonstration n’est injecté dans l’application. Les espaces Bibliothèque, Mises à jour et Compte sont accessibles ; le compte n’est pas nécessaire. Mises à jour et compte restent des états de navigation, sans prétendre implémenter leurs fonctions futures.

## Configuration hôte

KORTEX_DESKTOP_API_ORIGIN accepte uniquement l’origine HTTPS de staging existante ou api.kortex.site ; identifiants URL, chemins, query, fragments et autres origines sont refusés. Le canal est déduit de cette origine. Aucun service de production n’a été créé ou modifié.

Le dossier par défaut est %LOCALAPPDATA%/Kortex. KORTEX_DESKTOP_DATA_DIR permet au banc de test d’utiliser un dossier isolé sous target/. KORTEX_DESKTOP_OFFLINE=1 impose le mode hors connexion. KORTEX_DESKTOP_TRUST désigne exclusivement le bundle public de confiance fourni par l’hôte (racine épinglée, snapshot signé, version minimale). Il ne provient jamais d’un outil téléchargé. Aucun secret de signature n’est inclus dans l’application ; aucune confiance de production n’est inventée.

La bibliothèque lit installed/<id>/<version>/package.tool et package.tool.json, et vérifie taille, liens, SHA, signature, chaîne de confiance et identité avant affichage. Un paquet altéré reste désactivé. L’ouverture le vérifie à nouveau. Le test copie uniquement une fixture signée existante dans son dossier isolé ; il ne simule pas une installation utilisateur. Le téléchargement et l’installation atomique depuis le Store appartiennent à 13.2 et ne sont pas implémentés ici.

## Store et limites

Client HTTP typé pour /tools, /tools/:id et /tools/:id/actions ; les schémas shared sont compilés localement sans changer les contrats. Cohérence des identités, versions, empreintes et actions vérifiée. HTTPS, absence de redirection, connexion 5 s, requête 12 s, réponse 1 Mio. Aucun cookie ou jeton utilisateur transmis.

Cache mémoire des métadonnées : cinq minutes, 128 URL au plus, isolé par paramètres de recherche/tri/curseur. Le contrôleur refuse doublons, cycles de curseur et dépassement des 10 000 entrées parcourues ; les catégories reçues sont enrichies au fil des pages et conservées lors d’un changement de filtre. L’interface montre des libellés français pour la taxonomie connue.

Les tests Rust couvrent 10 000 entrées synthétiques dans le contrôleur, les requêtes HTTP locales réelles, le TTL, les entrées hostiles, 404/503, l’absence de réseau et un paquet signé altéré. Les essais Windows WebView2 utilisent le vrai catalogue Cloudflare, puis ouvrent et utilisent le convertisseur de casse signé sans réseau. Le compte n’est jamais simulé. Aucun benchmark de charge distante de 10 000 outils n’est revendiqué.

Périmètre : Windows ; Linux/macOS et Firefox exclus. WebKit non rejoué dans ce lot. Binaire de développement, sans installateur Windows ni signature Authenticode à cette étape. Preuves : docs/evidence/desktop-phase13-validation.json.
