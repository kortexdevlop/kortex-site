# Corpus ABI borné, phase 18

Seed pointeurs : 0x18011802, LCG 1664525/1013904223, 4 096 mutations par moteur. Corpus ZIP/WASM existant : seed 0x4b4f5254, 10 000 mutations, référence tests/fixtures/malformed/corpus.json inchangée. Les mutations structurellement valides ne sont pas présentées comme des attaques exécutées : le scanner statique n’instancie rien.

Native : adversarial.rs construit de vrais modules WAT signés par la fixture de test, puis exerce NativeGuestEngine/Wasmtime. Boucles dans alloc/dispatch/dealloc, trap, sortie débordante et chevauchement ; budget 300 ms, borne observée <5 s pour chaque scénario, arrêt explicite. Tables non bornées/énormes et mémoires excessives refusées avant instanciation.

Web : abi-limits.spec.ts construit des octets WASM et leur metadata signée avec une clé éphémère ; aucune admission unsigned. Worker réel, timeout 300 ms (≤30 s produit), ticker parent vivant, bornes et corrélation, tables/mémoire refusées avant instanciation. Tests existants de schéma UI, queue, stockage et fichiers ajoutés à la régression.

Il s’agit d’un corpus déterministe et reproductible, pas d’une campagne libFuzzer exhaustive. Aucun cargo fuzz/libFuzzer n’est prétendu exécuté. Les outils hostiles ne sont jamais publiés ; transport et identité contrôlés seulement dans les bancs locaux. Les fixtures V1 de calculatrice restent figées pour vérifier les anciennes signatures/goldens malgré la publication additive 1.0.1. Une clé de fixture expirée est régénérée par le packager éphémère, jamais rendue valide en contournant l’expiration.
