# Reprise Cloudflare — point d'arrêt demandé, 27 octobre 2026

## État courant — profil kortex activé et contrôlé

Le propriétaire a créé/activé le profil kortex. `wrangler auth list` confirme sa liaison au dossier du projet. Le contrôle whoami annonce désormais Workers, workers_scripts et D1, et les inventaires D1/R2 réussissent. Le blocage de droits est levé. La préparation locale 9.2 est validée ; 9.3 n'est pas démarrée.

Deux bases D1 de comptes sont visibles : kortex-accounts et kortex-accounts-fresh-20261017. Elles n'ont pas été interrogées en SQL, modifiées ou choisies comme catalogue. La liste R2 par défaut réussit sans retourner de bucket ; aucune conclusion sur d'autres juridictions non interrogées. Les Workers nommés kortex-api-staging et kortex-api-production ne sont pas trouvés (10007). Les paramètres distants Kortex restent vides ; ils seront fixés au provisionnement prévu à 9.6, sans utiliser le UUID local comme véritable ID distant.

Aucun login, changement de droit, création, upload ou déploiement par l'agent durant ce contrôle. Preuve expurgée : docs/evidence/cloudflare-kortex-profile-9.2.json. Les sections suivantes conservent l'historique du diagnostic ; une nouvelle réautorisation n'est pas demandée à ce stade.

## Historique — réautorisation du profil global

La connexion utilisateur a bien réussi et a demandé les scopes Workers/D1 prévus. Le contrôle agent suivant a néanmoins renouvelé un jeton dont les scopes ne couvrent toujours que Pages/user/account/zone ; D1/R2 restent refusés, code 10000. La cause n'est pas établie. Ni un échec de saisie utilisateur ni l'absence de ressources ne sont déduits. Preuve publique expurgée : docs/evidence/cloudflare-reauthorization-9.2.json.

Pour séparer Kortex de la session globale Pages, utiliser désormais un profil dédié. Depuis PowerShell dans C:\Users\µ\Desktop\KORTEX-, lancer ces commandes une par une (pnpm n'est pas dans le PATH de ce terminal) :

```powershell
& "C:\Program Files\nodejs\node.exe" ".\node_modules\wrangler\bin\wrangler.js" auth create kortex --scopes user:read account:read workers:write workers_scripts:write d1:write
```

Valider l'autorisation dans le navigateur et attendre le succès, puis lier ce profil au seul dossier du projet :

```powershell
& "C:\Program Files\nodejs\node.exe" ".\node_modules\wrangler\bin\wrangler.js" auth activate kortex
```

Ces commandes ont été vérifiées dans l'aide de Wrangler 4.152.0. La fonctionnalité de profils y est marquée expérimentale. whoami ne prend pas --profile dans cette version : l'activation du profil sur le dossier est nécessaire. Ne pas supprimer la session Pages existante. L'agent n'a lancé ni création/activation de profil ni nouveau login. Après confirmation du propriétaire, reprendre le contrôle read-only normal dans ce dossier ; aucun déploiement autorisé par cette manipulation.

## État initial vérifié (historique)

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

L'API expose GET/HEAD /api/v1/health et, depuis 9.3, la liste/recherche, les détails et actions sous /api/v1/tools. Voir api/README.md pour les paramètres et le secret serveur de curseur. La santé est une liveness, pas une interrogation D1/R2. La publication et le téléchargement restent aux étapes suivantes. Les DTO API restent candidats ; les six schémas gelés sont inchangés.

## Réinitialisation de l'ancien projet — 27 octobre 2026

Sur demande explicite du propriétaire, les neuf anciens Workers, leurs domaines, les deux bases de comptes, le widget Turnstile et la redirection WWW ont été supprimés. Les inventaires de calcul/stockage vérifiés sont maintenant vides ; les bases préexistantes citées plus haut sont un historique, pas des ressources à réutiliser. L'enregistrement et la zone active kortex.site sont conservés. Le propriétaire a expressément choisi de conserver la messagerie, ses huit DNS et sa règle de routage. Rapport : docs/reports/CLOUDFLARE-RESET-2026-10-27.md. Aucune ressource du nouveau catalogue n'a été créée ; aucun déploiement anticipé de 9.6.

## Nouveau staging — lot 9.4–9.6

L'état « inventaires vides » ci-dessus correspond au retrait de l'ancien projet. Le lot suivant autorisé crée maintenant un staging distinct : kortex-api-staging, kortex-catalog-staging (a406c1bf-cf12-4d1d-b8ff-6aa0955abb0a), kortex-tools-staging privé. L'API a effectivement été déployée sur https://kortex-api-staging.userpgl.workers.dev. Aucun changement DNS/messagerie, aucun binding de production. Les IDs et noms réels sont dans api/wrangler.jsonc ; aucun jeton ni clé privée dans cette configuration.

Les migrations sont exécutées avec le binding DB effectivement adopté en 9.2, donc `wrangler d1 migrations apply DB --remote --env staging --config api/wrangler.jsonc --profile kortex`, et non l'ancien alias KORTEX_DB du banc 9.0. Le déploiement distant est explicite et séparé du script build dry-run.

Consulter infra/staging-checklist.md pour l'identité temporaire d'opérateur, le PUT R2 conditionnel via remote binding, la reprise et le nettoyage. Le roundtrip et l'ouverture sur deux hôtes doivent être attestés dans docs/evidence/cloud-publish.md ; le simple succès du déploiement ou des tests locaux ne suffit pas au gate 9.6.
