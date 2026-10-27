# Retrait de l'ancien projet kortex.site

Date : 2026-10-27. Autorisation : demande explicite du propriétaire de supprimer l'ancien projet, puis réponse « Conserver la messagerie » à la confirmation groupée des réglages restants.

Les opérations distantes suivantes ont effectivement réussi :

| Ressource | Retirée |
|---|---|
| Workers | kortex-api, kortex-desk, kortex-flow, kortex-hub, kortex-mind, kortex-run, kortex-seek, kortex-site, kortex-store |
| Domaines Worker | kortex.site, api, workspace, flow, dashboard, agent, search, store et run sous kortex.site |
| D1 | kortex-accounts, kortex-accounts-fresh-20261017 |
| Turnstile | Kortex registration, associé au seul hostname kortex.site |
| Redirection HTTP | Redirect from WWW to root [Template] |

Le retrait des domaines Worker a supprimé leurs neuf enregistrements DNS gérés. Les secrets et assets attachés aux Workers ont disparu avec ces scripts. Aucun export des données des bases supprimées n'a été effectué.

La vérification API après suppression retourne des inventaires vides pour Workers, domaines Worker, services Worker, D1, R2, KV, Queues, Durable Objects et Pages. La zone kortex.site reste active. Le contrôle des routes Worker avec le profil kortex retournait également une liste vide. Les inventaires DNS/règles et Turnstile ne sont pas accessibles avec les droits OAuth API actuels ; ils ont donc été contrôlés dans le tableau de bord connecté : plus de widget Turnstile, aucune Page Rule, aucune règle de sécurité personnalisée ou de rate limiting, aucune règle HTTP de réécriture, redirection, origine, transformation, configuration ou cache dans la vue globale après suppression de la redirection WWW.

Contrôle complémentaire API : R2 dans la juridiction EU, Workflows et Vectorize retournent également des inventaires vides. La messagerie a été relue après toutes les suppressions : routage Enabled, une règle et une destination, DNS Locked.

Conservés volontairement : l'enregistrement acheté et la zone DNS, les protections Cloudflare par défaut, les huit DNS de messagerie (deux CNAME d'envoi, trois MX Cloudflare, les deux DKIM et le SPF), le routage mail admin@kortex.site et sa destination. Cette exception demandée par le propriétaire signifie que l'état final conserve la messagerie ; ce n'est pas une zone totalement vide. Le dépôt du nouveau projet et les autres domaines du compte ne sont pas supprimés. Les données historiques de facturation, de journalisation et de sauvegarde conservées par le fournisseur ne sont pas présentées comme purgées.

Preuve publique expurgée : docs/evidence/cloudflare-reset-2026-10-27.json. Les inventaires et journaux d'opérations locaux sont sous target/cloudflare-reset-*.json, ignorés par Git, sans jeton OAuth. Les captures de vérification sont sous target/cloudflare-turnstile-reset.png et target/cloudflare-rules-reset.png ; elles restent locales car le tableau de bord affiche l'identité du compte. Aucun identifiant de compte/zone ou email de destination n'est ajouté aux preuves versionnées.

Aucune ressource du nouveau catalogue n'a été créée ou déployée. La suite demandée est l'implémentation locale de 9.3 ; le provisionnement distant reste prévu en 9.6.
