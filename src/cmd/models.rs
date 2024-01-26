use super::utils::parse_key_val;
use clap::Parser;
use dco3::provisioning::NewCustomerRequest as NewCustomerRequestDco3;
use dco3::{
    auth::DracoonErrorResponse,
    provisioning::{CustomerAttributes, FirstAdminUser},
};
use serde::{Deserialize, Serialize};

#[derive(thiserror::Error, Debug)]
#[allow(dead_code)]
pub enum DcProvError {
    #[error("Invalid account")]
    InvalidAccount,
    #[error("Credential storage failed")]
    CredentialStorageFailed,
    #[error("Credential deletion failed")]
    CredentialDeletionFailed,
    #[error("Bad request")]
    BadRequest(DracoonErrorResponse),
    #[error("Unauthorized")]
    Unauthorized(DracoonErrorResponse),
    #[error("Forbidden")]
    Forbidden(DracoonErrorResponse),
    #[error("Not found")]
    NotFound(DracoonErrorResponse),
    #[error("Payment required")]
    PaymentRequired(DracoonErrorResponse),
    #[error("Conflict")]
    Conflict(DracoonErrorResponse),
    #[error("Internal server error")]
    Unknown(DracoonErrorResponse),
    #[error("IO error")]
    Io,
    #[error("Other error")]
    Other,
}

#[derive(Parser)]
#[clap(
    rename_all = "kebab-case",
    about = "DRACOON Provisioning API CLI tool (dcprov)"
)]
pub struct DcProv {
    /// optional X-SDS-Service-Token
    #[clap(short, long, help = "Optional X-SDS-Service-Token")]
    pub token: Option<String>,

    /// command
    #[clap(subcommand)]
    pub cmd: DCProvCommand,
}

#[derive(Parser)]
pub enum DCProvCommand {
    /// List all available customers for specific DRACOON url
    List {
        /// DRACOON url
        url: String,
        #[clap(short, long, help = "filter option – see API docs for details")]
        filter: Option<String>,
        #[clap(short, long, help = "sort option – see API docs for details")]
        sort: Option<String>,
        #[clap(
            short,
            long,
            help = "offset – max. 500 items returned, see API docs for details"
        )]
        offset: Option<u64>,
        #[clap(
            short,
            long,
            help = "limit – limits max. returned items, see API docs for details"
        )]
        limit: Option<u64>,
        #[clap(long, help = "csv flag – if passed, output will be comma-separated")]
        csv: bool,

        #[clap(long, help = "will fetch all items (default: paginated, 500 results)")]
        all: bool


    },

    /// Configure X-SDS-Service-Token for specific DRACOON url
    Config {
        /// DRACOON url
        url: String,
        #[clap(subcommand)]
        cmd: ConfigCommand,
    },

    /// Create a new customer for specific DRACOON url
    Create {
        /// DRACOON url
        url: String,
        #[clap(subcommand)]
        cmd: CreateCommand,
    },

    /// Get a customer by id for specific DRACOON url
    Get {
        /// DRACOON url
        url: String,
        /// Customer id
        id: u64,
        #[clap(long, help = "csv flag – if passed, output will be comma-separated")]
        csv: bool,
    },

    /// Update a customer by id for specific DRACOON url
    Update {
        /// DRACOON url
        url: String,
        /// Customer id
        id: u64,
        #[clap(subcommand)]
        cmd: UpdateCommand,
    },

    /// Delete a customer by id for specific DRACOON url
    Delete {
        /// DRACOON url
        url: String,
        /// Customer id
        id: u64,
    },

    /// Get customer attributes for a customer by customer id for specific DRACOON url
    GetAttributes {
        /// DRACOON url
        url: String,
        /// Customer id
        id: u64,
        #[clap(short, long, help = "filter option – see API docs for details")]
        filter: Option<String>,
        #[clap(short, long, help = "sort option – see API docs for details")]
        sort: Option<String>,
        #[clap(
            short,
            long,
            help = "offset – max. 500 items returned, see API docs for details"
        )]
        offset: Option<u64>,
        #[clap(
            short,
            long,
            help = "limit – limits max. returned items, see API docs for details"
        )]
        limit: Option<u64>,
        #[clap(long, help = "csv flag – if passed, output will be comma-separated")]
        csv: bool,
    },

    /// Set customer attributes for a customer by customer id for specific DRACOON url
    SetAttributes {
        /// DRACOON url
        url: String,
        /// Customer id
        id: u64,
        #[clap(short, value_parser = parse_key_val::<String, String>, number_of_values = 1)]
        attribs: Vec<(String, String)>,
    },

    /// Get customer users for a customer by customer id for specific DRACOON url
    GetUsers {
        /// DRACOON url
        url: String,
        /// Customer id
        id: u64,
        #[clap(short, long, help = "filter option – see API docs for details")]
        filter: Option<String>,
        #[clap(short, long, help = "sort option – see API docs for details")]
        sort: Option<String>,
        #[clap(
            short,
            long,
            help = "offset – max. 500 items returned, see API docs for details"
        )]
        offset: Option<u64>,
        #[clap(
            short,
            long,
            help = "limit – limits max. returned items, see API docs for details"
        )]
        limit: Option<u64>,
        #[clap(long, help = "csv flag – if passed, output will be comma-separated")]
        csv: bool,
    },

    /// Print version info and logo
    Version,
}

#[derive(Parser)]
pub enum ConfigCommand {
    /// Set X-SDS-Service-Token
    Set { token: String },
    /// Get (output) stored X-SDS-Service-Token
    Get,
    /// Delete stored X-SDS-Service-Token
    Delete,
}

#[derive(Parser)]
#[structopt(rename_all = "kebab-case")]
pub enum CreateCommand {
    /// Create a new customer from JSON file
    FromFile { path: String },
    /// Create a new customer via interactive prompt
    Prompt,
}

#[derive(Parser)]
#[structopt(rename_all = "kebab-case")]
pub enum UpdateCommand {
    /// Update maximum quota (in bytes!)
    QuotaMax { quota_max: u64 },
    /// Update maximum users
    UserMax { user_max: u64 },
    /// Update company name
    CompanyName { company_name: String },
}

// TODO: remove this when dco3 adds Deserialize for NewCustomerRequest
#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct NewCustomerRequest {
    pub customer_contract_type: String,
    pub quota_max: u64,
    pub user_max: u64,
    pub first_admin_user: FirstAdminUser,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub company_name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub trial_days: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub is_locked: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub customer_attributes: Option<CustomerAttributes>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub provider_customer_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub webhooks_max: Option<u64>,
}

impl From<NewCustomerRequest> for NewCustomerRequestDco3 {
    fn from(req: NewCustomerRequest) -> Self {
        Self {
            customer_contract_type: req.customer_contract_type,
            quota_max: req.quota_max,
            user_max: req.user_max,
            first_admin_user: req.first_admin_user,
            company_name: req.company_name,
            trial_days: req.trial_days,
            is_locked: req.is_locked,
            customer_attributes: req.customer_attributes,
            provider_customer_id: req.provider_customer_id,
            webhooks_max: req.webhooks_max,
        }
    }
}
