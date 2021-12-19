use chrono::{ DateTime, Utc};
use reqwest::{header, Client, StatusCode, Url};
use serde::{Deserialize, Serialize};
use url::ParseError;

const APP_USER_AGENT: &str = concat!(env!("CARGO_PKG_NAME"), "/", env!("CARGO_PKG_VERSION"));
const X_SDS_SERVICE_TOKEN_HEADER: &str = "X-Sds-Service-Token";
const DRACOON_PROVISIONING_API: &str = "api/v4/provisioning/";
const CUSTOMERS: &str = "customers";
const ATTRIBUTES: &str = "customerAttributes";
const USERS: &str = "users";

const DEFAULT_LIMIT: i32 = 500;
const DEFAULT_OFFSET: i32 = 0;
const DEFAULT_INCLUDE_ATTRIBUTES: bool = false;

async fn check_token_validity(
    token: &str,
    base_url: &str,
) -> Result<bool, DRACOONProvisioningError> {
    let http = Client::builder()
        .user_agent(APP_USER_AGENT)
        .build()
        .unwrap();

    let base_url = Url::parse(base_url).unwrap();
    let api_url = format!(
        "{}{}{}/?limit=1",
        base_url, DRACOON_PROVISIONING_API, CUSTOMERS
    );

    let res = http
        .get(api_url)
        .header(X_SDS_SERVICE_TOKEN_HEADER, token)
        .header(header::CONTENT_TYPE, "application/json")
        .send()
        .await?;
    match res.status() {
        StatusCode::OK => Ok(true),
        _ => Ok(false),
    }
}

