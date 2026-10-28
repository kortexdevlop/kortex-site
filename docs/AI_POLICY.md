# Politique IA V1 — 28 octobre 2026

Le service cloud propose des actions ; il ne charge et n’exécute aucun module WASM. Les outils invités restent utilisables sans compte et sans IA. Les étapes 15.2–15.4 ajoutent le médiateur local et l’interface Web/Desktop décrits ci-dessous.

## Consentement et données

`POST /api/v1/ai/plan` exige une session Web propriétaire avec CSRF, ou une session native dont le droit IA a été accordé séparément depuis les réglages Web. `consent_context: {ai_allowed:true, contains_private_data:false}` reste obligatoire à chaque demande. Le mode invité est désactivé. Les appareils existants et nouveaux n’obtiennent aucun droit IA par défaut : `ai_allowed=0`, activation/retrait Web propriétaire avec CSRF, effet immédiat. Le jeton natif ne peut pas s’octroyer ce droit ; ses demandes n’ont pas d’Origin et partagent les quotas du même utilisateur Web.

V1 accepte uniquement un message de 1 000 caractères maximum et dix références publiques d’outils au plus. Les fichiers, contenus binaires, handles et champs supplémentaires sont refusés, même si le demandeur affirme donner son consentement. Les assignations de secrets reconnues, Bearer et chemins locaux usuels sont refusés avant tout appel fournisseur. Les emails et formats de téléphone courants sont masqués. Ce filtre heuristique ne détecte pas toute donnée personnelle : l’utilisateur doit transmettre uniquement un besoin non confidentiel. Aucun document privé ne doit être collé dans le message.

Seuls le message filtré et au plus vingt actions publiques publiées/vérifiées sont envoyés à Workers AI. Les descriptions du catalogue et les messages sont traités comme données non fiables. Aucune clé d’authentification, identité utilisateur, archive ou donnée locale n’entre dans le prompt.

## Modèle et plafonds appliqués

