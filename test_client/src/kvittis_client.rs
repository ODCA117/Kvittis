use reqwest::{Client as HttpClient, Url};
use anyhow::Result;
use common::{
    ExpenseId, GroupId, UserId, api::{
        ApiResponse, BalanceRequest, CreateExpenseResponse, CreateGroupResponse,
        ExpenseRequest, GetExpenseResponse, GetGroupResponse, GetUserResponse,
        GroupRequest, RegisterResponse, UserRequest,
    }
};

const USER_ENDPOINT: &str = "/api/user";
const GROUP_ENDPOINT: &str = "/api/group";
const EXPENSE_ENDPOINT: &str = "/api/expense";
const BALANCE_ENDPOINT: &str = "/api/balance";

pub struct KvittisClient {
    pub http: HttpClient,
    pub url: Url,
}

impl KvittisClient {
    pub fn new(base_url: &str) -> Result<Self> {
        let url = Url::parse(base_url)?;
        Ok(KvittisClient {
            http: HttpClient::new(),
            url,
        })
    }

    pub async fn register_user(&self, name: &str) -> Result<RegisterResponse> {
        let req = UserRequest::Register { username: name.to_owned() };
        let resp = self
            .http
            .post(self.url.join(USER_ENDPOINT)?)
            .json(&req)
            .send()
            .await?;
        dbg!(&resp);
        let resp = resp.json::<ApiResponse<RegisterResponse>>().await?;
        match resp {
            ApiResponse::Success(r) => Ok(r),
            ApiResponse::Error { message } => Err(anyhow::anyhow!(message)),
        }
    }

    pub async fn delete_user(&self, id: UserId) -> Result<()> {
        let req = UserRequest::Delete { user_id: id };
        let resp = self
            .http
            .post(self.url.join(USER_ENDPOINT)?)
            .json(&req)
            .send()
            .await?;
        dbg!(&resp);
        match resp.status().is_success() {
            true => Ok(()),
            false => Err(anyhow::anyhow!("Failed to delete user")),
        }
    }

    pub async fn get_user(&self, id: UserId) -> Result<GetUserResponse> {
        let req = UserRequest::Get { user_id: id };
        let resp = self
            .http
            .post(self.url.join(USER_ENDPOINT)?)
            .json(&req)
            .send()
            .await?;
        dbg!(&resp);
        let resp = resp.json::<ApiResponse<GetUserResponse>>().await?;
        match resp {
            ApiResponse::Success(r) => Ok(r),
            ApiResponse::Error { message } => Err(anyhow::anyhow!(message)),
        }
    }

    pub async fn get_users(&self) -> Result<Vec<GetUserResponse>> {
        let req = UserRequest::List;
        let resp = self
            .http
            .post(self.url.join(USER_ENDPOINT)?)
            .json(&req)
            .send()
            .await?;
        dbg!(&resp);
        let resp = resp.json::<ApiResponse<Vec<GetUserResponse>>>().await?;
        match resp {
            ApiResponse::Success(r) => Ok(r),
            ApiResponse::Error { message } => Err(anyhow::anyhow!(message)),
        }
    }

    pub async fn search_users(&self, query: &str) -> Result<Vec<GetUserResponse>> {
        let req = UserRequest::Search { query: query.to_owned() };
        let resp = self
            .http
            .post(self.url.join(USER_ENDPOINT)?)
            .json(&req)
            .send()
            .await?;
        dbg!(&resp);
        let resp = resp.json::<ApiResponse<Vec<GetUserResponse>>>().await?;
        match resp {
            ApiResponse::Success(r) => Ok(r),
            ApiResponse::Error { message } => Err(anyhow::anyhow!(message)),
        }
    }

    pub async fn add_friend(&self, user_id: UserId, friend_id: UserId) -> Result<()> {
        let req = UserRequest::AddFriend { user_id, friend_id };
        let resp = self
            .http
            .post(self.url.join(USER_ENDPOINT)?)
            .json(&req)
            .send()
            .await?;
        dbg!(&resp);
        match resp.status().is_success() {
            true => Ok(()),
            false => Err(anyhow::anyhow!("Failed to add friend")),
        }
    }

