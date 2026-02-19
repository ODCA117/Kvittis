use common::api::{CreateGroupResponse, RegisterResponse};
use rust_kvittis_client::kvittis_client::KvittisClient;
use tokio;
use anyhow::Result;

async fn create_and_get_user(client: &KvittisClient, name: &str) -> anyhow::Result<()> {
    let register_response = client.register_user(name).await?;
    println!("Registered user: {:?}", register_response);

    let get_user_response = client.get_user(register_response.id).await?;
    println!("Fetched user: {:?}", get_user_response);

    Ok(())
}

#[tokio::main]
async fn main() -> Result<()> {
    /* Create client */
    let client = KvittisClient::new("http://localhost:3000")?;

    /* Create permanent Users */
    let RegisterResponse {id: alice_id, username: alice_un} = client.register_user("alice").await?;
    let RegisterResponse {id: bob_id, username: bob_un}  = client.register_user("bob").await?;
    let RegisterResponse {id: charlie_id, username: charlie_un}  = client.register_user("charlie").await?;

    /* Create permanent Groups */
    let CreateGroupResponse { id: fam_id, name: fam_name} = client.create_group("family", alice_id, vec![alice_id, bob_id]).await?;
    let CreateGroupResponse { id: friend_id, name: friend_name } = client.create_group("friends", charlie_id, vec![alice_id, charlie_id]).await?;
    
    /* Create permanent Expenses */
    Ok(())
}
