# Rapport 12.3 — Explorateur CSV : contrat et parseur borné

27/10/2026, Windows. Base d1450f0. Commit dans PROGRESS.md ; lot « Enchaîne les 3 prochaines étapes » (12.3, 12.4, 12.5), validation humaine formelle non inventée.

Nouvelle crate tool-explorateur-csv, manifeste, icône et contrat (`summarize_csv`, `export_report`, 1 000 000 d’octets maximum par action). Parseur CSV strict écrit sans dépendance (la crate `csv` n’est pas dans le verrou hors ligne) : UTF-8, BOM, LF/CRLF, guillemets entiers uniquement, 5 000 lignes, 50 colonnes, 10 Mio. Refus explicites : UTF-8 invalide, guillemet mal placé ou non fermé, ligne irrégulière, CR seul, caractère de contrôle, fichier vide, dépassement. Statistiques par colonne (valeurs, vides, min, max, moyenne) avec politique de nullité documentée ; aucun NaN ni infini en sortie JSON (débordement refusé).

Défaut trouvé et corrigé pendant l’étape : un analyseur à tampon complet dépassait le budget de 100 M d’instructions Wasm sur un fichier de 880 Ko (`Timeout` dans Wasmtime). Le parseur est devenu un flux qui consomme chaque bloc de 256 Kio, avec cellules dans une arène unique et statistiques accumulées au passage ; la statistique « distincts » a été retirée. Les résultats sont identiques quel que soit le découpage en blocs (testé de 1 à 64 octets, y compris caractères multi-octets, CRLF et guillemets à cheval).

Tests Rust (tests/parser.rs, 9) : guillemets/retours/BOM, 11 entrées hostiles, bornes exactes, nombres stricts et débordements, statistiques dorées, actions JSON, injection de formule, indépendance du découpage, chemin rapide décimal comparé bit à bit à `str::parse`. Preuves : tools/explorateur-csv/tests/parser.rs. Statut formel pending ; 12.4 suit dans le même lot.