    pub async fn create_expense(
        &self,
        payer: UserId,
        participants: Vec<UserId>,
        amount: i64,
        description: Option<String>,
        group_id: Option<GroupId>,
    ) -> Result<CreateExpenseResponse> {
        let req = ExpenseRequest::Create { payer, participants, amount, description, group_id };
        let resp = self
            .http
            .post(self.url.join(EXPENSE_ENDPOINT)?)
            .json(&req)
            .send()
            .await?;
        dbg!(&resp);
        let resp = resp.json::<ApiResponse<CreateExpenseResponse>>().await?;
        match resp {
            ApiResponse::Success(r) => Ok(r),
            ApiResponse::Error { message } => Err(anyhow::anyhow!(message)),
        }
    }

    pub async fn delete_expense(&self, expense_id: ExpenseId) -> Result<()> {
        let req = ExpenseRequest::Delete { id: expense_id };
        let resp = self
            .http
            .post(self.url.join(EXPENSE_ENDPOINT)?)
            .json(&req)
            .send()
            .await?;
        dbg!(&resp);
        match resp.status().is_success() {
            true => Ok(()),
            false => Err(anyhow::anyhow!("Failed to delete expense")),
        }
    }

    pub async fn get_expense(&self, expense_id: ExpenseId) -> Result<GetExpenseResponse> {
        let req = ExpenseRequest::Get { id: expense_id };
        let resp = self
            .http
            .post(self.url.join(EXPENSE_ENDPOINT)?)
            .json(&req)
            .send()
            .await?;
        dbg!(&resp);
        let resp = resp.json::<ApiResponse<GetExpenseResponse>>().await?;
        match resp {
            ApiResponse::Success(r) => Ok(r),
            ApiResponse::Error { message } => Err(anyhow::anyhow!(message)),
        }
    }

    pub async fn create_group(
        &self,
        name: &str,
        owner_id: UserId,
        members: Vec<UserId>,
    ) -> Result<CreateGroupResponse> {
        let req = GroupRequest::Create {
            name: name.to_owned(),
            owner_id,
            members,
        };
        let resp = self
            .http
            .post(self.url.join(GROUP_ENDPOINT)?)
            .json(&req)
            .send()
            .await?;
        dbg!(&resp);
        let resp = resp.json::<ApiResponse<CreateGroupResponse>>().await?;
        match resp {
            ApiResponse::Success(r) => Ok(r),
            ApiResponse::Error { message } => Err(anyhow::anyhow!(message)),
        }
    }

    pub async fn delete_group(&self, id: GroupId) -> Result<()> {
        let req = GroupRequest::Delete { group_id: id };
        let resp = self
            .http
            .post(self.url.join(GROUP_ENDPOINT)?)
            .json(&req)
            .send()
            .await?;
        dbg!(&resp);
        match resp.status().is_success() {
            true => Ok(()),
            false => Err(anyhow::anyhow!("Failed to delete group")),
        }
    }

    pub async fn search_group(&self, query: &str) -> Result<Vec<GetGroupResponse>> {
        let req = GroupRequest::Search { query: query.to_owned() };
        let resp = self
            .http
            .post(self.url.join(GROUP_ENDPOINT)?)
            .json(&req)
            .send()
            .await?;
        dbg!(&resp);
        let resp = resp.json::<ApiResponse<Vec<GetGroupResponse>>>().await?;
        match resp {
            ApiResponse::Success(r) => Ok(r),
            ApiResponse::Error { message } => Err(anyhow::anyhow!(message)),
        }
    }

    pub async fn get_group(&self, id: GroupId) -> Result<GetGroupResponse> {
        let req = GroupRequest::Get { group_id: id };
        let resp = self
            .http
            .post(self.url.join(GROUP_ENDPOINT)?)
            .json(&req)
            .send()
            .await?;
        dbg!(&resp);
        let resp = resp.json::<ApiResponse<GetGroupResponse>>().await?;
        match resp {
            ApiResponse::Success(r) => Ok(r),
            ApiResponse::Error { message } => Err(anyhow::anyhow!(message)),
        }
    }

    pub async fn add_user_to_group(&self, group_id: GroupId, user_id: UserId) -> Result<()> {
        let req = GroupRequest::AddMember { group_id, new_member: user_id };
        let resp = self
            .http
            .post(self.url.join(GROUP_ENDPOINT)?)
            .json(&req)
            .send()
            .await?;
        dbg!(&resp);
        match resp.status().is_success() {
            true => Ok(()),
            false => Err(anyhow::anyhow!("Failed to add user to group")),
        }
    }
}
