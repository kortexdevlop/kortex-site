# Étape 18.0 — Menaces et frontières de confiance

28/10/2026, base 6e67be3. Terminée techniquement ; revue humaine du modèle laissée en attente.

SECURITY.md couvre guest signé hostile, archive/auteur compromis, XSS, iframe, IA/workflow, SSRF, tokens et administration ; actifs, TCB, protections et sévérités Critique/Majeure/Mineure. ARCHITECTURE.md marque les frontières ; threat-matrix.md rattache chaque menace critique active à ses protections et essais. Le gate Google absent est explicitement BLOCKED, aucun fournisseur contrôlé assimilé à Google.

cargo-audit initialement absent a été installé puis exécuté réellement (0.22.2). Trois avis ZIP et neuf avis npm ont été trouvés, les dépendances ont été corrigées : ZIP 9.0.0, Zod 4.6.6, Undici 7.30.1. Après : zéro vulnérabilité détectée par les deux scanners, sans ignore. Cinq avertissements RustSec restent exposés : paste est dans la chaîne de compilation Windows ; les autres graphes examinés ne sont pas présents pour cette cible. Revue/expiration dans SECURITY_EXCEPTIONS.md ; aucun accord du propriétaire inventé.

Tests du lot : 50 natifs (+4 repris), 37 cœur/ABI/UI, 88 API, 23 Web sécurité (+ABI repris), 8 bureau staging. Corpus finis et limites d’assurance documentés ; sources et preuves reliées par security-validation.json. Windows/Chromium seulement. Ni production, DNS, mail ou quota IA modifié. Autorisation de poursuivre 18.1/18.2 donnée par la demande d’enchaîner les trois étapes ; pas de passage à 18.3.
