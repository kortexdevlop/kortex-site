# Calculatrice — implémentation et périmètre clavier

Les quatre actions add, subtract, multiply et divide reçoivent exactement {a:number,b:number} et renvoient {result:number}. Elles partagent State::calculate avec les boutons. Nombres IEEE-754 finis et bornés à ±9007199254740991 ; division par ±0 : DIVIDE_BY_ZERO, résultat non fini ou hors borne : INVALID_INPUT. L’affichage conserve la valeur calculée. Permissions : aucune.

Deux champs nommés Premier nombre et Deuxième nombre acceptent une saisie au clavier, y compris le séparateur décimal français. Le pavé comporte chiffres 0–9, Virgule, Changer le signe, Effacer un chiffre et Tout effacer. Des boutons choisissent explicitement le champ destinataire ; saisir dans un champ le sélectionne aussi. Les opérations et le pavé restent dans l’ordre natif de tabulation, avec focus visible et boutons d’au moins 44 × 44 px. Les résultats utilisent alert/success (role=status), les erreurs alert/error (role=alert). Le résultat précédent est effacé à la saisie suivante.

Ajustement explicite du design 7.0 : le schéma UI v1 figé ne transmet que click/input/change/select et ne déclare ni commande de focus guest ni keydown. Entrée/Espace activent donc le bouton focalisé, comportement natif vérifié ; les raccourcis globaux Entrée sur un champ et Échap pour remettre le focus ne sont pas implémentés. Tout effacer fournit la remise à zéro accessible. Les ajouter demanderait une évolution de contrat, hors du lot 7.1–7.3. Aucun raccourci ne détourne la saisie d’un autre champ.

Composition : column, row, grid, heading, input, button et alert, tous validés par le renderer UI partagé. Aucun style ou HTML arbitraire. Les clés de nœuds restent stables. Une erreur de calcul dans un événement reconnu est affichée, avec acquittement de l’événement ; action.invoke renvoie le code d’erreur métier structuré. Init et shutdown réinitialisent tout état ; une réouverture ne récupère aucun calcul précédent.

Le code guest et ses tests sont livrés en 7.1 ; le même paquet local signé est vérifié Web/Windows en 7.2 et les enveloppes sont figées en 7.3. Les cas de tests exécutent les opérations réelles, sans moteur simulé.
