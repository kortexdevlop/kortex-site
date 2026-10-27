# API locale Kortex

La santé répond sur `/api/v1/health`. Le catalogue V1 expose GET/HEAD :

- `/api/v1/tools?limit=20&sort=recent&q=calcul&category=calculation&tag=math`
- `/api/v1/tools/:id`
- `/api/v1/tools/:id/actions`

La liste renvoie `{items,next_cursor}`. Limite entière 1–100, défaut 20 ; tri `recent` (défaut, published_at DESC puis id DESC) ou `name` (name_norm ASC puis id ASC, collation SQLite BINARY). `category` est un slug exact ; `tag` est un tag exact unique de 1–40 caractères. `q` est borné à 200 caractères avant/après normalisation NFKC, espaces externes retirés, casse abaissée. La recherche est littérale dans name_norm et le résumé ; `%` et `_` ne sont pas des jokers. name_norm doit être normalisé au stockage par le futur producteur. La mise en minuscules SQLite du résumé reste celle de SQLite (ASCII) ; aucune recherche linguistique ou tolérance aux accents n'est annoncée. Paramètres inconnus, répétés, invalides ou non bornés : 400.

Pour continuer une page, renvoyer `cursor=...` avec les mêmes filtres, tri et limite. Le curseur est un jeton interne versionné HMAC-SHA256, base64url canonique, maximal 4096 caractères. Il n'est pas chiffré et ne contient aucun secret. Toute altération, version/forme inattendue ou réutilisation avec une autre requête est refusée. Le parcours est stable sur un ensemble figé ; il ne constitue pas un snapshot transactionnel entre requêtes lorsque le catalogue change.

Seuls les outils et versions courantes de statut published sont servis. Les actions proviennent du contrat immuable de cette même version, jamais d'un brouillon plus récent. Le détail inclut le descripteur public enregistré ; sa validation de forme et ses identités sont contrôlées, sans refaire une vérification réseau R2 ou cryptographique à chaque lecture. La publication vérifiée demeure la responsabilité des étapes 9.5 et suivantes. Les validateurs sont générés à la compilation depuis les schémas ; aucune compilation dynamique de schéma n'est exécutée dans le Worker.

Les réponses réussies ont un ETag SHA-256 et `Cache-Control: public, max-age=0, must-revalidate`. `If-None-Match` accepte aussi les ETags faibles et renvoie 304 sans corps si la représentation courante existe et est identique. Une dépublication est vérifiée avant le cache et renvoie 404. Erreurs : no-store. CORS reste limité à PUBLIC_WEB_ORIGIN ; If-None-Match est autorisé et ETag exposé.

## Configuration locale

Installer les dépendances, puis appliquer les migrations uniquement à D1 local :

```text
pnpm --filter @kortex/api exec wrangler d1 migrations apply DB --local --env=
```

Créer dans **api/.dev.vars**, fichier ignoré, `CURSOR_SIGNING_KEY=` suivi de 64 caractères hexadécimaux minuscules issus de 32 octets aléatoires. Garder cette clé côté serveur, distincte de toute clé de signature d'artefact. Aucun secret réel ni valeur par défaut n'est versionné. Ne pas remplacer un fichier de secrets existant. En staging/production, ce sera un secret Wrangler propre à chaque environnement au provisionnement, pas une variable PUBLIC ni une valeur dans wrangler.jsonc. Changer la clé invalide les curseurs en circulation ; aucune rotation multiclés n'est implémentée.

```text
pnpm --filter @kortex/api dev
pnpm --filter @kortex/api test
pnpm --filter @kortex/api typecheck
pnpm --filter @kortex/api build
```

La liste échoue volontairement en 503 sans clé valide, même vide ; la santé reste disponible. Une base migrée vide renvoie une liste vide, sans données artificielles de démarrage. Les 10 000 entrées et attestations synthétiques de tests n'entrent jamais dans la configuration déployée. Le build reste exclusivement un dry-run ; aucune ressource distante n'est provisionnée en 9.3. Le téléchargement R2 reste l'étape 9.4.

## Téléchargement et administration (9.4–9.5)

GET/HEAD `/api/v1/downloads/:id/:version` transmet l'objet privé publié et non révoqué. MIME `application/vnd.kortex.tool+zip`, headers digest/ETag et Link depuis le détail. Une version révoquée répond 410 même avec son ancien ETag. Les réponses exigent une revalidation D1, les erreurs sont no-store. SHA/signature restent vérifiés par l'hôte avant exécution.

Les POST d'administration sont réservés aux machines, sans prérequête CORS de publication depuis un navigateur :

- `/api/v1/admin/publications` : PublicationRequest existante, descripteur/manifest/contract/r2_key/request_id, ZIP déjà présent dans le bucket privé.
- `/api/v1/admin/tools/:id/rollback` : `{request_id,version,expected_current_version}`.
- `/api/v1/admin/artifacts/:sha256/revoke` : `{request_id,reason}`.

Corps JSON fermé maximal 256 Kio, UUID idempotent. Réponse PublicationStatus ; pour un rejeu, la réponse initiale est renvoyée sans nouveau basculement. Auth manquante/invalide : 401, action interdite : 403, forme invalide : 400, trop gros : 413, échec d'intégrité : 422, conflit/UUID réutilisé différemment : 409. Pas de fuite SQL ni de clé dans les erreurs.

`ADMIN_AUTH_POLICY` est une chaîne JSON serveur fermée : issuer, audience, subject, repository, repository_id, ref, workflow_ref et actions (`publish`, `revoke`, `rollback`). GitHub exige issuer `https://token.actions.githubusercontent.com` et RS256/JWKS officiel. `urn:kortex:staging-operator` accepte Ed25519 via jwks épinglé uniquement en local/staging. Aucun mode partagé par mot de passe ou jeton permanent ; les JWT exigent iat/exp/jti, durée maximale dix minutes. La politique absente rend l'administration indisponible. Les clés d'artefacts sont installées hors de cet endpoint depuis une configuration opérateur de confiance, jamais depuis le body de publication.

Les migrations locales incluent maintenant 0003_admin_operations.sql. Les contrôles avant commit peuvent matérialiser un ZIP de publication borné à 25 Mio pour l'inspection ; cela ne concerne pas le streaming des téléchargements. La mise en staging 9.6 doit être observée séparément du succès de ces tests locaux.
