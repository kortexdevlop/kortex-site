# Contrats publics Kortex 1.0 — candidat

Statut : proposé pour revue, non gelé. Source normative de conception : MASTER_PLAN partie B.2 et ADR-001 à ADR-010. Date : 23 octobre 2026. Les essais ne sont pas remplacés par cette transcription.

Les mots MUST, MUST NOT, SHOULD et MAY expriment respectivement obligation, interdiction, recommandation et permission selon RFC 2119. Les owners sont des rôles responsables ; leurs titulaires humains restent à désigner par le propriétaire. Chaque bloc identifié porte une règle normative collective couvrant toutes ses clauses.

## KTX-MANIFEST-001 — owner : SDK / schemas

Le producteur et le consommateur MUST respecter intégralement le contrat suivant ; ils MUST NOT accepter les comportements explicitement interdits.

**Format `manifest.json`** : `format_version: "1.0"`, `id` slug ASCII `[a-z0-9]+(?:-[a-z0-9]+)*` (3–64), `version` SemVer, `name`, `summary`, `description`, `category`, `tags[]`, `seo:{title,description}`, `icon_path`, `abi_version:"1.0"`, `ui_schema_version:"1.0"`, `permissions[]`, `entrypoint:"module.wasm"`, `min_runtime:"1.0.0"`. Les chemins sont relatifs, sans `..` ni chemin absolu ; les textes et tailles sont bornés dans le schéma.

## KTX-ACTION-001 — owner : SDK / actions

Le producteur et le consommateur MUST respecter intégralement le contrat suivant ; ils MUST NOT accepter les comportements explicitement interdits.

**Format `contract.json`** : `contract_version:"1.0"`, `tool_id`, `actions[]` de `{id, description, input_schema, output_schema, permissions[], side_effect:"none"|"local-write"|"network", idempotent:boolean, timeout_ms}`. `action.id` unique par outil ; identifiant global d’action = `{tool_id}.{action_id}` ; schémas JSON 2020-12 **dans un sous-ensemble validable par Rust et TS** (types, propriétés, `required`, bornes, tableaux, `enum`, pas de `$ref` externe ni code).

## KTX-ABI-001 — owner : runtime / SDK

Le producteur et le consommateur MUST respecter intégralement le contrat suivant ; ils MUST NOT accepter les comportements explicitement interdits.

**ABI WASM Core Kortex 1.0** (exports guest) : `memory`; `kortex_alloc(len:i32)->i32`, `kortex_dealloc(ptr:i32,len:i32)`, `kortex_dispatch(ptr:i32,len:i32)->i64`. Résultat `i64 = (ptr_u32 << 32) | len_u32`. L’hôte alloue/copie une requête UTF-8 JSON, appelle dispatch, décode pointer/longueur, lit la réponse, libère chaque allocation selon la convention du SDK. L'import guest `kortex_v1.host_enqueue(ptr:i32,len:i32)->i32` accepte une demande de capacité JSON bornée et retourne uniquement un statut de **mise en file** (`0=acceptée`, valeurs négatives = rejet) ; la réponse asynchrone est réinjectée par un futur `dispatch` de type `capability.result`. **Aucun accès host synchrone blocant, aucun WASI implicite.** L’ABI contrôle les pointeurs, longueurs, tailles, mémoire et réentrance ; une nouvelle ABI majeure utilisera un autre module d’import.

## KTX-PROTOCOL-001 — owner : runtime

Le producteur et le consommateur MUST respecter intégralement le contrat suivant ; ils MUST NOT accepter les comportements explicitement interdits.

**Enveloppe JSON** : requête `{v:1, request_id:string, kind:"init"|"render"|"ui.event"|"action.invoke"|"capability.result"|"state.restore"|"shutdown", payload:object}` ; réponse `{v:1,request_id,ok:boolean,data?:object,error?:{code:string,message:string,retryable:boolean}}`. `request_id` est un UUID opaque, jamais utilisé comme autorisation. La fonction dispatch est séquentielle, sans réentrance concurrente ; limites initiales **projet** : requête/réponse 1 MiB, UI ≤ 2 000 nœuds, ZIP `.tool` ≤ 25 MiB, guest WASM ≤ 20 MiB, mémoire guest ≤ 64 MiB (le module déclare un maximum de 1 024 pages WASM 64KiB ; le tool-builder passe et vérifie la limite linker), action ≤ 30 s, file de 128 événements maximum. Limites ajustables uniquement via évolution documentée et tests croisés (et non contraintes garanties par les plateformes).

## KTX-UI-001 — owner : UI hote

Le producteur et le consommateur MUST respecter intégralement le contrat suivant ; ils MUST NOT accepter les comportements explicitement interdits.

