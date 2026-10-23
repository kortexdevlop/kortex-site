# Architecture gate 3.4 — 23 octobre 2026

Décision technique proposée : **NO-GO pour gel et phase 4** tant que CR-2026-001 n'est pas corrigée et vérifiée. Aucun contrat FROZEN_V1. Ce constat ne révoque pas les validations humaines des livrables précédents : il ajoute le défaut découvert à la revue.

| Preuve | Résultat observé | Portée / décision |
|---|---|---|
| 3.0, da4dac8 | Guest réel, 10000 dispatchs, allocations reprises, restart | Chromium/WebKit ; Firefox omis sur accord explicite |
| 3.1, 989428d | Même hash WASM, 30 réponses identiques, pointeurs et OutOfFuel | Wasmtime Windows ; pile native du banc 32 MiB |
| 3.2, e94e0c3 | Dioxus Web/Desktop, DOM/events/focus et captures | Sous-ensemble du fixture, pas renderer complet ni événements guest |
| 3.3, 2a1b2f6 | Isolation/CSP/Worker et ZIP hostiles | Deux moteurs ; validation ZIP dans Node, parseur navigateur produit ultérieur |
| Audit ABI/doc 3.4 | Trois divergences codec/schéma reproduites | Bloquant pour gel ; CR-2026-001 |
| Linux/macOS | Non essayés sur instruction propriétaire | Aucune compatibilité déclarée |
| Firefox | Échec de lancement, omis sur instruction propriétaire | Aucune compatibilité déclarée |
| GitHub | Workflow présent, protection distante non activée | Accord propriétaire pour progression locale ; activation ultérieure |

## Audit exécuté

Rust 1.99.1, Node 24.15.0, AJV 8.20.0. Harness local .git/probe/audit-3.4 importe la crate kortex-abi actuelle et appelle Reply::decode / Envelope::decode. Comparaison indépendante .git/probe/audit-protocol.mjs charge le schéma officiel avec AJV strict.

Messages identiques dans les deux langages :

```json
[
 {"v":1,"request_id":"00000000-0000-4000-8000-000000000001","ok":true,"data":null},
 {"v":1,"request_id":"00000000-0000-4000-8000-000000000001","ok":true,"error":null},
 {"v":1,"request_id":"00000000-0000-4000-8000-000000000001","kind":"capability.result","payload":{}}
]
```

Résultats réels : Rust [true,true,true], schéma JSON [false,false,false]. Les tests croisés 2.4 couvraient manifeste/contrat et n'exerçaient pas ce corpus d'enveloppes. L'écart capability.result était déclaré dans la preuve candidate 2.3, sans alignement ultérieur du codec.

## Comparaison aux décisions

ADR-001/002 : faisabilité binaire et rendu hôte démontrés sur les plateformes essayées ; pas de généralisation aux OS absents. ADR-005/010 : refus de capacité et isolation testés, médiation complète/queue asynchrone positive encore à implémenter. ADR-007 : signature détachée prouvée avec clés éphémères de test, chaîne persistante et ZIP canonique encore à construire. ADR-003/004/006/008/009 : décisions de conception, catalogue/IA/OIDC et UX produit pas encore construits. Les versions Dioxus/dx 0.7.10 sont vérifiées localement, avec feature web-sys/Location explicitement requise.

## Conditions de levée

Approbation de CR-2026-001, correction ciblée, parité des messages invalides et positifs, nouvelle empreinte guest identifiée, prototypes Web/native rejoués sur exactement les mêmes octets, régressions vertes. Ensuite seulement nouvelle décision GO et approbation humaine du périmètre de gel. Ne pas geler la médiation complète, les payloads métier non spécifiés ni les plateformes omises.

Attestation : propriétaire a approuvé le livrable 3.3 par « Oui continue ». Approbation du gate 3.4, du correctif et du gel : non reçue. Responsables technique/architecture et sécurité : non attestés. Le gate est une proposition de revue, pas une signature de l'agent au nom d'un humain.
