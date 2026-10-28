# Menaces → protections → preuves, 28/10/2026

| Menace | Gravité locale | Protection | Tests exécutés / limite |
|---|---|---|---|
| Archive falsifiée / signataire inconnu | Critique | hash, Ed25519, racine/expiration, ZIP strict en mémoire, pas d’extraction disque | pack archive/security, native package_security, Web loader/signature/package-security |
| Auteur signé hostile / import natif | Critique | ABI statique, imports restreints, Wasmtime/Worker, courtiers | native adversarial et guest, Web abi-limits : vrais modules signés ; pas de signature contournée |
| Boucle alloc/dispatch/dealloc, pointeur invalide | Majeure | interruption Worker, fuel/epoch, copie bornée, non-chevauchement, corrélation | security-abi-web.json ; adversarial ; core lifecycle ; corpus fixe 4 096 pointeurs par moteur et 10 000 ZIP/WASM |
| Mémoire/table/UI/file d’attente excessive | Majeure | mémoire ≤64 Mio, table ≤65 536/max obligatoire, UI validée avant rendu, queue 128 | abi-limits, adversarial, core lifecycle, UI render ; pas de plafond RAM totale du processus revendiqué |
| iframe étrangère / commande vers autre instance | Critique | source/origine/nonce/session, lookup propriétaire, générations retirées | Web bridge et workspace-router/isolation ; preuves 17.3 et régression staging du présent lot |
| XSS guest / catalogue / contenu UI | Critique | schéma UI, texte échappé, pas de HTML guest, CSP sans script inline arbitraire | UI render html_injection, API contracts, Web CSP ; aucun pentest universel de tous navigateurs revendiqué |
| Prompt injection / abus de parcours | Critique | plan strict, actions connues, contrat et consentement local, scopes de capacité distincts | API device-ai, core ai_bridge/lifecycle, Web ai-bridge et annulation workspace ; modèle non fiable |
| SSRF / DNS rebinding / redirection privée | Critique | Windows : toutes IP publiques et fixées, chaque redirection revalidée ; Web : DNS non vérifiable refusé | gateway DNS rebinding, backend privé/mixte, network-files et capabilities TLS local ; zéro scan public |
| Nom de fichier / jonction / handle volé | Critique | aucun chemin guest ; sélection hôte, handle propriétaire, contrôles reparse Windows, sauvegarde bornée | native capabilities jonction réelle, Web filename traversal, workspace deux lectures / handle étranger |
| Cookie/CSRF / accès à un compte tiers | Critique | cookie host-only, HMAC serveur, Origin/CSRF/session, propriétaire SQL et scopes Desktop | 88 tests API dont security-abuse, auth, password-auth, users, device-ai ; auth-abuse HTTP staging |
| Token volé / export / effacement | Critique | expiration/révocation, CAS et suppression en cascade, export sans credentials | users/security-abuse ; un token bearer volé reste utilisable dans son scope avant révocation/expiration, sans preuve de possession supplémentaire |
| Publication opérateur / dépôt usurpé | Critique | JWT/JWKS/claims, autorité configurée, signature d’artefact et immutabilité | API publication/downloads/auth-schema ; pas d’exécution GitHub OIDC réelle ajoutée dans cet audit |
| Données privées / logs / IA | Majeure | message volontaire, filtrage de formes connues, aucun upload automatique de fichiers/résultats ; logs bornés | API security-abuse/device-ai, audit local capabilities, inventaire PRIVACY ; filtrage non assimilé à une DLP |
| Dépendance vulnérable | Majeure | mises à jour exactes, lockfiles et scans réels non ignorés | cargo audit et pnpm audit : avant/après ; avertissements et dépendances hors Windows dans SECURITY_EXCEPTIONS |
| Login Google réel | Gate | fournisseur désactivé, clients absents | BLOCKED : clients Web/Desktop et parcours Google réels requis ; fournisseur de test ≠ Google |

Un succès est celui du corpus et du périmètre indiqués. Les scanners absents ou non exécutés ne deviennent pas des succès ; l’absence initiale de cargo-audit a été résolue par installation et exécution. Revue humaine du modèle encore à effectuer.
