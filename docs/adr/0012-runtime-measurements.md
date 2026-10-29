# Décision de mesure RSS Windows

Les premiers essais ont alterné entre environ 16 et 28 Mo de RSS après fermeture, même après attente de la fin réelle des threads. Les deux séries échouées sont conservées. Le contrôle comparait une seule valeur après huit instances non initialisées avec une seule valeur après cinquante instances initialisées : base non équivalente et phases différentes des arènes d’allocation JIT Windows.

Décision : huit cycles d’échauffement complets identiques aux cinquante cycles mesurés, puis comparaison de leur enveloppe RSS p95 avec le maximum du RSS final et du p95 des dix derniers cycles. Toutes les observations sont publiées, seuil inchangé à 10 %. Aucune exemption mémoire, aucun masquage de dépassement. Le test mesure une croissance de mémoire résidente soutenue après échauffement, pas une preuve générale d’absence de fuite. Le tas JS Web reste une métrique distincte.

L’arrêt Windows attend les deux threads, avec fermeture du canal de capacités et interruption epoch préalables ; les tests de guest hostile et annulation passent. Aucun cache global de modules, permission ou contrat ABI ajouté.

Un smoke Web local tardif a échoué après expiration normale de la racine de test éphémère (validité une heure), pendant le changement de bundle. Il reste joint (runtime-web-performance-expired-local-key.json) ; la preuve antérieure réussie est conservée séparément et le système complet est reprovisionné avec une nouvelle clé locale avant le nouveau gate. Aucun allongement de confiance ni contournement de signature.
