# Étape 4.4 — Renderer Dioxus partagé et styles hôtes

23 octobre 2026 ; base 9d86c17. Étape 4.3 approuvée par « Oui continue ».

Livrable : crate kortex-ui basée sur Dioxus 0.7.10 déjà vérifié au prototype. UiRenderer reçoit un ValidatedUiTree immutable et un Callback<UiEvent>. UiBoundary est la frontière pour JSON non fiable : elle valide puis rend, ou affiche un message hôte accessible sans rendu partiel. Limites 1 Mio, 2 000 nodes et profondeur UI 32 ; clés globalement uniques ; conformité des séries aux labels. Aucun HTML outil exécuté, aucun style ou lien extérieur accepté depuis une tool.

Écart rédactionnel du plan : il annonce 16 types, mais le schéma FROZEN_V1 en définit 17, toolbar et tabs inclus. Le registre couvre exactement ces 17 types, sans changer le schéma. Chaque type est exercé dans le snapshot d'arborescence et dispose du refus commun explicite des propriétés invalides ; les types inconnus affichent une erreur visible.

Composants : layouts column/row/grid, card, texte, six niveaux de titres, bouton, input, textarea, select, checkbox, table, progress, alert, graphiques bar/line, toolbar et tabs. Les enfants sont rendus avec leurs clés stables. Les contrôles natifs conservent leur ordre de tabulation ; labels associés et identifiants préfixés par instance évitent les collisions entre outils. Onglets avec tablist/tab/tabpanel, sélection, panels cachés, focus mobile par flèches/Home/End et appel MountedData::set_focus ; la modification de active_key ou des clés enfants réinitialise leur sélection. Toolbar utilise un groupe nommé et l'ordre natif des contrôles, sans revendiquer un modèle ARIA toolbar partiel.

Événements : click/null, input/string, change/string ou booléen, select/clé du panel. Le type number conserve la chaîne native, autorisée par le contrat existant. Les contrôles désactivés n'émettent pas. Les événements sont testés au travers des vrais handlers du VirtualDom Dioxus et validés contre la définition event gelée. Le résultat reste à envoyer au moteur par l'adaptateur hôte des phases suivantes.

Graphiques : géométrie numérique SVG hôte uniquement, valeurs négatives/zéro et bornes finies couvertes ; labels échappés ; tableau accessible de toutes les séries. Styles neutres locaux avec tokens, focus visible, largeur mobile et palette par série. La revue des captures a permis de corriger la valeur initiale textarea en SSR et la largeur des tables.

Validation :
- cargo test -p kortex-ui --locked : 7 tests réussis.
- cargo check -p kortex-ui --locked : réussi.
- cargo check -p kortex-ui --target wasm32-unknown-unknown --locked : réussi.
- cargo fmt --all -- --check : réussi.
- node crates/kortex-ui/tests/browser.cjs : Chromium et WebKit réussis, ordre de Tab, labels, IDs uniques, valeur textarea, largeur 360 px et audit axe WCAG A/AA/2.1AA sans violation sur le corpus.
- Cinq empreintes des schémas gelés inchangées.

Les vérifications navigateur utilisent le HTML SSR réellement produit par Dioxus, pas une reconstitution. Elles ne revendiquent pas l'interactivité d'un adaptateur Web déjà livré : handlers et changement d'onglet sont exercés séparément dans VirtualDom ; la mise au focus MountedData sur un vrai adaptateur sera revalidée lors du raccordement Web/desktop. Aucun test Linux/macOS ou Firefox demandé. Pas de mock permanent.

Fichiers supplémentaires nécessaires : Cargo.lock pour verrouiller le nouveau membre ; tests/browser.cjs et tests/snapshots/tree.txt pour reproduire les critères de tabulation et snapshot ; captures et preuve JSON. La fixture HTML de test est générée dans target uniquement. Variables de compilation utilisées pour économiser le disque : CARGO_PROFILE_DEV_DEBUG=0, CARGO_PROFILE_TEST_DEBUG=0, CARGO_INCREMENTAL=0. Pour reproduire les contrôles navigateur, définir KORTEX_UI_PROOF_HTML sur le chemin absolu target/ui-4.4.html avant cargo test, puis lancer browser.cjs depuis la racine.

Statut : awaiting_approval ; commit résultat : commit contenant ce rapport. Détails : docs/evidence/ui-4.4.md.
