# Parcours locaux — étapes 16.0 à 16.2

La page `/workflows` permet de préparer un enchaînement à partir des actions du catalogue publié. Choisir un outil et une action, ajouter une étape, renseigner ses valeurs ou sélectionner le résultat d’une autre étape. Les liens et les paramètres sont vérifiés immédiatement. Une vue carte et une vue liste donnent accès aux mêmes étapes ; ajout, déplacement et modification sont utilisables au clavier.

L’éditeur peut importer/exporter le format workflow V1, annuler une modification et annuler un import sans remplacer le document courant. Les imports résolvent leurs actions dans le catalogue réel. Les documents et les valeurs restent en mémoire ; aucun enregistrement automatique dans localStorage, aucune synchronisation, aucun envoi du document au serveur. L’export est un fichier local et peut contenir les valeurs saisies.

**Ce lot prépare et vérifie les parcours. L’exécution depuis l’éditeur et son branchement produit Web/Desktop appartiennent à 16.3.** Aucun bouton ne prétend exécuter un outil dans cette page. Le moteur Rust est disponible et son fonctionnement réel est prouvé séparément avec un adaptateur de test vers le Runtime signé : multiplication 1200 puis addition 1210. Cet adaptateur n’est pas un moteur fictif dans le produit.

## Profil d’exécution V1

- Workflow JSON : 1 Mio maximum, 50 étapes, 100 liens, profondeur 50. Le schéma gelé reste inchangé ; ces bornes sont un profil plus strict de ses limites structurelles générales.
- Ordre topologique stable selon l’ordre des étapes, exécution séquentielle, arrêt à la première erreur. Chaque résultat utilisé exige une dépendance préalable, directe ou transitive.
- Paramètres via JSON Pointer RFC 6901, échappements `~0` et `~1`, indices de tableau canoniques de 0 à 999. Les champs doivent être déclarés dans le schéma. Les chemins racine et descendants ne peuvent pas se recouvrir.
- Contrôle statique des types et bornes, puis contrôle de la valeur réellement reçue avant invocation. Une propriété de sortie facultative absente arrête le parcours avant l’étape destinataire.
- Entrée/sortie d’étape : 1 Mio chacune ; résultats cumulés : 4 Mio. Pas de contenu d’entrée/sortie dans le journal.
- Durée totale : cinq minutes maximum ; chaque action conserve son délai du contrat, plafonné par le temps restant et l’expiration du consentement. Annulation réveillant et abandonnant la future en attente.
- Zéro nouvel essai par défaut. Deux au plus si explicitement demandés, uniquement pour des actions idempotentes et des erreurs signalées temporaires. Une configuration de retry comprenant une action non idempotente est refusée.

## Consentement et effets

Le registre est construit à partir de paquets `VerifiedTool`, avec identité, version, instance et empreinte d’artefact. Les contrats et permissions sont épinglés à la préparation et revérifiés avant chaque tentative et après son retour. Aucun client ne peut injecter une action inventée dans ce registre.

La prévisualisation donne les permissions agrégées et les étapes sensibles. Une action à effet, une permission déclarée ou `requires_confirmation=true` impose une décision explicite. Le mode autorisant les actions pures est une politique de l’utilisateur fournie par l’hôte ; il n’est pas lu depuis le JSON du parcours.

L’approbation d’étape ou de lot est liée à un Run unique, au document et aux contrats épinglés, aux bornes de retry et à une échéance de cinq minutes. Elle est consommée et ne s’applique pas à une autre exécution. `AWAITING_CONSENT` préserve les résultats précédents ; une reprise ne rejoue pas les étapes terminées. Refus/annulation/échec terminal scellent le consentement.

Cette approbation ne crée **aucun** droit de capacité. Les scopes réels passent par le même `Policy` que l’IA et le Runtime, puis par le courtier normal lors de l’action. Les grants à usage unique ne sont pas confondus avec leur révocation : un compteur de retrait du Policy permet d’arrêter une exécution en cas de retrait, même pendant une attente. Le retrait d’une autre session du même Policy arrête aussi le Run par prudence.

