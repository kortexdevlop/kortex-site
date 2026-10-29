# Distribution Desktop Windows

Périmètre autorisé : Windows x64 et Chromium/WebView2. Les essais Linux/macOS/Firefox ont été exclus explicitement ; aucune compatibilité correspondante n'est déclarée.

Le binaire est compilé par Dioxus CLI 0.7.10 à partir de kortex-desktop. `node scripts/desktop-package.mjs` appelle la commande réellement vérifiée : `dx bundle --desktop --release --package kortex-desktop --package-types nsis --out-dir <dossier> --locked`. Le format retenu est l'installateur EXE NSIS, pas un MSI fictif. Le runtime WebView2 hors ligne est inclus et installé silencieusement.

NSIS 3.11 fourni par Dioxus lit par défaut certains scripts sans BOM en ACP. Le chemin Windows contenant µ a reproduit ce défaut ; le wrapper reprend uniquement cet échec identifié avec le même script généré et `/INPUTCHARSET UTF8 /OUTPUTCHARSET UTF8`, sans réécrire le contenu de l'application. L'installateur est contrôlé comme exécutable PE et son hash SHA-256 est enregistré.

Le smoke réel vérifie installation silencieuse, présence du binaire, processus vivant, désinstallation et disparition du binaire. Il utilise un dossier neuf sous target et refuse d'écraser une installation ou des raccourcis Kortex existants. Il ne constitue pas une preuve d'interaction visuelle exhaustive. La configuration embarque les racines publiques et la confiance staging ; aucune clé privée n'est embarquée.

Les préversions GitHub sont réservées au staging et restent explicitement non signées Authenticode. Leur provenance et leur descripteur sont signés par la clé de release certifiée ; le canal public signé contient le lien et le checksum. Aucun téléchargement n'est exécuté automatiquement. La production est refusée sans certificat Windows valide ; aucune signature Apple n'est inventée.

Les releases sont immuables par version : une relivraison avec d'autres octets échoue. Pour reprendre une publication, relancer seulement le job de publication avec son artefact d'origine ; pour changer le binaire, augmenter la version. Le contrôle de mise à jour Desktop vérifie la signature, l'expiration, le compteur monotone et la révocation de la clé ; il n'offre jamais un downgrade automatique.

Sources des outils : [bundling Dioxus 0.7](https://dioxuslabs.com/learn/0.7/tutorial/bundle/), [options MakeNSIS](https://nsis.sourceforge.io/Docs/Chapter3.html), code de la version installée dioxus-cli 0.7.10, fichiers de configuration et journaux dans target/desktop-package.

Une reprise peut fournir artifact_run au workflow Desktop : elle vérifie le dépôt, la branche, le workflow et la réussite effective de la compilation d’origine, exige sa CI verte et refuse toute différence de source affectant le binaire. Le même EXE est téléchargé, contrôlé et retesté ; la provenance et le tag conservent son commit de compilation. Un changement de code Desktop, de contrat, de confiance ou de packageur exige une reconstruction. Les journaux stdout/stderr du démarrage sont archivés, le profil WebView2 est isolé sous target, et la désinstallation est tentée même après un échec de démarrage.

Le paquet embarque explicitement le WebView2 x64 officiel épinglé (infra/desktop/webview2-x64.json), vérifié par SHA-256 et signature Microsoft. Le téléchargement x64 erroné du bundler Dioxus 0.7.10 est corrigé par préchargement de son cache ; un ancien fichier différent est sauvegardé sous target/desktop-webview2 avant remplacement. Le code source du bundler reste intact. Le smoke vérifie l’enregistrement du Runtime après installation et conserve les diagnostics de démarrage. La compilation sans secret et la validation commune peuvent tourner en parallèle ; la publication protégée exige toujours une CI verte sur le même commit.