**UI JSON V1** : arbre `{type, key, props, children}` ; types autorisés `column,row,grid,card,text,heading,button,input,textarea,select,checkbox,table,progress,alert,svg-chart,toolbar,tabs`. Seules les clés déclarées et des propriétés sérialisables sont permises ; pas de HTML arbitraire, de script, d’URL non vérifiée, ni de CSS provenant du paquet. Événements `{node_key,event_type,value}`. Les graphiques sont rendus par l’hôte à partir de données ; le WebView Desktop ne reçoit pas d’appel WebGPU du guest.

## KTX-CAPABILITY-001 — owner : securite / runtime

Le producteur et le consommateur MUST respecter intégralement le contrat suivant ; ils MUST NOT accepter les comportements explicitement interdits.

**Capacités V1** : `storage.read`, `storage.write` (namespace privé), `file.pick`, `file.read`, `file.save` (handles éphémères et octets bornés), `network.fetch` (origines HTTP(S) explicitement autorisées et vérification DNS/redirects côté hôte), `clipboard.write` (geste et approbation) ; `gpu.direct` absent. Un outil sans permission déclarée ne peut obtenir une capacité par simple message JSON. Aucune commande shell, socket brute, accès hors répertoire, ou exécution dynamique.

## KTX-TRUST-001 — owner : distribution / securite

Le producteur et le consommateur MUST respecter intégralement le contrat suivant ; ils MUST NOT accepter les comportements explicitement interdits.

**Preuve de publication** : objet externe `{schema_version:1,tool_id,version,abi_version,sha256,signing_key_id,signature}` où `signature` est Ed25519 sur le texte UTF-8 exactement `kortex-artifact-v1\n{tool_id}\n{version}\n{abi_version}\n{sha256_hex_lower}\n` ; ce document est signé après formation du ZIP. La signature n’est pas intégrée au ZIP ; le catalogue fournit le descripteur authentifié, dont les hôtes vérifient la signature par une clé de confiance embarquée. Rotation de clés via manifeste de clés signé par une racine hors ligne.

## KTX-CATALOG-001 — owner : API catalogue

Le producteur et le consommateur MUST respecter intégralement le contrat suivant ; ils MUST NOT accepter les comportements explicitement interdits.

**Catalogue API V1** : `{items:ToolSummary[],next_cursor:string|null}` ; pagination curseur et limite 1–100 ; ordre stable `(published_at DESC,id DESC)` ou `(name_norm ASC,id ASC)`, filtre `status=published`, cursor opaque signé ; codes normalisés `BAD_REQUEST`, `NOT_FOUND`, `CONFLICT`, `UNAUTHORIZED`, `FORBIDDEN`, `RATE_LIMITED`, `UNAVAILABLE`, `INTEGRITY_ERROR`. Tous les POST sensibles ont `request_id` et protection idempotence.

## KTX-MIGRATION-001 — owner : API / stockage

Le producteur et le consommateur MUST respecter intégralement le contrat suivant ; ils MUST NOT accepter les comportements explicitement interdits.

**Migrations** : SQL forward-only numéroté, contraintes et index explicites, intégration `schema_version`/`contract_version` aux données ; D1 n’héberge jamais le ZIP ou le WASM.

## KTX-ABI-002 — owner : runtime / SDK

Les offsets et longueurs MUST être interprétés comme u32 dans la mémoire linéaire et leurs bornes MUST être vérifiées sans débordement avant toute lecture ou écriture. Le résultat i64 MUST être décomposé sans perte de précision (BigInt en JavaScript). Les octets de réponse MUST être copiés avant libération. Chaque allocation MUST être libérée exactement une fois conformément au SDK ; aucun pointeur guest ne devient un pointeur natif. Les entrées invalides MUST être rejetées sans accès hors bornes. La convention définitive d’allocation et ses preuves appartiennent au SDK et aux prototypes, avant gel.

## Décisions normatives

### KTX-ADR-001 — owner : architecture

L’implémentation MUST respecter : WebAssembly **Core**, cible `wasm32-unknown-unknown`, même binaire Web/Desktop. Preuve et étape de gel : 3.0, 3.1 puis 2.5.

### KTX-ADR-002 — owner : architecture

L’implémentation MUST respecter : Dioxus exclusivement dans les **hôtes** ; UI déclarative JSON versionnée issue du guest. Preuve et étape de gel : 3.2, 4.4.

### KTX-ADR-003 — owner : architecture

L’implémentation MUST respecter : API Worker source unique des règles métier catalogue. Preuve et étape de gel : 9.4, 11.1.

### KTX-ADR-004 — owner : architecture

L’implémentation MUST respecter : IA et workflows agissent sur le client actif ; aucune exécution cloud de `.tool`. Preuve et étape de gel : 15.2, 16.3.

