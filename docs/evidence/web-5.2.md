# Preuves finales du lot Web — 5.2

Windows, 23 octobre 2026. Commandes : pnpm --filter @kortex/web-adapter typecheck ; pnpm --filter @kortex/web-adapter test (22 tests) ; pnpm exec playwright test tests/web (6 tests) ; cargo check -p kortex-runtime-web --locked ; dx build --platform web --debug-symbols false. Compilation, typecheck et tests exécutés avec les versions épinglées. Les scripts construisent automatiquement la fixture guest de test ; RUSTFLAGS limite sa mémoire à 64 Mio. Le certificat réseau et la clé privée de test restent exclusivement dans target ignoré.

Scénarios : sept capacités refusées sans grant ; fausse approbation JS refusée ; refus utilisateur ; annulation/révocation ; IndexedDB réel isolé par profil/outil/version et quota atomique 5 Mio ; persistance non confirmée refusée ; vrai sélecteur de fichiers avec annulation et fichier 10 Mio+1 ; lecture de hello en base64 et vrai téléchargement ; jeton révoqué refusé ; allowlist réseau, vrai redirect TLS non suivi, offline et gros corps refusés ; clipboard exigeant un geste et acceptant seulement succès natif ou refus explicite.

Parcours complet prouvé : guest SDK Rust signé, host_enqueue(storage.read), prompt Dioxus, clic réel, lecture IndexedDB de prefs=123, capability.result dans le Worker, render actualisé avec resolved:123. Captures Chromium et WebKit inspectées. Pour ce test positif seulement, la sonde persisted est pilotée ; aucun backend stockage simulé et aucune affirmation d'une permission durable réellement accordée par ces profils headless.

Six tests Web : shell, transitions, modes, Tab, offline et focus réel par ArrowRight/Home/End sur les onglets. Les huit tests Worker 5.1 restent inclus : module signé, refus non signé/tampering, 100 actions, corpus wire, pointeurs invalides, mauvaise corrélation et boucle infinie interrompue sans blocage du thread UI.

Reproduction : pnpm --filter @kortex/web-adapter build copie les bundles dans runtime/web/public/adapter (généré, ignoré) ; depuis runtime/web, dx build avec CARGO_TARGET_DIR absolu vers le cache de build. Copier la sortie public de dx dans target/web-dist pour le serveur Playwright. Celui-ci ne sert plus web-adapter/dist séparément : il exerce la distribution complète. OpenSSL est utilisé uniquement par le test TLS (Git for Windows sur cet hôte).

Limites : pas de détection privée heuristique ; durable seulement si le navigateur confirme, sans garantie contre effacement ; clipboard dépend des droits natifs. ZIP .tool, téléchargement et iframe séparée déployée ne sont pas revendiqués avant 5.3/5.4.

API canonique du shell distribué : importer VerifiedTool et WebBridge depuis /adapter/host.js, partagé avec le composant hôte. Les sources TS restent modulaires ; dx rebundle les entrées publiques et les registres d’admission de deux bundles distincts ne doivent pas être mélangés.
