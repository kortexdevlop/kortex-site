# Inventaire technique des données — 18.2

État du 28/10/2026, staging. Ce document décrit le code et les essais ; il n’invente ni certification ni politique de sauvegardes du prestataire.

| Donnée / finalité | Emplacement | Durée et suppression |
|---|---|---|
| Email et hash de mot de passe / connexion | D1 ; email lisible, mot de passe seulement sous hash scrypt salé et pepper secret serveur | Jusqu’à suppression du compte. Email non vérifié ; pas de service de vérification/récupération par mail. |
| Session navigateur | Cookie Secure/HttpOnly/host-only, token haché HMAC dans D1 | Validité 7 jours maximum ; logout/révocation et purge du compte invalident l’accès. |
| Flux OAuth / formulaire mot de passe | État navigateur lié à un enregistrement serveur | Validité 10 minutes ; états expirés refusés, nettoyage opportuniste lors des requêtes. Google réel désactivé faute de clients. |
| Appareil Windows | Token dans le coffre Windows, hash/scopes/propriétaire côté serveur | Expiration serveur et révocation par appareil ou compte ; l’option IA est distincte et révocable. |
| Favoris, réglages et définitions de parcours | Local ; synchronisation D1 seulement après opt-in | Jusqu’à effacement local ou purge du compte. Conflits/CAS et tombstones ; contenu de fichiers/handles/chemins refusé dans les parcours synchronisés. Une constante métier textuelle peut néanmoins contenir des données personnelles. |
| Fichier choisi et résultat d’un outil | Handle opaque et mémoire locale de l’instance ; jamais upload automatique | Handle et droits temporaires, généralement 5 minutes ; fermer/recharger détruit l’instance. L’expiration du droit ne garantit pas l’effacement immédiat des octets déjà lus dans la mémoire guest. |
| Stockage d’un outil | IndexedDB Web ou stockage hôte Windows, namespace profil/outil, quota 5 Mio | Permission préalable ; l’outil peut mémoriser ses données autorisées. Pas de purge automatique de tout stockage à la fermeture ; effacement du namespace ou option explicite de désinstallation. |
| Disposition et historique du bureau | LocalStorage du site, aucune synchronisation du bureau | Layout de métadonnées seulement ; historique limité à 100 date/outil/action/état. Résultats du panneau/fenêtre en mémoire, non persistés. |
| Message à l’IA | Envoi volontaire vers l’API puis Workers AI ; catalogue d’actions public | Consentement explicite requis. Aucun fichier ou résultat local ajouté automatiquement. Filtrage de formes connues de secrets/PII, sans garantie de détection de tout contenu privé ; le propriétaire doit configurer la rétention du fournisseur. |
| Quotas IA et limites d’authentification | Compteurs agrégés/pseudonymisés D1 | Fenêtres logiques minute/15 min/heure/jour selon la limite ; enregistrements expirés nettoyés à une requête suivante. Pas de remise à zéro par cet audit. |
| Événements d’authentification et audit de capacités | Événements bornés, identifiants/catégories techniques ; audit de capacités local sans contenu fichier | Auth events : validité jusqu’à 7 jours, nettoyage opportuniste ; audit local borné. Ni mots de passe, bearer, prompts ou données de fichiers dans les logs applicatifs prévus/testés. |

## Export et effacement

L’export authentifié fournit les données du propriétaire, jamais les hashes de mot de passe/session, secrets ou données d’un autre compte. L’effacement exige CSRF, confirmation et révision CAS courante ; DELETE users entraîne les cascades documentées, révoque les sessions/appareils et invalide la répétition de la demande. Les tests gardent un deuxième compte pour vérifier qu’il est conservé. Les compteurs pseudonymisés du jour et le flux navigateur courant sont nettoyés par la route de purge ; aucune promesse d’effacement de toutes traces agrégées historiques ou sauvegardes Cloudflare n’est faite.

Le stockage local d’un navigateur ou d’une installation hors ligne n’est pas supprimé à distance par cette purge SQL. Le propriétaire doit effacer ces espaces depuis cet appareil. Rétention des sauvegardes et journaux Cloudflare : non vérifiée dans ce lot, gate opérateur avant production (SEC-OPS-18). Tests locaux à identités fictives et requêtes négatives staging ; aucun compte utilisateur réel supprimé.


## Supervision 21.0

Métadonnées serveur autorisées uniquement ; aucun corps, fichier, prompt, email, IP, cookie, token ou identifiant utilisateur journalisé par le nouveau module. Identifiant de trace serveur aléatoire et minute UTC ; familles de routes bornées, statuts et durées. Champs, limites des métriques et preuves : docs/reports/TELEMETRY.md. Paramètres staging préparés : sans persistence de logs, sans invocation automatique et sans traces. Aucune nouvelle collecte Desktop ou transmission locale.

Cible disponibilité 99,5 % mensuelle ; alerte 5xx >2 % sur cinq minutes avec au moins 100 requêtes. Évaluateur testé, mais collecte globale continue, destinataire/astreinte, rétention des agrégats et plafonds financiers restent à configurer avant production. Aucun message envoyé ni forfait modifié. Procédure : infra/alerts.md.