### KTX-ADR-005 — owner : architecture / securite

L’implémentation MUST respecter : Capacités hôte médiées ; **pas** de GPU guest direct en V1. Preuve et étape de gel : 4.2, 18.1.

### KTX-ADR-006 — owner : architecture

L’implémentation MUST respecter : Simple/Medium/Pro = densité/complexité visuelle ; **aucun** plan payant. Preuve et étape de gel : 10.2, 11.2.

### KTX-ADR-007 — owner : architecture / securite

L’implémentation MUST respecter : ZIP `.tool` signé via descripteur **détaché** Ed25519, hachage SHA-256. Preuve et étape de gel : 8.2, 8.3.

### KTX-ADR-008 — owner : architecture

L’implémentation MUST respecter : Versions immuables, alias `current_version` transactionnel en D1. Preuve et étape de gel : 9.5, 19.5.

### KTX-ADR-009 — owner : architecture / securite

L’implémentation MUST respecter : OIDC Authorization Code + PKCE pour clients ; session Web HttpOnly ; aucune clé longue durée en JS. Preuve et étape de gel : 14.0, 14.1.

### KTX-ADR-010 — owner : architecture / securite

L’implémentation MUST respecter : Un outil = un namespace d’état, des demandes hôte et une liste de capacités déclarées. Preuve et étape de gel : 4.2, 18.2.

## KTX-VERSION-001 — owner : architecture / maintenance

Les versions de paquet MUST suivre SemVer majeur.mineur.correctif. Le protocole JSON v:1 et les versions de format, ABI, UI et contrat MUST être vérifiés séparément ; la version du paquet ne remplace aucune de ces versions. Un correctif MUST préserver les contrats observables. Une évolution mineure MAY ajouter des champs optionnels tolérés et documentés ; elle MUST préserver les anciens messages et champs obligatoires. Un changement d’export/import ABI, retrait de champ obligatoire ou changement de sémantique d’action MUST ouvrir une version majeure et un adaptateur versionné. Les validateurs MUST appliquer la politique de champs de leur schéma : cette règle n’autorise pas des propriétés UI arbitraires.

## KTX-DEPRECATION-001 — owner : architecture / maintenance

Une dépréciation MUST être annoncée dans le journal avec surface, remplacement, migration et versions concernées. Elle MUST NOT retirer une surface gelée en 1.x. Une suppression MUST attendre une nouvelle majeure avec adaptateur et preuves de migration ; aucune durée de support non approuvée n’est promise.

## KTX-ERROR-001 — owner : runtime / API

Une erreur de dispatch MUST utiliser error.code, error.message et error.retryable dans l’enveloppe de réponse avec le request_id correspondant. Une réussite et une erreur MUST NOT être présentées simultanément. Le message MUST NOT divulguer un secret. Les erreurs catalogue MUST utiliser les codes KTX-CATALOG-001 ; le catalogue n’impose pas implicitement ces mêmes codes à toutes les erreurs guest. La sémantique des erreurs d’action MUST être documentée et testée avec le contrat correspondant.

## KTX-EXCEPTION-001 — owner : architecture / securite / proprietaire

Une modification incompatible gelée MUST arrêter la chaîne et ouvrir docs/CHANGE_REQUESTS/CR-YYYY-NNN.md avec incident, impact, contrats, options, migration, tests, compatibilité, rollback et signataires. Elle MUST obtenir les approbations architecture, sécurité et propriétaire avant réalisation. Le plan source MUST NOT être réécrit silencieusement. Une dérogation de tests MUST garder les résultats non vérifiés visibles ; elle ne constitue pas une preuve technique.

## Journal des changements

- 2026-10-23 : transcription candidate B.2 et ADR-001…010 ; règles de version, dépréciation, erreur et exception. Aucun gel ni approbation de revue humaine des noms/types/limites déclaré.

## KTX-SCHEMA-001 — owner : SDK / schemas

Précisions candidates 2.1 : les exemples canoniques sont shared/fixtures/manifest.valid.json et contract.valid.json. Les validateurs MUST préserver la distinction slug/SemVer et MUST NOT dépendre de l'ordre des clés. Ils MUST refuser les propriétés inconnues du manifeste, du contrat et du sous-ensemble de schéma d'action. Les références internes des deux schémas officiels sont locales ; un schéma input/output fourni par un outil MUST NOT contenir de $ref, externe ou récursif.

Bornes proposées pour rendre les limites textuelles B.2 exécutables : name 100, summary 240, description 10000, category 64, tag 40 (20 tags uniques), seo.title 70, seo.description 160, icon_path 256 caractères. Chaînes obligatoires non vides. Version SemVer limitée à 128 caractères. Action : id 64 caractères, motif [a-z][a-z0-9_-]* ; description 1000 ; 1 à 100 actions ; timeout_ms entier 1 à 30000. Permissions uniques parmi les sept capacités B.2. Chemins ASCII relatifs : ni segment point/point-point, ni antislash, deux-points ou double barre oblique. Aucun trim ou changement de casse implicite.

