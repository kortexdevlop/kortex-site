# Rapport 13.0 — Application Desktop produit

28/10/2026. Base 6a47c5e. Lot 12.6–13.1 expressément demandé par le propriétaire. Résultat technique réalisé et testé ; le statut de revue humaine historique reste distinct.

Application Windows Dioxus démarrable, navigation Store/Bibliothèque/Mises à jour/Compte et thème workspace. Réutilisation du composant du shell natif, du moteur Wasmtime et de kortex-ui. Bibliothèque contrôlée par la confiance hôte, sans compte obligatoire. Preuve réelle : lancement WebView2 sans réseau, ouverture d’un paquet signé déjà présent dans le dossier de test et conversion Unicode réussie. Aucun dialogue de permission pour cet outil sans permission ; les capacités sensibles gardent le courtier existant. Aucun installateur ou téléchargement Store ajouté.

Documentation et commandes : docs/DESKTOP.md. Preuves et empreintes : docs/evidence/desktop-phase13-validation.json ; captures desktop-store-windows.png et desktop-offline-windows.png. Windows uniquement selon le périmètre demandé. 13.2 non engagée. Aucun changement des contrats publics runtime/API, de DNS, de production ou de messagerie.
