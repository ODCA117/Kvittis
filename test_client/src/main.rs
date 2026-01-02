use anyhow::Result;
use clap::Parser;
use common::{
    api::{
        CreateExpenseRequest, CreateGroupRequest, CreateGroupResponse, FriendRequest,
        GetUserResponse, RegisterRequest, RegisterResponse,
    },
    Expense, User, UserId,
};
use rand::Rng;
use reqwest::{Client as HttpClient, Url};

const REGISTER_ENDPOINT: &str = "/register";
const GET_USER_ENDPOINT: &str = "/user/";
const GET_USERS_ENDPOINT: &str = "/users";
const ADD_FRIEND_ENDPOINT: &str = "/friend";
const CREATE_EXPENSE_ENDPOINT: &str = "/expense";
const CREATE_GROUP_ENDPOINT: &str = "/group";

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
        let url = self.url.join(ADD_FRIEND_ENDPOINT)?;
        let friend_request = FriendRequest { user_id, friend_id };
        let resp = self.http.post(url).json(&friend_request).send().await?;
        dbg!(&resp);
        match resp.status().is_success() {
            true => Ok(()),
            false => Err(anyhow::anyhow!("Failed to add friend")),
        }
    }

    async fn create_expense(
        &self,
        payer: UserId,
        participants: Vec<UserId>,
        amount: u64,
        description: Option<String>,
    ) -> Result<CreateExpenseRequest> {
        let url = self.url.join(CREATE_EXPENSE_ENDPOINT)?;
        let request = CreateExpenseRequest {
            payer,
            amount,
            description,
            participants,
            group_id: None,
        };

        let resp = self.http.post(url).json(&request).send().await?;
        dbg!(&resp);
        let resp = resp.json::<CreateExpenseRequest>().await?;
        Ok(resp)
    }

    async fn create_group(
        &self,
        name: &str,
        owner_id: UserId,
        members: Vec<UserId>,
    ) -> Result<CreateGroupResponse> {
        let url = self.url.join(CREATE_GROUP_ENDPOINT)?;
        let request = CreateGroupRequest {
            name: name.to_owned(),
            owner_id,
            members,
        };
        let resp = self.http.post(url).json(&request).send().await?;
        dbg!(&resp);
        let resp = resp.json::<CreateGroupResponse>().await?;
        Ok(resp)
    }
}

fn generate_name() -> String {
    let n = rand::rng().random_range(1..10000);
    let name = format!("user_{}", n);
    name
}

#[tokio::main]
async fn main() -> Result<()> {
    let args = Cli::parse();

    let test_client = KvittisClient::new(&args.url)?;
    let user: User = test_client.register_user(&generate_name()).await?.into();
    let user2: User = test_client.register_user(&generate_name()).await?.into();
    test_client.add_friend(user.id, user2.id).await?;

    let usr: User = test_client.get_user(user.id).await?.into();
    dbg!(usr);

    let users: Vec<User> = test_client
        .get_users()
        .await?
        .into_iter()
        .map(|f| User::from(f))
        .collect();

    dbg!(users);

    // let group = test_client
    //     .create_group("test_group", user.id, vec![user.id, user2.id])
    //     .await?;
    // dbg!(&group);
    //
    // let expense = test_client
    //     .create_expense(
    //         user.id,
    //         vec![user.id, user2.id],
    //         1000,
    //         Some("Lunch".to_string()),
    //     )
    //     .await?;
    // dbg!(&expense);
    Ok(())
}
