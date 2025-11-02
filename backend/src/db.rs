use sqlx::PgPool;
use sqlx::postgres::PgPoolOptions;
use std::time::Duration;
use tokio::time::sleep;

async fn connect_to_db(url: &str) -> Result<PgPool, sqlx::Error> {
    // Try a few times in case Postgres just became healthy but isn't accepting TCP yet.
    let mut backoff = 1u64;
    for attempt in 1..=6 {
        match PgPoolOptions::new()
            .max_connections(10)
            .acquire_timeout(Duration::from_secs(10))
            .connect(url)
            .await
        {
            Ok(pool) => return Ok(pool),
            Err(e) => {
                eprintln!(
                    "⏳ Connexion DB tentative {attempt}/6 échouée: {e} (re-tentative dans {backoff}s)"
                );
                sleep(Duration::from_secs(backoff)).await;
                backoff = (backoff * 2).min(8);
            }
        }
    }
    // Dernière tentative avec timeout un peu plus long pour surface l'erreur claire si ça persiste
    PgPoolOptions::new()
        .max_connections(10)
        .acquire_timeout(Duration::from_secs(30))
        .connect(url)
        .await
}

async fn run_migrations(pool: &sqlx::PgPool) -> Result<(), sqlx::Error> {
    println!("🔄 Exécution des migrations...");

    let migration_result = sqlx::migrate!("./migrations").run(pool).await;

    match migration_result {
        Ok(_) => println!("✅ Migrations exécutées avec succès!"),
        Err(e) => println!("⚠️ Erreur lors des migrations : {}", e),
    }

    Ok(())
}

async fn seed(pool: &PgPool, json_files: &[&str]) -> Result<(), Box<dyn std::error::Error>> {
    let count: i64 = match sqlx::query_scalar("SELECT COUNT(*) FROM pokemon")
        .fetch_one(pool)
        .await
    {
        Ok(c) => c,
        Err(e) => {
            return Err(format!("pokemon table not ready: {e}").into());
        }
    };
    if count > 0 {
        return Ok(());
    }

    for path in json_files {
        seed_from_json(pool, path).await?;
    }

    Ok(())
}

async fn seed_from_json(pool: &PgPool, json_path: &str) -> Result<(), Box<dyn std::error::Error>> {
    dotenvy::dotenv().ok();
    let len: usize = 649; // Taille des 5 premieres générations
    let data = std::fs::read_to_string(json_path)?;
    let value: serde_json::Value = serde_json::from_str(&data)?;
    let arr_ref: &Vec<serde_json::Value> = if let Some(a) = value.as_array() {
        a
    } else if let Some(obj) = value.as_object() {
        if let Some(a) = obj.get("pokemon").and_then(|v| v.as_array()) {
            a
        } else if let Some(a) = obj.values().find_map(|v| v.as_array()) {
            a
        } else {
            return Err("Le JSON doit être un tableau ou contenir un tableau".into());
        }
    } else {
        return Err("Le JSON doit être un tableau".into());
    };

    println!(
        "🌱 fichier: {} — insertion de {} pokémons...",
        json_path, len
    );
    for item in arr_ref.iter().take(len) {
        let name = item
            .get("name")
            .and_then(|v| v.get("english").and_then(|x| x.as_str()))
            .ok_or("champ 'name.english' manquant")?;
        // types
        let types = item
            .get("type")
            .and_then(|v| v.as_array())
            .ok_or("champ 'type' manquant")?;
        let type1 = types
            .first()
            .and_then(|v| v.as_str())
            .ok_or("type[0] manquant")?;
        let type2 = types.get(1).and_then(|v| v.as_str());
        // id, image
        let dex_no: i32 = item
            .get("id")
            .and_then(|v| v.as_i64())
            .ok_or("champ 'id' manquant")? as i32;
        let image_url = item
            .get("image")
            .and_then(|img| img.get("hires").and_then(|v| v.as_str()))
            .ok_or("image.hires manquant")?;
        let height_m: Option<f64> = item.get("profile").and_then(|p| {
            p.get("height").and_then(|h| {
                h.as_str()
                    .and_then(|s| s.split_whitespace().next())
                    .and_then(|n| n.replace(',', ".").parse::<f64>().ok())
            })
        });
        let weight_kg: Option<f64> = item.get("profile").and_then(|p| {
            p.get("weight").and_then(|w| {
                w.as_str()
                    .and_then(|s| s.split_whitespace().next())
                    .and_then(|n| n.replace(',', ".").parse::<f64>().ok())
            })
        });
        let description: Option<String> = item
            .get("description")
            .and_then(|v| v.as_str())
            .map(|s| s.to_string());
        // stats
        let (hp, atk, def, spa, spd, spe) = {
            if let Some(stats) = item.get("base") {
                let get = |k: &str| stats.get(k).and_then(|v| v.as_i64()).map(|x| x as i32);
                (
                    get("HP"),
                    get("Attack"),
                    get("Defense"),
                    get("Sp. Attack"),
                    get("Sp. Defense"),
                    get("Speed"),
                )
            } else {
                return Err("champ 'base' manquant".into());
            }
        };

        sqlx::query(
            r#"
                        INSERT INTO pokemon (
                                name, type1, type2,
                                base_hp, base_attack, base_defense, base_sp_attack, base_sp_defense, base_speed,
                                dex_no, image_url, height_m, weight_kg, description
                        )
                        VALUES ($1,$2,$3,$4,$5,$6,$7,$8,$9,$10,$11,$12,$13,$14)
            ON CONFLICT (name) DO UPDATE SET
              type1 = EXCLUDED.type1,
              type2 = EXCLUDED.type2,
              base_hp = EXCLUDED.base_hp,
              base_attack = EXCLUDED.base_attack,
              base_defense = EXCLUDED.base_defense,
              base_sp_attack = EXCLUDED.base_sp_attack,
              base_sp_defense = EXCLUDED.base_sp_defense,
              base_speed = EXCLUDED.base_speed,
              dex_no = EXCLUDED.dex_no,
              image_url = EXCLUDED.image_url,
              height_m = EXCLUDED.height_m,
                            weight_kg = EXCLUDED.weight_kg,
                            description = EXCLUDED.description
            "#,
        )
        .bind(name)
        .bind(type1)
        .bind(type2)
        .bind(hp)
        .bind(atk)
        .bind(def)
        .bind(spa)
        .bind(spd)
        .bind(spe)
        .bind(dex_no)
        .bind(image_url)
        .bind(height_m)
        .bind(weight_kg)
        .bind(description)
        .execute(pool)
        .await?;
    }

    println!("🌱 seed terminé");
    Ok(())
}

pub async fn init_db(url: &str) -> PgPool {
    let pool = connect_to_db(url)
        .await
        .unwrap_or_else(|e| panic!("Echec connexion DB: {e}"));
    let _ = run_migrations(&pool).await;
    if let Err(e) = seed(&pool, &["data/pokedex.json"]).await {
        eprintln!("🌱 seed ignoré: {e}");
    }
    pool
}
