# KORTEX

Plateforme d'outils Rust Core WASM pour Web et Desktop, catalogue commun.

Initialisation 1.0 uniquement. Aucune application active ni infrastructure déployée. Plan : docs/MASTER_PLAN.md ; progression : docs/PROGRESS.md. Commandes Cargo et pnpm à activer aux étapes suivantes. Contrôles actuels : git status --short et git diff --check.

Installation locale : Node.js et Corepack requis. Activer Corepack puis lancer pnpm install --lockfile-only. Les versions seront figées en 1.2. Les scripts lint, typecheck, test, build et verify délèguent aux paquets présents ; aucun paquet applicatif n'existe encore. Une absence de paquet ne constitue pas un test réussi. Chaque futur paquet doit fournir ses vrais scripts de contrôle.

Configuration : copier .env.example vers .env pour le local ; ne jamais versionner les valeurs réelles. Injecter les secrets staging/production dans les paramètres protégés GitHub/Cloudflare. La clé privée de signature reste exclusivement en CI. Vérifications : node scripts/check-env.mjs --example et node --test tests/config/secrets.test.mjs. Voir infra/environments.md.
