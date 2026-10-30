# Alertes opérationnelles

Périmètre : staging Windows/Chromium ; production non activée. SLO cible API : 99,5 % par mois, pas un résultat déjà atteint. L’absence de trafic ne prouve pas la disponibilité. Une mesure mensuelle nécessite un agrégateur et des probes externes avant production.

| Signal | Seuil / action | État |
|---|---|---|
| 5xx | >2 % sur cinq cases UTC d’une minute, volume ≥100 ; vérifier configuration, dépendances et quota D1 | Évaluateur testé localement et via sonde privée Cloudflare |
| 429 | Séparer limite applicative IA et fournisseur ; ne jamais augmenter automatiquement le budget | Compteur disponible |
| Latence | SLO catalogue p95 ≤500 ms liste / ≤300 ms détail ; benchmarks dédiés et matériel documenté | Rapports existants ; p95 du collecteur est un échantillon borné distinct |
| Publication | Réponse 5xx sur famille publication ; rapprocher du journal de reprise | Événement disponible |
| R2 / IA | Appels et erreurs de dépendances ; coûts réels via dashboard du fournisseur | Compteurs applicatifs, aucune estimation de facture inventée |

Usage : obtenir un export NDJSON des seuls événements structurés autorisés, exécuter node scripts/health-summary.mjs export.ndjson. Code 2 = alerte, code 0 = pas de seuil dépassé. Le CLI ne transmet rien. Une entrée de plus de 16 Ko ou invalide est ignorée et comptée rejetée.

Configuration prête dans api/wrangler.jsonc : logs sans persistence, sans invocation automatique ni traces, requêtes URL masquées. Vérifiée contre le schéma Wrangler épinglé 4.152.0 et par dry-run. Rien n’a été publié sur le Worker produit. Avant activation durable : choisir collecteur, destinataire/astreinte, rétention des agrégats et plafonds financiers. Aucun achat, abonnement, notification ou automation créé par cette étape.

Drill réseau : node scripts/qa-telemetry-cloud.mjs avec credential en environnement, jamais en argument. Crée un Worker authentifié et expirant sans bindings de données ; vérifie propriétaire puis supprime sa seule ressource. Maximum normal : 232 lectures HTTP, zéro lecture D1 et zéro inférence. Échec du nettoyage = gate incomplet.

## Préparation production sans D1

Workflow production-health.yml et production-monitor.json préparés ; trois probes statiques au maximum par exécution, aucune lecture D1, cible horaire. Le moniteur est désactivé tant que production/DNS ne sont pas réellement actifs. Il ne constitue pas un collecteur global d’erreurs ni un SLO end-to-end. Échec du job = signal GitHub selon les préférences existantes du propriétaire ; aucune destination email/webhook créée. Quatre tests setup/moniteur et simulation locale zéro réseau du mode désactivé réussis.
