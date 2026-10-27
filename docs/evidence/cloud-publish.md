# Démonstration de publication cloud — 27 octobre 2026

Résultat : roundtrip réel de staging et démarrage vérifié sur Web et Windows réussis. Ce rapport ne revendique ni livraison du site web, ni production, ni exécution GitHub Actions réelle. Les signatures et ancres du gate sont TEST et temporaires.

| Ressource / contrôle | Observation |
|---|---|
| API | https://kortex-api-staging.userpgl.workers.dev |
| Worker | kortex-api-staging |
| D1 | kortex-catalog-staging, UUID a406c1bf-cf12-4d1d-b8ff-6aa0955abb0a |
| R2 | kortex-tools-staging, privé, sans URL publique ni domaine direct |
| Migrations | 0001_catalog.sql, 0002_publication.sql, 0003_admin_operations.sql |
| Version saine finale | calculatrice 1.0.0 |
| Version de contrôle révoquée | calculatrice 1.1.0 |
| Octets téléchargés | 264 845, strictement identiques au ZIP canonique du pipeline |
| SHA-256 | 1d4c1a0967b583489f69921d80d1c062dbd57ecb0d9dabea8337575e4ab32787 |
| Hôtes | Chromium, WebKit et Windows/Wasmtime ; même archive et preuve détachée |

Séquence réellement exécutée :

1. Création de la base et du bucket dédiés, vérification de la confidentialité R2, migrations distantes et déploiement du Worker. Le bucket a été créé dans le tableau de bord après deux refus API/CLI 500/10001 ; aucune activation de plan ou achat n'a été effectué.
2. Préparation du paquet calculatrice par le CLI Rust générique et le pipeline existant. Dépôt par binding R2 distant, PUT conditionnel If-None-Match. Un deuxième dépôt divergent sous la même clé a été refusé, sans modifier l'objet.
3. POST admin non authentifié : 401. Publication authentifiée par JWT court de l'opérateur staging : 200. Les contrôles de signature/hash/archive/ABI s'exécutent dans le Worker réellement déployé ; D1 fournit le catalogue courant.
4. Publication d'une seconde version signée de staging, lecture du nouveau pointeur, POST rollback vers 1.0.0 avec expected_current_version, puis révocation de 1.1.0. L'ancien téléchargement renvoie 410 ; 1.0.0 demeure visible et téléchargeable en 200. Aucun DELETE/REPLACE de version ni réactivation de tombstone.
5. Téléchargement HTTPS depuis R2 via l'API ; empreinte recalculée et comparaison octet par octet au paquet du pipeline. GET détail fournit exactement le descripteur utilisé par les hôtes.
6. Les tests Web retéléchargent les octets distants, puis les servent tels quels au shell local pour l'admission normale (racine épinglée, snapshot signé, SHA, signature, ZIP, ABI). Chromium et WebKit démarrent la calculatrice et exécutent multiply(25,48)=1200. Aucun site staging n'est inventé : son UI arrivera aux phases suivantes.
7. Windows ouvre le fichier effectivement téléchargé via verify_package_with_trust/Wasmtime et le shell habituel. Pavé numérique et action retournent 1200 ; DIVIDE_BY_ZERO, annulation, fermeture/réouverture passent. Le processus de test est arrêté proprement.

L'opérateur temporaire est autorisé par la session Cloudflare préexistante et une clé publique de JWT épinglée par le déploiement. Sa clé privée n'est pas persistée, ses JWT durent cinq minutes et ce mode est interdit en production. Les permissions/claims exacts de GitHub sont implémentés mais ne sont pas présentés comme éprouvés par un runner GitHub réel. Les clés d'artefacts sont séparées du JWT ; les hôtes reçoivent leur racine explicitement par le banc d'essai opérateur.

Incidents corrigés et conservés comme historique : rejet du flag --profile par whoami (seul profil actif accepté), création R2 API/CLI refusée puis succès UI, reconstruction native relancée sur le cache D: déjà prévu, horloge locale environ six secondes en avance affectant not_before d'une clé de fixture, et refus d'écraser une preuve existante (sortie par clé désormais distincte). Les contrôles de sécurité ont été conservés ; le dernier cycle complet et les deux hôtes passent.

Preuve structurée expurgée : validation-9.4-9.6.json (déploiement courant vérifié après installation du secret, migrations, états, logs et SHA-256). Captures inspectées visuellement : staging9-chromium.png, staging9-webkit.png et staging9-windows.png. Les captures du tableau de bord restent sous target/staging-cloud car elles affichent l'identité du compte. Aucun token, email de compte ou clé privée dans les preuves versionnées. Domaine, DNS et messagerie inchangés ; aucune ressource de production utilisée.