Le sous-ensemble input/output impose type et autorise properties, required, additionalProperties booléen, items, enum scalaire, minimum/maximum, minLength/maxLength et minItems/maxItems. Bornes numériques finies de magnitude au plus 9007199254740991 ; longueurs au plus 1048576 ; tableaux au plus 2000 ; 100 propriétés et 100 valeurs enum ; profondeur au plus 16. Les bornes inversées et required sans propriété correspondante sont rejetés. Ces limites supplémentaires sont proposées pour revue, sans gel.

L'unicité des action.id et la profondeur sont vérifiées en complément d'AJV dans scripts/validate-schema.mjs : le JSON Schema seul ne les garantit pas. Le runner compile les validateurs JavaScript depuis les schémas, importables par les futurs consommateurs TypeScript ; aucun fichier TypeScript généré ni package partagé n'est livré avant l'étape 2.4.

## KTX-ABI-003 — owner : runtime / SDK — implémentation candidate 2.2

La crate portable kortex-abi expose RequestKind, Envelope, Reply et HostError. Les sept kinds B.2 sont sérialisés avec leurs noms exacts. Le codec vérifie v=1, UUID textuel 8-4-4-4-12 hexadécimal opaque, payload/data objets, réponse cohérente et taille UTF-8 JSON de 1 à 1048576 octets. Une erreur requiert code et message non vides ; les codes d'erreur guest restent extensibles. Les champs inconnus sont rejetés dans cette version candidate.

Les constantes MAX_* reprennent les plafonds de B.2. pack/unpack conservent les 64 bits du résultat i64, y compris son bit de signe ; ptr et len sont des u32. checked_range refuse longueur nulle, message trop long, mémoire excédant 64 MiB, addition débordante et plage hors mémoire. L'offset zéro reste valide ; aucune conversion vers un pointeur natif n'est fournie. Cette vérification de plage porte sur les messages JSON, pas sur la sémantique d'un allocateur guest.

EnqueueStatus accepte 0 comme mise en file acceptée et tout négatif comme rejet ; un code positif est inconnu et rejeté. Aucune attribution arbitraire de sous-codes négatifs. Aucun Wasmtime, WASI ou Dioxus dans cette crate. Les effets d'alloc/dealloc et le contrôle de réentrance restent à implémenter dans le SDK et les hôtes.

## KTX-UI-002 — owner : UI hôte — candidat 2.3

Les 18 types B.2 possèdent chacun un objet props fermé défini dans ui.schema.json. Tout nœud MUST contenir type, key, props et children. Les clés sont stables et uniques dans l'arbre, de 1 à 64 caractères ASCII selon [A-Za-z][A-Za-z0-9_-]*. Aucune propriété HTML, CSS, style ou URL n'est acceptée. L'hôte MUST traduire les props vers ses composants sûrs. Input, textarea, select, checkbox et progress exigent label ; les boutons exigent text ; aria_label permet une précision accessible. L'hôte MUST assurer navigation clavier et annonce des contrôles ; le schéma seul ne prouve pas leur accessibilité.

Bornes candidates : 2000 nœuds au total, profondeur 32, écarts layout 0–32, grille 1–12 colonnes. svg-chart MUST avoir kind bar/line, labels (au plus 500), series (au plus 20) de label/values ; chaque série MUST avoir autant de valeurs que de labels. Chaque valeur numérique MUST être finie dans la plage sûre définie dans les schémas. Le renderer MUST construire les graphiques depuis ces données, sans SVG brut fourni par le guest. Les événements déclarés dans $defs/event sont node_key, event_type (click/input/change/select), value scalaire ; le runtime MUST vérifier la clé cible et l'événement autorisé par le contrôle.

## KTX-WORKFLOW-001 — owner : orchestration locale — candidat 2.3

Workflow 1.0 : id, name, nodes et edges. Chaque nœud MUST posséder id unique, action_ref sous forme tool-id.action_id, input_bindings et requires_confirmation booléen. Maximum 100 nœuds et 1000 arêtes ; aucune arête dupliquée, cyclique ou vers un nœud absent. Chaque input_binding associe un JSON Pointer cible soit à {value:...}, soit à {from_node,source_pointer}. Les pointeurs suivent l'échappement ~0/~1 ; le pointeur vide désigne la racine. Une source MUST être reliée au consommateur par une arête explicite. Les cycles sont interdits en V1.

