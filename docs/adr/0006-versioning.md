# ADR-0006 — Versions et refus de négociation

Statut : candidat au gel, 23 octobre 2026. Owners : architecture, runtime, distribution/sécurité.

Les versions de paquet SemVer, ABI, format ZIP/manifeste, schéma UI, contrat d'actions, protocole JSON et API sont distinctes. La version 1.0 du format n'implique pas le support d'une ABI inconnue. Le runtime MUST vérifier chaque version prise en charge et min_runtime avant instanciation. tool.abi_version non supporté MUST produire UNSUPPORTED_ABI sans exécuter le module. Format, contrat, UI incompatibles produisent respectivement UNSUPPORTED_FORMAT, UNSUPPORTED_CONTRACT, UNSUPPORTED_UI ; runtime antérieur au minimum : RUNTIME_TOO_OLD. Une version malformée produit INVALID_VERSION. Les marqueurs de build SemVer ne modifient pas la priorité, une préversion précède sa version stable.

Ces codes sont des erreurs de négociation runtime ; ils ne modifient pas la liste des codes catalogue. Le refus n'autorise aucun downgrade silencieux ni nouvelle permission. Les mises à jour doivent conserver les contrats gelés ou ouvrir une majeure avec adaptateur et migration, conformément à KTX-VERSION-001 et KTX-EXCEPTION-001.

Les artefacts de mise à jour MUST être vérifiés par empreinte et descripteur Ed25519 détaché selon KTX-TRUST-001. Les changements de clés MUST utiliser un manifeste signé par la racine hors ligne. Les changements de règles/corpus exigent revue et approbation humaine tracée ; une approbation textuelle ne vaut pas signature cryptographique d'un artefact. Aucun mécanisme de signature de release n'est prétendu installé à cette étape.

Le gel de portabilité MUST attendre les preuves P1–P5 des prototypes ; tests locaux de corpus seuls insuffisants. La dérogation propriétaire Linux/macOS reste visible et ne prouve pas ces systèmes. Aucun nouveau gel par cet ADR.
