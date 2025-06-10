pub mod db {
    use futures_util::stream::StreamExt; 
    use mongodb::Database;
    use mongodb::bson::doc;
    use mongodb::{Client, options::ClientOptions};
    use serde::{Deserialize, Serialize};
    use mongodb::bson::{Document};
    use serde_json;

    #[derive(Debug, Serialize, Deserialize)]
   pub struct Player {
        username: String,
        score: i32,
    }

    pub async fn connect_db() -> mongodb::error::Result<Database> {
        let uri = "mongodb+srv://shrivastavavaibhav7:U9Ql3IgU3YEhniMx@cluster0.hksxvwm.mongodb.net/?retryWrites=true&w=majority&appName=Cluster0";
        let mut client_options = ClientOptions::parse(uri).await?;
        client_options.app_name = Some("SimpleRustApp".to_string());

        let client = Client::with_options(client_options)?;
        Ok(client.database("gameDB"))
    }

    pub async fn insert_player() -> mongodb::error::Result<()> {
        let db = connect_db().await?;
        let players = db.collection::<Player>("players");

        let new_player = Player {
            username: "vaibhav_simple new module".to_string(),
            score: 159999,
        };

        players.insert_one(new_player, None).await?;
        println!("✅ Player inserted!");
        Ok(())
    }

    pub async fn get_players() -> mongodb::error::Result<()> {
        let db = connect_db().await?;
        let players =  db.collection::<Document>("players");

        let mut cursor = players.find(None, None).await?;
        //let mut player_list = Vec::new();

        while let Some(result) = cursor.next().await {
            match result {
            Ok(doc) => {
                match serde_json::to_string_pretty(&doc) {
                    Ok(json_str) => println!("📄 JSON:\n{}", json_str),
                    Err(e) => eprintln!("❌ JSON conversion failed: {}", e),
                }
            }
            Err(e) => eprintln!("❌ Failed to read document: {}", e),
        }
        }
        Ok(())
    }

    pub async fn update_player_score(username: &str, new_score: i32) -> mongodb::error::Result<()> {
        let db = connect_db().await?;
        let players = db.collection::<Player>("players");

        let filter = doc! { "username": username };
        let update = doc! { "$set": { "score": new_score } };

        let result = players.update_one(filter, update, None).await?;
        if result.matched_count > 0 {
            println!("✅ Player score updated!");
        } else {
            println!("❌ No player found with username: {}", username);
        }
        Ok(())
    }
}
