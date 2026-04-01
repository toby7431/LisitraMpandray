# FJKM Ambalavao Isotry — Système de Gestion d'Église

Application desktop de gestion des membres et des cotisations de l'église FJKM Ambalavao Isotry (Madagascar).

---

## Stack technique

| Couche | Technologie |
|---|---|
| Frontend | Rust + [Leptos 0.7](https://leptos.dev) (WebAssembly) + Tailwind CSS 4 |
| Desktop | [Tauri 2](https://tauri.app) |
| Base de données | SQLite via `sqlx 0.7` |
| API réseau | [Axum 0.7](https://github.com/tokio-rs/axum) (serveur) + `reqwest 0.12` (client) |
| Export | `rust_xlsxwriter` (Excel), CSV natif |
| Finances | `rust_decimal` (précision Ariary) |
| Installeur | NSIS (Windows `.exe`) |

---

## Fonctionnalités

- **Gestion des membres** — Communiants et Cathekomens (nom, adresse, téléphone, travail, genre, numéro de carte)
- **Cotisations** — Enregistrement et suivi des cotisations en Ariary avec date et période
- **Transfert** — Passage d'un membre de Cathekomen à Communiant
- **Archives annuelles** — Clôture automatique de fin d'année avec résumé financier
- **Export** — CSV et Excel (liste des membres + totaux)
- **Import** — Import CSV de membres
- **Thème** — Mode clair / sombre / système
- **Réseau** — Mode serveur/client sur réseau local (RJ45) pour 2 postes

---

## Structure du projet

```
EgliseManager/
├── src/                          # Frontend Leptos (WASM)
│   ├── app.rs                    # Composant racine + vérification config
│   ├── components/               # Composants UI réutilisables
│   ├── pages/
│   │   ├── accueil.rs            # Tableau de bord + versets
│   │   ├── communiants.rs        # Page membres Communiants
│   │   ├── cathekomens.rs        # Page membres Cathekomens
│   │   ├── archives.rs           # Archives annuelles
│   │   └── setup.rs              # Wizard de configuration (1er lancement)
│   ├── services/
│   │   ├── db_service.rs         # Appels aux commandes Tauri (données)
│   │   └── config_service.rs     # Appels aux commandes Tauri (config réseau)
│   └── models/                   # Modèles de données (Member, Contribution, YearSummary)
│
├── src-tauri/                    # Backend Tauri (Rust natif)
│   ├── src/
│   │   ├── lib.rs                # Commandes Tauri + DataSource enum
│   │   ├── config.rs             # Configuration réseau (AppConfig)
│   │   ├── remote_client.rs      # Client HTTP reqwest (mode client)
│   │   ├── api_server.rs         # Serveur Axum (mode serveur)
│   │   ├── export.rs             # Fonctions CSV/Excel partagées
│   │   └── db/
│   │       ├── repo.rs           # Repository SQLite (toutes les requêtes)
│   │       ├── models.rs         # Modèles de données Rust
│   │       └── error.rs          # Types d'erreur
│   ├── migrations/
│   │   └── 0001_initial.sql      # Schéma de la base de données
│   └── resources/
│       └── README.txt            # Guide utilisateur (inclus dans l'installeur)
│
├── style/                        # CSS / Tailwind
└── assets/                       # Icônes et ressources statiques
```

---

## Reconfiguration du mode réseau

Un bouton **⚙** (engrenage) est disponible dans la barre de navigation, à droite du sélecteur de thème.

1. Cliquer sur **⚙** → une confirmation s'affiche inline : **"Reconfigurer ? [Oui] [Non]"**
2. Cliquer **Oui** → la configuration est effacée, le wizard de configuration s'affiche à nouveau
3. L'utilisateur peut choisir un nouveau mode (Serveur / Client) sans relancer l'app

Techniquement : la commande Tauri `reset_config` supprime `config.json` et remet `DataSource` à `Unconfigured`. Le signal `is_configured` (contexte Leptos `ConfigCtx`) passe à `Some(false)` et l'app affiche la `SetupPage`.

---

## Architecture réseau (mode multi-postes)

Pour utiliser l'application sur **2 PC reliés par câble RJ45** :

```
PC Serveur                         PC Client
┌─────────────────┐                ┌─────────────────┐
│  Tauri App      │◄── RJ45 ──────►│  Tauri App      │
│  Axum API :7654 │                │  (reqwest)      │
│  SQLite (fjkm.db)│               │  (pas de DB)    │
└─────────────────┘                └─────────────────┘
```

**DataSource** (dans `lib.rs`) est un enum qui dispatche chaque commande Tauri vers :
- `Local(Repository)` — requêtes SQLite directes (mode serveur)
- `Remote(RemoteClient)` — appels HTTP vers l'API Axum (mode client)
- `Unconfigured` — état initial avant configuration

La configuration est stockée dans `{AppData}/mg.fjkm.ambalavao.isotry/config.json`.

---

## Développement

### Prérequis

- Rust (stable) + `cargo`
- Node.js + `npm` (pour Tailwind)
- `trunk` : `cargo install trunk`
- `cargo-tauri` : `cargo install tauri-cli`
- Cible WASM : `rustup target add wasm32-unknown-unknown`

### Lancer en mode développement

```bash
cargo tauri dev
```

### Construire l'installeur

```bash
cargo tauri build
# → target/release/bundle/nsis/FJKM Ambalavao Isotry_x.x.x_x64-setup.exe
```

---

## Base de données

La base SQLite (`fjkm.db`) est créée automatiquement dans le dossier de données de l'application :

- **Windows** : `C:\Users\{nom}\AppData\Roaming\mg.fjkm.ambalavao.isotry\`

Les migrations sont embarquées dans le binaire (`src-tauri/migrations/`) et s'exécutent automatiquement au premier lancement.

---

## Licence

Usage interne — FJKM Ambalavao Isotry, Antananarivo, Madagascar.
