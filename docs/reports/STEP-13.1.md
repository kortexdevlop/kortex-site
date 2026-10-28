# Rapport 13.1 — Store Desktop alimenté par l’API

28/10/2026. Base 6a47c5e. Lot 12.6–13.1 expressément demandé par le propriétaire. Résultat technique réalisé et testé ; le statut de revue humaine historique reste distinct.

Client HTTP typé et validation via les schémas shared ; catalogue, fiches et actions réels. Recherche, filtres issus des métadonnées, tri, curseur et cache cinq minutes. Tests Rust : contrôleur parcourant 10 000 entrées synthétiques sans doublon, erreurs 404/503, cache TTL, encodage de filtres et réponses hostiles. Essai Windows contre le catalogue Cloudflare : cinq outils, recherche, filtre, tri, fiche/actions et état vide. Aucun catalogue codé en dur ni base de catalogue Desktop.

Documentation et commandes : docs/DESKTOP.md. Preuves et empreintes : docs/evidence/desktop-phase13-validation.json ; captures desktop-store-windows.png et desktop-offline-windows.png. Windows uniquement selon le périmètre demandé. 13.2 non engagée. Aucun changement des contrats publics runtime/API, de DNS, de production ou de messagerie.