pub struct DRACOONProvisioning {
    http: Client,
    base_url: Url,
    x_sds_service_token: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct RangeResponse {
    pub offset: i64,
    pub limit: i64,
    pub total: i64,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct KeyValueEntry {
    pub key: String,
    pub value: String,
}
#[derive(Debug, Serialize, Deserialize)]
pub struct CustomerAttributes {
    items: Vec<KeyValueEntry>,
}

impl CustomerAttributes {
    pub fn new() -> CustomerAttributes {
        let items = Vec::new();
        CustomerAttributes { items }
    }

    pub fn add_attribute(&mut self, key: String, value: String) -> () {
        let attrib = KeyValueEntry { key, value };
        self.items.push(attrib);
    }
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Customer {
    pub id: i64,
    pub company_name: String,
    pub customer_contract_type: String,
    pub quota_max: i64,
    pub quota_used: i64,
    pub user_max: i64,
    pub user_used: i64,
    pub created_at: DateTime<Utc>,
    pub customer_attributes: Option<CustomerAttributes>,
    pub updated_at: Option<DateTime<Utc>>,
    pub last_login_at: Option<DateTime<Utc>>,
    pub trial_days_left: Option<i32>,
    pub is_locked: Option<bool>,
    pub customer_uuid: Option<String>,
}
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GetCustomersResponse {
    pub range: RangeResponse,
    pub items: Vec<Customer>,
}

#[derive(Debug, Deserialize)]
pub struct AttributesResponse {
    pub range: RangeResponse,
    pub items: Vec<KeyValueEntry>,
}

#[derive(Debug, Deserialize)]
#[allow(dead_code)]
pub struct Right {
    id: i64,
    name: String,
    description: String,
}

#[derive(Debug, Deserialize)]
#[allow(dead_code)]
pub struct Role {
    id: i64,
    name: String,
    description: String,
    items: Option<Vec<Right>>,
}

#[derive(Debug, Deserialize)]
#[allow(dead_code)]
pub struct UserRoles {
    items: Vec<Role>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UserItem {
    pub id: i64,
    pub first_name: String,
    pub last_name: String,
    pub user_name: String,
    pub is_locked: bool,
    pub avatar_uuid: String,
    pub created_at: Option<DateTime<Utc>>,
    pub expire_at: Option<DateTime<Utc>>,
    pub last_login_success_at: Option<DateTime<Utc>>,
    pub is_encryption_enabled: Option<bool>,
    pub email: Option<String>,
    pub phone: Option<String>,
    pub home_room_id: Option<i64>,
    pub user_roles: Option<UserRoles>,
}

#[derive(Debug, Deserialize)]
pub struct UserList {
    pub range: RangeResponse,
    pub items: Vec<UserItem>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct NewCustomerResponse {
    pub id: i64,
    pub company_name: String,
    pub customer_contract_type: String,
    pub quota_max: i64,
    pub user_max: i64,
    pub is_locked: Option<bool>,
    pub trial_days: Option<i64>,
    pub created_at: Option<DateTime<Utc>>,
    pub first_admin_user: FirstAdminUser,
    pub customer_attributes: Option<CustomerAttributes>,
    pub provider_customer_id: Option<String>,
    pub webhooks_max: Option<i64>,
    pub customer_uuid: String,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct FirstAdminUser {
    pub first_name: String,
    pub last_name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub user_name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub auth_data: Option<UserAuthData>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub receiver_language: Option<String>,
    pub notify_user: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub email: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub phone: Option<String>,
}

impl FirstAdminUser {
    pub fn new_local(
        first_name: String,
        last_name: String,
        user_name: Option<String>,
        email: String,
        receiver_language: Option<String>,
    ) -> FirstAdminUser {
        let auth_data = UserAuthData {
            method: "basic".to_string(),
            login: None,
            password: None,
            must_change_password: Some(true),
            ad_config_id: None,
            oid_config_id: None,
        };

        FirstAdminUser {
            first_name,
            last_name,
            user_name,
            auth_data: Some(auth_data),
            receiver_language,
            notify_user: None,
            email: Some(email),
            phone: None,
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UserAuthData {
    pub method: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub login: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub password: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub must_change_password: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub ad_config_id: Option<i64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub oid_config_id: Option<i64>,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct NewCustomerRequest {
    pub customer_contract_type: String,
    pub quota_max: i64,
    pub user_max: i64,
    pub first_admin_user: FirstAdminUser,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub company_name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub trial_days: Option<i64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub is_locked: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub customer_attributes: Option<CustomerAttributes>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub provider_customer_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub webhooks_max: Option<i64>,
}

impl NewCustomerRequest {
    pub fn new(
        customer_contract_type: String,
        quota_max: i64,
        user_max: i64,
        first_admin_user: FirstAdminUser,
        company_name: Option<String>,
        trial_days: Option<i64>,
        is_locked: Option<bool>,
        customer_attributes: Option<CustomerAttributes>,
        provider_customer_id: Option<String>,
        webhooks_max: Option<i64>,
    ) -> NewCustomerRequest {
        NewCustomerRequest {
            customer_contract_type,
            quota_max,
            user_max,
            first_admin_user,
            company_name,
            trial_days,
            is_locked,
            customer_attributes,
            provider_customer_id,
            webhooks_max,
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UpdateCustomerResponse {
    pub id: i64,
    pub company_name: String,
    pub customer_contract_type: String,
    pub quota_max: i64,
    pub user_max: i64,
    pub customer_uuid: String,
    pub is_locked: Option<bool>,
    pub trial_days: Option<i64>,
    pub created_at: Option<DateTime<Utc>>,
    pub updated_at: Option<DateTime<Utc>>,
    pub customer_attributes: Option<CustomerAttributes>,
    pub provider_customer_id: Option<String>,
    pub webhooks_max: Option<i64>,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UpdateCustomerRequest {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub company_name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub customer_contract_type: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub quota_max: Option<i64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub user_max: Option<i64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub is_locked: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub provider_customer_id: Option<i64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub webhooks_max: Option<i64>,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DRACOONErrorResponse {
    pub code: i64,
    pub message: String,
    pub debug_info: Option<String>,
    error_code: Option<i64>,
}

#[derive(Debug)]
#[allow(dead_code)]
pub enum DRACOONProvisioningError {
    RequestFailed(reqwest::Error),
    InvalidUrl(ParseError),
    Unauthorized(Option<DRACOONErrorResponse>),
    BadRequest(DRACOONErrorResponse),
    Forbidden(DRACOONErrorResponse),
    NotFound(DRACOONErrorResponse),
    PaymentRequired(DRACOONErrorResponse),
    NotAcceptable(DRACOONErrorResponse),
    Conflict(DRACOONErrorResponse),
    Undocumented(DRACOONErrorResponse),
    InvalidAccount,
}

impl From<reqwest::Error> for DRACOONProvisioningError {
    fn from(error: reqwest::Error) -> Self {
        DRACOONProvisioningError::RequestFailed(error)
    }
}

impl From<ParseError> for DRACOONProvisioningError {
    fn from(error: ParseError) -> Self {
        DRACOONProvisioningError::InvalidUrl(error)
    }
}

impl DRACOONProvisioning {
    pub async fn new(
        base_url: String,
        service_token: String,
    ) -> Result<DRACOONProvisioning, DRACOONProvisioningError> {
        let http = Client::builder().user_agent(APP_USER_AGENT).build()?;

        let base_url = Url::parse(&base_url)?;

        match check_token_validity(&service_token, base_url.as_str()).await {
            Ok(valid) => match valid {
                true => Ok(DRACOONProvisioning {
                    x_sds_service_token: service_token,
                    http: http,
                    base_url: base_url,
                }),
                false => Err(DRACOONProvisioningError::Unauthorized(None)),
            },
            Err(e) => return Err(e),
        }
    }

    pub async fn get_customers(
        &self,
        filter: Option<String>,
        sort: Option<String>,
        limit: Option<i64>,
        offset: Option<i64>,
        include_attributes: Option<bool>,
    ) -> Result<GetCustomersResponse, DRACOONProvisioningError> {
        let mut api_url = format!("{}{}{}", self.base_url, DRACOON_PROVISIONING_API, CUSTOMERS);

        match limit {
            Some(limit) => api_url += format!("/?limit={}", limit).as_str(),
            None => api_url += format!("/?limit={}", DEFAULT_LIMIT).as_str(),
        }

        match offset {
            Some(offset) => api_url += format!("&offset={}", offset).as_str(),
            None => api_url += format!("&offset={}", DEFAULT_OFFSET).as_str(),
        }

        match filter {
            Some(filter) => api_url += format!("&filter={}", filter).as_str(),
            None => (),
        }

        match sort {
            Some(sort) => api_url += format!("&sort={}", sort).as_str(),
            None => (),
        }

        match include_attributes {
            Some(include_attributes) => {
                api_url += format!("&include_attributes={}", include_attributes).as_str()
            }
            None => {
                api_url += format!("&include_attributes={}", DEFAULT_INCLUDE_ATTRIBUTES).as_str()
            }
        }

        let api_url = Url::parse(&api_url)?;

        let response = self
            .http
            .get(api_url)
            .header(X_SDS_SERVICE_TOKEN_HEADER, &self.x_sds_service_token)
            .header(header::CONTENT_TYPE, "application/json")
            .send()
            .await?;

        match response.status() {
            StatusCode::OK => return Ok(response.json::<GetCustomersResponse>().await?),
            StatusCode::NOT_FOUND => {
                return Err(DRACOONProvisioningError::NotFound(
                    response.json::<DRACOONErrorResponse>().await?,
                ))
            }
            StatusCode::BAD_REQUEST => {
                return Err(DRACOONProvisioningError::BadRequest(
                    response.json::<DRACOONErrorResponse>().await?,
                ))
            }
            StatusCode::UNAUTHORIZED => {
                return Err(DRACOONProvisioningError::Unauthorized(Some(
                    response.json::<DRACOONErrorResponse>().await?,
                )))
            }
            StatusCode::NOT_ACCEPTABLE => {
                return Err(DRACOONProvisioningError::NotAcceptable(
                    response.json::<DRACOONErrorResponse>().await?,
                ))
            }
            _ => {
                return Err(DRACOONProvisioningError::Undocumented(
                    response.json::<DRACOONErrorResponse>().await?,
                ))
            }
        };
    }

    pub async fn create_customer(
        &self,
        customer: NewCustomerRequest,
    ) -> Result<NewCustomerResponse, DRACOONProvisioningError> {
        let api_url = format!("{}{}{}", self.base_url, DRACOON_PROVISIONING_API, CUSTOMERS);
        let api_url = Url::parse(&api_url)?;

        let response = self
            .http
            .post(api_url)
            .header(X_SDS_SERVICE_TOKEN_HEADER, &self.x_sds_service_token)
            .header(header::CONTENT_TYPE, "application/json")
            .json(&customer)
            .send()
            .await?;

        match response.status() {
            StatusCode::CREATED => return Ok(response.json::<NewCustomerResponse>().await?),
            StatusCode::PAYMENT_REQUIRED => {
                return Err(DRACOONProvisioningError::PaymentRequired(
                    response.json::<DRACOONErrorResponse>().await?,
                ))
            }
            StatusCode::NOT_FOUND => {
                return Err(DRACOONProvisioningError::NotFound(
                    response.json::<DRACOONErrorResponse>().await?,
                ))
            }
            StatusCode::BAD_REQUEST => {
                return Err(DRACOONProvisioningError::BadRequest(
                    response.json::<DRACOONErrorResponse>().await?,
                ))
            }
            StatusCode::UNAUTHORIZED => {
                return Err(DRACOONProvisioningError::Unauthorized(Some(
                    response.json::<DRACOONErrorResponse>().await?,
                )))
            }
            StatusCode::NOT_ACCEPTABLE => {
                return Err(DRACOONProvisioningError::NotAcceptable(
                    response.json::<DRACOONErrorResponse>().await?,
                ))
            }
            StatusCode::CONFLICT => {
                return Err(DRACOONProvisioningError::Conflict(
                    response.json::<DRACOONErrorResponse>().await?,
                ))
            }
            _ => {
                return Err(DRACOONProvisioningError::Undocumented(
                    response.json::<DRACOONErrorResponse>().await?,
                ))
            }
        };
    }

    pub async fn get_customer(
        &self,
        id: i64,
        include_attributes: Option<bool>,
    ) -> Result<Customer, DRACOONProvisioningError> {
        let attrib = match include_attributes {
            Some(include_attributes) => include_attributes,
            None => false,
        };
        let api_url = format!(
            "{}{}{}/{}/?include_attributes={}",
            self.base_url,
            DRACOON_PROVISIONING_API,
            CUSTOMERS,
            id,
            attrib.to_string()
        );

        let api_url = Url::parse(&api_url)?;

        let response = self
            .http
            .get(api_url)
            .header(X_SDS_SERVICE_TOKEN_HEADER, &self.x_sds_service_token)
            .header(header::CONTENT_TYPE, "application/json")
            .send()
            .await?;

        match response.status() {
            StatusCode::OK => return Ok(response.json::<Customer>().await?),
            StatusCode::NOT_FOUND => {
                return Err(DRACOONProvisioningError::NotFound(
                    response.json::<DRACOONErrorResponse>().await?,
                ))
            }
            StatusCode::BAD_REQUEST => {
                return Err(DRACOONProvisioningError::BadRequest(
                    response.json::<DRACOONErrorResponse>().await?,
                ))
            }
            StatusCode::UNAUTHORIZED => {
                return Err(DRACOONProvisioningError::Unauthorized(Some(
                    response.json::<DRACOONErrorResponse>().await?,
                )))
            }
            StatusCode::NOT_ACCEPTABLE => {
                return Err(DRACOONProvisioningError::NotAcceptable(
                    response.json::<DRACOONErrorResponse>().await?,
                ))
            }
            _ => {
                return Err(DRACOONProvisioningError::Undocumented(
                    response.json::<DRACOONErrorResponse>().await?,
                ))
            }
        }
    }

    pub async fn update_customer(
        &self,
        id: i64,
        update: UpdateCustomerRequest,
    ) -> Result<UpdateCustomerResponse, DRACOONProvisioningError> {
        let api_url = format!(
            "{}{}{}/{}",
            self.base_url, DRACOON_PROVISIONING_API, CUSTOMERS, id
        );
        let api_url = Url::parse(&api_url)?;

        let response = self
            .http
            .put(api_url)
            .header(X_SDS_SERVICE_TOKEN_HEADER, &self.x_sds_service_token)
            .header(header::CONTENT_TYPE, "application/json")
            .json(&update)
            .send()
            .await?;

        match response.status() {
            StatusCode::OK => return Ok(response.json::<UpdateCustomerResponse>().await?),
            StatusCode::PAYMENT_REQUIRED => {
                return Err(DRACOONProvisioningError::PaymentRequired(
                    response.json::<DRACOONErrorResponse>().await?,
                ))
            }
            StatusCode::NOT_FOUND => {
                return Err(DRACOONProvisioningError::NotFound(
                    response.json::<DRACOONErrorResponse>().await?,
                ))
            }
            StatusCode::BAD_REQUEST => {
                return Err(DRACOONProvisioningError::BadRequest(
                    response.json::<DRACOONErrorResponse>().await?,
                ))
            }
            StatusCode::UNAUTHORIZED => {
                return Err(DRACOONProvisioningError::Unauthorized(Some(
                    response.json::<DRACOONErrorResponse>().await?,
                )))
            }
            StatusCode::NOT_ACCEPTABLE => {
                return Err(DRACOONProvisioningError::NotAcceptable(
                    response.json::<DRACOONErrorResponse>().await?,
                ))
            }
            _ => {
                return Err(DRACOONProvisioningError::Undocumented(
                    response.json::<DRACOONErrorResponse>().await?,
                ))
            }
        };
    }

    pub async fn delete_customer(&self, id: i64) -> Result<(), DRACOONProvisioningError> {
        let api_url = format!(
            "{}{}{}/{}",
            self.base_url, DRACOON_PROVISIONING_API, CUSTOMERS, id
        );

        let api_url = Url::parse(&api_url)?;

        let response = self
            .http
            .delete(api_url)
            .header(X_SDS_SERVICE_TOKEN_HEADER, &self.x_sds_service_token)
            .header(header::CONTENT_TYPE, "application/json")
            .send()
            .await?;

        match response.status() {
            StatusCode::NO_CONTENT => return Ok(()),
            StatusCode::NOT_FOUND => {
                return Err(DRACOONProvisioningError::NotFound(
                    response.json::<DRACOONErrorResponse>().await?,
                ))
            }
            StatusCode::BAD_REQUEST => {
                return Err(DRACOONProvisioningError::BadRequest(
                    response.json::<DRACOONErrorResponse>().await?,
                ))
            }
            StatusCode::UNAUTHORIZED => {
                return Err(DRACOONProvisioningError::Unauthorized(Some(
                    response.json::<DRACOONErrorResponse>().await?,
                )))
            }
            StatusCode::NOT_ACCEPTABLE => {
                return Err(DRACOONProvisioningError::NotAcceptable(
                    response.json::<DRACOONErrorResponse>().await?,
                ))
            }
            _ => {
                return Err(DRACOONProvisioningError::Undocumented(
                    response.json::<DRACOONErrorResponse>().await?,
                ))
            }
        }
    }

    pub async fn get_customer_attributes(
        &self,
        id: i64,
        filter: Option<String>,
        sort: Option<String>,
        offset: Option<i64>,
        limit: Option<i64>,
    ) -> Result<AttributesResponse, DRACOONProvisioningError> {
        let mut api_url = format!(
            "{}{}{}/{}/{}",
            self.base_url, DRACOON_PROVISIONING_API, CUSTOMERS, id, ATTRIBUTES
        );

        match limit {
            Some(limit) => api_url += format!("/?limit={}", limit).as_str(),
            None => api_url += format!("/?limit={}", DEFAULT_LIMIT).as_str(),
        }

        match offset {
            Some(offset) => api_url += format!("&offset={}", offset).as_str(),
            None => api_url += format!("&offset={}", DEFAULT_OFFSET).as_str(),
        }

        match filter {
            Some(filter) => api_url += format!("&filter={}", filter).as_str(),
            None => (),
        }

        match sort {
            Some(sort) => api_url += format!("&sort={}", sort).as_str(),
            None => (),
        }

        let api_url = Url::parse(&api_url)?;
        let response = self
            .http
            .get(api_url)
            .header(X_SDS_SERVICE_TOKEN_HEADER, &self.x_sds_service_token)
            .header(header::CONTENT_TYPE, "application/json")
            .send()
            .await?;

        match response.status() {
            StatusCode::OK => return Ok(response.json::<AttributesResponse>().await?),
            StatusCode::NOT_FOUND => {
                return Err(DRACOONProvisioningError::NotFound(
                    response.json::<DRACOONErrorResponse>().await?,
                ))
            }
            StatusCode::BAD_REQUEST => {
                return Err(DRACOONProvisioningError::BadRequest(
                    response.json::<DRACOONErrorResponse>().await?,
                ))
            }
            StatusCode::UNAUTHORIZED => {
                return Err(DRACOONProvisioningError::Unauthorized(Some(
                    response.json::<DRACOONErrorResponse>().await?,
                )))
            }
            StatusCode::NOT_ACCEPTABLE => {
                return Err(DRACOONProvisioningError::NotAcceptable(
                    response.json::<DRACOONErrorResponse>().await?,
                ))
            }
            _ => {
                return Err(DRACOONProvisioningError::Undocumented(
                    response.json::<DRACOONErrorResponse>().await?,
                ))
            }
        };
    }

    pub async fn update_customer_attributes(
        &self,
        id: i64,
        attribs: CustomerAttributes,
    ) -> Result<Customer, DRACOONProvisioningError> {
        let api_url = format!(
            "{}{}{}/{}/{}",
            self.base_url, DRACOON_PROVISIONING_API, CUSTOMERS, id, ATTRIBUTES
        );

        let api_url = Url::parse(&api_url)?;

        let response = self
            .http
            .put(api_url)
            .header(X_SDS_SERVICE_TOKEN_HEADER, &self.x_sds_service_token)
            .header(header::CONTENT_TYPE, "application/json")
            .json(&attribs)
            .send()
            .await?;

        match response.status() {
            StatusCode::OK => return Ok(response.json::<Customer>().await?),
            StatusCode::NOT_FOUND => {
                return Err(DRACOONProvisioningError::NotFound(
                    response.json::<DRACOONErrorResponse>().await?,
                ))
            }
            StatusCode::BAD_REQUEST => {
                return Err(DRACOONProvisioningError::BadRequest(
                    response.json::<DRACOONErrorResponse>().await?,
                ))
            }
            StatusCode::UNAUTHORIZED => {
                return Err(DRACOONProvisioningError::Unauthorized(Some(
                    response.json::<DRACOONErrorResponse>().await?,
                )))
            }
            StatusCode::NOT_ACCEPTABLE => {
                return Err(DRACOONProvisioningError::NotAcceptable(
                    response.json::<DRACOONErrorResponse>().await?,
                ))
            }
            _ => {
                return Err(DRACOONProvisioningError::Undocumented(
                    response.json::<DRACOONErrorResponse>().await?,
                ))
            }
        };
    }

    pub async fn get_customer_users(
        &self,
        id: i64,
        filter: Option<String>,
        sort: Option<String>,
        offset: Option<i64>,
        limit: Option<i64>,
    ) -> Result<UserList, DRACOONProvisioningError> {
        let mut api_url = format!(
            "{}{}{}/{}/{}",
            self.base_url, DRACOON_PROVISIONING_API, CUSTOMERS, id, USERS
        );

        match limit {
            Some(limit) => api_url += format!("/?limit={}", limit).as_str(),
            None => api_url += format!("/?limit={}", DEFAULT_LIMIT).as_str(),
        }

        match offset {
            Some(offset) => api_url += format!("&offset={}", offset).as_str(),
            None => api_url += format!("&offset={}", DEFAULT_OFFSET).as_str(),
        }

        match filter {
            Some(filter) => api_url += format!("&filter={}", filter).as_str(),
            None => (),
        }

        match sort {
            Some(sort) => api_url += format!("&sort={}", sort).as_str(),
            None => (),
        }

        let api_url = Url::parse(&api_url)?;
        let response = self
            .http
            .get(api_url)
            .header(X_SDS_SERVICE_TOKEN_HEADER, &self.x_sds_service_token)
            .header(header::CONTENT_TYPE, "application/json")
            .send()
            .await?;

        match response.status() {
            StatusCode::OK => return Ok(response.json::<UserList>().await?),
            StatusCode::NOT_FOUND => {
                return Err(DRACOONProvisioningError::NotFound(
                    response.json::<DRACOONErrorResponse>().await?,
                ))
            }
            StatusCode::BAD_REQUEST => {
                return Err(DRACOONProvisioningError::BadRequest(
                    response.json::<DRACOONErrorResponse>().await?,
                ))
            }
            StatusCode::UNAUTHORIZED => {
                return Err(DRACOONProvisioningError::Unauthorized(Some(
                    response.json::<DRACOONErrorResponse>().await?,
                )))
            }
            StatusCode::NOT_ACCEPTABLE => {
                return Err(DRACOONProvisioningError::NotAcceptable(
                    response.json::<DRACOONErrorResponse>().await?,
                ))
            }
            _ => {
                return Err(DRACOONProvisioningError::Undocumented(
                    response.json::<DRACOONErrorResponse>().await?,
                ))
            }
        };
    }
}
