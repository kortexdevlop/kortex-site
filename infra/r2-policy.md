# Politique R2 — préparation 9.1

État : politique et invariants SQL vérifiés localement. Aucun bucket distant créé, aucune requête R2 distante ni publication effectuée. L'activation distante reste à 9.6.

| Environnement | Nom prévu, à confirmer avant création | Accès |
|---|---|---|
| local | kortex-tools-local | émulateur uniquement |
| staging | kortex-tools-staging | bucket privé distinct |
| production | kortex-tools-production | bucket privé distinct |

Le futur Worker API reçoit le binding `TOOLS_BUCKET`. Désactiver r2.dev et tout domaine public direct ; les lecteurs passent par la route de téléchargement autorisée. Aucun token R2 ni secret dans le navigateur, le paquet ou les métadonnées publiques. Les noms ci-dessus sont une proposition, pas la preuve d'un compte ou d'une ressource existante.

## Identité et écriture

La clé est exactement `tools/{tool_id}/{version}/{sha256}.tool`, SHA-256 minuscule du ZIP final. Le couple outil/version est immuable, même si un autre hash donnerait une autre clé. Toute divergence de hash est un conflit ; ne jamais remplacer le contenu ou le descripteur sous la même version. Utiliser un PUT conditionnel atomique avec `If-None-Match: *`, pas seulement HEAD puis PUT. Si l'objet existe, vérifier ses octets et sa taille avant d'accepter une répétition ; tout écart est un conflit. Les [conditions de l'API R2](https://developers.cloudflare.com/r2/api/workers/workers-api-reference/) sont documentées par Cloudflare ; leur intégration applicative sera testée à l'étape de publication.

Les métadonnées prévues sont content-type `application/zip`, identité/version, SHA et longueur. ETag ne doit jamais être assimilé à SHA-256. Avant de passer un job à validated : contrôler HEAD (taille/identité/ETag), relire le corps avec un plafond de 25 Mio, recalculer SHA-256 et vérifier descripteur Ed25519, schémas et ABI. Consigner ensemble taille HEAD, ETag, taille/hash relus et dates de vérification. Seul le service de publication de confiance écrit ces attestations ; une valeur fournie par le client n'est pas une preuve.

Le SQL empêche de publier sans ces attestations cohérentes et une clé active. Il ne peut pas effectuer HEAD/GET ni vérifier Ed25519 : les fixtures SQL sont des attestations synthétiques de test d'état, jamais une simulation permanente du service. L'API de publication et les contrôles R2 réels arrivent aux étapes suivantes.

## Publication, cache et rollback

Les états sont uploaded→validated→published, avec failed ou revoked en sortie des états admissibles ; published peut seulement aller à revoked. Failed/revoked sont terminaux. Une reprise après failed utilise un nouvel UUID et un nouveau job, sur les mêmes octets immuables. Un UUID répété avec la même empreinte de requête ne crée aucun job/événement ; une charge différente est un conflit. Le service doit calculer cette empreinte, jamais faire confiance à celle du client.

Après validation externe, effectuer dans une transaction D1 le passage du job à published, celui de tool_versions à published et le basculement de tools.published_version avec le journal. Les opérations R2 et D1 ne sont pas une transaction distribuée : conserver le staging/job en cas d'interruption et reprendre de façon idempotente. La route de téléchargement devra servir uniquement une version publiée et non révoquée.

Le cache interne des octets adressés par SHA peut être immutable ; les décisions d'autorisation, le pointeur courant et les révocations ne doivent pas être cachés comme immuables. Aucun header public long TTL ni CDN n'est activé ici. La politique de téléchargement définitive doit tester le retrait après révocation avant activation distante.

Un rollback légitime déplace le pointeur vers une ancienne version encore publiée, vérifiée et non révoquée. Il ne remet pas un job en uploaded, ne remplace pas les octets et ne réactive ni clé ni artefact révoqué. Avant une révocation, déplacer ou retirer tous les pointeurs courants concernés dans la même transaction, puis écrire revocations ; les triggers révoquent jobs/versions et, si demandé, la clé. Le journal et les tombstones sont append-only.

## Rétention

Conserver les artefacts publiés et les descripteurs sans expiration automatique ; conserver le journal et les tombstones de révocation durablement. Ne pas appliquer de règle lifecycle générale sur le préfixe tools/. Un objet orphelin ou un upload échoué peut être candidat au nettoyage après sept jours, uniquement après vérification des références D1, absence de job actif/publié et revue opérateur tracée. Un objet révoqué reste privé et non servi ; une éventuelle suppression d'urgence exige une décision opérateur documentée, sans effacer son identité ni sa révocation. Aucun nettoyeur ni lifecycle distant n'est installé à cette étape.
