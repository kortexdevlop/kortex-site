# Compte Desktop Windows

Depuis « Compte », « Se connecter dans le navigateur » ouvre le navigateur système. Le navigateur demande une connexion Web puis une confirmation de l’appareil. Le mot de passe n’est jamais saisi dans la WebView Desktop.

Le client ouvre un listener `127.0.0.1` sur un port éphémère, crée un vérificateur PKCE S256 et un `state` aléatoires, puis demande une autorisation de dix minutes. Seuls les ports non privilégiés et le chemin `/callback` sont autorisés. Le navigateur renvoie un code à usage unique et le state ; seul le détenteur du vérificateur peut obtenir la session. L’écoute native s’arrête après cinq minutes ou annulation.

Une session opaque de sept jours est protégée par le Gestionnaire d’identifiants Windows. Aucune solution de repli en fichier clair. Le serveur conserve seulement son HMAC. Pas de refresh token ; une nouvelle connexion est nécessaire à expiration. Vingt appareils actifs maximum. Les droits limités couvrent l’identité, les favoris/préférences et la déconnexion : aucun export, suppression de compte, gestion d’autres appareils ou appel IA par ce jeton.

La synchronisation est désactivée par défaut. L’option locale exige un consentement sur cet appareil et respecte aussi la désactivation du compte Web. Favoris et quatre préférences sont des métadonnées ; aucun fichier ne remonte et un favori ne déclenche aucune installation. Les écritures utilisent les révisions courantes et montrent les conflits. Les réglages de thème, densité et texte sont appliqués ; la langue préférée est synchronisée, l’interface native reste en français dans cette version.

Les sessions sont révocables dans les réglages Web. La déconnexion efface le coffre local et le cache du compte ; elle révoque le serveur si celui-ci est joignable. Hors connexion, elle indique de révoquer aussi l’appareil depuis le Web. Les outils installés et leurs données restent intacts. La connexion n’est pas nécessaire à leur usage hors ligne.

La preuve réelle utilise le compte email/mot de passe ajouté à la demande du propriétaire. Le transport Desktop et son retour navigateur sont réels, ainsi que le coffre Windows et la synchronisation bidirectionnelle. Le helper de test appelle le client de production puis pilote une vraie interface Windows ; il ne prouve pas le lancement du navigateur par le bouton système lui-même.

Google reste désactivé : les deux clients OAuth n’existent pas encore. Le retour vers l’autorisation Desktop est préparé dans le flux OIDC Web existant, mais le gate « issuer OIDC réel » de 14.5 ne peut pas être déclaré réussi. La clôture complète OIDC attend cette configuration ; aucun remplacement silencieux par un fournisseur fictif. Linux/macOS et Firefox restent exclus conformément aux instructions.
