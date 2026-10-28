# Store Desktop — vérification 13.5

Périmètre demandé : Windows ; Linux/macOS et Firefox exclus par le propriétaire. Ne pas transformer cette exclusion en preuve de compatibilité.

Commande PowerShell : `$env:KORTEX_PHASE135='1'; node scripts/test-desktop-lifecycle.mjs`. Le parcours 13.2–13.4 est réutilisé. Aucun mock de téléchargement, signature ou exécution guest. Données isolées sous target ; processus Windows possédé par le test.

- Cinq outils publics : fiche, confirmation, téléchargement signé, activation dans library.json, lancement Wasmtime et rendu prêt dans WebView2, capture, désinstallation et disparition de l'index.
- Publier ensuite un sixième outil temporaire absent du catalogue initial, après ouverture de l'application. Rechercher, installer et convertir un texte Unicode sans redémarrage ni reconstruction.
- Basculer hors connexion et exécuter l'outil installé.
- Publier sa nouvelle version ; refuser puis accepter la permission supplémentaire, conserver données et ancienne version. Ouvrir le nouveau guest puis revenir à l'ancienne version.
- Publier un vrai guest défaillant ; son initialisation doit être rejetée sans modifier version courante ni données.
- Annuler une désinstallation, conserver les données, réinstaller, demander leur suppression ; les autres profils/outils restent intacts.
- Comparer les octets de l'exécutable avant/après. Révoquer les versions et la clé temporaires, supprimer les objets R2, restaurer la politique opérateur. Aucun DNS, mail ou production modifié.

Les cinq outils publics n'ont pas tous une deuxième version : download/install/launch/uninstall est couvert individuellement ; update/rollback par le sixième outil signé réel. Les vecteurs fonctionnels détaillés restent tracés dans tests/desktop/five-tools.md et phase12. Les dialogues Windows de sélection/enregistrement de fichiers ne sont pas automatisés ici.
