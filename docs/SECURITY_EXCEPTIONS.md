# Gates et réserves de sécurité — 28/10/2026

Aucun avis n’est ignoré dans cargo-audit/pnpm. Aucune exception n’est présentée comme acceptée par le propriétaire. Responsable de suivi : maintenance Kortex ; les échéances ci-dessous imposent une revue, pas une autorisation automatique de production.

| ID | Réserve | État / mesure | Échéance |
|---|---|---|---|
| SEC-GOOGLE-18 | Clients OAuth Web/Desktop absents | Gate réel Google BLOCKED, fournisseur désactivé. Ajouter les clients puis vérifier login, refus, callback/rejeu, logout et révocation avec Google avant clôture complète 18.2. Email/password et fournisseur contrôlé testés séparément. | 04/11/2026, ou avant activation Google |
| SEC-UPSTREAM-18 | RustSec : fxhash, paste, proc-macro-error non maintenus | Avis conservés dans le rapport ; pas de vulnérabilité active signalée après ZIP 9. Revue de remplacement dans la chaîne Dioxus/outillage requise, sans mise à jour majeure aveugle. | 27/11/2026, ou avant production |
| SEC-PLATFORM-18 | RustSec : glib 0.18.5 et rand 0.7.3, avertissements unsound | Absents du graphe Windows vérifié ; avertissements globaux non masqués. Linux/macOS exclus par le propriétaire ; toute réactivation de ces plateformes exige revue/migration et tests propres. | Avant réactivation ; revue le 27/11/2026 |
| SEC-WEB-DNS-18 | Le navigateur ne permet pas de fixer la résolution DNS | Mitigation active, pas une dérogation SSRF : DNS refusé UNSUPPORTED. IPv4 publiques littérales seulement avec allowlist, consentement et CSP. Pas de promesse d’un accès réseau Web général. | Réexaminer avant tout élargissement réseau |
| SEC-OPS-18 | Sauvegardes et rétention des logs de l’hébergeur non vérifiées | La purge SQL logique et la révocation sont testées ; aucun effacement physique instantané des sauvegardes n’est affirmé. Politique opérateur à établir avant production. | Avant production |

Limites d’assurance : mémoire linéaire ≠ mémoire totale, corpus fini ≠ preuve générale, contrôles automatisés ≠ audit indépendant. Les problèmes de contraste connus du bureau ne sont pas requalifiés en conformité AA.
