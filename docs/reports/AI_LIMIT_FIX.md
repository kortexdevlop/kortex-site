# Correction de la limite IA — 28/10/2026

Demande : « Règle ça », puis « Termine ». Le compteur Cloudflare n’était pas épuisé ; le plafond applicatif de trois demandes staging empêchait de continuer.

Limites Kortex : 50 demandes/compte et 100 au total par jour UTC, réservation quotidienne cohérente de 1 689 600 tokens. Compteurs conservés, aucun changement de forfait Cloudflare. La limite reste applicative et ne prétend pas mesurer le quota de neurones du compte.

L’API marque un refus interne avec `X-Kortex-Limit-Source: application` ; la façade Web conserve cette provenance et le délai jusqu’à minuit UTC. Messages Web/Windows corrigés : limite Kortex explicitement nommée ; une limitation non identifiée n’est pas attribuée à Cloudflare.

Validation : 11 tests API ciblés réussis, dont une quatrième réservation acceptée au-delà de l’ancien plafond, types API, build Web et Windows, scénario navigateur refus/quota/déconnexion réussi. Déploiements API et Web staging effectués. Nouvelle inférence réelle (catalogue complet comme l’interface) : statut 200, `calculatrice/multiply`, a=25 et b=48 ; aucune exécution automatique. Compte de test purgé. Preuve : `docs/evidence/ai-limit-fix.json`.

Production, DNS et messagerie non touchés. Les traces précédentes de trois réservations / 429 sont historiques : elles correspondent à l’ancien réglage et ne prouvent pas un épuisement Cloudflare.
