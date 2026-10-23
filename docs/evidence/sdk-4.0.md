# SDK 4.0 — preuve locale

23 octobre 2026. Rust/Cargo 1.99.1, Node 24.15.0, Windows. Base 8a57069.

Résultats : cargo test -p kortex-sdk --locked : sept tests intégration et un exemple de documentation réussis ; cargo check -p kortex-sdk --target wasm32-unknown-unknown --locked : code 0. Régression workspace : neuf tests ABI, sept SDK, deux shared et un doctest réussis. Contrats croisés : 46 cas identiques, trois tests Node réussis.

Les tests natifs couvrent 1000 actions, erreurs/corrélation, panic Tool intercepté, ownership, 20000 couples pseudo-aléatoires pointeur/longueur (corpus déterministe), file limitée à 128 demandes, refus transport et callbacks hors ordre avec rejet du replay terminé. Les stubs de transport native sont exclusivement des tests unitaires, sans service réel ni effet de capacité revendiqué. Les IDs doivent être uniques dans la session ; le SDK ne confère aucune autorisation par ID.

Un guest cdylib de diagnostic a effectivement utilisé export_tool! et Host::default : build WASM release avec --max-memory=67108864, puis instanciation sous le moteur WebAssembly Node. SHA-256 de ce module SDK distinct du prototype 3.0 : d18ce8965275a522566ba91f4e36d2a36d6647fc809693d16cf6d63d6bc986b1. Aucun remplacement des preuves de portabilité 3.0/3.1 par ce nouveau module.

Sortie réelle :

```text
SDK WASM exports, 10000 dispatches, double-free rejection, invalid pointer and async capability correlation: PASS
[{"module":"kortex_v1","name":"host_enqueue","kind":"function"}]
```

Le smoke test appelle dealloc deux fois sur la même allocation ; la seconde libération est ignorée, sans dereference, puis le guest continue. Son transport JS accepte la mise en file, sans exécuter de storage.read. Le résultat est réinjecté par capability.result et le second résultat pour le même pending ID est rejeté. Ce n'est pas un service de stockage implémenté.

Les offsets natifs de BufferArena sont virtuels pour les tests ; aucun pointeur natif 64 bits n'est tronqué puis exporté. Sur WASM, le registre possède les vraies allocations ; il contrôle pointer/len exacts et budget de buffers avant copie. Les pointeurs restent des offsets guest 32 bits. Le budget de buffers n'est pas une mesure de mémoire totale ; le tool-builder doit imposer le maximum de mémoire du module.

Panic Tool natif : catch_unwind dans le dispatcher avant toute FFI. WASM panic=abort : trap de l'instance, pas d'unwind vers l'hôte et pas de Reply garantie. Épuisement de mémoire de réponse : trap, jamais un pointeur inventé. L'hôte devra arrêter/remplacer l'instance après trap. Aucun accès WASI/OS importé dans le guest vérifié.

## Reproduction du smoke test de diagnostic

Les fichiers suivants sont conservés localement sous .git/probe ; leur contenu intégral est donné ici pour une reproduction depuis un checkout propre. Le SDK et ses sept tests versionnés constituent la validation principale de l'étape.

### .git/probe/sdk-smoke/Cargo.toml

```toml
[package]
name="kortex-sdk-smoke"
version="0.0.0"
edition="2024"
[workspace]
[lib]
crate-type=["cdylib"]
[dependencies]
kortex-sdk={path="../../../crates/kortex-sdk"}
serde_json="=1.0.154"
[profile.release]
panic="abort"
```

### .git/probe/sdk-smoke/src/lib.rs

```rust
use kortex_sdk::*;
#[derive(Default)]struct Guest{host:Host}
impl Tool for Guest {
 fn handle(&mut self,r:Request)->Reply{
  if r.kind==RequestKind::Init {
   let pending=self.host.enqueue(CapabilityRequest{request_id:"00000000-0000-4000-8000-000000000002".into(),capability:"storage.read".into(),payload:Default::default()});
   return success(r.request_id,serde_json::json!({"queued":pending.is_ok()}).as_object().unwrap().clone())
  }
  success(r.request_id,serde_json::json!({"result":1200}).as_object().unwrap().clone())
 }
 fn host(&mut self)->Option<&mut Host>{Some(&mut self.host)}
}
kortex_sdk::export_tool!(Guest);
```

### .git/probe/sdk-smoke.cjs

```javascript
const fs=require('fs');const assert=require('assert/strict');
(async()=>{let queued;const bytes=fs.readFileSync('.git/probe/sdk-smoke/target/wasm32-unknown-unknown/release/kortex_sdk_smoke.wasm');const module=await WebAssembly.compile(bytes);let instance;instance=await WebAssembly.instantiate(module,{kortex_v1:{host_enqueue(p,n){queued=JSON.parse(new TextDecoder().decode(new Uint8Array(instance.exports.memory.buffer,p>>>0,n>>>0)));return 0}}});const e=instance.exports;const id='00000000-0000-4000-8000-000000000001';
function read(result){const bits=BigInt.asUintN(64,result),p=Number(bits>>32n),n=Number(bits&0xffffffffn);const reply=JSON.parse(new TextDecoder().decode(new Uint8Array(e.memory.buffer,p,n).slice()));e.kortex_dealloc(p,n);return reply}
function call(kind,payload={}){const input=new TextEncoder().encode(JSON.stringify({v:1,request_id:id,kind,payload}));const p=e.kortex_alloc(input.length)>>>0;new Uint8Array(e.memory.buffer,p,input.length).set(input);const reply=read(e.kortex_dispatch(p,input.length));e.kortex_dealloc(p,input.length);e.kortex_dealloc(p,input.length);return reply}
assert.equal(call('init').data.queued,true);assert.equal(queued.capability,'storage.read');assert.equal(call('capability.result',{capability_request_id:queued.request_id,ok:true,data:{}}).ok,true);assert.equal(call('capability.result',{capability_request_id:queued.request_id,ok:true,data:{}}).error.code,'UNKNOWN_CAPABILITY_RESULT');for(let i=0;i<10000;i++)assert.equal(call('action.invoke').data.result,1200);assert.equal(read(e.kortex_dispatch(-1,1)).error.code,'INVALID_BUFFER');assert.equal(e.kortex_alloc(-1),0);console.log('SDK WASM exports, 10000 dispatches, double-free rejection, invalid pointer and async capability correlation: PASS');console.log(JSON.stringify(WebAssembly.Module.imports(module)));})();
```

Depuis la racine, créer les dossiers puis ces fichiers ; fixer RUSTFLAGS à -C link-arg=--max-memory=67108864, exécuter cargo build --manifest-path .git/probe/sdk-smoke/Cargo.toml --target wasm32-unknown-unknown --release, supprimer cette variable de la session, puis node .git/probe/sdk-smoke.cjs. Les deps exactes sont celles du Cargo.toml montré ci-dessus ; le module est un diagnostic local non publié.
