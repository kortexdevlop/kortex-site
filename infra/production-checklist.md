# Checklist production — plafond 0 €

État faisant foi : `infra/production-state.json`, `infra/deploy/channels.json` et `docs/evidence/production-readiness.json`. Le propriétaire a confirmé licence propriétaire et budget mensuel 0 €. Les cases suivantes ne sont cochées qu'après preuve.

- [x] Licence propriétaire choisie explicitement ; fichier [LICENSE](../LICENSE).
- [x] Zone `kortex.site` accessible et active ; relevé en lecture seule conservé sans contenu DNS sensible.
- [x] Messagerie et DNS existants préservés.
- [x] Ordre de bascule DB → API → paquets → Runtime → site → Windows décrit dans [le plan](production-release-plan.json).
- [x] Contact sécurité confirmé par le propriétaire : admin@kortex.site.
- [ ] Nom/identité juridique de l’éditeur confirmée.
- [ ] Revue CGU/confidentialité et obligations des dépendances redistribuées ; aucune approbation juridique automatique.
- [ ] Ressources production distinctes configurées : compte, DB, bucket, Workers et trois origines HTTPS réelles.
- [ ] Conditions gratuites et limites de facturation vérifiées pour tous les services ; aucune souscription ni hausse automatique.
- [ ] Quotas suffisants pour le trafic retenu ; service dégradé explicite si le palier gratuit est atteint.
- [x] Google OAuth reporté par le propriétaire ; email/mot de passe conservé.
- [ ] Connexion email/mot de passe validée sur la production réelle.
- [x] Signature Windows Authenticode reportée explicitement par le propriétaire ; état du certificat toujours non validé.
- [ ] CI complète verte et RC gelée sur le même commit, cinq paquets vérifiés et artefacts traçables.
- [ ] Drill D1/R2 distant réussi, RPO/RTO observés et copie de retour disponibles.
- [ ] Sauvegardes, rétention, collecteur et destinataire/astreinte réellement activés et testés.
- [ ] Secrets et environnement GitHub production protégés, TLS/CORS/CSP et droits minimaux vérifiés.
- [ ] Revue finale de la livraison concrète et smoke par bascule.

Contrôles : `node scripts/production-readiness.mjs` produit la liste des gates ouverts et sort avec code 2 si incomplet. `node scripts/check-env.mjs --production` exige réellement le mode production et refuse les gates ouverts ; il ne valide pas le template local comme une production. Un succès de tests synthétiques de ce contrôleur n'est pas un succès de mise en production.

Un budget 0 € ne permet pas de promettre une disponibilité sans limite ou de souscrire un service payant pour lever un blocage. Le plan Free de la zone ne prouve pas à lui seul le plan Workers/R2/IA. L'accès à l'inventaire des abonnements est actuellement refusé : ce point reste non vérifié.
