use reqwest::{Client as HttpClient, Url};
use anyhow::Result;
use common::{
    Expense, GroupId, User, UserId, api::{
        CreateExpenseRequest, CreateGroupRequest, CreateGroupResponse, FriendRequest, GetGroupResponse, GetUserResponse, NewGroupMemberRequest, RegisterRequest, RegisterResponse, SearchGroupRequest, SearchUserRequest
    }
};

const REGISTER_ENDPOINT: &str = "/register";
const GET_USER_ENDPOINT: &str = "/user/";
const SEARCH_USER_ENDPOINT: &str = "/search_user";
const GET_USERS_ENDPOINT: &str = "/users";
const ADD_FRIEND_ENDPOINT: &str = "/friend";
const CREATE_EXPENSE_ENDPOINT: &str = "/expense";
const CREATE_GROUP_ENDPOINT: &str = "/create_group";
const GET_GROUP_ENDPOINT: &str = "/group/";
const DELETE_GROUP_ENDPOINT: &str = "/group/";
const SEARCH_GROUP_ENDPOINT: &str = "/search_group";
const NEW_GROUP_MEMBER_ENDPOINT: &str = "/new_group_member";

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

    pub async fn delete_user(&self, id: UserId) -> Result<()> {
        let url = self.url.join(GET_USER_ENDPOINT)?.join(&id.to_string())?;
        let resp = self.http.delete(url).send().await?;
        dbg!(&resp);
        match resp.status().is_success() {
            true => Ok(()),
            false => Err(anyhow::anyhow!("Failed to delete user")),
        }
    }

    pub async fn get_user(&self, id: UserId) -> Result<GetUserResponse> {
        let url = self.url.join(GET_USER_ENDPOINT)?.join(&id.to_string())?;
        let resp = self.http.get(url).send().await?;
        dbg!(&resp);
        let resp = resp.json::<GetUserResponse>().await?;
        Ok(resp)
    }

    pub async fn get_users(&self) -> Result<Vec<GetUserResponse>> {
        let url = self.url.join(GET_USERS_ENDPOINT)?;
        let resp = self.http.get(url).send().await?;
        dbg!(&resp);
        let resp = resp.json::<Vec<GetUserResponse>>().await?;
        Ok(resp)
    }

    pub async fn search_users(&self, query: &str) -> Result<Vec<GetUserResponse>> {
        let url = self.url.join(SEARCH_USER_ENDPOINT)?;
        let search_request = SearchUserRequest {
            query: query.to_owned(),
        };
        let resp = self.http.post(url).json(&search_request).send().await?;
        dbg!(&resp);
        let resp = resp.json::<Vec<GetUserResponse>>().await?;
        Ok(resp)
    }

    pub async fn add_friend(&self, user_id: UserId, friend_id: UserId) -> Result<()> {
        let url = self.url.join(ADD_FRIEND_ENDPOINT)?;
        let friend_request = FriendRequest { user_id, friend_id };
        let resp = self.http.post(url).json(&friend_request).send().await?;
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
        amount: u64,
        description: Option<String>,
        group_id: Option<GroupId>,
    ) -> Result<CreateExpenseRequest> {
        let url = self.url.join(CREATE_EXPENSE_ENDPOINT)?;
        let request = CreateExpenseRequest {
            payer,
            amount,
            description,
            participants,
            group_id,
        };

        let resp = self.http.post(url).json(&request).send().await?;
        dbg!(&resp);
        let resp = resp.json::<CreateExpenseRequest>().await?;
        Ok(resp)
    }

    pub async fn create_group(
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

    pub async fn delete_group(&self, id: GroupId) -> Result<()> {
        let url = self.url.join(DELETE_GROUP_ENDPOINT)?.join(&id.to_string())?;
        let resp = self.http.delete(url).send().await?;
        dbg!(&resp);
        match resp.status().is_success() {
            true => Ok(()),
            false => Err(anyhow::anyhow!("Failed to delete group")),
        }
    }

    pub async fn search_group(&self, query: &str) -> Result<Vec<GetGroupResponse>> {
        let url = self.url.join(SEARCH_GROUP_ENDPOINT)?;
        let search_request = SearchGroupRequest {
            query: query.to_owned(),
        };
        let resp = self.http.post(url).json(&search_request).send().await?;
        dbg!(&resp);
        let resp = resp.json::<Vec<GetGroupResponse>>().await?;
        Ok(resp)
    }

    pub async fn get_group(&self, id: GroupId) -> Result<GetGroupResponse>{
        let url = self.url.join(GET_GROUP_ENDPOINT)?.join(&id.to_string())?;
        let resp = self.http.get(url).send().await?;
        dbg!(&resp);
        let resp = resp.json::<GetGroupResponse>().await?;
        Ok(resp)
    }

    pub async fn add_user_to_group(&self, group_id: GroupId, user_id: UserId) -> Result<()> {
        let url = self.url.join(NEW_GROUP_MEMBER_ENDPOINT)?;
        let req = NewGroupMemberRequest {
            group_id: group_id,
            new_member: user_id,
        };
        let resp = self.http.post(url).json(&req).send().await?;
        dbg!(&resp);
        match resp.status().is_success() {
            true => Ok(()),
            false => Err(anyhow::anyhow!("Failed to add user to group")),
        }
    }

    // pub async fn get_group_expenses(&self, group_id: GroupId) -> Result<Vec<Expense>> {
    //     let url = self.url.join("/group/")?.join(&group_id.to_string())?.join("/expenses")?;
    //     let resp = self.http.get(url).send().await?;
    //     dbg!(&resp);
    //     let resp = resp.json::<Vec<Expense>>().await?;
    //     Ok(resp)
    // }

    // pub async fn get_group_balances(&self, group_id: GroupId) -> Result<Vec<Expense>> {
}
