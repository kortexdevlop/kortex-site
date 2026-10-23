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
