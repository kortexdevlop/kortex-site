# Préparation production — 22.0

**État : non prête.** Choix reçus du propriétaire : licence propriétaire et plafond mensuel **0 €**. Le fichier LICENSE reflète ce choix sans attribuer une identité juridique inventée ; les droits des composants tiers restent distincts. Aucun achat, souscription ou hausse automatique de forfait n'a été exécuté.

Inventaire réel en lecture seule : zone `kortex.site` active, plan de zone Free Website ; trois Workers, une DB et un bucket Kortex, tous staging. `infra/deploy/channels.json` contient encore `production: null`. Les huit enregistrements DNS, dont trois MX, sont préservés ; leur contenu n'est pas copié dans la preuve. L'accès aux abonnements du compte a été refusé (403) : le plan de zone ne prouve pas la facturation gratuite de tous les services.

Métadonnées d'identité vérifiées sur le Worker staging : Google désactivé, clients Web/Desktop et binding de secret Google absents. Aucun certificat de signature de code valide avec clé privée n'est disponible dans le magasin utilisateur Windows inspecté. Les environnements GitHub staging/production existent et sont protégés ; cela ne prouve pas que les secrets applicatifs et le certificat de production sont configurés.

Le dernier run observé 40914787626 sur 05dece0 a réussi contrats/sécurité et les contrôles locaux Web, mais a échoué sur un transport HTTP loopback Windows et trois tests staging pendant le blocage D1. Le transport des fixtures debug a été isolé des proxys/pools hérités ; dix tests installateur/update repassent localement. Le prochain passage CI complet reste nécessaire, sans réduction de couverture ni suppression des tests distants.

Le nouveau contrôleur `production-readiness.mjs` refuse configuration absente, stockage partagé, origines fictives, dépassement du budget autorisé, certificat/identité/manuels requis manquants, RC non gelée, drill distant incomplet et CI de mauvaise révision. `check-env --production` exige maintenant le vrai mode et les gates de production ; les placeholders `.invalid/.test/.example` ne passent plus pour des domaines de production. Les tests positifs sont des données synthétiques uniquement, pas une attestation de production.

Blocages exacts et non secrets : [production-readiness.json](../evidence/production-readiness.json). Checklist et ordre : [production-checklist](../../infra/production-checklist.md), [plan non exécutable](../../infra/production-release-plan.json). Contact sécurité confirmé : admin@kortex.site. Identité d’éditeur, revue juridique/licences, Google, signature Windows, ressources et secrets production, facturation gratuite vérifiée, restauration distante et alertes continues restent à confirmer. L'autorisation de préparer/déployer ne crée pas ces moyens matériels.

Aucun site, API ou installateur n'est annoncé en production sur la base d'un contrôle local. Les réserves du gate 20.5 restent applicables ; les formats de paquets, clés publiques et ABI ne sont pas affaiblis pour les lever.
