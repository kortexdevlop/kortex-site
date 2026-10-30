# Livraison V1.0.0 — 22.1

**Non exécutée : prérequis 22.0 non satisfaits.** Aucun tag `v1.0.0`, Worker/DB/bucket production, bascule DNS ou installateur de production n'a été créé par ce lot. Les services staging, la zone et la messagerie sont conservés.

Le plan concret, limité à Windows x64 / Chromium-WebView2, est [infra/production-release-plan.json](../../infra/production-release-plan.json) : DB/sauvegarde/migrations → API → cinq paquets signés et catalogue → Runtime Web séparé → Astro SSR → nouvel installateur Windows signé. Chaque bascule exige un smoke et s'arrête en cas d'erreur. Le rollback de code ne restaure pas implicitement D1 ou les droits révoqués.

Les scripts de publication/déploiement actuels gardent leurs refus production. Ils ne sont pas débloqués par un JSON de test, des domaines fictifs ou la réutilisation d'un ancien EXE non signé. Le workflow site existant publie site puis Runtime pour staging : il ne doit pas être présenté comme l'orchestrateur de production dans l'ordre du plan ci-dessus.

À réunir avant exécution : RC/CI vérifiées sur la révision retenue, ressources/origines production réelles et isolées, secrets et règles OIDC protégés, budget gratuit vérifié, restauration distante réussie, certificat et installateur signés, contacts/revues et acceptation de la livraison concrète. Ensuite seulement créer le tag, effectuer la bascule ordonnée et archiver les reçus/checksums/version IDs réels.

Les notes [v1.0.0](../releases/v1.0.0.md) restent un brouillon. La recette production 22.2 et le gel V1 22.3 ne sont pas engagés.

Actualisation du périmètre : Google OAuth et signature Windows Authenticode sont reportés explicitement par le propriétaire. Ils ne bloquent plus la poursuite de la préparation ; leurs états techniques restent non configurés/non validés. Connexion email/mot de passe conservée. Les autres prérequis et signatures des paquets/canaux restent applicables.

Actualisation du 30/10 : DB/R2 et douze migrations production préparés dans le lot 22.0, indépendants du staging ; configurations de services générées mais non déployées. Cette préparation modifie l’inventaire des ressources, sans créer le tag v1.0.0 ni exécuter la livraison 22.1.
