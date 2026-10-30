# Récupération et réponse aux incidents

Périmètre : Windows x64 / Chromium-WebView2, budget mensuel autorisé **0 €**. Les objectifs du catalogue sont RPO ≤24 h et RTO ≤4 h ; ils ne sont pas des garanties mesurées en production. Le rapport [DISASTER_DRILL](reports/DISASTER_DRILL.md) distingue l'essai local de la restauration Cloudflare encore bloquée par le quota gratuit.

## Préparer avant l'incident

Conserver l'inventaire non secret des environnements, les versions Worker/Runtime/site, les manifests et SHA des paquets, les documents publics de confiance et révocation, les bookmarks D1 et les reçus de publication. La racine privée hors ligne et les clés CI restent dans leurs emplacements protégés, pas dans ces rapports. Les clés du système QA ne sont pas une sauvegarde des identités de production.

Les objets R2 sont adressés par hash et les opérations de publication sont journalisées. Cela n'est pas une sauvegarde indépendante : un administrateur du compte peut supprimer objets et journaux. Prévoir une copie hors de la zone de panne et vérifier ses hashes. Rétention et fréquence opérationnelles doivent être choisies et réellement activées avant la production ; une procédure seule ne les active pas.

D1 Time Travel est disponible sur le moteur `version: production`, y compris au palier gratuit. La fenêtre documentée est **7 jours en Free**, 30 en Paid ; la page générale parlant de 30 jours ne doit pas faire oublier cette différence. Source : [limites D1](https://developers.cloudflare.com/d1/platform/limits/) et [Time Travel](https://developers.cloudflare.com/d1/reference/time-travel/), vérifiées le 29/10/2026. Ne pas changer de forfait avec le plafond de 0 €.

## Qualifier et contenir

1. Noter heure UTC, symptômes, versions et identifiants de trace sans contenu privé. Distinguer panne, dépassement de quota, erreur de configuration et compromission.
2. Identifier précisément environnement/compte/DB/bucket. Suspendre les publications, migrations et modifications de compte pendant une restauration. Éviter toute action sur le DNS/mail pour résoudre une panne applicative.
3. En cas de compromission, retirer les accès suspects avant de rouvrir les écritures. Conserver une copie expurgée des indices et journaux d'audit ; ne pas réinjecter une ancienne clé compromise.
4. Choisir rollback de code, correctif compensatoire ou restauration de données. Un rollback Worker ne restaure pas D1. Une restauration D1 ne restaure pas R2, les secrets, les révocations déjà connues des clients ou un canal monotone.

## Procédures

| Incident | Procédure |
|---|---|
| Fiche, migration ou base corrompue | [Restaurer D1](runbooks/restore-d1.md) |
| Outil ou version compromis | [Retirer un outil](runbooks/revoke-tool.md) |
| Clé de signature ou identité CI compromise | [Rotation et révocation](runbooks/rotate-keys.md) |
| Cloud, IA, quota ou perte de zone | [Panne du cloud](runbooks/cloud-outage.md) |

Pour R2 : retrouver une copie dont les octets correspondent au SHA du descripteur signé ; vérifier identité/version/signature avec une confiance actuellement autorisée, rétablir uniquement la clé d'objet attendue, puis vérifier HEAD, téléchargement complet et exécution. Si les octets d'origine manquent, ne pas fabriquer un paquet sous le même hash/version : nouvelle version, nouvelle signature et nouvel état de publication sont nécessaires. Une ancienne signature ne réhabilite pas une clé révoquée.

Pour une compromission OIDC : suspendre le workflow ou client concerné, retirer les accès et secrets exposés, vérifier issuer/audience/repository ID/ref/workflow des règles de publication, révoquer les sessions concernées, puis effectuer un vrai parcours de connexion avec les nouveaux paramètres. Le fournisseur Google utilisateur et OIDC GitHub du publisher sont deux frontières distinctes.

## Vérifier et communiquer

Vérifier schéma et données récupérés, hashes des paquets, endpoint API catalogue/détail, SSR/sitemap, cadre Runtime et lancement signé, puis identité, synchronisation et parcours. Consigner perte nette de données et chronologie RPO/RTO. Garder l'état précédant la restauration récupérable jusqu'à la clôture de l'incident.

La communication publique relève du propriétaire désigné : publier seulement service affecté, plage horaire, fonctions disponibles, action attendue et heure de prochaine mise à jour réellement tenue. Ne pas publier emails, traces personnelles, secrets ou détails exploitables. Aucun message public ni notification n'est envoyé automatiquement par ce dépôt ; contact sécurité et astreinte restent à désigner.

Clôture : cause, périmètre, preuve de restauration, pertes, retrait des ressources de test, correctif, revue des droits et suivi. Une simulation locale ne suffit pas à lever le gate de restauration distante de V1.

Actualisation : contact sécurité confirmé admin@kortex.site, éditeur Tiago Antunes. L’astreinte permanente reste non configurée. Pour le drill cloud, préparer une fixture fraîche locale dans un nouveau dossier target, puis fournir son release-report.json par KORTEX_DRILL_FIXTURE_REPORT ; une fixture expirée doit être refusée avant création réseau. Le drill n’utilise aucun import synthétique massif.
