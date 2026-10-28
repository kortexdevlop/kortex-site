# Étape 15.0 — Politique et budget IA

Base : `b00a6af`. Autorisation : propriétaire, « Enchaines ces 3 étapes ». Date : 28/10/2026.

Politique écrite dans `docs/AI_POLICY.md`, configuration validée, schéma de proposition V1 additif et contrats documentés. Modèle réellement disponible : `@cf/meta/llama-3.3-70b-instruct-fp8-fast`, JSON Mode confirmé par documentation officielle et inférence native réussie sur le compte staging.

Mode invité désactivé. Session Web + CSRF + consentement explicite, aucun fichier privé accepté en V1. Masquage emails/téléphones courants et refus des secrets/chemins reconnus ; limite heuristique documentée. Aucune conservation des prompts/réponses en D1 ou logs Kortex ; compteurs HMAC journaliers seulement. Engagements du fournisseur sourcés avec exceptions, aucune garantie absolue inventée.

Plafonds appliqués : 2 demandes/utilisateur/jour UTC, 3 globales/jour, réservation 16 896 tokens/demande, 50 688/jour, 512 tokens sortants et 20 s. Réservation atomique, pas de remboursement/essai automatique après échec. Estimation inférieure à 0,02 USD/jour pour ce Worker selon les tarifs consultés ; ce n’est pas un plafond de facturation global. Lecture REST du catalogue/paramètres AI refusée par le jeton (403), disponibilité néanmoins prouvée par le binding. Aucun achat ni changement de forfait.

Validation : 81 tests API dont confidentialité/consentement/quota concurrent/plan strict et fournisseur contrôlé dans les tests seulement ; types et 3 tests de contrats réussis. Vraie inférence arithmetic publique et `429` réel après consommation du quota. Trois réservations consommées aujourd’hui, aucune remise à zéro. Voir `device-14.5-15.1-validation.json` et preuves staging.

Décision : **réalisation et validation techniques 15.0 terminées**, sous réserve de la clôture OIDC du prédécesseur 14.5. Revue humaine non inventée. Aucun développement de 15.2.

Fichiers : `docs/AI_POLICY.md`, `docs/CONTRACTS.md`, `api/src/{env,services/ai-config,security/ai-rate-limit}.ts`, schéma `ai-plan`, migration de compteurs et tests associés. Production, DNS et messagerie conservés.
