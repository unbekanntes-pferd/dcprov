mod cmd;
mod credentials;
mod provisioning;
use cmd::{UpdateType, PrintType};
use structopt::StructOpt;

#[derive(StructOpt)]
#[structopt(rename_all = "kebab-case")]
enum DCProv {
    List {
        url: String,
        #[structopt(short, long)]
        filter: Option<String>,
        #[structopt(short, long)]
        sort: Option<String>,
        #[structopt(short, long)]
        offset: Option<i64>,
        #[structopt(short, long)]
        limit: Option<i64>,
        #[structopt(long)]
        csv: bool,
    },

    Config {
        url: String,
        #[structopt(subcommand)]
        cmd: ConfigCommand,
    },

    Create {
        url: String,
        #[structopt(subcommand)]
        cmd: CreateCommand,
    },

    Get {
        url: String,
        id: u32,
        #[structopt(long)]
        csv: bool,
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
    Set { token: String },
    Get,
    Delete,
}

#[derive(StructOpt)]
#[structopt(rename_all = "kebab-case")]
enum CreateCommand {
    FromFile { path: String },
    Prompt,
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
            csv
        } => {
            let provider = cmd::init_provisioning(url).await;
            let print_type = match csv {
                true => PrintType::Csv,
                false => PrintType::Pretty
            };
            cmd::list_customers(provider, filter, sort, offset, limit, print_type).await
        }

        DCProv::Config { url, cmd } => match cmd {
            ConfigCommand::Set { token } => match credentials::set_dracoon_env(&url, &token) {
                true => println!("Stored credentials for {}", url),
                false => println!("Error storing credentials"),
            },
            ConfigCommand::Get => match credentials::get_dracoon_env(&url) {
                Ok(token) => println!("Stored token for {} is {}", url, token),
                Err(e) => println!("An error ocurred: account not found {} ({:?})", url, e),
            },
            ConfigCommand::Delete => match credentials::delete_dracoon_env(&url) {
                true => println!("Deleted credentials for {}", url),
                false => println!("Error deleting credentials: account not found for {}", url),
            },
        },

        DCProv::Create { url, cmd } => {
            let provider = cmd::init_provisioning(url).await;
            let new_customer = match cmd {
                CreateCommand::FromFile { path } => cmd::parse_customer_json_from_file(&path),

                CreateCommand::Prompt => cmd::prompt_new_customer(),
            };
            cmd::create_customer(provider, new_customer).await;
        }

        DCProv::Get { url, id, csv} => {
            let provider = cmd::init_provisioning(url).await;
            let print_type = match csv {
                true => PrintType::Csv,
                false => PrintType::Pretty
            };
            cmd::get_customer(provider, id, print_type).await;
        }

        DCProv::Update { url, id, cmd } => {
            let provider = cmd::init_provisioning(url).await;

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
            let provider = cmd::init_provisioning(url).await;
            cmd::delete_customer(provider, id).await;
        }
    }
}
