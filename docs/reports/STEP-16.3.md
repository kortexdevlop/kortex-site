# Étape 16.3 — Exécution locale Web et Windows

28/10/2026. Base : `16de346`. Autorisation du propriétaire : « Enchaînes les 4 étape ».

L’éditeur lance les actions des paquets signés sur l’origine runtime dédiée, avec mapping typé, instantané du document et des contrats, confirmations, progression, arrêt, résultats locaux et journal sans paramètres ni résultats. Le protocole vérifie origine, fenêtre source, nonce, session et corrélation ; une seule invocation de parcours est admise par frame. La suppression de la frame arrête son Worker. Le conteneur garde une identité stable pendant le dialogue de consentement, correction prouvée par le scénario fichier.

Windows possède une section Parcours : JSON, confirmation du lot immuable, résolution des outils installés et signés, vérification de la version publiée en ligne, maintien des leases, `Runtime::invoke_action`, progression, annulation et résultats locaux. Le profil est limité à huit outils distincts, 50 étapes et cinq minutes, sans retry automatique. Le journal local exclut les entrées et sorties. La sélection de fichier reste séparée du document et sa lecture bornée exige un consentement distinct ; les handles du backend NativeFiles restent locaux.

Preuves réelles : date UTC → calculatrice, 1200 → 1210, sur Chromium staging et Wasmtime Windows hors connexion. Arrêt avant confirmation lors de la fermeture de la page ; aucun appel d’exécution cloud. Test isolé du protocole : faux nonce/session rejetés, rejeu et deuxième invocation rejetés, isolation/CSP et récupération par nouvelle session.

**Clôture non prononcée.** Les actions Desktop déclarant des capacités hôte sont actuellement refusées. Le dialogue générique de scopes et son raccordement au NativeServices/Gateway existant restent à faire ; une confirmation de parcours ne doit jamais devenir un grant de capacité. Les actions du catalogue actuel et le transfert de fichier choisi par l’utilisateur fonctionnent, mais cela ne prouve pas cette dernière partie du périmètre 16.3.

Preuves : `docs/evidence/orchestration-validation.json`, `orchestration-native.json`, `orchestration-web.json`, `orchestration-file.json` et logs associés. Production, DNS, messagerie, quotas IA et réserve Google/OIDC 14.5 conservés.
