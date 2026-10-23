# Étape 4.3 — Stockage abstrait et état récupérable

23 octobre 2026 ; base 46a7528. Étape 4.2 approuvée par « Oui continue ».

Livrables : StorageBackend, ToolStorage et erreurs stables QUOTA, PERMISSION, UNAVAILABLE, CANCELLED, CONFLICT, CORRUPT ; snapshots avec identité outil, version outil, version stockage et empreinte SHA-256. Aucun contrat gelé modifié et aucune dépendance ajoutée. Les deux lib.rs exposent les modules nécessaires.

Un service est lié par l'hôte à (profil, outil, version stockage). Les clés ne permettent pas de changer cette identité. Le quota de 5 Mio compte clés et valeurs de toutes les versions d'un même outil/profil ; il ne peut donc pas être multiplié par création de versions. Chaque modification utilise un remplacement atomique conditionné par la révision. Les migrations N-1 remplacent leur namespace source dans la même transaction ; un échec de conversion ou de quota ne publie rien. Suppression explicite du namespace courant. Export du namespace uniquement après approbation explicite transmise par l'UI hôte.

Limites : aucun backend durable fourni ; le backend mémoire existe uniquement dans le fichier de tests. Les backends futurs doivent respecter comparaison de révision, quota, isolation et absence de modification sur échec. L'hôte choisit l'identité depuis le contexte chargé et vérifie la policy 4.2 avant de déléguer au service ; ce module n'est pas une nouvelle entrée guest. L'approbation d'export est une décision de l'UI hôte, pas un booléen accepté d'un outil. Cette API est réservée à l'état non secret ; elle ne détecte pas un secret arbitraire caché dans des octets et ne fournit aucun coffre de secrets. Le hash détecte la corruption et ne garantit pas l'authenticité. Restore exige identité et version stockage attendues ; une version N-1 nécessite la migration explicite.

Validation Windows : cargo test -p kortex-core --locked : 17 tests réussis, dont 6 nouveaux ; cargo test -p kortex-capabilities --locked : 16 tests réussis ; cargo fmt --all -- --check : réussi. Pas d'essais Linux/macOS ou Firefox, conformément aux instructions du propriétaire. Preuve : docs/evidence/storage-4.3.md.

Statut : awaiting_approval ; commit résultat : commit contenant ce rapport.
