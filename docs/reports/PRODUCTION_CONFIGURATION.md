# Configuration production séparée — préparation

Le lot prépare les ressources suivantes pour `kortex.site` : DB `kortex-catalog-production`, bucket privé `kortex-tools-production`, Workers API/site/Runtime distincts et origines `api.kortex.site`, `kortex.site`, `run.kortex.site`. Les noms sont un plan ; aucun identifiant de base fictif n’est inséré dans le canal de production.

`node scripts/production-setup.mjs --plan` produit un plan non secret sous target. `--apply` est explicitement bloqué avant tout appel réseau tant que le forfait gratuit n’est pas vérifié. Deux accès Cloudflare ont refusé la lecture des abonnements (403). La zone Free et le champ Workers `default_usage_model: standard` ne constituent pas une preuve de forfait Workers Free. La confirmation demandée au propriétaire reste nécessaire pour le plafond de 0 €.

Après cette vérification seulement, le script peut créer les stockages isolés, appliquer les douze migrations connues et produire les configurations API/Runtime/site. Les hashes de migration sont épinglés dans `infra/production-schema.json` : toute nouvelle migration ou modification exige une nouvelle revue avant réseau. Les réponses SQL sont comptées par le budget de reprise (2 000 lectures / 1 000 écritures / 70 appels). Le script ne supprime aucune ressource, refuse l’adoption d’une base/bucket non attesté et conserve un reçu en cas d’échec partiel. Il n’active aucun forfait ou service payant.

Cette préparation ne déploie pas de Worker, ne modifie pas le DNS ou les MX, n’ouvre pas de bucket public et ne copie ni secret ni racine de confiance du staging. Les secrets de production et les documents de confiance des paquets devront être créés séparément. Google reste désactivé, email/mot de passe prévu, IA désactivée initialement tant que son enveloppe gratuite n’est pas vérifiée. La signature Windows reportée ne désactive pas les signatures des paquets, canaux ou révocations.

## Alertes sans D1

Le workflow `production-health.yml` et `infra/production-monitor.json` préparent trois sondes statiques par heure, sur runner Windows, sans installation de dépendances ni secret. Les endpoints de santé n’exécutent aucune requête D1. Réponse inattendue, redirection ou erreur réseau font échouer le job ; la notification dépend des réglages GitHub existants du propriétaire. Aucun email ou webhook externe n’est envoyé par ce lot.

Le moniteur reste désactivé jusqu’au déploiement réel et à la vérification des domaines. Sa planification ne devient effective qu’après présence sur la branche par défaut. Ce contrôle de santé ne mesure pas le catalogue, la connexion ni le SLO réel : le collecteur d’erreurs agrégées et le destinataire opérationnel restent à activer séparément. `continuous_alerts_configured` n’est donc pas marqué vrai par la seule création du workflow.

## Ordre de poursuite

Vérifier le forfait → provisionner les stockages et leurs configurations → créer les secrets/confiances de production → tester les configurations → déployer sous les gates existants → configurer les trois domaines en conservant la messagerie → activer et tester les alertes. Le script de déploiement protégé conserve son refus production tant que les prérequis de livraison sont ouverts.

## Exécution réelle après confirmation du propriétaire

Workers Free confirmé explicitement. Création des stockages isolés et application des douze migrations réussies ; [reçu et vérification](../evidence/production-storage-setup.json). Base production vide, indépendante du staging : 331d46e7-009a-4e38-b5d2-1e262485dc92 ; bucket privé kortex-tools-production. Canal de production et api/wrangler.jsonc désormais renseignés. Les trois configurations sont générées sous target/production-setup avec les chemins réels. 626 lectures / 203 écritures D1, puis 12 lectures de vérification, zéro test de charge.

Cette exécution limitée ne souscrit aucun forfait, n’upload aucun objet R2 et ne déploie aucun Worker. La création du stockage vide est distincte de la vérification de facturation globale pour le trafic public : cette dernière reste ouverte, ainsi que secrets, confiance de paquets, DNS/TLS et activation des alertes. Le moniteur préparé reste désactivé.

La configuration réelle du canal production rend désormais son contexte CI lisible ; les fonctions de publication et déploiement conservent un refus explicite RELEASE_GATE_NOT_APPROVED avant tout accès réseau pour ce canal. Un test couvre ce refus avec un canal réellement configuré, indépendamment de l’ancien état production:null. Les protections CI/OIDC et les formats gelés ne sont pas affaiblis.
