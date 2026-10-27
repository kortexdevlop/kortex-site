# Reprise Cloudflare — point d'arrêt demandé, 27 octobre 2026

## État vérifié

9.1 est terminée localement. La préparation locale de 9.2 fonctionne : API Workers, D1/R2 locaux, deux migrations, tests et build sans upload. Aucun Worker, base ou bucket distant n'a été créé/modifié par ces étapes. Aucun login n'a été lancé.

Wrangler 4.152.0 est **déjà authentifié par OAuth**, avec un compte accessible. Son autorisation annonce user:read, account:read, offline_access, pages:write et zone:read. Elle ne contient pas les droits Workers/D1 requis. Les consultations distantes D1 et R2 sont refusées avec le code Cloudflare 10000 ; les consultations des déploiements kortex-api-staging/production échouent aussi. Leur existence et leurs permissions effectives ne sont donc pas validées. Un refus d'accès n'est pas une preuve d'absence de ressources.

Le projet ne renseigne aucun account_id, aucune liaison D1/R2 distante et aucune origine staging/production réelle. Le UUID du binding DB par défaut est exclusivement local. Les tableaux de bindings distants sont vides et les origines .invalid sont rejetées par l'application. AI n'est pas activé. La recherche de connecteur Cloudflare dans les intégrations disponibles n'a renvoyé aucun résultat ; le chemin de reprise est Wrangler.

Arrêt à cette limite, conformément à la demande du propriétaire. 9.3 n'est pas démarrée. La validation distante de 9.2 reste en attente ; la partie locale est prouvée et conservée.

## Ce que le propriétaire doit faire

Dans le terminal du projet, relancer l'autorisation OAuth avec les droits nécessaires. La commande suivante utilise des scopes reconnus par la version installée, conserve les droits Pages/zone déjà présents et ajoute Workers/D1 :

```text
pnpm exec wrangler login --scopes user:read account:read pages:write zone:read workers:write workers_scripts:write d1:write
```

Valider le bon compte et les autorisations affichées dans le navigateur. Les scopes OAuth sélectionnés permettent des écritures ; **obtenir une autorisation ne crée ni ne déploie une ressource**. La prochaine reprise commencera uniquement par des lectures. Aucun token, mot de passe ou fichier OAuth ne doit être collé dans le chat. Les droits du membre sur le compte doivent également autoriser ces produits ; renouveler un jeton ne donne pas un rôle que le membre n'a pas.

Dire ensuite « Cloudflare réautorisé, reprends la vérification ». Si la commande échoue, transmettre seulement le message d'erreur sans secrets. Sources officielles : [connexion et choix des scopes](https://developers.cloudflare.com/workers/wrangler/commands/general/), [rôles et permissions](https://developers.cloudflare.com/workers/authorization/). La liste des scopes a aussi été contrôlée avec `wrangler login --scopes-list`, qui n'ouvre pas de connexion.

## Vérifications à reprendre, sans déployer

```text
node api/scripts/check-cloudflare.mjs --resources
```

Le rapport sous target/cloudflare-connection-report.json ne conserve ni token, ni email, ni ID de compte. Vérifier authentification et accès aux inventaires D1/R2/Workers. Sélectionner ensuite explicitement le compte Kortex et identifier les ressources existantes à réutiliser ; ne pas remplacer des ressources existantes sur la seule base de leurs noms. Si les droits R2 sont rétablis mais son activation est demandée par Cloudflare, le propriétaire devra vérifier cet écran ; aucune activation ni donnée de paiement n'est effectuée par l'agent dans cette tâche.

Avant toute activation distante prévue à 9.6 : renseigner compte, IDs D1, noms R2 privés et origines HTTPS distinctes dans les environnements dédiés, confirmer les responsabilités, appliquer les migrations par l'outil prévu et contrôler les ressources réelles. Les noms de infra/r2-policy.md restent des propositions. Aucun ID imaginaire dans une configuration distante. L'étape 9.2 prépare les bindings ; elle n'autorise pas un déploiement anticipé de la phase.

## Continuer les essais locaux sans compte

```text
pnpm install --frozen-lockfile
pnpm --filter @kortex/api test
pnpm --filter @kortex/api typecheck
pnpm --filter @kortex/api build
node api/scripts/smoke-local.mjs
pnpm --filter @kortex/api dev
```

Le build est exclusivement `wrangler deploy --dry-run --env=`. Le serveur de développement utilise --local et 127.0.0.1:8787 ; le smoke utilise un port libre puis arrête uniquement son propre processus. Les migrations peuvent être appliquées à l'émulateur avec `pnpm --filter @kortex/api exec wrangler d1 migrations apply DB --local`. Ne pas ajouter --remote pour les essais locaux.

L'API expose seulement GET/HEAD /api/v1/health et les prérequêtes CORS autorisées. Les autres routes renvoient 404 JSON. La santé est une liveness, pas une interrogation D1/R2. Le catalogue/recherche arrivent en 9.3 et les opérations de publication plus tard. Le schéma api.schema.json pose leurs DTO candidats ; il ne signifie pas que ces routes existent ni que les signatures y sont déjà vérifiées.
