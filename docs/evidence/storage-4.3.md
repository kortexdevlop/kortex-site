# Preuves locales — 4.3

Exécution Windows du 23 octobre 2026, verrou de dépendances inchangé.

- Core : 11 tests de cycle de vie existants + 6 tests d'état, tous réussis.
- Capabilities : 16 tests existants, tous réussis.
- Format Rust : réussi.

Scénarios nouveaux dans crates/kortex-core/tests/state.rs : roundtrip et altération du snapshot ; refus de restauration dans un autre outil ou version ; isolation entre deux profils et deux outils ; refus d'export sans approbation ; clé traversante refusée ; get/set/delete et nettoyage namespace ; quota exact puis dépassement depuis une autre version sans changement des données ; migration N-1 à quota plein sans double comptage ; annulation puis migration réussie ; erreurs PERMISSION/UNAVAILABLE/CANCELLED récupérables ; conflit de révision ; migrations produisant clé invalide ou dépassement de quota annulées ; snapshot surdimensionné refusé.

Memory est défini uniquement dans le test. Aucune durabilité, UI de consentement ou détection automatique de secrets dans une valeur arbitraire n'est revendiquée. Le contrat StorageBackend exige que les implémentations durables à venir maintiennent les mêmes garanties transactionnelles.
