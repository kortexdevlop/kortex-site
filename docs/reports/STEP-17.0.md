# Étape 17.0 — Modèle du bureau

28/10/2026. Base : `16de346`. Modèle terminé et testé techniquement, sans interface 17.1.

Schéma additif `workspace.schema.json`, types partagés et modèle TypeScript : format 1.0, au plus huit fenêtres, UUID distincts pour fenêtre et instance, slug et SemVer conformes au manifeste, bornes minimales 320×240, modes normal/minimisé/maximisé, focus et ordre z normalisé. La fabrique produit de nouveaux UUID ; restaurer ne crée aucune instance runtime.

Transitions d’ouverture, fermeture, focus, déplacement, minimisation, maximisation et restauration ; doublons d’instances et de fenêtres refusés, recadrage au viewport et retour au dernier focus visible. En dessous des dimensions minimales, le modèle utilise un espace logique 320×240 ; l’adaptation visuelle appartient à 17.1. Les bounds normaux sont conservés lors d’une maximisation.

Sérialisation locale avec validation et lecture bornée, récupération d’un état vide si la sauvegarde est invalide. Schéma à champs stricts : aucun contenu de fichier, état guest, handle, token ou champ étranger ne peut être persisté. Les tests couvrent huit fenêtres, neuvième refusée, collision, focus/z, fermeture, relecture identique, recadrage et rejet de champs privés ; ils confrontent aussi les sorties au schéma JSON réel.

Statut formel humain laissé en attente. Les réserves de 16.3/16.5 restent visibles ; l’interface de fenêtres 17.1 n’a pas été commencée.