Modèle réel : `@cf/meta/llama-3.3-70b-instruct-fp8-fast`, compatible [JSON Mode](https://developers.cloudflare.com/workers-ai/features/json-mode/), vérifié par une inférence réussie sur le compte staging le 28/10/2026. Le jeton opérateur refuse la consultation REST du catalogue AI (403), mais le binding natif autorise effectivement l’inférence. Pas de changement de forfait ni achat.

| Limite staging | Valeur |
| --- | --- |
| Requêtes par compte / jour UTC | 50 |
| Requêtes totales de ce service / jour UTC | 100 |
| Prompt et schéma, UTF-8 cumulés | 16 000 octets |
| Sortie fournisseur | 512 tokens |
| Réservation prudente par demande | 16 896 tokens |
| Réservation quotidienne totale | 1 689 600 tokens |
| Attente fournisseur | 20 secondes |
| Plan accepté | 10 étapes, 32 Kio maximum |

Réservation atomique D1 avant l’inférence. Les erreurs ne remboursent pas la réservation ; aucun nouvel essai automatique. Limites quotidiennes à 00:00 UTC, globales à tous les utilisateurs de ce Worker, avec `429` et `Retry-After` conservateur. Les autres applications du compte Cloudflare ne passent pas par ce compteur.

Selon les [tarifs Cloudflare](https://developers.cloudflare.com/workers-ai/platform/pricing/) consultés, l’estimation conservatrice de ce service est inférieure à 0,60 USD/jour pour ces limites. Ce calcul n’est ni une facture, ni un plafond de facturation de tout le compte. Les réglages de facturation/consommation globale n’ont pas pu être inspectés avec le jeton disponible. Ne pas présenter le budget applicatif comme un coupe-circuit de facturation Cloudflare.

## Validation indépendante et refus

Le fournisseur reçoit aussi les schémas exacts des actions candidates : une grammaire d’objet libre avait produit des paramètres invalides, correctement refusés lors du premier essai. La réponse reste validée indépendamment : structure stricte, version, nombre d’étapes, consentement obligatoire, action existante et schéma d’entrée du catalogue courant. Le sous-ensemble JSON Schema V1 est interprété sans `eval` ; un mot-clé non pris en charge provoque un refus. Champs de fichier/handle/binaire interdits dans la réponse. Aucun outil/action absent, brouillon ou révoqué n’est accepté.

Erreurs fournisseur, réponse mal formée ou entrée invalide : `503`, sans aucune exécution. Seuls des codes fixes et éventuellement un code numérique fournisseur sortent dans l’erreur ; jamais son texte libre ni sa réponse.

## Médiation et interface locales — 15.2 à 15.4

Une proposition reçue ne démarre aucun outil. L’utilisateur voit l’action et ses paramètres, puis approuve ou refuse chaque étape. Une décision est à usage unique, un nouveau plan exige de nouvelles décisions, et la proposition expire après cinq minutes. L’identité du compte est revérifiée avant d’exécuter une proposition cloud. Les plans et résultats restent uniquement en mémoire.

Au clic d’approbation, le Web revalide le catalogue publié, charge un outil signé dans l’origine runtime séparée et contrôle le plan contre le contrat vérifié de cet outil. Messages liés à l’origine, la fenêtre, un nonce et une session, plus une corrélation par requête. L’exécution passe par le dispatcher universel et conserve le courtier de permissions. Aucune décision IA n’accorde un accès fichier/réseau/stockage. Les résultats affichés sont bornés et les champs handles/chemins/binaires masqués ; aucun résultat n’est envoyé à l’IA.

Windows utilise le même `Runtime::invoke_action` après revue du paquet installé/signé, pin de l’instance et contrôle de version publiée lorsqu’en ligne. Aucun téléchargement ni installation automatique. En V1, l’assistant natif refuse les actions demandant des permissions ou des effets sensibles et renvoie vers l’interface de l’outil ; il ne contourne pas son consentement. Les cinq actions testées sont pures. Les outils déjà installés et l’exemple local restent disponibles hors connexion ; une demande cloud exige une connexion.

Le hash d’un fichier possède un parcours **local explicite**, distinct d’une demande IA : sélection du fichier dans le navigateur, limite 16 Ko pour cet exemple, paramètres masqués, puis approbation. Le contenu est transmis en mémoire au module WASM signé, sans requête Workers AI ni écriture du fichier dans localStorage. Ce parcours n’est pas présenté comme une génération du modèle. Les fichiers plus grands restent traitables dans l’outil manuel selon ses propres limites.

La politique CSP de `/ai` autorise uniquement les hashes exacts du bootstrap de rendu Astro et les scripts de même origine ; aucun `unsafe-inline` pour les scripts. Les champs restent désactivés jusqu’à l’hydratation et au chargement du catalogue, afin qu’une saisie précoce ne soit pas perdue.

Preuves : `docs/evidence/assistant-privacy.json`, `assistant-four-tools.json`, `assistant-native.json`, `assistant-account-staging.json` et matrice `ai-action-matrix.md`. Les propositions fixes sont injectées exclusivement par Playwright dans les tests : catalogue, admission signée, modules et exécution sont réels. La véritable inférence Cloudflare est déjà prouvée par 15.1 ; aucun quota n’a été remis à zéro ni relevé pour ces essais. La génération réelle de cinq plans supplémentaires n’a pas été tentée une fois le plafond quotidien atteint.

## Conservation

Kortex ne conserve ni prompts ni réponses de production, et ne les écrit pas dans ses logs. D1 ne stocke que des compteurs, réservations et échéances ; la clé quotidienne par utilisateur est un HMAC. L’expiration logique est quotidienne ; le nettoyage physique intervient à la prochaine activité. La purge de compte retire son compteur courant, sans réinitialiser le budget partagé.

L’appel est une inférence standard synchrone, sans AI Gateway, cache, batch, entraînement ni stockage explicite. Les [engagements Workers AI sur les données](https://developers.cloudflare.com/workers-ai/platform/data-usage/) s’appliquent, avec leurs exceptions et métadonnées opérationnelles. Aucune durée de conservation absolue du fournisseur n’est promise par Kortex.

Les preuves de test contiennent uniquement un besoin arithmétique public et sa proposition. Les comptes temporaires ont été supprimés. Les trois réservations du 28/10/2026 sont consommées par les essais (deux refus sûrs puis succès) ; aucun compteur global n’a été remis à zéro.

## Correction du plafond staging — 28/10/2026

Le plafond initial de 2 demandes par compte et 3 globales était un garde-fou de test Kortex, pas le quota Cloudflare. Il est relevé à 50 par compte / 100 globales par jour UTC, avec réservation totale cohérente de 1 689 600 tokens. Les compteurs existants sont conservés. Le budget applicatif ne mesure pas les neurones disponibles ni la consommation des autres applications du compte. Aucun forfait Cloudflare n’a été modifié.

Une limite interne renvoie `X-Kortex-Limit-Source: application`, conservé par le proxy Web, et un délai correspondant à minuit UTC. Le Web et Windows nomment explicitement la limite Kortex ; un 429 sans cette provenance reste une limitation temporaire non attribuée à Cloudflare. Vraie demande après correction : statut 200, calculatrice/multiply, a=25 et b=48, sans exécution. Compte temporaire supprimé ; preuve `docs/evidence/ai-limit-fix.json`. Les mentions d’épuisement à trois appels dans les rapports précédents décrivent l’ancien réglage.
