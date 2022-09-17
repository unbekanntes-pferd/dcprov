pub(crate) mod cmd;
mod credentials;
mod provisioning;

use std::error::Error;

use cmd::{PrintType, UpdateType, print_version};
use clap::Parser;

use colored::*;



fn parse_key_val<T, U>(s: &str) -> Result<(T, U), Box<dyn Error + Send + Sync + 'static>>
where
    T: std::str::FromStr,
    T::Err: Error + Send + Sync + 'static,
    U: std::str::FromStr,
    U::Err: Error + Send + Sync + 'static,
{
    let pos = s
        .find('=')
        .ok_or_else(|| format!("invalid KEY=value: no `=` found in `{}`", s))?;
    Ok((s[..pos].parse()?, s[pos + 1..].parse()?))
}

#[derive(Parser)]
#[clap(rename_all = "kebab-case", about="DRACOON Provisioning API CLI tool (dcprov)")]
enum DCProv {
    /// List all available customers for specific DRACOON url
    List {
        /// DRACOON url
        url: String,
        #[clap(short, long, help="filter option – see API docs for details")]
        filter: Option<String>,
        #[clap(short, long, help="sort option – see API docs for details")]
        sort: Option<String>,
        #[clap(short, long, help="offset – max. 500 items returned, see API docs for details")]
        offset: Option<i64>,
        #[clap(short, long, help="limit – limits max. returned items, see API docs for details")]
        limit: Option<i64>,
        #[clap(long, help="csv flag – if passed, output will be comma-separated")]
        csv: bool,
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
        id: u32,
        #[clap(long, help="csv flag – if passed, output will be comma-separated")]
        csv: bool,
    },
    
    /// Update a customer by id for specific DRACOON url
    Update {
        /// DRACOON url
        url: String,
        /// Customer id
        id: u32,
        #[clap(subcommand)]
        cmd: UpdateCommand,
    },
    
    /// Delete a customer by id for specific DRACOON url
    Delete {
        /// DRACOON url
        url: String,
        /// Customer id
        id: u32,
    },
    
    /// Get customer attributes for a customer by customer id for specific DRACOON url
    GetAttributes {
        /// DRACOON url
        url: String,
        /// Customer id
        id: u32,
        #[clap(short, long, help="filter option – see API docs for details")]
        filter: Option<String>,
        #[clap(short, long, help="sort option – see API docs for details")]
        sort: Option<String>,
        #[clap(short, long, help="offset – max. 500 items returned, see API docs for details")]
        offset: Option<i64>,
        #[clap(short, long, help="limit – limits max. returned items, see API docs for details")]
        limit: Option<i64>,
        #[clap(long, help="csv flag – if passed, output will be comma-separated")]
        csv: bool,
    },

    /// Set customer attributes for a customer by customer id for specific DRACOON url
    SetAttributes {
        /// DRACOON url
        url: String,
        /// Customer id
        id: u32,
        #[clap(short, parse(try_from_str = parse_key_val), number_of_values = 1)]
        attribs: Vec<(String, String)>,
    },

    /// Get customer users for a customer by customer id for specific DRACOON url
    GetUsers {
        /// DRACOON url
        url: String,
        /// Customer id
        id: u32,
        #[clap(short, long, help="filter option – see API docs for details")]
        filter: Option<String>,
        #[clap(short, long, help="sort option – see API docs for details")]
        sort: Option<String>,
        #[clap(short, long, help="offset – max. 500 items returned, see API docs for details")]
        offset: Option<i64>,
        #[clap(short, long, help="limit – limits max. returned items, see API docs for details")]
        limit: Option<i64>,
        #[clap(long, help="csv flag – if passed, output will be comma-separated")]
        csv: bool,
    },
    
    /// Print version info and logo
    Version {}
}

#[derive(Parser)]
enum ConfigCommand {
    /// Set X-SDS-Service-Token 
    Set { token: String },
    /// Get (output) stored X-SDS-Service-Token 
    Get,
    /// Delete stored X-SDS-Service-Token 
    Delete,
}

#[derive(Parser)]
#[structopt(rename_all = "kebab-case")]
enum CreateCommand {
    /// Create a new customer from JSON file
    FromFile { path: String },
    /// Create a new customer via interactive prompt
    Prompt,
}

