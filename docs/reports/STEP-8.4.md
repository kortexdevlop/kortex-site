# Étape 8.4 — Préparation locale de publication

Date : 2026-10-27. Base du lot : 02b0f6d. Résultat : 8ffb37d. Prérequis 8.3 satisfait dans ce lot. Autorisation : propriétaire, « Enchaîne les 4 prochaines étapes ».

release-tool.mjs orchestre le CLI générique build→validate→package→hash→sign→verify et crée PublicationRequest, ancre/snapshot publics et rapport dans un staging sous target/. La sortie machine du CLI fournit le manifeste et le contrat effectivement contenus dans le ZIP. La requête est validée par les schémas manifest, contract et artifact puis par cohérence croisée. La clé R2 est une chaîne de destination, sans upload. Le dry-run utilise de vraies signatures avec clés TEST en mémoire et documents publics expirant après 24 h. Aucun secret privé n'est conservé en fichier ou log.

| Commande | Date / environnement | Sortie | Résultat |
|---|---|---:|---|
| node scripts/release-tool.mjs --tool calculatrice --dry-run | 2026-10-27, Windows/Node 24.15/Rust, Cargo offline | 0 | Paquet signé/vérifié, remote_writes=0 |
| node --test tests/publication/local-pipeline.test.mjs | même environnement | 0 | Deux répertoires indépendants : mêmes octets et SHA ; répétition idempotente sans nouvelle signature |
| node --test tests/publication/golden.test.mjs | Windows/Node | 0 | Référence paquet et six schémas gelés inchangés |

SHA du paquet : 1d4c1a0967b583489f69921d80d1c062dbd57ecb0d9dabea8337575e4ab32787. Les profils TEST/CI, UUID, clés R2 et identités incohérents ainsi que les conflits de versions sont refusés. Le packageur n'écrase pas un .tool existant avec des octets différents. Les clés éphémères font varier la signature entre deux stagings, pas le ZIP ; la répétition d'un même staging conserve UUID et preuve.

shared/fixtures/publication-request.json est un exemple public utilisant la preuve TEST permanente déjà vérifiée par Rust/Web. TOOL_PUBLISHING.md décrit usage, récupération d'un staging incomplet et rollback futur par pointeur. Il ne revendique ni API, ni R2, ni D1 de publication déployés. Preuve structurée : docs/evidence/validation-8.3-9.0.json. Critère local satisfait.
