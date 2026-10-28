# Rapport 12.5 — Cinq outils sur les deux hôtes

## Clôture technique — 28/10/2026

Demande du propriétaire : « Termines les correctement et clotures les avant de passer à la suite ». Les étapes 12.3, 12.4 et 12.5 sont techniquement closes ; 12.6 n’est pas engagée. Implémentation CSV initiale : fd0657c ; publication : 61f712c ; premier essai Cloudflare : 3e2d138 ; commit final consigné dans PROGRESS.md.

Les cinq outils sont publiés sur le staging. Le vrai site charge les vrais paquets signés, puis exécute les neuf vecteurs partagés avec Wasmtime Windows. La matrice est dans docs/evidence/five-tools-matrix.md. Aucun identifiant de ces outils ne figure dans le code hôte/catalogue/API ; la fixture partagée de publication de calculatrice est un exemple, pas une branche produit.

Le défaut d’export a été corrigé dans le site : ajout de allow-downloads au sandbox de l’iframe, toujours sur origine distincte, sans navigation top ni fenêtre supplémentaire. Cette correction de la capacité file.save est couverte par la demande explicite de terminer les points restants ; l’amendement du contrat documentaire est tracé dans CONTRACTS.md. Le rapport téléchargé est comparé octet pour octet à l’action export_report. Refus de permission et CSV mal formé restent testés.

La mémorisation utilise réellement IndexedDB dans un profil Chrome temporaire, après permission navigateur persistent-storage accordée par CDP, sans mock de persist(), persisted() ou IndexedDB. Le nom est retrouvé après rechargement du site ; un refus de permission ne le révèle pas. Un contexte sans persistance confirmée renvoie UNAVAILABLE sans fausse confirmation. Cette autorisation de test ne modifie pas le profil personnel du propriétaire. Le stockage durable reste soumis aux décisions du navigateur ; l’outil n’annonce aucune garantie en mode privé ou évictable.

Vérifications et comptes exacts : docs/evidence/phase12-closure.json. Les logs de clôture et rapports Playwright sont joints. Le site de staging a été redéployé (version c18c4f20-3a2d-4f55-9e67-9b66d4107534) ; les cinq paquets restent immuables. Production, DNS, domaine et messagerie inchangés.

Limites explicites : Linux/macOS et Firefox exclus sur instruction du propriétaire. WebKit non rejoué pour ce lot ; les essais antérieurs restent historiques. Sélection/sauvegarde native positive non automatisée ; vrais fichiers/handles/permissions et persistance après redémarrage couverts côté Wasmtime. Preuve WebView2 calculatrice héritée des phases 7 à 9. Clôture technique enregistrée séparément de la revue visuelle historique, sans approbation humaine inventée.
