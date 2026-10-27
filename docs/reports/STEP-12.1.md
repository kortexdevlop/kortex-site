# Rapport 12.1 — SHA-256 sur fichier local

27/10/2026, Windows. Base 0547b00. Commit dans PROGRESS.md ; lot expressément autorisé, validation humaine formelle non inventée.

Nouvelle crate tool-calculateur-hash : file.pick/read via Host et host_enqueue existants. UUID de corrélation sans horloge système, lecture incrémentale de blocs ≤256 Kio jusqu’à 10 Mio, handle opaque, nom local borné, progression et annulation par génération. Offsets, taille, base64, EOF et résultats tardifs contrôlés. Le fichier complet n’est pas copié dans le guest et aucun réseau guest n’est autorisé. Refus de permission explicite sans lecture ; annulation sans empreinte finale ni nouveaux blocs, tout en reconnaissant qu’une opération bornée déjà en cours peut se terminer.

Action pure sha256_bytes({data_base64})->{hex}, aucune permission sensible et sortie limitée au digest. Le plafond d’enveloppe ABI de 1 Mio est conservé : base64 ≤1 048 000 caractères, donc ≤786 000 octets décodés effectivement transportables. La limite théorique d’octets n’autorise pas à dépasser l’enveloppe gelée. Fichiers plus grands via les fragments UI. Cette distinction est documentée ; aucune extension silencieuse du codec.

Deux tests Rust : vecteurs vide/abc, base64 hostile, bornes, enchaînement de blocs, annulation/résultat tardif, refus et offset corrompu. Tests navigateur locaux : vrai picker, refus avant lecture, calcul incrémental connu, action équivalente, annulation et zéro requête réseau après chargement. Sur Cloudflare : même archive signée, quatre blocs d’un fichier choisi localement, digest exact et absence d’envoi réseau dans Chromium/WebKit.

Test Windows Wasmtime avec véritable fichier disque, trois blocs, gateway/handles/permissions et SDK guest identiques ; lecture refusée et annulation vérifiées. La sélection positive est contrôlée à la frontière du picker dans ce test de backend, sans mock des octets ni de l’I/O. L’interface WebView2 couvre l’action réelle et le refus du picker ; le dialogue Windows de sélection positive n’est pas automatisé. Aucun changement du bridge générique ou des fichiers hôte n’a été nécessaire.

Le contrat gelé ne permet pas pattern dans les schémas de valeurs ; la validation initiale l’a détecté, puis le contrat a été corrigé avant signature. La syntaxe base64 et les chiffres du digest sont contrôlés dans le guest, avec taille exacte 64 en sortie du schéma. Empreintes, signatures et preuves finales dans docs/evidence/phase12-validation.json. Aucun fichier utilisateur envoyé au cloud, aucune donnée brute retournée par défaut à une future IA.
