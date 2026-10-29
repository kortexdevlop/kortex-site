# Résilience Windows x64

Linux/macOS et autres moteurs exclus à la demande du propriétaire. Aucun résultat émulé pour ces plateformes.

Préparer les fixtures locales avec node scripts/ci-prepare.mjs --native, puis CARGO_INCREMENTAL=0, CARGO_PROFILE_DEV_DEBUG=0 et KORTEX_TEST_RELEASES=<workspace>/target/ci-fixtures ; exécuter cargo test --workspace --locked. Les anciennes archives target/phase12 ou staging-cloud ne sont pas la source de ce gate et peuvent avoir une confiance expirée.

Cas obligatoires : cinq archives signées et vecteurs partagés Web/Wasmtime ; guest hostile, dépassements mémoire/fuel/délais, annulation ; fichier choisi/refusé et droits à usage unique ; parcours sensible/capacité hors scope/retrait de consentement ; interruption/troncature/altération/redirection/annulation d’installation ; migration ratée et premier lancement non confirmé ; rollback conservant les octets utilisateur ; révocation et tombstones. Les tests Windows de vrais fichiers, stockage et HTTP loopback utilisent des dossiers temporaires distincts.

L’ancienne fixture HTTP fermait brutalement son socket après écriture : fermeture en écriture puis attente de fin du pair pour éviter des resets Windows parasites. Le test d’archive corrompue rejette maintenant explicitement un simple échec de transport au lieu de le compter comme validation d’intégrité. Cinq tests installateur et cinq tests mises à jour réussis après correction.

Le smoke HTTPS externe est explicitement ignoré par défaut et doit être exécuté séparément si requis ; aucune preuve live fabriquée. Logs complets : target/ci-reports/resilience/native*.log ; synthèse vérifiée dans docs/evidence/resilience-validation.json.
