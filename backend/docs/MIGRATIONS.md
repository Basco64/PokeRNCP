# Migrations SQLx (PostgreSQL)

Ce backend utilise `sqlx::migrate!` au démarrage/test pour appliquer les migrations présentes dans `backend/migrations`.

## Principe

- Chaque migration a une paire de fichiers:
  - `<timestamp>_<name>.up.sql` — à appliquer en avant
  - `<timestamp>_<name>.down.sql` — à appliquer en arrière (rollback)
- Les fichiers sont ordonnés par timestamp croissant.
- Au démarrage, on exécute `sqlx::migrate!("./migrations").run(&pool)` pour appliquer tout ce qui manque.

## Commandes utiles (sqlx-cli)

Assurez-vous d'avoir `sqlx-cli` installé et vos variables d'environnement (DATABASE_URL ou TEST_DATABASE_URL) définies.

```bash
# Installer l'outil (si besoin)
cargo install sqlx-cli --no-default-features --features rustls,postgres

# Créer la base (si pas encore créée)
sqlx database create --database-url "$DATABASE_URL"

# Créer une nouvelle migration (ex)
sqlx migrate add your_feature

# Appliquer toutes les migrations en attente
sqlx migrate run --database-url "$DATABASE_URL"

# Revenir en arrière d'une migration
sqlx migrate revert --database-url "$DATABASE_URL"

# Voir l'état
sqlx migrate info --database-url "$DATABASE_URL"
```

Note: Sous Windows avec bash.exe, exportez la variable si nécessaire:

```bash
export DATABASE_URL='postgres://user:pass@localhost:5432/PokeRNCP'
```

## Exemples dans ce repo

- `20251019141600_init` — Schéma initial (users, pokemon, user_pokemon, index)
- `20251029120000_add_user_profile_fields` — Ajout de colonnes facultatives/sûres (`bio`, `is_active`)
- `20251029121000_add_audit_logs` — Nouvelle table annexe avec indexes (`audit_logs`)
- `20251029122000_email_lowercase_data_migration` — Migration de données (mise en minuscule des emails)

### Bonnes pratiques

- Toujours ajouter des colonnes avec `DEFAULT` et/ou `NULL` pour ne pas casser les INSERT existants.
- Écrire un `DOWN` quand c'est possible; documenter quand ce n'est pas réversible (ex: migrations de données).
- Prévoir des index pour les requêtes fréquentes.
- Regrouper les modifications atomiques par migration et les nommer clairement.
- En test/intégration, laisser l'app/les tests appliquer les migrations automatiquement.

### Seed et migrations

Le démarrage (et les tests) exécutent aussi un seed Gen1 si la table `pokemon` est vide.

- Le seed est idempotent (UPSERT par `name`).
- Pour re-seed: `TRUNCATE TABLE pokemon RESTART IDENTITY CASCADE;` puis relancer.
