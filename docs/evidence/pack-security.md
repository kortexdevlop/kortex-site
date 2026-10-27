# Sécurité paquet — 27 octobre 2026

L'inspection WASM statique a été extraite dans `kortex-pack::wasm_check::check`. Elle valide le binaire complet puis les signatures ABI, l'unique mémoire 32 bits non partagée avec maximum déclaré ≤1 024 pages, les exports requis et l'import optionnel unique `kortex_v1.host_enqueue(i32,i32)->i32`. Seuls les deux globals de compilation `__heap_base` et `__data_end` sont tolérés en plus. WASI, start et exports arbitraires sont refusés. Le packageur refuse les sources dont l'identité/version/ABI ont changé pendant le build.

Le test déterministe effectue 10 000 mutations ZIP/WASM : 9 952 refus structurels, 48 structures encore valides, zéro panic, zéro instanciation. Recettes et hash : `tests/fixtures/malformed/README.md` et `corpus.json`. Une structure valide ne dispense jamais de vérifier le hash et la signature détachée.

Six archives correctement signées mais malformées sont refusées par les deux admissions : fichier supplémentaire, conflit d'identité, WASI, mémoire partagée, mémoire sans maximum, mauvais export. Le test Web compte zéro Worker créé. Le test natif n'instancie aucun moteur. Les tests de confiance couvrent également altération des octets/descripteurs, clé inconnue, expirée/révoquée/retirée, snapshot falsifié et rollback de version.

Les produits utilisent les contrôles partagés : lecture ZIP stricte et validation complète côté natif ; règles équivalentes ZIP/métadonnées et inspection ABI avant compilation côté Web. Le Worker et Wasmtime contrôlent aussi l'ABI avant instanciation. Les tests unitaires du moteur qui fabriquent volontairement des guests invalides passent par la couche Core de test ; l'application et le gate passent par l'admission complète.

Résultats détaillés et limites : `validation-8.3-9.0.json`, `../reports/PHASE8_GATE.md`. Il s'agit d'essais locaux reproductibles, pas d'un audit de sécurité indépendant ni d'une preuve exhaustive de l'absence de vulnérabilité.
