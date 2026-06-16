use anyhow::Result;
use common::{
    ExpenseId, FriendRequestAction, FriendRequestId, GroupId, GroupRole, NewUser, UserId, api::{
        ApiResponse, AuthorizedUserRequest, BalanceEntry, BalanceRequest, CreateExpenseResponse,
        CreateGroupResponse, ExpenseRequest, FriendRequestResponse, GetExpenseResponse,
        GetGroupResponse, GetUserResponse, GroupBalance, GroupRequest, HandleFriendRequestResponse,
        LoginResponse, PendingFriendRequestResponse, RegisterResponse, SearchUserResponse,
        UnauthorizedUserRequest,
    }
};
use reqwest::{Client as HttpClient, Url};

const AUTH_USER_ENDPOINT: &str = "/api/auth_user";
const UNAUTH_USER_ENDPOINT: &str = "/api/unauth_user";
const GROUP_ENDPOINT: &str = "/api/group";
const EXPENSE_ENDPOINT: &str = "/api/expense";
const BALANCE_ENDPOINT: &str = "/api/balance";

/// Unauthenticated client - entry point for the API.
/// Can only perform registration and login operations.
#[derive(Debug, Clone)]
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

    pub async fn register_user(&self, user: NewUser) -> Result<RegisterResponse> {
        let req = UnauthorizedUserRequest::Register { user };

        dbg!(req.clone());

        let resp = self
            .http
            .post(self.url.join(UNAUTH_USER_ENDPOINT)?)
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

    pub async fn login_user(
        &self,
        username: String,
        password: String,
    ) -> Result<AuthenticatedKvittisClient> {
        let req = UnauthorizedUserRequest::Login { username, password };

        dbg!(&req);
        let resp = self
            .http
            .post(self.url.join(UNAUTH_USER_ENDPOINT)?)
            .json(&req)
            .send()
            .await?;
        dbg!(&resp);
        let resp = resp.json::<ApiResponse<LoginResponse>>().await?;
        match resp {
            ApiResponse::Success(r) => Ok(AuthenticatedKvittisClient {
                http: self.http.clone(),
                url: self.url.clone(),
                token: r.token,
            }),
            ApiResponse::Error { message } => Err(anyhow::anyhow!(message)),
        }
    }
}

/// Authenticated client - holds a JWT and exposes all protected API methods.
/// Token is automatically sent via bearer authentication on all requests.
#[derive(Debug, Clone)]
pub struct AuthenticatedKvittisClient {
    pub http: HttpClient,
    pub url: Url,
    pub token: String,
}

impl AuthenticatedKvittisClient {
    /// Consumes self, discards the token, and returns an unauthenticated client.
    #[must_use]
    pub fn logout(self) -> KvittisClient {
        KvittisClient {
            http: self.http,
            url: self.url,
        }
    }

    // ========== User Methods ==========

    pub async fn delete_user(self) -> Result<()> {
        let req = AuthorizedUserRequest::Delete;
        let resp = self
            .http
            .post(self.url.join(AUTH_USER_ENDPOINT)?)
            .bearer_auth(&self.token)
            .json(&req)
            .send()
            .await?;
        dbg!(&resp);
        match resp.status().is_success() {
            true => Ok(()),
            false => Err(anyhow::anyhow!("Failed to delete user")),
        }
    }

    pub async fn get_user(&self) -> Result<GetUserResponse> {
        let req = AuthorizedUserRequest::Get;

        let send_req = self
            .http
            .post(self.url.join(AUTH_USER_ENDPOINT)?)
            .bearer_auth(&self.token)
            .json(&req);

        dbg!(&send_req);
        let resp = send_req.send().await?;
        dbg!(&resp);
        let resp = resp.json::<ApiResponse<GetUserResponse>>().await?;
        match resp {
            ApiResponse::Success(r) => Ok(r),
            ApiResponse::Error { message } => Err(anyhow::anyhow!(message)),
        }
    }

