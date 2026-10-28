# Rapport 14.3 — API utilisateur, export et purge

28/10/2026. Base 60a7986. Autorisation : « Enchaine maintenant 14.2 jusqu'à 14.4 ».

Library GET/PUT/DELETE, preferences GET/PATCH, export GET et data DELETE. Session propriétaire, CSRF, précondition de compte, If-Match, 409 sans effet perdant et quotas. Aucun droit sync déduit du bearer d'identité Desktop. Export des métadonnées personnelles sans matériel d'authentification. Suppression atomique du compte avec cascade des identifiants, sessions, identités et données synchronisées, en préservant un autre utilisateur.

Isolation A/B, mauvais CSRF, compte changé, rejeu, opt-in, ETag, export et purge éprouvés en D1/API ; parcours réel de deux sessions sur staging. Correction du comptage des cascades via DELETE RETURNING et conservation des ETags forts par no-transform sur Cloudflare. Résultats du lot : 71 tests API, 18 Web, types/builds et navigateur réussis. Guide docs/SYNC.md, preuves docs/evidence/account-14.2-14.4-validation.json. Production, DNS et mail intacts.
