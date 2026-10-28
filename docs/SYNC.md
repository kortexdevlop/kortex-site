# Synchronisation facultative — 14.2 à 14.4

28/10/2026. Favoris et quatre réglages d'affichage seulement ; aucun fichier, nom de fichier, contenu d'outil ou workflow. Désactivée par défaut. Connexion email opérationnelle ; Google attend toujours ses clients OAuth. 14.5 non engagée.

## Modèle

Migration additive **0008_sync.sql**, car 0004 à 0007 sont déjà occupées. Tables user_sync, user_library, user_preferences et sync_events, toutes supprimées en cascade avec users. Révisions entières monotones serveur ; transactions D1 batch et CAS. Aucune horloge cliente utilisée pour l'ordre.

Bibliothèque OR-set : chaque ajout a un UUID stable (tag) et un tool_id slug. Retirer supprime les tags observés seulement ; les tombstones sont conservées, y compris lorsqu'un retrait précède son ajout. Un ancien tag supprimé ne revient jamais ; un nouvel ajout concurrent survit à un retrait qui ne l'avait pas observé. Un tag ne peut changer d'outil. CAS et écritures annexes sont dans une transaction avec UUID de mutation, sans effet d'une écriture perdante.

If-Match exact `"library-N"` ou `"preferences-N"` exigé pour les mutations ; 409 en cas de révision dépassée. La bibliothèque relit puis rejoue les opérations originales avec les mêmes tags observés. Les préférences refusent la perte silencieuse : 409, saisie conservée et rechargement explicite dans l'interface. Les timestamps clients et champs inconnus sont refusés.

## API

| Route /api/v1/me | Méthodes | Données |
|---|---|---|
| /library | GET, PUT, DELETE | items {tool_id,tags[]}, tombstones, révision ; PUT {adds:[{tool_id,tag}],removes:[{tool_id,tags:[]}]} |
| /preferences | GET, PATCH | sync_enabled et preferences partielles : theme light/dark, density simple/medium/pro, type standard/large, lang fr/en |
| /export | GET | téléchargement JSON et ETag `"account-L-P"` |
| /data | DELETE | If-Match composé et {confirm:"delete-account"} |

DELETE library sans corps retire au plus 100 tags observés sous la révision courante ; conserve les tombstones. Library exige sync_enabled. Activer peut transmettre les premières préférences ; désactiver bloque immédiatement les écritures de bibliothèque et les uploads de préférences, mais conserve les anciennes métadonnées pour lecture/export ou suppression explicite. Une référence d'outil ne constitue pas une installation : disponibilité, signature et permissions restent vérifiées au lancement.

## Confidentialité

Session Web HttpOnly via binding privé AuthService uniquement. Aucun sélecteur user_id accepté ; toutes les requêtes SQL utilisent le propriétaire de session. Origin exact et CSRF pour mutations. X-Kortex-Account est une précondition d'identité contre les onglets restés ouverts après changement de compte, jamais une sélection de propriétaire. Le bearer Google Desktop limité à identity:read ne reçoit aucun droit sync implicite ; 14.5 devra fournir ces droits.

Réponses privées no-store, sans PII dans les pages publiques. Le relais ajoute no-transform aux réponses de compte pour préserver les ETags forts à travers [la compression Cloudflare](https://developers.cloudflare.com/speed/optimization/content/compression/) ; les validateurs faibles restent refusés pour If-Match. Les pages /account sont protégées côté SSR : invité redirigé, panne avec cookie explicitement signalée, cookie transmis au seul Service Binding. Aucun token client dans le rendu SSR public.

Export : profil, email local/statut non vérifié, identités externes, tous les tags/tombstones, préférences, événements conservés et dates des sessions. Jamais les hashes de mots de passe, index email, secrets, jetons ou hashes de session. Purge atomique de users et cascade vers identités, identifiants, sessions et tables sync. Compteurs propres à l'identifiant et transactions anonymes connues du navigateur retirés ; compteurs globaux et autres formulaires anonymes suivent leur TTL. Fichiers et favoris invités locaux restent intacts.

## Quotas et clients

JSON 16 Kio, 100 opérations de tag par mutation, 2000 tags/tombstones par compte, 100 événements sans contenu par entité, export 1 Mio, 100 sessions avec conservation garantie de la nouvelle. Pas de compaction susceptible de ressusciter un ancien ajout offline.

Le workspace utilise les favoris distants après consentement, sans écraser ni uploader automatiquement les favoris invités. Import local explicite dans la bibliothèque. Aucun cache personnel persistant ajouté dans localStorage ; état distant en mémoire. Les réglages partagés se modifient dans la page dédiée. Le Web requiert une connexion pour confirmer une modification ; les opérations préparées offline et leur rebase sont éprouvées au niveau modèle/API, sans prétendre à une application Web entièrement offline. Une modification non confirmée est signalée et la vue peut être rechargée.

Preuves : docs/evidence/account-14.2-14.4-validation.json. D1 émulée réelle : isolation A/B, CAS concurrent, tombstones, quotas, export/purge. Chromium Windows staging : deux sessions, conflits, favoris du workspace, export téléchargé, purge et calculatrice invitée. Linux/macOS et Firefox exclus selon les instructions du propriétaire.
