# Modèle de menace V1 — 18.0

28/10/2026. Périmètre exécuté : Windows et Chromium, site/API/runtime de staging. Document technique soumis à revue du propriétaire ; aucune approbation humaine ni certification n’est présumée. Référence de méthode : [OWASP ASVS 5.0](https://asvs.owasp.org/), sans revendication de conformité à tous ses niveaux.

## Actifs et acteurs

Actifs : fichiers choisis, stockage d’outil, résultats, identité et sessions, définitions de parcours, clés de publication et intégrité des paquets. Adversaires : guest hostile même correctement signé, auteur compromis, archive modifiée, iframe étrangère, contenu XSS, proposition IA injectée, compte tiers, porteur d’un token volé et appelant des routes opérateur. Un administrateur Cloudflare ou un binaire hôte déjà compromis possède les secrets du système : ce sont des frontières de confiance, pas des attaques prétendument contenues par le guest.

## Frontières de confiance

- Les octets R2 et leurs métadonnées sont des entrées : taille, SHA-256, signature, identité/version, ZIP et ABI sont vérifiés avant exécution. R2 ne calcule rien ; D1 contient catalogue et données de compte autorisées, jamais un exécuteur guest.
- Le guest WASM est non fiable. Pas de WASI, DOM, chemin de fichier ou réseau libre. Worker navigateur / Wasmtime Windows distincts ; seuls les services hôte interprètent les demandes de capacités. Le renderer Dioxus et les courtiers font partie de l’hôte de confiance.
- Site et runtime Web ont des origines distinctes. Les iframes partagent l’origine runtime, ce ne sont pas des origines différentes par outil. Le guest n’obtient pas leur JavaScript : il tourne dans son Worker. Les messages parent exigent source/origine/nonce/session/requête ; pas de pont direct entre outils.
- Une capacité vient du manifeste chargé et d’un contexte hôte : outil/version/instance/ressource/expiration. Une approbation de parcours ou d’IA n’accorde pas une permission de fichier. Les gestes sensibles sont frais, les droits révocables et les handles opaques.
- L’IA et ses descriptions d’outils sont non fiables. Le service propose un plan borné ; l’hôte vérifie le contrat et attend le consentement. Aucun exécuteur cloud ; les résultats locaux ne sont pas retournés au modèle.
- Cookies navigateur : Secure/HttpOnly/host-only, session serveur et preuve CSRF liée à la session pour mutation. Les tokens Desktop ont leurs scopes, propriétaire et révocation ; la synchronisation est opt-in. Un sélecteur de compte fourni par le client ne choisit pas le propriétaire SQL.
- Administration : JWT signé, émetteur/audience/identité du dépôt et droits vérifiés, requêtes idempotentes et versions immuables. Une clé d’auteur valide n’autorise ni le déploiement de l’hôte ni une capacité non déclarée.

## Priorisation et constats

Critique : exécution native arbitraire, accès inter-compte, publication sans autorité, fichier/réseau sans permission. Majeure : indisponibilité de l’hôte, DNS vers réseau privé, dépassement d’un budget, fuite de secrets. Mineure : dette de maintenance sans chemin démontré dans le périmètre, défaut ergonomique sans élévation de droits. Cette classification locale n’altère pas la sévérité publiée par les auteurs des dépendances.

Constats corrigés dans ce lot : tables WASM sans maximum ou au-delà de 65 536 entrées refusées avant instanciation ; vérification d’adresses publiques également au backend réseau Windows ; requêtes Web vers noms DNS refusées UNSUPPORTED, car l’API du navigateur ne permet pas de contrôler/réutiliser les adresses résolues. Seules des IPv4 publiques littérales explicitement autorisées peuvent passer ce courtier ; la CSP du runtime peut encore les refuser. Redirections Web jamais suivies, cookies réseau omis. Windows résout, refuse toute réponse privée/mixte, fixe les adresses et recontrôle chaque redirection.

Zod 4.6.6, Undici 7.30.1 et ZIP 9.0.0 remplacent les versions signalées. Le writer ZIP fixe explicitement le système Unix et les permissions 0644 afin de garder les mêmes octets canoniques sur Windows ; le corpus historique de 10 000 mutations reste inchangé. Les anciens paquets ne sont pas réécrits. Les fixtures de calculatrice V1 sont figées au lieu de suivre le manifeste d’une nouvelle publication.

Budgets : mémoire linéaire guest 64 Mio, message 1 Mio, file d’attente 128, invocation au plus 30 s, fichiers 10 Mio et blocs de 256 Kio, stockage 5 Mio par outil/profil, bureau huit fenêtres. Ces limites ne bornent pas toute la RAM du navigateur, de la compilation ou du processus. Les corpus finis ne constituent pas une preuve de sécurité de tous les modules possibles.

Matrice et preuves : tests/security/threat-matrix.md et docs/evidence/security-*. Les exceptions restent visibles dans SECURITY_EXCEPTIONS.md, sans suppression d’avis du scanner. La validation Google/OIDC réelle est bloquée par les clients absents ; le protocole est testé avec un fournisseur local contrôlé, sans présenter cela comme une connexion Google réelle.

## Publication et suite

Seul le staging propriétaire reçoit les correctifs. Production, DNS, messagerie, secrets, clés de confiance actives et quotas IA conservés. Aucun scan de réseau public ; transport TLS local et réponses DNS contrôlées dans les tests. Phase 18.3 non engagée. Une promotion en production exige la revue humaine et la résolution ou acceptation explicite des gates ouverts.