Le moteur MUST valider action_ref contre les contrats réels puis les entrées et sorties contre leurs schémas ; les pointeurs MUST se résoudre dans les données. Une sortie hors schéma MUST arrêter l'action selon la politique d'erreur. La validité structurelle d'un workflow ne prouve ni disponibilité d'un outil ni validité d'un résultat. requires_confirmation MUST NOT remplacer les permissions et consentements locaux.

## KTX-PROTOCOL-002 — owner : runtime — candidat 2.3

protocol.schema.json décrit requêtes et réponses B.2. capability.result utilise payload {capability_request_id,ok,data?,error?} ; capability_request_id est l'UUID opaque de corrélation de la demande hôte, jamais une autorisation. ok=true interdit error ; ok=false exige error et interdit data. Les autres payloads restent des objets à spécialiser dans les étapes responsables ; aucune validation de payload métier complète n'est revendiquée ici. Les codecs Rust de 2.2 ne spécialisent pas encore les payloads : alignement partagé prévu en 2.4.

Le runner applique les invariants d'arbre et de DAG en complément des schémas ; un validateur JSON Schema seul MUST NOT être présenté comme vérifiant ces invariants globaux.

## KTX-CONFORMANCE-001 — owner : architecture / runtime

La suite tests/conformance/v1.cases.json MUST rester indépendante du code des runtimes. Le runner valide les traces et la négociation sans instanciation ; les hôtes réels MUST ultérieurement fournir des traces et preuves comparables. UNSUPPORTED_ABI MUST être retourné avant instanciation pour une ABI non prise en charge. Les codes et priorités de négociation suivent adr/0006-versioning.md.

Un échange MUST commencer par init réussi, puis permettre render/action.invoke et les autres messages admis ; un second init et tout message après shutdown sont refusés. Les request_id MUST être uniques dans la trace et les réponses MUST reprendre l'identifiant reçu. Les entrées et sorties d'action MUST correspondre au schéma déclaré, même si l'enveloppe JSON est valide. Une capacité MUST être connue, déclarée et accordée ; aucune chaîne fournie par le guest ne vaut consentement.

Le gel de la portabilité et des hypothèses runtime MUST attendre les preuves P1–P5 prévues par le plan. Le corpus local ne prouve ni allocation, ni interruption, ni sécurité cross-host réelle. Toute mise à jour gelée MUST suivre la procédure exceptionnelle, les approbations tracées et la politique de signature des artefacts/clés ; aucune signature cryptographique du corpus initial n'est revendiquée.

## KTX-ARCHIVE-001 — owner : distribution / sécurité — précisions candidates 3.3

Les plafonds B.2 restent inchangés : ZIP 25 MiB et module WASM 20 MiB. Le prototype ajoute un budget de décompression de 64 MiB total, 20 MiB par entrée, 128 entrées et ratio maximal 100:1. Ces valeurs sont des politiques candidates de validation d'archives, pas une garantie de consommation mémoire totale du processus. L'implémentation finale MUST borner les sorties réelles du décompresseur, vérifier longueur/CRC et cohérence local/central, et refuser extraction hors namespace ou liens symboliques.

Le banc 3.3 accepte volontairement un sous-ensemble ZIP strict : méthodes stored/deflate, noms ASCII relatifs déclarés, entrées uniques, aucun chiffrement, descriptor de données, ZIP64 ou multidisque. Ce sous-ensemble expérimental ne définit pas encore tous les octets du ZIP canonique de l'étape 8. La preuve de signature est externe à l'archive et vérifiée avant décompression ; les clés éphémères des tests MUST NOT être réutilisées comme clés de confiance produit.

Le runtime iframe MUST utiliser une origine distincte et vérifier event.origin, event.source et nonce de session. sandbox allow-scripts allow-same-origin allow-downloads n'est admis que sur cette origine distincte. Tout postMessage MUST cibler l'origine exacte ; aucune permission de navigation top n'est accordée. La CSP de prototype restreint les scripts inline par nonce, Worker/connect à self et frame-ancestors au parent attendu. La CSP de production sera construite aux étapes responsables.

## État du gel à la revue 3.4 — 23 octobre 2026

Tous les blocs restent candidats. Aucun statut FROZEN_V1 attribué. L'audit a reproduit trois divergences entre le codec Rust et protocol.schema.json ; voir evidence/ARCHITECTURE_GATE.md et CHANGE_REQUESTS/CR-2026-001.md. Toute décision GO et gel est suspendue jusqu'à correction, nouvelle preuve Web/native sur le binaire réidentifié, et approbation humaine du gate. Les autorisations antérieures de poursuivre ne sont pas une attestation de conformité des messages découverts ici.

