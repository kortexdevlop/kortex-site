# Preuve 5.1

Windows, 23 octobre 2026. Typecheck strict réussi, build esbuild réussi, huit tests réels Chromium/WebKit réussis. Tests : web-adapter/tests/wasm.spec.ts. Guest : tests/spikes/core-guest/target/wasm32-unknown-unknown/release/kortex_core_guest_spike.wasm, instructions inchangées ; sections de métadonnées et signature éphémère ajoutées uniquement dans le banc.

La vérification de signature est exécutée dans les deux navigateurs, y compris le repli strict WebKit. Source primaire du repli et audit version 2.3.0 : https://github.com/paulmillr/noble-curves/blob/main/README.md. Le binaire WASM est instancié exclusivement dans Worker ; boucle infinie d'un module adversarial arrêté, ticks du thread UI observés pendant son exécution. Contrôles mémoire adversariaux et corrélation refusés. L'API ne permet pas de construire un VerifiedTool avec un objet structurel, et modifier la copie publique des octets ne modifie pas l'artefact admis.

Limite assumée de phase : pas de chargeur ZIP/API/cache avant 5.3. Aucun outil factice permanent ; fixtures et modules adversariaux uniquement dans les tests.
