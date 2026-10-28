# Politique IA V1 — 28 octobre 2026

Le service propose des actions ; il ne charge et n’exécute aucun module WASM. Les outils invités restent utilisables sans compte et sans IA. Aucun médiateur d’exécution ni chat produit n’est livré dans ce lot (15.2 et suivantes).

## Consentement et données

`POST /api/v1/ai/plan` exige une session Web propriétaire, un jeton CSRF et `consent_context: {ai_allowed:true, contains_private_data:false}`. Le mode invité est désactivé. Les appareils Desktop ne possèdent pas le droit `ai:plan`.

V1 accepte uniquement un message de 1 000 caractères maximum et dix références publiques d’outils au plus. Les fichiers, contenus binaires, handles et champs supplémentaires sont refusés, même si le demandeur affirme donner son consentement. Les assignations de secrets reconnues, Bearer et chemins locaux usuels sont refusés avant tout appel fournisseur. Les emails et formats de téléphone courants sont masqués. Ce filtre heuristique ne détecte pas toute donnée personnelle : l’utilisateur doit transmettre uniquement un besoin non confidentiel. Aucun document privé ne doit être collé dans le message.

Seuls le message filtré et au plus vingt actions publiques publiées/vérifiées sont envoyés à Workers AI. Les descriptions du catalogue et les messages sont traités comme données non fiables. Aucune clé d’authentification, identité utilisateur, archive ou donnée locale n’entre dans le prompt.

## Modèle et plafonds appliqués

Modèle réel : `@cf/meta/llama-3.3-70b-instruct-fp8-fast`, compatible [JSON Mode](https://developers.cloudflare.com/workers-ai/features/json-mode/), vérifié par une inférence réussie sur le compte staging le 28/10/2026. Le jeton opérateur refuse la consultation REST du catalogue AI (403), mais le binding natif autorise effectivement l’inférence. Pas de changement de forfait ni achat.

| Limite staging | Valeur |
| --- | --- |
| Requêtes par compte / jour UTC | 2 |
| Requêtes totales de ce service / jour UTC | 3 |
| Prompt et schéma, UTF-8 cumulés | 16 000 octets |
| Sortie fournisseur | 512 tokens |
| Réservation prudente par demande | 16 896 tokens |
| Réservation quotidienne totale | 50 688 tokens |
| Attente fournisseur | 20 secondes |
| Plan accepté | 10 étapes, 32 Kio maximum |

Réservation atomique D1 avant l’inférence. Les erreurs ne remboursent pas la réservation ; aucun nouvel essai automatique. Limites quotidiennes à 00:00 UTC, globales à tous les utilisateurs de ce Worker, avec `429` et `Retry-After` conservateur. Les autres applications du compte Cloudflare ne passent pas par ce compteur.

Selon les [tarifs Cloudflare](https://developers.cloudflare.com/workers-ai/platform/pricing/) consultés, l’estimation conservatrice de ce service est inférieure à 0,02 USD/jour pour ces limites. Ce calcul n’est ni une facture, ni un plafond de facturation de tout le compte. Les réglages de facturation/consommation globale n’ont pas pu être inspectés avec le jeton disponible. Ne pas présenter le budget applicatif comme un coupe-circuit de facturation Cloudflare.

## Validation indépendante et refus

Le fournisseur reçoit aussi les schémas exacts des actions candidates : une grammaire d’objet libre avait produit des paramètres invalides, correctement refusés lors du premier essai. La réponse reste validée indépendamment : structure stricte, version, nombre d’étapes, consentement obligatoire, action existante et schéma d’entrée du catalogue courant. Le sous-ensemble JSON Schema V1 est interprété sans `eval` ; un mot-clé non pris en charge provoque un refus. Champs de fichier/handle/binaire interdits dans la réponse. Aucun outil/action absent, brouillon ou révoqué n’est accepté.

Erreurs fournisseur, réponse mal formée ou entrée invalide : `503`, sans aucune exécution. Seuls des codes fixes et éventuellement un code numérique fournisseur sortent dans l’erreur ; jamais son texte libre ni sa réponse. La validation locale avec revue/permissions avant exécution appartient à 15.2.

## Conservation

Kortex ne conserve ni prompts ni réponses de production, et ne les écrit pas dans ses logs. D1 ne stocke que des compteurs, réservations et échéances ; la clé quotidienne par utilisateur est un HMAC. L’expiration logique est quotidienne ; le nettoyage physique intervient à la prochaine activité. La purge de compte retire son compteur courant, sans réinitialiser le budget partagé.

L’appel est une inférence standard synchrone, sans AI Gateway, cache, batch, entraînement ni stockage explicite. Les [engagements Workers AI sur les données](https://developers.cloudflare.com/workers-ai/platform/data-usage/) s’appliquent, avec leurs exceptions et métadonnées opérationnelles. Aucune durée de conservation absolue du fournisseur n’est promise par Kortex.

Les preuves de test contiennent uniquement un besoin arithmétique public et sa proposition. Les comptes temporaires ont été supprimés. Les trois réservations du 28/10/2026 sont consommées par les essais (deux refus sûrs puis succès) ; aucun compteur global n’a été remis à zéro.
