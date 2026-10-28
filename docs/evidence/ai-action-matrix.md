# Matrice IA et confidentialité — 15.2 à 15.4

Date : 28/10/2026, Windows/Chromium. Catalogue et modules signés staging réels ; propositions déterministes injectées **uniquement par les tests navigateur** pour ne pas contourner les trois réservations Workers AI déjà consommées. Inférence réelle antérieure : `device-ai-staging.json`. Aucun mock ni générateur fictif dans le produit.

| Outil/action | Validation Web réelle | Validation native signée |
| --- | --- | --- |
| calculatrice.multiply | Approbation 25×48 → 1200 | Interface Windows, 1200 hors connexion |
| timestamp-utc.to_unix | 2024-01-01 UTC → 1704067200 | Même action via médiateur/Wasmtime |
| convertisseur-casse.upper | « Été ß » → « ÉTÉ SS » | Même action via médiateur/Wasmtime |
| explorateur-csv.summarize_csv | Deux lignes, deux colonnes | Même action pure via médiateur/Wasmtime |
| calculateur-hash.sha256_bytes | Vrai fichier sélectionné et approuvé ; SHA-256 attendu | Données de test locales « abc », empreinte standard |

Le cas fichier utilise un parcours local explicite de 16 Ko maximum, sans demander au modèle de manipuler des octets privés. Le contenu ne passe pas par `file.read` du module : le navigateur lit le fichier choisi et transmet ses octets au contrat pur `sha256_bytes` après approbation. Aucun droit supplémentaire n’est donné au module. L’interface manuelle de l’outil conserve son propre parcours de permissions.

Les refus, schéma mal formé, action `delete-files` inconnue, paramètres faux, expiration, consommation unique et déconnexion ne démarrent aucune action. L’arrivée du plan crée zéro iframe. L’approbation seule crée l’instance signée. Le moteur et les sorties sont contrôlés indépendamment des affirmations du fournisseur.

Les requêtes réseau du hash privé sont inventoriées dans `assistant-privacy.json` : ni texte sentinelle, ni base64, ni nom de fichier dans une requête ; aucun appel `/ai/plan`, aucun contenu fichier dans localStorage. Les résultats ne remontent jamais au cloud. Les données de session ne figurent pas dans l’inventaire.

Autorisation IA d’appareil : refus par défaut, activation Web sous session/CSRF, vraie demande native atteignant le quota 429, retrait immédiatement effectif (403). Aucun compteur global réinitialisé. Le compte temporaire a été supprimé et le coffre local nettoyé par le client.

L’assistant natif refuse les actions à permission/effet sensible et invite à ouvrir l’outil pour le consentement habituel. Le médiateur Web conserve le courtier de permissions dans l’iframe. Un refus n’accorde aucune capacité. Les preuves ne revendiquent pas une exécution sensible native automatisée ni cinq générations réelles du modèle après épuisement du quota.

Les outils installés et l’exemple natif fonctionnent hors connexion. L’IA indisponible conserve le lien vers les outils manuels ; une proposition cloud ne survit pas à un changement de compte. Google/OIDC réel demeure en attente de configuration (14.5), sans blocage des comptes email existants. Linux/macOS/Firefox exclus par instruction du propriétaire.
