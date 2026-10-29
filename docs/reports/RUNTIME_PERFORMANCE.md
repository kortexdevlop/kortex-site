# Runtime : 20.2 — 29 octobre 2026

Implémentation 1c77871215fd405d02835d55f90c1d4890b355f9 après ff6764085e60434966492c769cefd12e0201da7a. Windows x64, Chromium 156/Playwright 1.64.1, Wasmtime 50. CPU Intel(R) Core(TM) i7-2600K CPU @ 3.40GHz, 8 threads, RAM 25713045504 octets, Windows 10.0.22621.

Cinq archives signées réelles, mêmes SHA256 Web/Windows ; 10 lancements froids et 100 actions après 5 échauffements par outil. Actions vérifiées avec les vecteurs partagés. Distributions complètes et verify/instantiate dans les JSON.

| Outil | Archive octets | Windows froid p50 ms | Windows froid p95 ms | Windows action p95 ms | Web froid p50 ms | Web froid p95 ms | Web action p95 ms |
|---|---:|---:|---:|---:|---:|---:|---:|
| calculatrice | 264845 | 492.07 | 594.95 | 1.55 | 86.20 | 98.00 | 0.50 |
| convertisseur-casse | 258718 | 498.90 | 826.49 | 1.55 | 68.60 | 76.80 | 0.40 |
| calculateur-hash | 299737 | 574.16 | 751.82 | 1.56 | 70.30 | 80.70 | 0.40 |
| timestamp-utc | 274751 | 527.96 | 541.35 | 1.58 | 82.90 | 103.10 | 0.40 |
| explorateur-csv | 441418 | 826.74 | 849.34 | 1.52 | 77.10 | 91.60 | 0.50 |

Budgets respectés : froid ≤3000 ms, action ≤100 ms, guest ≤64 MiB. Guest natif observé : 1179648 octets. Profil desktop-release optimisé, inherits release et strip false, réglages Dioxus Desktop. Lectures locales natives ; téléchargement HTTP loopback Web, pas une mesure réseau public. Chromium neuf par lancement, démarrage du navigateur exclu ; cache disque OS non purgé. Aucun cache ajouté sans gain mesuré.

Huit instances natives : RSS 27897856 octets. Cinquante cycles, enveloppe RSS 27963392 → 27971584 (+0.03 %). Méthode : p95 de huit échauffements identiques, comparé au maximum du RSS final et du p95 des dix derniers cycles. Toutes les valeurs restent jointes. Les arènes JIT Windows font osciller le RSS ; la photographie unique initiale comparait des états non équivalents. Deux calibrations échouées conservées (before-shutdown-fix et single-snapshot). Le seuil reste 10 %. La fermeture attend maintenant les threads guest/timer après interruption et révocation. Treize tests natifs ciblés passent : annulation, guest hostile, limites, vrais fichiers et cinq outils.

Web : 50 cycles, tas JS principal après GC +3.88 %, zéro Worker retenu. Cette métrique n’est pas le RSS total du navigateur. Huit fenêtres réelles, API/D1/R2/Runtime locaux sans mock, aucune erreur page/crash et rAF <500 ms. Premiers relevés sur shell 1.0.1 avec adaptateur reconstruit ; bundle immuable 1.0.2 compilé ensuite, non publié. Nouvelle exécution complète locale enregistrée séparément si ses mesures changent. Paquets inchangés, descripteurs locaux signés par clé éphémère ; publication et vérification API réelles.

Installateur : preuve historique 19.3, 225520032 octets ; aucun nouvel installateur livré ici. Exclusions Linux/macOS/Firefox maintenues. Commandes : node scripts/measure-native.mjs ; node scripts/qa-verify.mjs après préparation fixtures/builds. Preuves runtime-native-performance.json, runtime-web-performance*.json et runtime-eight-windows.png. Staging non clos : D1_QUOTA_INCIDENT.md.


Exécution complète finale avec bundle 1.0.2 : node scripts/qa-verify.mjs réussi (33 interface + 7 performance). Rapports courants issus de cette exécution ; mesures antérieures conservées dans runtime-web-performance-prior-bundle.json. Les processus locaux créés par le runner ont été arrêtés automatiquement. ADR : docs/adr/0012-runtime-measurements.md.