    pub async fn get_users(&self) -> Result<Vec<GetUserResponse>> {
        let req = AuthorizedUserRequest::List;
        let resp = self
            .http
            .post(self.url.join(AUTH_USER_ENDPOINT)?)
            .bearer_auth(&self.token)
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

    pub async fn search_users(&self, query: &str) -> Result<SearchUserResponse> {
        let req = AuthorizedUserRequest::Search {
            query: query.to_owned(),
        };
        let resp = self
            .http
            .post(self.url.join(AUTH_USER_ENDPOINT)?)
            .bearer_auth(&self.token)
            .json(&req)
            .send()
            .await?;
        dbg!(&resp);
        let resp = resp.json::<ApiResponse<SearchUserResponse>>().await?;
        match resp {
            ApiResponse::Success(r) => Ok(r),
            ApiResponse::Error { message } => Err(anyhow::anyhow!(message)),
        }
    }

    pub async fn send_friend_request(&self, friend_id: UserId) -> Result<FriendRequestResponse> {
        let req = AuthorizedUserRequest::SendFriendRequest { friend_id };
        let resp = self
            .http
            .post(self.url.join(AUTH_USER_ENDPOINT)?)
            .bearer_auth(&self.token)
            .json(&req)
            .send()
            .await?;
        dbg!(&resp);
        let resp = resp.json::<ApiResponse<FriendRequestResponse>>().await?;
        match resp {
            ApiResponse::Success(r) => Ok(r),
            ApiResponse::Error { message } => Err(anyhow::anyhow!(message)),
        }
    }

    pub async fn get_pending_friend_requests(&self) -> Result<PendingFriendRequestResponse> {
        let req = AuthorizedUserRequest::GetPendingFriendRequests;
        let resp = self
            .http
            .post(self.url.join(AUTH_USER_ENDPOINT)?)
            .bearer_auth(&self.token)
            .json(&req)
            .send()
            .await?;
        dbg!(&resp);
        let resp = resp
            .json::<ApiResponse<PendingFriendRequestResponse>>()
            .await?;
        match resp {
            ApiResponse::Success(r) => Ok(r),
            ApiResponse::Error { message } => Err(anyhow::anyhow!(message)),
        }
    }

    pub async fn handle_friend_request(
        &self,
        request_id: FriendRequestId,
        action: FriendRequestAction,
    ) -> Result<()> {
        let req = AuthorizedUserRequest::HandleFriendRequest {
            request_id,
            request_action: action,
        };
        dbg!(&req);
        let resp = self
            .http
            .post(self.url.join(AUTH_USER_ENDPOINT)?)
            .bearer_auth(&self.token)
            .json(&req)
            .send()
            .await?;
        dbg!(&resp);
        let resp = resp
            .json::<ApiResponse<HandleFriendRequestResponse>>()
            .await?;
        match resp {
            ApiResponse::Success(_r) => Ok(()),
            ApiResponse::Error { message } => Err(anyhow::anyhow!(message)),
        }
    }
    // ========== Expense Methods ==========

    pub async fn create_expense(
        &self,
        payer: UserId,
        participants: Vec<UserId>,
        amount: i64,
        description: Option<String>,
        group_id: Option<GroupId>,
    ) -> Result<CreateExpenseResponse> {
        let req = ExpenseRequest::Create {
            payer,
            participants,
            amount,
            description,
            group_id,
        };
        let resp = self
            .http
            .post(self.url.join(EXPENSE_ENDPOINT)?)
            .bearer_auth(&self.token)
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
            .bearer_auth(&self.token)
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
            .bearer_auth(&self.token)
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

    pub async fn list_expenses_for_user(&self, user_id: UserId) -> Result<Vec<GetExpenseResponse>> {
        let req = ExpenseRequest::ListForUser { user_id };
        let resp = self
            .http
            .post(self.url.join(EXPENSE_ENDPOINT)?)
            .bearer_auth(&self.token)
            .json(&req)
            .send()
            .await?;
        dbg!(&resp);
        let resp = resp.json::<ApiResponse<Vec<GetExpenseResponse>>>().await?;
        match resp {
            ApiResponse::Success(r) => Ok(r),
            ApiResponse::Error { message } => Err(anyhow::anyhow!(message)),
        }
    }

    pub async fn list_expenses_for_group(
        &self,
        group_id: GroupId,
    ) -> Result<Vec<GetExpenseResponse>> {
        let req = ExpenseRequest::ListForGroup { group_id };
        let resp = self
            .http
            .post(self.url.join(EXPENSE_ENDPOINT)?)
            .bearer_auth(&self.token)
            .json(&req)
            .send()
            .await?;
        dbg!(&resp);
        let resp = resp.json::<ApiResponse<Vec<GetExpenseResponse>>>().await?;
        match resp {
            ApiResponse::Success(r) => Ok(r),
            ApiResponse::Error { message } => Err(anyhow::anyhow!(message)),
        }
    }

    // ========== Group Methods ==========

    pub async fn create_group(
        &self,
        name: &str,
    ) -> Result<CreateGroupResponse> {
        let req = GroupRequest::Create {
            name: name.to_owned(),
        };
        let resp = self
            .http
            .post(self.url.join(GROUP_ENDPOINT)?)
            .bearer_auth(&self.token)
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

    pub async fn delete_group(&self, group_id: GroupId) -> Result<()> {
        let req = GroupRequest::Delete { group_id };
        let resp = self
            .http
            .post(self.url.join(GROUP_ENDPOINT)?)
            .bearer_auth(&self.token)
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
        let req = GroupRequest::Search {
            query: query.to_owned(),
        };
        let resp = self
            .http
            .post(self.url.join(GROUP_ENDPOINT)?)
            .bearer_auth(&self.token)
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

    pub async fn get_group(&self, group_id: GroupId) -> Result<GetGroupResponse> {
        let req = GroupRequest::Get { group_id };
        let resp = self
            .http
            .post(self.url.join(GROUP_ENDPOINT)?)
            .bearer_auth(&self.token)
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
        let req = GroupRequest::AddMember {
            group_id,
            new_member: user_id,
            role: GroupRole::Member,
        };
        let resp = self
            .http
            .post(self.url.join(GROUP_ENDPOINT)?)
            .bearer_auth(&self.token)
            .json(&req)
            .send()
            .await?;
        dbg!(&resp);
        match resp.status().is_success() {
            true => Ok(()),
            false => Err(anyhow::anyhow!("Failed to add user to group")),
        }
    }

    // ========== Balance Methods ==========

    pub async fn get_user_balances(&self, user_id: UserId) -> Result<Vec<BalanceEntry>> {
        let req = BalanceRequest::User { user_id };
        let resp = self
            .http
            .post(self.url.join(BALANCE_ENDPOINT)?)
            .bearer_auth(&self.token)
            .json(&req)
            .send()
            .await?;
        dbg!(&resp);
        let resp = resp.json::<ApiResponse<Vec<BalanceEntry>>>().await?;
        match resp {
            ApiResponse::Success(r) => Ok(r),
            ApiResponse::Error { message } => Err(anyhow::anyhow!(message)),
        }
    }

    pub async fn get_group_balances(&self, group_id: GroupId) -> Result<Vec<GroupBalance>> {
        let req = BalanceRequest::Group { group_id };
        let resp = self
            .http
            .post(self.url.join(BALANCE_ENDPOINT)?)
            .bearer_auth(&self.token)
            .json(&req)
            .send()
            .await?;
        dbg!(&resp);
        let resp = resp.json::<ApiResponse<Vec<GroupBalance>>>().await?;
        match resp {
            ApiResponse::Success(r) => Ok(r),
            ApiResponse::Error { message } => Err(anyhow::anyhow!(message)),
        }
    }
}
