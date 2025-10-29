use sqlx::PgPool;
use sqlx::postgres::PgPoolOptions;

async fn connect_to_db(url: &str) -> Result<PgPool, sqlx::Error> {
    let db_pool = PgPoolOptions::new()
        .max_connections(30)
        .connect(url)
        .await?;

    Ok(db_pool)
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
        json_path,
        arr_ref.len()
    );
    for item in arr_ref {
        let name = item
            .get("name")
            .and_then(|v| v.as_str())
            .ok_or("champ 'name' manquant")?;
        // types
        let (type1, type2) = if let Some(types) = item.get("type").and_then(|v| v.as_array()) {
            let t1 = types.first().and_then(|v| v.as_str()).unwrap_or("");
            let t2 = types.get(1).and_then(|v| v.as_str());
            (t1.to_string(), t2.map(|s| s.to_string()))
        } else if let Some(types) = item.get("types").and_then(|v| v.as_array()) {
            let t1 = types.first().and_then(|v| v.as_str()).unwrap_or("");
            let t2 = types.get(1).and_then(|v| v.as_str());
            (t1.to_string(), t2.map(|s| s.to_string()))
        } else {
            let t1 = item.get("type1").and_then(|v| v.as_str()).unwrap_or("");
            let t2 = item.get("type2").and_then(|v| v.as_str());
            (t1.to_string(), t2.map(|s| s.to_string()))
        };
        if type1.is_empty() {
            return Err(format!("type1 manquant pour {name}").into());
        }
        // new optional fields
        let dex_no: Option<i32> = item
            .get("dex_no")
            .and_then(|v| v.as_i64())
            .map(|x| x as i32)
            .or_else(|| {
                item.get("num")
                    .and_then(|v| v.as_str())
                    .and_then(|s| s.trim().parse::<i32>().ok())
            });
        let image_url: Option<String> = item
            .get("image_url")
            .and_then(|v| v.as_str())
            .map(|s| s.to_string())
            .or_else(|| {
                item.get("img")
                    .and_then(|v| v.as_str())
                    .map(|s| s.to_string())
            });
        let height_m: Option<f64> = item.get("height_m").and_then(|v| v.as_f64()).or_else(|| {
            item.get("height")
                .and_then(|v| v.as_str())
                .and_then(|s| s.split_whitespace().next())
                .and_then(|n| n.replace(',', ".").parse::<f64>().ok())
        });
        let weight_kg: Option<f64> = item.get("weight_kg").and_then(|v| v.as_f64()).or_else(|| {
            item.get("weight")
                .and_then(|v| v.as_str())
                .and_then(|s| s.split_whitespace().next())
                .and_then(|n| n.replace(',', ".").parse::<f64>().ok())
        });
        let weaknesses: Option<Vec<String>> = item
            .get("weaknesses")
            .and_then(|v| v.as_array())
            .map(|arr| {
                arr.iter()
                    .filter_map(|x| x.as_str().map(|s| s.to_string()))
                    .collect()
            });
        // stats
        let (hp, atk, def, spa, spd, spe) = {
            if let Some(stats) = item.get("stats").or_else(|| item.get("baseStats")) {
                let get = |k: &str| stats.get(k).and_then(|v| v.as_i64()).map(|x| x as i32);
                (
                    get("hp"),
                    get("attack"),
                    get("defense"),
                    get("sp_attack").or_else(|| get("spAttack").or_else(|| get("special-attack"))),
                    get("sp_defense")
                        .or_else(|| get("spDefense").or_else(|| get("special-defense"))),
                    get("speed"),
                )
            } else {
                let get = |k: &str| item.get(k).and_then(|v| v.as_i64()).map(|x| x as i32);
                (
                    get("base_hp"),
                    get("base_attack"),
                    get("base_defense"),
                    get("base_sp_attack"),
                    get("base_sp_defense"),
                    get("base_speed"),
                )
            }
        };

        sqlx::query(
            r#"
            INSERT INTO pokemon (
                name, type1, type2,
                base_hp, base_attack, base_defense, base_sp_attack, base_sp_defense, base_speed,
                dex_no, image_url, height_m, weight_kg, weaknesses
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
              weaknesses = EXCLUDED.weaknesses
            "#,
        )
        .bind(name)
        .bind(&type1)
        .bind(type2.as_deref())
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
        .bind(weaknesses)
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
    if let Err(e) = seed(&pool, &["data/gen1.json"]).await {
        eprintln!("🌱 seed ignoré: {e}");
    }
    pool
}
