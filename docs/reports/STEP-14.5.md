# Étape 14.5 — Compte Desktop Windows

Base : `b00a6af`. Autorisation : propriétaire, « Enchaines ces 3 étapes ». Date : 28/10/2026.

## Résultat

Connexion email réelle dans le navigateur, confirmation d’appareil, code à usage unique avec PKCE S256/state/loopback, session opaque limitée à sept jours protégée par le coffre Windows. Aucun mot de passe dans la WebView, aucune session en clair sur disque. Appareils listés/révocables sur le Web. Export/purge complétés sans export des hashes de jetons.

Consentement local et compte pour la sync ; favoris sans installation automatique, préférences avec révisions et erreurs de conflit. Interface Windows reliée, thème/densité/texte appliqués. Déconnexion locale/serveur et conservation des fichiers ; hors connexion préservé. Langue préférée synchronisée, UI native toujours française.

Parcours réel staging : navigateur anonyme redirigé vers login puis autorisation, client Rust de production, vrai coffre Windows, favori Desktop visible Web, thème Desktop visible Web et taille Web visible dans l’application Windows, révocation Web puis coffre effacé, marqueur de fichier intact. Le helper appelle le client sans ouvrir lui-même le navigateur système ; son ouverture par le bouton produit n’a pas été automatisée.

## Validation et limites

Quatre tests natifs spécifiques réussis (PKCE/state, expiry, coffre/logout/offline, références sans installation), build Windows, 81 tests API, 23 Web, 3 contrats et parcours réels. La suite Desktop complète finale réussit : **22 tests**, en série. Des essais antérieurs ont rencontré des erreurs intermittentes Windows `WSAECONNABORTED/10053` sur les serveurs TCP locaux ; le résultat final et les logs figurent dans `device-14.5-15.1-validation.json`. Aucun contournement ni modification du transport produit ou du serveur de test conservé pour faire passer ces essais.

Google n’a toujours pas ses clients OAuth. Le gate obligatoire avec issuer OIDC réel n’est pas réalisé. Le retour vers l’approbation Desktop est préparé dans le login OIDC Web, mais la connexion prouvée est email + PKCE interne, et ne vaut pas preuve Google.

Décision : **partie email/Windows réalisée et prouvée ; clôture complète 14.5 en attente du gate Google**. Aucune approbation humaine inventée. Le propriétaire a expressément autorisé les trois étapes du lot ; préparation et vérification 15.0/15.1 effectuées avec cette réserve documentée.

Chemins : `desktop/src/{account,account_ui,secure_store,app,library,config,lib}.rs`, CSS/Cargo, API auth/users/device/session, migration 0009, Web login/approval/devices/proxy, tests et helper staging. Interfaces additives, outils invités conservés. Production, DNS et messagerie non touchés. Preuves : `device-sync-ui-staging.json`, `device-ai-staging.json`, captures Windows/Web et logs agrégés.
