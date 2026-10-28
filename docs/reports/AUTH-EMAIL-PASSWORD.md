# Rapport — ajout email/mot de passe avant 14.2

28/10/2026. Base aee812d. Ajout expressément demandé par le propriétaire, hors numérotation du plan ; aucune étape sautée ou approbation visuelle inventée.

Inscription et connexion Web par email/mot de passe, compte minimal et déconnexion. Stockage scrypt natif + sel + pepper, formulaire CSRF privé, limites persistantes, sessions communes indépendantes de Google, nouveaux endpoints bornés et interfaces responsive. Email non vérifié et récupération par mail absente selon l'absence de service confirmée par le propriétaire. Aucun lien automatique entre identités Google et locales. Migration 0007 additive ; aucun KDF JavaScript dans le produit, @types/node épinglé uniquement pour le typage.

56 tests API au total (9 nouveaux + 47 régressions), 14 tests proxy, 3 contrats ; analyse/typecheck et build réussis. Parcours réel Chromium sur Cloudflare : inscription, session/rechargement, déconnexion avec CSRF, mauvais puis bon mot de passe, cookie sécurisé, UI mobile/bureau. Compte de test supprimé. Preuves/hashs et IDs de déploiement : docs/evidence/password-auth-validation.json.

Résultat : complément email/mot de passe terminé et actif sur staging. Google reste en attente de ses clients OAuth. 14.2 non engagée. DNS, production et routage mail conservés. Guide : docs/AUTH-PASSWORD.md.
