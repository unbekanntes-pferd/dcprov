pub(crate) mod cmd;
mod credentials;
use cmd::{
    handle_errors, print_version, ConfigCommand, CreateCommand, DCProvCommand, DcProv, PrintType,
    UpdateCommand, UpdateType, DcProvError,
};

use clap::Parser;
use colored::*;
use credentials::SERVICE_NAME;
use keyring::Entry;

#[tokio::main]
async fn main() {
    let opt = DcProv::parse();

    match opt.cmd {
        DCProvCommand::List {
            url,
            filter,
            sort,
            offset,
            limit,
            csv,
        } => {
            let provider = cmd::init_provisioning(&url, opt.token).await;
            let print_type = match csv {
                true => Some(PrintType::Csv),
                false => Some(PrintType::Pretty),
            };
            if let Err(ref e) = provider {
                handle_errors(e)
            }
            let provider = provider.unwrap();
            cmd::list_customers(provider, filter, sort, offset, limit, print_type).await
        }

        DCProvCommand::Config { url, cmd } => {
            let entry = Entry::new(SERVICE_NAME, &url).map_err(|_| DcProvError::CredentialStorageFailed);
            if let Err(ref e) = entry {
                handle_errors(e)
            }

            let entry = entry.unwrap();
            match cmd {
            ConfigCommand::Set { token } => match credentials::set_dracoon_env(&entry, &token) {
                Ok(_) => println!("{}{}{}", "Success ".green(), "Credentials saved for ", url),
                Err(ref e) => handle_errors(e),
            },
            ConfigCommand::Get => match credentials::get_dracoon_env(&entry) {
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
            ConfigCommand::Delete => match credentials::delete_dracoon_env(&entry) {
                Ok(_) => println!(
                    "{}{}{}",
                    "Success ".green(),
                    "Credentials deleted for ",
                    url
                ),
                Err(ref e) => handle_errors(e),
            },
        }},

        DCProvCommand::Create { url, cmd } => {
            let provider = cmd::init_provisioning(&url, opt.token).await;
            let new_customer = match cmd {
                CreateCommand::FromFile { path } => cmd::parse_customer_json_from_file(&path),

                CreateCommand::Prompt => cmd::prompt_new_customer(),
            };
            if let Err(ref e) = provider {
                handle_errors(e)
            }
            if let Err(ref e) = new_customer {
                handle_errors(e)
            }
            let provider = provider.unwrap();
            let new_customer = new_customer.unwrap();
            cmd::create_customer(provider, new_customer).await;
        }

        DCProvCommand::Get { url, id, csv } => {
            let provider = cmd::init_provisioning(&url, opt.token).await;
            let print_type = match csv {
                true => Some(PrintType::Csv),
                false => Some(PrintType::Pretty),
            };
            if let Err(ref e) = provider {
                handle_errors(e)
            }
            let provider = provider.unwrap();
            cmd::get_customer(provider, id, print_type).await;
        }

        DCProvCommand::Update { url, id, cmd } => {
            let provider = cmd::init_provisioning(&url, opt.token).await;

            let update_type = match cmd {
                UpdateCommand::CompanyName { company_name } => {
                    UpdateType::CompanyName(company_name)
                }
                UpdateCommand::QuotaMax { quota_max } => UpdateType::QuotaMax(quota_max),
                UpdateCommand::UserMax { user_max } => UpdateType::UserMax(user_max),
            };

            if let Err(ref e) = provider {
                handle_errors(e)
            }
            let provider = provider.unwrap();

            cmd::update_customer(provider, id, update_type).await;
        }

        DCProvCommand::Delete { url, id } => {
            let provider = cmd::init_provisioning(&url, opt.token).await;
            if let Err(ref e) = provider {
                handle_errors(e)
            }
            let provider = provider.unwrap();
            cmd::delete_customer(provider, id).await;
        }
        DCProvCommand::GetAttributes {
            url,
            id,
            filter,
            sort,
            offset,
            limit,
            csv,
        } => {
            let provider = cmd::init_provisioning(&url, opt.token).await;
            if let Err(ref e) = provider {
                handle_errors(e)
            }
            let provider = provider.unwrap();
            let print_type = match csv {
                true => Some(PrintType::Csv),
                false => Some(PrintType::Pretty),
            };
            cmd::get_customer_attributes(provider, id, filter, sort, offset, limit, print_type)
                .await
        }
        DCProvCommand::SetAttributes { url, id, attribs } => {
            let provider = cmd::init_provisioning(&url, opt.token).await;
            if let Err(ref e) = provider {
                handle_errors(e)
            }
            let provider = provider.unwrap();
            cmd::update_customer_attributes(provider, id, attribs).await;
        }
        DCProvCommand::GetUsers {
            url,
            id,
            filter,
            sort,
            offset,
            limit,
            csv,
        } => {
            let provider = cmd::init_provisioning(&url, opt.token).await;
            let print_type = match csv {
                true => Some(PrintType::Csv),
                false => Some(PrintType::Pretty),
            };
            if let Err(ref e) = provider {
                handle_errors(e)
            }
            let provider = provider.unwrap();
            cmd::get_customer_users(provider, id, filter, sort, offset, limit, print_type).await;
        }
        DCProvCommand::Version {} => print_version(),
    }
}