#[derive(Parser)]
#[structopt(rename_all = "kebab-case")]
enum UpdateCommand {
    /// Update maximum quota (in bytes!)
    QuotaMax { quota_max: i64 },
    /// Update maximum users 
    UserMax { user_max: i64 },
    /// Update company name
    CompanyName { company_name: String },
}

#[tokio::main]
async fn main() {
    let opt = DCProv::from_args();

    match opt {
        DCProv::List {
            url,
            filter,
            sort,
            offset,
            limit,
            csv,
        } => {
            let provider = cmd::init_provisioning(&url).await;
            let print_type = match csv {
                true => Some(PrintType::Csv),
                false => Some(PrintType::Pretty),
            };
            cmd::list_customers(provider, filter, sort, offset, limit, print_type).await
        }

        DCProv::Config { url, cmd } => match cmd {
            ConfigCommand::Set { token } => match credentials::set_dracoon_env(&url, &token) {
                true => println!("{}{}{}", "Success ".green(), "Credentials saved for ", url),
                false => println!(
                    "{} {}{}",
                    "Error".white().on_red(),
                    "Could not save credentials for ",
                    url
                ),
            },
            ConfigCommand::Get => match credentials::get_dracoon_env(&url) {
                Ok(token) => println!(
                    "{}{}{}{}{}",
                    "Success ".green(),
                    "Credentials for ",
                    url,
                    ": ",
                    token
                ),
                Err(e) => println!(
                    "{} {}{}\n{:?}",
                    "Error".white().on_red(),
                    "Could not get credentials – account not found for ",
                    url,
                    e
                ),
            },
            ConfigCommand::Delete => match credentials::delete_dracoon_env(&url) {
                true => println!(
                    "{}{}{}",
                    "Success ".green(),
                    "Credentials deleted for ",
                    url
                ),
                false => println!(
                    "{} {}{}",
                    "Error".white().on_red(),
                    "Could not delete credentials – account not found for ",
                    url
                ),
            },
        },

        DCProv::Create { url, cmd } => {
            let provider = cmd::init_provisioning(&url).await;
            let new_customer = match cmd {
                CreateCommand::FromFile { path } => cmd::parse_customer_json_from_file(&path),

                CreateCommand::Prompt => cmd::prompt_new_customer(),
            };
            cmd::create_customer(provider, new_customer).await;
        }

        DCProv::Get { url, id, csv } => {
            let provider = cmd::init_provisioning(&url).await;
            let print_type = match csv {
                true => Some(PrintType::Csv),
                false => Some(PrintType::Pretty),
            };
            cmd::get_customer(provider, id, print_type).await;
        }

        DCProv::Update { url, id, cmd } => {
            let provider = cmd::init_provisioning(&url).await;

            let update_type = match cmd {
                UpdateCommand::CompanyName { company_name } => {
                    UpdateType::CompanyName(company_name)
                }
                UpdateCommand::QuotaMax { quota_max } => UpdateType::QuotaMax(quota_max),
                UpdateCommand::UserMax { user_max } => UpdateType::UserMax(user_max),
            };

            cmd::update_customer(provider, id, update_type).await;
        }

        DCProv::Delete { url, id } => {
            let provider = cmd::init_provisioning(&url).await;
            cmd::delete_customer(provider, id).await;
        }
        DCProv::GetAttributes {
            url,
            id,
            filter,
            sort,
            offset,
            limit,
            csv,
        } => {
            let provider = cmd::init_provisioning(&url).await;
            let print_type = match csv {
                true => Some(PrintType::Csv),
                false => Some(PrintType::Pretty),
            };
            cmd::get_customer_attributes(provider, id, filter, sort, offset, limit, print_type)
                .await
        }
        DCProv::SetAttributes { url, id, attribs } => {
            let provider = cmd::init_provisioning(&url).await;
            cmd::update_customer_attributes(provider, id, attribs).await;
        },
        DCProv::GetUsers {url, id, filter,sort , offset, limit, csv} => {
            let provider = cmd::init_provisioning(&url).await;
            let print_type = match csv {
                true => Some(PrintType::Csv),
                false => Some(PrintType::Pretty),
            };
            cmd::get_customer_users(provider, id, filter, sort, offset, limit, print_type).await;

        }
        DCProv::Version { } => print_version()
        
    }
}
