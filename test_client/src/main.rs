use anyhow::Result;
use clap::Parser;
use common::{
    api::{FriendRequest, GetUserResponse, RegisterRequest, RegisterResponse}, User, UserId,
};
use reqwest::{Client as HttpClient, Url};

const REGISTER_ENDPOINT: &str = "/register";
const GET_USER_ENDPOINT: &str = "/user/";
const GET_USERS_ENDPOINT: &str = "/users";
const GET_FRIEND_ENDPOINT: &str = "/friend";

#[derive(Parser)]
#[command(version, about, long_about = None)]
struct Cli {
    #[arg(short, long, default_value = "http://localhost:3000")]
    url: String,
}

struct KvittisClient {
    http: HttpClient,
    url: Url,
}

impl KvittisClient {
    fn new(base_url: &str) -> Result<Self> {
        let url = Url::parse(base_url)?;
        Ok(KvittisClient {
            http: HttpClient::new(),
            url,
        })
    }

    async fn register_user(&self, name: &str) -> Result<RegisterResponse> {
        let req = RegisterRequest {
            username: name.to_owned(),
        };

        let resp = self
            .http
            .post(self.url.join(REGISTER_ENDPOINT)?)
            .json(&req)
            .send()
            .await?;
        dbg!(&resp);
        let resp = resp.json::<RegisterResponse>().await?;
        Ok(resp)
    }

    async fn get_user(&self, id: UserId) -> Result<GetUserResponse> {
        let url = self.url.join(GET_USER_ENDPOINT)?.join(&id.to_string())?;

        let resp = self.http.get(url).send().await?;
        dbg!(&resp);
        let resp = resp.json::<GetUserResponse>().await?;
        Ok(resp)
    }

    async fn get_users(&self) -> Result<Vec<GetUserResponse>> {
        let url = self.url.join(GET_USERS_ENDPOINT)?;
        let resp = self.http.get(url).send().await?;
        dbg!(&resp);
        let resp = resp.json::<Vec<GetUserResponse>>().await?;
        Ok(resp)
    }

    async fn add_friend(&self, user_id: UserId, friend_id: UserId) -> Result<()> {
        let url = self.url.join(GET_FRIEND_ENDPOINT)?;
        let friend_request = FriendRequest { user_id, friend_id };
        let resp = self.http.post(url).json(&friend_request).send().await?;
        dbg!(&resp);
        match resp.status().is_success() {
            true => Ok(()),
            false => Err(anyhow::anyhow!("Failed to add friend")),
        }
    }
}

#[tokio::main]
async fn main() -> Result<()> {
    let args = Cli::parse();

    let test_client = KvittisClient::new(&args.url)?;
    let user: User = test_client.register_user("test_name").await?.into();
    let user2: User = test_client.register_user("test2_name").await?.into();
    dbg!(&user);
    let user = test_client.get_user(user.id).await?;
    dbg!(&user);
    let users = test_client.get_users().await?;
    dbg!(&users);
    test_client.add_friend(user.id, user2.id).await?;
    let user = test_client.get_user(user.id).await?;
    dbg!(&user);
    let user2 = test_client.get_user(user2.id).await?;
    dbg!(&user2);
    Ok(())
}
