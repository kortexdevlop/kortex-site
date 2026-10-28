# Parcours entre outils — 28/10/2026

| Scénario | Web Chromium | Windows Wasmtime |
|---|---|---|
| Date UTC → nombre → addition | 1200 puis 1210, paquets signés | 1200 puis 1210, paquets signés, hors connexion |
| Number calculatrice → integer timestamp | Refus de type avant invocation | Refus avec contrats réels vérifiés |
| Statistiques texte absentes | Action rejetée à l’import | Action rejetée à la compilation |
| Fichier `abc` → SHA-256 | 0 lecture avant consentement, 1 après ; téléchargement explicite | Bind refusé sans consentement ; vrai NativeFiles puis paquet signé |
| Définition sauvegardée | Aucun contenu de fichier ni résultat | Définition originale inchangée |

Empreinte commune : `ba7816bf8f01cfea414140de5dae2223b00361a396177a9cb410ff61f20015ad`.

Les IDs des scénarios figurent dans les tests, jamais dans l’ordonnanceur produit. Les deux clients utilisent l’ABI `action.invoke` et les contrats signés existants ; aucune coercition implicite n’a été ajoutée au format workflow gelé. Le sens date → calcul remplace le positif calcul → date impossible avec ces types actuels ; le sens initial demeure un négatif verrouillé.

Réserve : prise en charge des capacités hôte déclarées dans les actions Desktop encore absente, donc clôture universelle 16.3/16.5 laissée ouverte. Voir les rapports correspondants et le manifeste de preuves.