Reprise CR-2026-001 du 23 octobre 2026 : le codec Rust refuse désormais data/error explicitement null et spécialise capability.result selon KTX-PROTOCOL-002. L’absence optionnelle des champs est préservée. La divergence initiale du gate 3.4 est résolue avec 46 cas croisés et nouveaux essais Web/native. Les contrats restent candidats ; décision humaine de gel toujours attendue.

## FROZEN_V1 — approbation explicite du 23 octobre 2026

Le propriétaire a répondu « Oui continue » à la demande explicite de validation de la revue 3.4 corrigée et du gel avant SDK 4.0. Le gel couvre ABI 1.0 (signatures, module d'import et conventions B.2), enveloppes protocole et les cinq schémas existants ci-dessous. Aucun nouveau schéma futur ni comportement non spécifié n'est gelé par anticipation. Les plateformes omises et limites de prototype restent celles du gate. Les évolutions suivent SemVer et la procédure de changement déjà définie. Pas de signature cryptographique inventée.

| Schéma gelé | SHA-256 source |
|---|---|
| shared/schemas/manifest.schema.json | c63ea6b9cfd1bfe07bd66e7ac8d3c6298868ffc3c2a06889c52edd00be6f9142 |
| shared/schemas/contract.schema.json | e6b34131ecff08083ab7112db8d38ced5374822d9d69753e17c7b23e307b65b6 |
| shared/schemas/ui.schema.json | 3ca83e92605639c261b97ab8694bc249c7c3e2999123be552060e8c58013b55b |
| shared/schemas/protocol.schema.json | c1a289b65c13894ab8d8953c6ef6b4e1a5de884275923ad1b32e8b440dc637cf |
| shared/schemas/workflow.schema.json | 28fdaa1b42a5d2fadc3b8d6fc81db7faacfeb208508c2da22b06eb3ab53dbb6d |

### Clarification d'ownership 4.0 — aucune modification d'ABI

L'hôte possède l'allocation de requête obtenue par kortex_alloc et MUST la libérer avec son ptr/len d'origine après copie et dispatch. Le SDK possède l'allocation de réponse jusqu'au retour ; l'hôte MUST copier les octets puis appeler kortex_dealloc une seule fois avec ptr/len retournés. Le SDK ne libère pas implicitement la requête. L'import host_enqueue MUST copier ses octets pendant l'appel ; son pointeur emprunté expire au retour et ne peut pas servir d'autorisation. La réponse de capacité arrive dans un futur dispatch, corrélée par capability_request_id ; le request_id de l'enveloppe de réponse reste celui du dispatch courant.

Le SDK tient un registre exact des buffers ; il refuse lecture et libération sur pointeur inconnu ou longueur différente. Un dealloc invalide est ignoré sans dereference ni double libération. Un alloc invalide/refusé retourne 0, que l'hôte MUST vérifier. Si aucune allocation de réponse ne peut être produite, l'instance WASM termine en trap ; aucun pointeur forgé n'est rendu. Les erreurs Tool natives sont interceptées avant frontière FFI ; un panic du code guest compilé avec abort produit un trap WASM, sans unwind vers l'hôte. L'hôte devra abandonner l'instance interrompue, sans revendiquer de Reply disponible dans cette situation.

## Mise en œuvre native et déclaration 7.0 — 27 octobre 2026

Aucun schéma ou ABI gelé modifié. Le format de record natif version 1 est interne au backend ; il ne crée pas de capacité publique. Les droits restent ceux du manifeste et les CapabilityRequest SDK existantes. Les extensions Rust internes (rendu après callback, lecture de décision de consentement) ne modifient pas les enveloppes. Le contrat calculatrice est une instance valide du schéma gelé et non une exception ; son implémentation reste en 7.1.

## Référence de compatibilité outil 1 — 7.3

ABI 1.0, UI 1.0 et cinq schémas gelés restent inchangés. tests/golden/calculatrice.ndjson contient 13 échanges Request/Reply exécutés par le SDK, le Worker Web et Wasmtime sur le vrai guest. tests/golden/calculatrice-reference.json consigne les SHA-256 du module, du ZIP signé de test et du corpus. tests/fixtures/calculatrice.test.cjs vérifie empreintes et signature B.2 ; le test natif réadmet le ZIP et refuse son altération. Ces références sont à conserver dans la validation des futurs packagers/publications ; elles ne doivent pas être régénérées automatiquement pendant un test.

Les erreurs d’action conservent INVALID_INPUT et DIVIDE_BY_ZERO. Un événement UI reconnu est acquitté même si le calcul échoue, puis render expose l’erreur dans un alert. Aucun nouveau message, événement clavier, propriété UI ou permission. La correction de sérialisation Rust→JavaScript Web porte sur la représentation des objets déjà prescrite, sans changement du wire JSON. Guide : TOOL_SDK.md ; gate : reports/PHASE7_GATE.md.

## Paquet et descripteur candidats — 8.0 à 8.2

Aucun des cinq schémas FROZEN_V1 n’est modifié. artifact.schema.json formalise les sept champs B.2, leurs identifiants/versions, SHA hexadécimal minuscule, key ID et signature Ed25519 base64 canonique. Les champs inconnus sont refusés. Ce nouveau schéma n’est pas encore gelé ; PACKAGE_FORMAT_V1 relève de 8.5.

Le packageur produit un ZIP lexical stored, timestamps DOS 1980-01-01 et permissions 0644, métadonnées JSON triées. Les plafonds de publication sont 25 Mio ZIP et cumul décompressé, 20 Mio par fichier, 256 entrées et ratio 20:1. Le parseur refuse les ambiguïtés ZIP, liens, traversals, doublons/collisions de casse et CRC incohérents avant toute utilisation des fichiers. Les rapports sont sérialisables, avec codes et chemins. Ces règles candidates remplacent les valeurs de prototype pour le nouveau pipeline de publication, sans revendiquer une migration de tous les anciens lecteurs des runtimes.

La signature Ed25519 reste sur le texte B.2 terminé par LF, après hash SHA-256 du ZIP final. La chaîne de clés utilise un snapshot SignedKeys v1 signé par racine hors ligne, dates en ms UNIX et version minimale fournie par l’hôte ; canonisation et limitations sont décrites dans SIGNING.md. La signature du snapshot ne suffit pas à autoriser un rollback si l’hôte connaît déjà une version supérieure. Il reste responsable du maintien de ce minimum et de son ancre. Les fixtures et ancres TEST sont uniquement des preuves locales, sans confiance de production.

## PACKAGE_FORMAT_V1 — gate local 8.5, 27 octobre 2026

Le lot demandé « Enchaîne les 4 prochaines étapes » couvre 8.3, 8.4, le gate 8.5 et 9.0. Son autorisation couvre ce gel local et la poursuite du lot ; elle ne constitue pas une revue de sécurité indépendante. Cette section remplace l'état candidat ci-dessus pour le format paquet et le descripteur. Les cinq contrats FROZEN_V1 antérieurs restent inchangés. Preuves : reports/PHASE8_GATE.md et tests/golden/calculatrice-artifact.json.

| Schéma gelé | SHA-256 source |
|---|---|
| shared/schemas/artifact.schema.json | 3712fe66d1ff069315b19e87fae71ffe7d523b9843ff1caed02632af84f7926c |

Le producteur PACKAGE_FORMAT_V1 MUST produire un ZIP stored, ordre lexical ASCII des chemins, timestamps DOS 1980-01-01 00:00:00, fichiers réguliers Unix 0644, aucun champ extra/commentaire, aucun dossier explicite. Les deux JSON sont UTF-8 avec clés récursivement triées, nombres selon serde_json verrouillé et aucun whitespace ajouté. Les fichiers sont manifest.json, contract.json, module.wasm et l'icône non vide déclarée sous assets/. Toute extension de ce contenu demande une évolution documentée du format, pas une exception par outil.

Les limites gelées du pipeline sont ZIP ≤25 Mio, cumul extrait ≤25 Mio, fichier ≤20 Mio, métadonnée ≤1 Mio, ≤256 entrées, ratio ≤20:1. Les lecteurs vérifient les budgets avant et pendant la décompression, CRC, bornes, cohérence des en-têtes et absence de données cachées. Ils refusent liens, chemins dangereux/réservés, collisions, chiffrement, multidisque, ZIP64 et data descriptors. Ils peuvent admettre l'ancien sous-ensemble sûr stored/deflate et ses dates, sans exiger que chaque ancienne archive soit canonique ; les nouveaux producteurs/publications MUST être canoniques. Aucun branchement par identifiant d'outil.

Le WASM MUST respecter ABI 1.0 et les signatures gelées, mémoire unique 32 bits non partagée avec maximum déclaré ≤1 024 pages, aucun start ni WASI. L'import host_enqueue est optionnel et unique ; les seuls exports supplémentaires tolérés sont les globals de compilation __heap_base et __data_end. L'inspection est statique, suivie de la validation complète du module, avant instanciation.

Le descripteur détaché MUST contenir exactement les sept champs du schéma ; sa signature Ed25519 porte sur le texte B.2 UTF-8 terminé par LF. Le hash SHA-256 couvre tous les octets du ZIP final. L'identité (tool_id, version) MUST désigner à vie ces mêmes octets et ce SHA ; aucun remplacement sous la même version. Les anciennes versions restent adressables par leur hash. Un rollback déplace le pointeur de catalogue vers une version autorisée ; il ne réécrit pas l'artefact et ne contourne pas une révocation de clé.

SignedKeys v1 et son texte canonique décrit dans SIGNING.md sont la chaîne de confiance du gate : ancre racine épinglée par l'hôte, snapshot signé, version minimale et fenêtres temporelles, révocations. Le paquet ne choisit jamais sa racine. Le maintien durable du minimum et le canal distant restent des responsabilités hôte à implémenter aux étapes prévues. Les clés TEST du gate ne confèrent aucune confiance de production. Toute évolution incompatible de PACKAGE_FORMAT_V1 exige une nouvelle version et une migration explicite ; les six empreintes et la référence sont contrôlées par tests/publication/golden.test.mjs.

## API candidate — préparation locale 9.2

api.schema.json ajoute les DTO API V1 santé/erreurs, outils/détail/actions, pagination et publication. Ce septième schéma n'est pas gelé ; les six empreintes précédentes restent inchangées. Les références aux champs manifest/contract/artifact réutilisent leurs définitions gelées. Le schéma ne remplace pas les contrôles d'identité croisée, de signature, de curseur signé ou d'état côté service.

Seule /api/v1/health est implémentée à ce stade, avec GET/HEAD ; les routes catalogue/publication restent à leurs étapes. Les erreurs contiennent exactement error:{code,message,request_id}, sans trace ni configuration. Le request_id corrèle, sans autoriser. La migration 0002 ajoute des invariants internes de stockage sans modifier l'ABI ou les payloads gelés. L'étape 9.2 reste en attente d'accès Cloudflare suffisants à la demande du propriétaire ; aucune nouvelle validation distante revendiquée.

## Implémentation catalogue 9.3

L'état d'implémentation ci-dessus décrit le point initial 9.2. Les GET/HEAD liste, détail et actions existent maintenant sous /api/v1/tools, selon les DTO candidats inchangés. Les accès Cloudflare ont été rétablis puis l'ancien projet supprimé à la demande du propriétaire, avec exception pour la messagerie. Le nouveau catalogue demeure local, sans provisionnement distant. Les paramètres (dont tag exact unique), le curseur HMAC serveur et le cache conditionnel sont décrits dans api/README.md. Aucun schéma public ou format gelé n'est modifié ; les lectures ne remplacent pas la vérification de publication du futur service.

Amendement du 28/10/2026 (clôture 12.3–12.5 demandée par le propriétaire) : allow-downloads permet la capacité file.save après consentement explicite du courtier de permissions. Les scripts des outils restent exécutés dans le Worker Wasm ; aucun accès direct au DOM, aucune navigation top ni ouverture de fenêtre supplémentaire.
# Ajouts 14.5–15.1 — sessions d’appareil et propositions IA

Les contrats existants restent compatibles. Le nouveau schéma `shared/schemas/ai-plan.schema.json` décrit `{v:1,steps:[{tool_id,action_id,input,requires_consent:true}],summary}` : dix étapes et 32 Kio maximum. Ce plan est une proposition ; aucune invocation de module dans le Worker. La validation d’exécution locale et sa revue ne sont pas livrées avant 15.2.

Amendement 15.2–15.4 : la revue et l’exécution locales sont maintenant implémentées, sans changer ce schéma. Les décisions sont à usage unique, expirent après cinq minutes et n’accordent aucune capacité. Le dispatcher universel demeure l’unique voie d’invocation. Le droit `ai:plan` d’un appareil se donne/reprend séparément par `PATCH /api/v1/me/devices {id,ai_allowed}`, session Web propriétaire/CSRF ; migration 0010 avec valeur initiale zéro pour tous les appareils. Le natif ne peut pas modifier cette autorisation. Voir `AI_POLICY.md` pour le périmètre et les limites d’exécution sensible.

`POST /api/v1/ai/plan` accepte uniquement `{message,available_tool_ids,consent_context:{ai_allowed:true,contains_private_data:false}}`, sous session Web/CSRF. Pas de fichiers ou champs supplémentaires, pas de comptes invités, pas d’accès via jeton Desktop. Les schémas d’entrée publiés sont contrôlés indépendamment au retour du fournisseur. Voir `AI_POLICY.md` pour quotas et conservation.

Les nouveaux endpoints `/api/v1/auth/desktop/{start,request,approve,token,logout}` assurent le lien navigateur/native avec S256, state et code à usage unique. `request` et `approve` passent exclusivement par le binding privé Web, sous session cookie ; `start`, `token` et `logout` exigent un client natif sans Origin. Les sessions d’appareil opaques n’élargissent pas les droits des anciens ID tokens Google. Elles permettent uniquement l’identité et la synchronisation opt-in, sans export/purge/IA. `/api/v1/me/devices` liste/révoque les appareils de la session Web. L’export inclut leur métadonnée sans hash/token ; la purge cascade sur les sessions d’appareil.
