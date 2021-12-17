mod cmd;
mod credentials;
mod provisioning;
use cmd::UpdateType;
use structopt::StructOpt;

#[derive(StructOpt)]
#[structopt(rename_all = "kebab-case")]
enum DCProv {
    List {
        url: String,

        #[structopt(short)]
        filter: Option<String>,

        #[structopt(short)]
        sort: Option<String>,

        #[structopt(short)]
        offset: Option<i64>,

        #[structopt(short)]
        limit: Option<i64>,
    },

    Config {
        #[structopt(subcommand)]
        cmd: ConfigCommand,
    },

    Create {
        #[structopt(subcommand)]
        cmd: CreateCommand,
    },

    Get {
        url: String,
        id: u32,
    },

    Update {
        url: String,
        id: u32,
        #[structopt(subcommand)]
        cmd: UpdateCommand,
    },

    Delete {
        url: String,
        id: u32,
    },
}

#[derive(StructOpt)]
enum ConfigCommand {
    Set { url: String, token: String },
    Get { url: String },
    Delete { url: String },
}

#[derive(StructOpt)]
#[structopt(rename_all = "kebab-case")]
enum CreateCommand {
    FromFile { url: String, path: String },
    Prompt { url: String },
}

#[derive(StructOpt)]
#[structopt(rename_all = "kebab-case")]
enum UpdateCommand {
    QuotaMax { quota_max: i64 },
    UserMax { user_max: i64 },
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
        } => {
            let provider = cmd::init_provisioning(url).await;
            cmd::list_customers(provider, filter, sort, offset, limit).await
        }

        DCProv::Config { cmd } => match cmd {
            ConfigCommand::Set { url, token } => match credentials::set_dracoon_env(&url, &token) {
                true => println!("Stored credentials for {}", url),
                false => println!("Error storing credentials"),
            },
            ConfigCommand::Get { url } => match credentials::get_dracoon_env(&url) {
                Ok(token) => println!("Stored token for {} is {}", url, token),
                Err(e) => println!("An error ocurred: account not found {} ({:?})", url, e),
            },
            ConfigCommand::Delete { url } => match credentials::delete_dracoon_env(&url) {
                true => println!("Deleted credentials for {}", url),
                false => println!("Error deleting credentials: account not found for {}", url),
            },
        },

        DCProv::Create { cmd } => match cmd {
            CreateCommand::FromFile { url, path } => {
                let provider = cmd::init_provisioning(url).await;
                let new_customer = cmd::parse_customer_json_from_file(&path);
                cmd::create_customer(provider, new_customer).await;
            }
            CreateCommand::Prompt { url } => {
                let provider = cmd::init_provisioning(url).await;
                let new_customer = cmd::prompt_new_customer();
                cmd::create_customer(provider, new_customer).await;
            }
        },

        DCProv::Get { url, id } => {
            let provider = cmd::init_provisioning(url).await;
            cmd::get_customer(provider, id).await;
        }

        DCProv::Update { url, id, cmd } => {
            let provider = cmd::init_provisioning(url).await;

            let update_type = match cmd {
                UpdateCommand::CompanyName { company_name } => UpdateType::CompanyName(company_name),
                UpdateCommand::QuotaMax { quota_max } => UpdateType::QuotaMax(quota_max),
                UpdateCommand::UserMax { user_max } => UpdateType::UserMax(user_max),
            };

            cmd::update_customer(provider, id, update_type).await;
        }

        DCProv::Delete { url, id } => {
            let provider = cmd::init_provisioning(url).await;
            cmd::delete_customer(provider, id).await;
        }
    }
}