L’hôte de 16.3 devra maintenir les instances du registre durant le Run, résoudre les ressources exactes après mapping, fournir les gestes nécessaires au courtier, et invoquer exclusivement `Runtime::invoke_action`. Il devra arrêter/décharger les instances et effacer les résultats lors de la fermeture. Aucun transport produit n’est ajouté prématurément ici.

Le journal local ne contient que l’étape, le statut, la tentative, l’heure et l’indication qu’un effet peut avoir eu lieu. Un effet externe déjà accompli n’est jamais annoncé comme annulé ou restauré. Le statut terminal doit être présenté avec le journal et les résultats partiels, sans promesse de rollback de fichier.

## Validation

Tests Rust du graphe, mapping, retry, limites, autorisation réelle, reprise, révocation, expiration et abandon d’une opération pendante. Test Wasmtime sur paquet calculatrice signé : 25×48=1200 puis +10=1210, sans cloud. Tests TypeScript et trois parcours Chromium staging : deux étapes au clavier, carte/liste, boucle refusée, export téléchargé, import identique, annulation conservant le document et mobile 390 px. Les deux cartes sont aussi vérifiées comme distinctes visuellement ; Preact 11 demande des unités CSS explicites.

Preuves : `docs/evidence/workflows-validation.json`, captures, `workflows-runtime.json` et `workflows-browser.json`. Windows/Chromium seulement selon les exclusions demandées. API, runtime distant, quotas IA, production, DNS et messagerie inchangés. La réserve Google de 14.5 demeure indépendante.

## Lot suivant 16.3–17.0 — état au 28/10/2026

Les paragraphes précédents décrivent le lot historique 16.0–16.2. Le Web staging possède désormais un lancement local dans le runtime signé isolé, progression/arrêt, confirmation, résultats partiels locaux et journal de métadonnées. Le nonce/session n’admet qu’une invocation par frame ; une nouvelle session est nécessaire après arrêt/crash. Windows possède le même parcours sur les paquets installés et signés, avec huit outils distincts au maximum et leases maintenus jusqu’à l’arrêt.

La sélection de fichier est éphémère, séparée de la définition ; une autorisation de lecture distincte précède la lecture bornée. Aucune étape ne reçoit automatiquement une capacité hôte grâce à son approbation de parcours. **Les actions déclarant des capacités restent refusées dans le lanceur Desktop.** Le raccordement de leurs scopes/dialogues au NativeServices existant manque encore : 16.3 et la clôture universelle 16.5 restent ouvertes, même si les scénarios des outils actuellement publiés passent.

La sauvegarde locale et la synchronisation du compte sont explicites, sans transfert des résultats/fichiers. Migration 0011, propriétaire, révisions/If-Match, tombstones et refus de conflits ; le token natif reste dans le coffre. Le layout 17.0 est un modèle strict de métadonnées avec huit fenêtres, focus/z et restauration recadrée ; aucune interface 17.1 n’est engagée. Voir `docs/evidence/orchestration-validation.json` et les quatre rapports pour le statut exact.

## Clôture du complément 16.3 / 16.5

La réserve historique ci-dessus est levée : les actions sensibles Windows utilisent maintenant le courtier existant, un dialogue par capacité/ressource et un droit à usage unique. La demande réelle du guest est aussi vérifiée contre les permissions de son action et le scope choisi. Les initialisations sont médiées séparément. Refus/annulation/expiration/fermeture bloquent la suite et révoquent les services ; un journal abandonné ne provoque aucune reprise.

Le Web attend les résultats des capacités, conserve au plus huit instances isolées pendant le Run et admet jusqu’à 50 requêtes distinctes par session ; un identifiant consommé ne peut être rejoué. Les files et limites des courtiers restent actives. Le test de stockage Web utilise une autorisation navigateur réelle de stockage durable dans un profil Chromium persistant et des transactions IndexedDB réelles, sans remplacer les sondes ni le backend. Les restrictions de la plateforme restent fail-closed.

Les scénarios signés, normaux et adversariaux, sont consignés dans docs/evidence/workflow-permissions-validation.json. Aucun changement de contrat workflow gelé, outil métier, API, quota IA, production, DNS ou messagerie. 16.3 et 16.5 closes techniquement ; 17.1 non engagée.
