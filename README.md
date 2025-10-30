## Backend — PokeRNCP

### Prérequis

- PostgreSQL 14+
- Rust (toolchain stable) + Cargo
- sqlx-cli (pour gérer la DB en ligne de commande)

### Variables d'environnement

- DATABASE_URL: URL Postgres (ex: postgres://user:pass@localhost:5432/PokeRNCP)
- JWT_SECRET: secret pour signer les tokens d'accès
- JWT_REFRESH_SECRET (optionnel): secret pour le refresh (sinon fallback sur JWT_SECRET)
- JWT_EXP_SECONDS (optionnel, défaut 900)
- JWT_REFRESH_EXP_SECONDS (optionnel, défaut 2592000)
- RESET_SECRET (optionnel, défaut JWT_SECRET)
- RESET_TOKEN_EXP_SECONDS (optionnel, défaut 3600)
- FRONTEND_ORIGIN: ex http://localhost:3000 (CORS)
- BACKEND_URL: adresse d'écoute du backend. Accepte soit "hôte:port" (ex: 0.0.0.0:8080), soit une URL complète (ex: http://0.0.0.0:8080).
- PRODUCTION_MODE: "true" en prod pour ajouter Secure sur les cookies.

### Initialisation base de données

Option sqlx-cli:

```bash
sqlx database create --database-url "$DATABASE_URL"
sqlx migrate run   --database-url "$DATABASE_URL"
```

Au premier démarrage du serveur, un seed auto insère les `LEN_POKEDEX` premières entrées du fichier `backend/data/pokedex.json` si la table `pokemon` est vide (par défaut 649).
Le seed est idempotent (UPSERT par name). Pour re-seed: vider la table puis relancer.

Vider la table proprement (PostgreSQL):

```sql
TRUNCATE TABLE pokemon RESTART IDENTITY CASCADE;
```

### Lancer le serveur

```bash
cargo run
```

### Tests (runner recommandé)

Pour une exécution plus lisible et rapide des tests, on utilise cargo-nextest via des alias Cargo.

Pré-requis (une fois):

```bash
rustup update stable
cargo install cargo-nextest
```

Variables utiles pour les tests:

- TEST_DATABASE_URL: URL Postgres de test; sinon fallback sur DATABASE_URL.

Lancer les tests:

```bash
# Runner nextest (alias défini dans .cargo/config.toml)
cargo tester
# équivalent plus court
cargo t

# Runner natif en mode concis (si besoin)
cargo test -q
```

Dans VS Code:

- Menu “Run Task…” → “Test (nextest)” est la tâche de test par défaut.

### API — Auth

- POST /api/auth/login

  - Body: { username: string, password: string }
  - Effet: set-cookie httpOnly "auth" (access) + "refresh"

- POST /api/auth/refresh-token

  - Lit le refresh token via Authorization: Bearer <token> OU cookie httpOnly "refresh"
  - Effet: set-cookie httpOnly "auth" (access) régénéré

- POST /api/auth/logout

  - Effet: supprime les cookies "auth" et "refresh"

- GET /api/auth/me

  - Requiert cookie "auth" (ou Authorization: Bearer access)
  - Retour: { id, username, email }

- PUT /api/auth/change-password

  - Body: { current_password, new_password }

- POST /api/auth/request-password-reset

  - Body: { email_or_username }
  - Dev: peut renvoyer un reset_token (en prod: à envoyer par email)

- POST /api/auth/confirm-password-reset
  - Body: { token, new_password }

### API — Users

- POST /api/users

  - Body: { username, email?, password }
  - Conflit unique -> 409

- PATCH /api/users/{id}

  - Body: { username?, email?, password? }
  - Requiert CurrentUser = id

- DELETE /api/users/{id}
  - Requiert CurrentUser = id

Note: La lecture du profil se fait via GET /api/auth/me (GET /api/users/{id} retiré).

### API — Pokémons

- GET /api/pokemons

  - Retourne la liste complète avec flag "caught" pour l'utilisateur courant

- GET /api/pokemons/search?q=prefix

  - 10 suggestions max, avec flag "caught"

- POST /api/pokemons/catch

  - Body: { name, nickname? }
  - Marque comme capturé (idempotent)

- GET /api/pokemons/{pokemon_id}
  - Détails du pokémon + flag "caught"

### Plan d'appel côté Frontend

1. Démarrage d'app

   - POST /api/auth/refresh-token (sans header): le serveur lit cookie "refresh" et replace "auth"
   - Si 401: aller à la page login

2. Login

   - POST /api/auth/login -> cookies posés
   - Ensuite GET /api/auth/me pour afficher le profil

3. Profil (lecture/modification)

   - GET /api/auth/me
   - PATCH /api/users/{id} pour changer username/email/password

4. Mot de passe oublié

   - POST /api/auth/request-password-reset -> récup token en dev
   - POST /api/auth/confirm-password-reset avec { token, new_password }

5. Pokédex
   - GET /api/pokemons -> lister tout (griser si caught=false)
   - GET /api/pokemons/search?q=… -> suggestions
   - POST /api/pokemons/catch { name, nickname? } -> déverrouiller/attraper
   - GET /api/pokemons/{id} -> page détail

### Notes

- Les endpoints protégés utilisent CurrentUser qui lit en priorité le cookie httpOnly "auth" (ou Authorization: Bearer access).
- CORS est configuré via FRONTEND_ORIGIN.
- Le backend écoute strictement sur BACKEND_URL (PORT n'est plus pris en charge dans le code). Si votre plateforme fournit uniquement PORT, définissez `BACKEND_URL=0.0.0.0:$PORT` au démarrage.
- Seed JSON: `backend/data/pokedex.json`. Pour ajouter d'autres seed: utiliser `seed_from_json(&pool, "data/genX.json")`.
