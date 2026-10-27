# Rapport 10.1 — tokens candidats, thèmes et densités

Date : 2026-10-27. Base : 9b0db4c. Résultat : commit contenant ce rapport, identifié dans PROGRESS.md.

Statut : **implémentation préparée ; validation formelle pending derrière 10.0**. Autorisation : lot explicite du propriétaire. La capture originale n’est pas disponible ; les couleurs, espaces et dimensions sont des choix candidats, sans mesure prétendue.

Source JSON unique, génération CSS déterministe, palette neutre et neuf pastels, polices système, espace de 4 px, rayons/ombres, points de rupture, focus, clair/sombre et densités gratuites simple/medium/pro. Les valeurs de densité sont exposées en préférences locales ; aucune relation avec un abonnement. Tailwind 4 via Vite et variables sémantiques partagées. theme.css est un fichier généré, son contenu exact est contrôlé au test et au lint.

| Commande | Date | Plateforme | Code | Résultat |
|---|---|---|---|---|
| node scripts/build-design-tokens.mjs | 2026-10-27 | Windows / Node | 0 | CSS synchronisé |
| node --test tests/design/tokens.test.mjs | 2026-10-27 | Windows / Node | 0 | 2 groupes réussis : contrastes et source unique |
| pnpm --filter @kortex/website lint | 2026-10-27 | Windows | 0 | export CSS identique à la source |

Les tests imposent ≥4,5:1 au texte normal/secondaire sur toutes les surfaces et pastels des deux thèmes ; boutons ≥4,5 et focus/contours ≥3. Les textes de grande taille utilisent aussi ces couleurs conformes. Aucun hex/rgb/hsl local divergent dans global.css, grille d’espacements vérifiée. Les contrôles de navigateur complètent la vérification des couleurs source.

Preuves et SHA-256 : docs/evidence/validation-10.0-10.2.json. Restant : confronter les tokens à la capture originale, ajuster puis revoir. Les seuils de fidélité ne sont pas évalués. Contrats gelés inchangés ; aucune approbation de design après résultat inventée.
