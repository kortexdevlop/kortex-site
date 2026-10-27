# Publication à chaud — preuve distante

27/10/2026 : gate réel réussi sur Cloudflare, avec fiche, carte, actions, sitemap et exécution signée.

| Contrôle | Résultat |
|---|---|
| Visibilité complète après publication | 381 ms |
| Calcul dans Chromium / WebKit | 25 × 48 = 1200 |
| Fiche et app refusées après révocation | 517 ms |
| Descripteur Runtime après révocation | 404 |
| Archive Runtime après révocation | 410 |
| Hash build Web avant/après | 5387ddbcb813c97bf7547bf1b57cc6f7e7b0dd788f040a2370912814a59836f2 |
| Déploiement Web avant/après | f5912646-b1f6-4b46-b31d-aa9f30b0053c |
| Politique de l’opérateur temporaire restaurée | oui |
| DNS, messagerie, production modifiés | non |

Les preuves machine complètes sont dans web-hot-publish.json et hot-publish-playwright.json. L’opérateur local simule la CI ; ce n’est pas une exécution GitHub OIDC. Les lectures de validation demandent une revalidation HTTP ; les TTL annoncés restent ≤60 s pour le catalogue et ≤900 s pour le sitemap. Le fixture est révoqué et ne pollue pas le catalogue après l’essai ; son artefact et l’audit sont conservés.
