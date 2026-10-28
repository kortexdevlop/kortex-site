# Rapport 13.5 — Store Desktop et publication à chaud

28/10/2026. Base ad00aa5. Autorisation : « Enchaines ces 2 étapes rapidement ».

Parcours Windows des cinq outils publics : téléchargement signé, installation, lancement/rendu, désinstallation. Sixième outil réellement publié après ouverture ; conversion Unicode en ligne/hors connexion, mise à jour avec permissions, retour arrière et refus d'un guest défaillant, conservation puis purge isolée. Preuves : docs/evidence/desktop-store135-validation.json ; matrice : docs/evidence/desktop-store-matrix.md.

La première tentative a correctement refusé la calculatrice : le Desktop utilisait une autre clé dry-run du même nom. Le paquet distant n'a pas été remplacé ; la clé publique de sa publication réelle a été rétablie et sa signature vérifiée avec le constructeur. Recompilation avant le parcours final, sans contournement de vérification.

Windows seulement sur instruction du propriétaire. La validation multi-OS du plan n'est pas revendiquée. Tests fonctionnels détaillés des guests tracés phase12 ; sélecteurs natifs de fichiers manuels. Les autres outils n'ayant pas de nouvelle version, les updates sont éprouvées avec la fixture signée réelle. Aucun contrat public changé. 14.1 non engagée.
