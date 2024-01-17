use crate::credentials::{get_dracoon_env, set_dracoon_env, SERVICE_NAME};
use colored::*;
use dco3::{
    auth::{DracoonErrorResponse, Provisioning},
    provisioning::{
        Customer, CustomerAttributes, FirstAdminUser, NewCustomerRequest as NewCustomerRequestDco3,
        UpdateCustomerRequest,
    },
    users::{AuthMethod, UserAuthData, UserItem},
    CustomerProvisioning, Dracoon, DracoonClientError, KeyValueEntry, ListAllParams,
};
use keyring::Entry;
use std::fs;

mod models;
mod utils;
pub use {models::*, utils::parse_key_val};

// header for CSV output (list customers)
const CUSTOMER_CSV_HEADER: &str =
    "companyName,contractType,userUsed,userMax,quotaUsed,quotaMax,id,createdAt";

// header for CSV output (list customer users)
const CUSTOMER_USERS_CSV_HEADER: &str = "id,firstName,lastName,userName,isLocked";

const CUSTOMER_ATTRIBUTES_CSV_HEADER: &str = "key,value";

// supported update types
pub enum UpdateType {
    CompanyName(String),
    QuotaMax(u64),
    UserMax(u64),
}

// supported customer print output
#[derive(Clone, Copy)]
pub enum PrintType {
    Pretty,
    Csv,
}

fn print_dracoon_error(err: &DracoonErrorResponse) {
    println!("{} {}", "Error".white().on_red(), err.error_message());
    if let Some(debug_info) = err.debug_info() {
        println!("{} {}", "Error details".white().on_red(), debug_info);
    };
}

fn handle_dracoon_errors(err: &DracoonClientError, msg: Option<&str>) -> () {
    let msg = msg.unwrap_or("Unknown error");

    println!("{} {}", "Error".white().on_red(), msg);

    match err {
        DracoonClientError::Http(err) => print_dracoon_error(err),
        _ => println!("{} {}", "Error".white().on_red(), "Uncaught error."),
    }
}

pub fn handle_errors(err: &DcProvError) {
    match err {
        DcProvError::BadRequest(err) => print_dracoon_error(err),
        DcProvError::Unauthorized(err) => print_dracoon_error(err),
        DcProvError::Forbidden(err) => print_dracoon_error(err),
        DcProvError::NotFound(err) => print_dracoon_error(err),
        DcProvError::PaymentRequired(err) => print_dracoon_error(err),
        DcProvError::Conflict(err) => print_dracoon_error(err),
        DcProvError::Unknown(err) => print_dracoon_error(err),
        DcProvError::Io => println!("{} {}", "Error".white().on_red(), "IO error."),
        DcProvError::Other => println!("{} {}", "Error".white().on_red(), "Uncaught error."),
        _ => println!("{} {}", "Error".white().on_red(), "Uncaught error."),
    }

    std::process::exit(1)
}

pub async fn init_provisioning(
    url: &str,
    token: Option<String>,
) -> Result<Dracoon<Provisioning>, DcProvError> {
    let url = if url.starts_with("https://") {
        url.to_string()
    } else if url.starts_with("http://") {
        url.replace("http://", "https://")
    } else {
        format!("https://{}", url)
    };

    let ask_for_token = || {
        dialoguer::Password::new()
            .with_prompt("Please enter X-SDS-Service-Token: ")
            .interact()
            .or(Err(DcProvError::Io))
    };

    let entry = Entry::new(SERVICE_NAME, &url).map_err(|_| DcProvError::CredentialStorageFailed);

    let (token, store) = match token {
        // Provided token, don't store
        Some(token) => (token, false),
        None => {
            // Entry present and holds a secret
            if let Ok(entry) = &entry {
                if let Ok(stored_secret) = get_dracoon_env(entry) {
                    (stored_secret, false)
                } else {
                    // Entry present but no secret, ask and store
                    (ask_for_token()?, true)
                }
            } else {
                // No entry, ask but don't store
                (ask_for_token()?, false)
            }
        }
    };

    // If necessary, create a new entry to store the secret
    if store {
        let entry =
            Entry::new(SERVICE_NAME, &url).map_err(|_| DcProvError::CredentialStorageFailed)?;
        set_dracoon_env(&entry, &token)?;
    }

    Ok(Dracoon::builder()
        .with_base_url(&url)
        .with_provisioning_token(token)
        .build_provisioning()
        .map_err(|_| DcProvError::InvalidAccount)?)
}

fn customer_to_string(customer: Customer, print_type: PrintType) -> String {
    match print_type {
        PrintType::Csv => {
            let cus_line = format!(
                "{},{},{},{},{},{},{},{}",
                customer.company_name,
                customer.customer_contract_type,
                customer.user_used,
                customer.user_max,
                customer.quota_used,
                customer.quota_max,
                customer.id,
                customer.created_at
            );
            cus_line
        }
        PrintType::Pretty => {
            let cus_line = format!("company: {} | contract: {} | users used: {} | users max: {} | quota used: {} | quota max: {} | id: {} | created_at: {}", customer.company_name, customer.customer_contract_type, customer.user_used, customer.user_max, customer.quota_used, customer.quota_max, customer.id, customer.created_at);
            cus_line
        }
    }
}

fn user_to_string(user: UserItem, print_type: PrintType) -> String {
    match print_type {
        PrintType::Csv => {
            let user_line = format!(
                "{},{},{},{},{}",
                user.id, user.first_name, user.last_name, user.user_name, user.is_locked
            );
            user_line
        }
        PrintType::Pretty => {
            let user_line = format!(
                "id: {} | first name: {} | last name: {} | user name: {} | is locked: {}",
                user.id, user.first_name, user.last_name, user.user_name, user.is_locked
            );
            user_line
        }
    }
}

fn customer_attribute_to_string(attrib: KeyValueEntry, print_type: PrintType) -> String {
    match print_type {
        PrintType::Csv => {
            let attrib_line = format!("{},{}", attrib.key, attrib.value);
            attrib_line
        }
        PrintType::Pretty => {
            let cus_line = format!("key: {} | value: {}", attrib.key, attrib.value);
            cus_line
        }
    }
}

pub async fn list_customers(
    provider: Dracoon<Provisioning>,
    filter: Option<String>,
    sort: Option<String>,
    offset: Option<u64>,
    limit: Option<u64>,
    print_type: Option<PrintType>,
    all: bool,
) {
    let print_type = print_type.unwrap_or(PrintType::Pretty);

    let params = build_params(filter.clone(), sort.clone(), offset, limit);

    let customers = provider.get_customers(Some(params)).await;

    if let Err(ref e) = customers {
        handle_dracoon_errors(e, Some("Could not list customers."));
        std::process::exit(1)
    };

    let mut customers = customers.unwrap();

    match print_type {
        PrintType::Csv => {
            println!("{}", CUSTOMER_CSV_HEADER);
        }
        PrintType::Pretty => {
            println!(
                "total customers: {} | offset: {} | limit: {}",
                customers.range.total, customers.range.offset, customers.range.limit
            );
        }
    };

    if all {
        for offset in 500..=customers.range.total {
            let params = build_params(filter.clone(), sort.clone(), Some(offset), limit);

            let next_customers = provider.get_customers(Some(params)).await;

            if let Err(ref e) = next_customers {
                handle_dracoon_errors(e, Some("Could not list customers."));
                std::process::exit(1)
            };

            let next_customers = next_customers.unwrap();

            customers.items.extend(next_customers.items);
        }
    }

    for customer in customers.items {
        let cus_line = customer_to_string(customer, print_type);
        println!("{}", cus_line);
    }
}

pub async fn get_customer(
    provider: Dracoon<Provisioning>,
    id: u64,
    print_type: Option<PrintType>,
) -> () {
    let print_type = print_type.unwrap_or(PrintType::Pretty);

    let customer = provider.get_customer(id, None).await;

    if let Err(ref e) = customer {
        handle_dracoon_errors(e, Some("Could not get customer info."));
        std::process::exit(1)
    };

    let customer = customer.unwrap();

    let cus_line = customer_to_string(customer, print_type);
    println!("{}", cus_line);
}

fn create_update_request(update_type: UpdateType) -> UpdateCustomerRequest {
    match update_type {
        UpdateType::CompanyName(name) => UpdateCustomerRequest::builder()
            .with_company_name(name)
            .build(),
        UpdateType::QuotaMax(quota_max) => UpdateCustomerRequest::builder()
            .with_quota_max(quota_max)
            .build(),
        UpdateType::UserMax(user_max) => UpdateCustomerRequest::builder()
            .with_user_max(user_max)
            .build(),
    }
}

pub async fn update_customer(provider: Dracoon<Provisioning>, id: u64, update_type: UpdateType) {
    let update_customer = create_update_request(update_type);

    let customer = provider.update_customer(id.into(), update_customer).await;

    if let Err(ref e) = customer {
        handle_dracoon_errors(e, Some("Could not update customer."));
        std::process::exit(1)
    };

    let customer = customer.unwrap();

    println!(
        "{}{}{}",
        "Success ".green(),
        "Updated customer with id ",
        id
    );

    let cus_line = format!(
        "company: {} | contract: {} | users max: {} | quota max: {} | id: {}",
        customer.company_name,
        customer.customer_contract_type,
        customer.user_max,
        customer.quota_max,
        customer.id
    );
    println!("{}", cus_line);
}

pub async fn delete_customer(provider: Dracoon<Provisioning>, id: u64) {
    match provider.delete_customer(id.into()).await {
        Ok(_) => {
            println!(
                "{}{}{}",
                "Success ".green(),
                "Deleted customer with id ",
                id
            );
            std::process::exit(0)
        }
        Err(ref e) => {
            handle_dracoon_errors(e, Some("Could not delete customer."));
            std::process::exit(1);
        }
    };
}

/// This function takes in a path to a JSON file (as string slice) and returns a request struct to create a new customer.
pub fn parse_customer_json_from_file(path: &str) -> Result<NewCustomerRequestDco3, DcProvError> {
    let raw_json = fs::read_to_string(path);

    let raw_json = match raw_json {
        Ok(res) => res,
        Err(e) => {
            println!(
                "{} {}{}",
                "Error".white().on_red(),
                "Could not open file from path ",
                path
            );
            println!("{:?}", e);
            std::process::exit(1)
        }
    };

    let new_customer = match serde_json::from_str::<NewCustomerRequest>(&raw_json) {
        Ok(customer) => customer,
        Err(e) => {
            println!(
                "{} {}{}",
                "Error".white().on_red(),
                "Could not parse customer from file ",
                path
            );
            println!("{:?}", e);
            std::process::exit(1)
        }
    };

    Ok(new_customer.into())
}

/// This function prompts for required fields via stdout and returns a request struct to create a new customer.
pub fn prompt_new_customer() -> Result<NewCustomerRequestDco3, DcProvError> {
    // first admin user
    println!("{}", "Step 1: Enter first admin user".white().on_blue());

    let first_name: String = dialoguer::Input::new()
        .with_prompt("Please enter first name: ")
        .interact()
        .or(Err(DcProvError::Io))?;

    let last_name: String = dialoguer::Input::new()
        .with_prompt("Please enter last name: ")
        .interact()
        .or(Err(DcProvError::Io))?;

    let email: String = dialoguer::Input::new()
        .with_prompt("Please enter email address: ")
        .interact()
        .or(Err(DcProvError::Io))?;

    let user_name: Option<String> = if dialoguer::Confirm::new()
        .with_prompt("Provide username?")
        .interact()
        .or(Err(DcProvError::Io))?
    {
        Some(
            dialoguer::Input::new()
                .with_prompt("Please enter username: ")
                .interact()
                .or(Err(DcProvError::Io))?,
        )
    } else {
        None
    };

    // customer
    println!("{}", "Step 2: Configure customer".white().on_blue());

    let company_name: String = dialoguer::Input::new()
        .with_prompt("Please enter company name: ")
        .interact()
        .or(Err(DcProvError::Io))?;

    let quota_max = loop {
        let quota_max: String = dialoguer::Input::new()
            .with_prompt("Please enter maxium quota (in bytes): ")
            .interact()
            .or(Err(DcProvError::Io))?;

        match quota_max.trim().parse::<u64>() {
            Ok(num) => {
                if num > 0 {
                    break num;
                }
            }
            Err(_) => {
                println!(
                    "{} {}",
                    "Error".white().on_red(),
                    "Please enter a valid positive number."
                );
            }
        };
    };

    let user_max = loop {
        let user_max: String = dialoguer::Input::new()
            .with_prompt("Please enter maxium users: ")
            .interact()
            .or(Err(DcProvError::Io))?;

        match user_max.trim().parse::<u64>() {
            Ok(num) => {
                if num > 0 {
                    break num;
                }
            }
            Err(_) => {
                println!(
                    "{} {}",
                    "Error".white().on_red(),
                    "Please enter a valid positive number."
                );
            }
        };
    };

    let user_name = user_name.unwrap_or(email.clone());

    // TODO: remove manual build once dco3 fixes bug with must_change_password
    let auth_data = UserAuthData::builder(AuthMethod::Basic)
        .with_must_change_password(true)
        .build();

    let first_admin_user = FirstAdminUser {
        first_name,
        last_name,
        user_name: Some(user_name),
        email: Some(email),
        auth_data: Some(auth_data),
        notify_user: Some(true),
        receiver_language: None,
        phone: None,
    };

    Ok(
        NewCustomerRequestDco3::builder("pay", quota_max, user_max, first_admin_user)
            .with_company_name(company_name)
            .build(),
    )
}

pub async fn create_customer(
    provider: Dracoon<Provisioning>,
    new_customer: NewCustomerRequestDco3,
) -> () {
    let customer = provider.create_customer(new_customer).await;

    if let Err(ref e) = customer {
        handle_dracoon_errors(e, Some(" customer info."));
        std::process::exit(1)
    };

    let customer = customer.unwrap();

    println!("{}{}", "Success ".green(), "Customer creeated.");
    println!(
        "Company name: {} | user max: {} | quota max: {} | id: {}",
        customer.company_name, customer.user_max, customer.quota_max, customer.id
    );
}

pub async fn get_customer_attributes(
    provider: Dracoon<Provisioning>,
    id: u64,
    filter: Option<String>,
    sort: Option<String>,
    offset: Option<u64>,
    limit: Option<u64>,
    print_type: Option<PrintType>,
) {
    let print_type = print_type.unwrap_or(PrintType::Pretty);

    let params = build_params(filter, sort, offset, limit);

    let attribs = provider
        .get_customer_attributes(id.into(), Some(params))
        .await;

    if let Err(ref e) = attribs {
        handle_dracoon_errors(e, Some("Could not get customer attributes."));
        std::process::exit(1)
    };

    let attribs = attribs.unwrap();

    match print_type {
        PrintType::Csv => {
            println!("{}", CUSTOMER_ATTRIBUTES_CSV_HEADER);
        }
        PrintType::Pretty => {
            println!("Customer attributes for customer with id: {}", id);
        }
    };

    if attribs.items.len() == 0 {
        println!("Customer has no customer attributes.")
    }

    for attrib in attribs.items {
        let attrib_line = customer_attribute_to_string(attrib, print_type);
        println!("{}", attrib_line);
    }
}

pub async fn update_customer_attributes(
    provider: Dracoon<Provisioning>,
    id: u64,
    attribs: Vec<(String, String)>,
) {
    let mut customer_attribs = CustomerAttributes::new();
    attribs.iter().for_each(|(key, value)| {
        customer_attribs.add_attribute(key, value);
    });

    let customer = provider
        .update_customer_attributes(id.into(), customer_attribs)
        .await;

    if let Err(ref e) = customer {
        handle_dracoon_errors(e, Some("Could not update customer attributes."));
        std::process::exit(1)
    };

    let customer = customer.unwrap();

    println!(
        "{}{}{}",
        "Success ".green(),
        "Updated customer attributes of customer with id ",
        customer.id
    );
}

pub async fn get_customer_users(
    provider: Dracoon<Provisioning>,
    id: u64,
    filter: Option<String>,
    sort: Option<String>,
    offset: Option<u64>,
    limit: Option<u64>,
    print_type: Option<PrintType>,
) -> () {
    let print_type = print_type.unwrap_or(PrintType::Pretty);

    let params = build_params(filter, sort, offset, limit);

    let user_list = provider.get_customer_users(id.into(), Some(params)).await;

    if let Err(ref e) = user_list {
        handle_dracoon_errors(e, Some("Could not get customer users."));
        std::process::exit(1)
    };

    let user_list = user_list.unwrap();

    match print_type {
        PrintType::Csv => {
            println!("{}", CUSTOMER_USERS_CSV_HEADER);
        }
        PrintType::Pretty => {
            println!(
                "total users: {} | offset: {} | limit: {}",
                user_list.range.total, user_list.range.offset, user_list.range.limit
            );
        }
    };

    for user in user_list.items {
        let user_line = user_to_string(user, print_type);
        println!("{}", user_line);
    }
}

pub fn print_version() {
    println!("@@@@@@@@@@@@@   @@@@@@@@@@@@@   @@@@@@@@@@@@@  @@@@@@@@@@@@@%   @@@@@@@@@@@@  @@@@@@   @@@@@  ");
    println!("@@@@@@@@@@@@@@  @@@@@@@@@@@@@@ @@@@@@@@@@@@@@  @@@@@@@@@@@@@@  @@@@@@@@@@@@@@ @@@@@@   @@@@@  ");
    println!("@@@@@   @@@@@@  @@@@@   @@@@@@ @@@@@@   @@@@@  @@@@@    @@@@@  @@@@@   @@@@@@ @@@@@@   @@@@@  ");
    println!("@@@@@   @@@@@@  @@@@@          @@@@@@   @@@@@  @@@@@    @@@@@  @@@@@   @@@@@@ @@@@@@   @@@@@  ");
    println!("@@@@@   @@@@@@  @@@@@          @@@@@@@@@@@@@@  @@@@@@@@@@@@@@  @@@@@   @@@@@@ @@@@@@   @@@@@ ");
    println!("@@@@@   @@@@@@  @@@@@          @@@@@@@@@@@@@   @@@@@@@@@@      @@@@@   @@@@@@ @@@@@@   @@@@@ ");
    println!("@@@@@   @@@@@@  @@@@@          @@@@@@@@@       @@@@@@@@@@@@@   @@@@@   @@@@@@ @@@@@@   @@@@@  ");
    println!("@@@@@   @@@@@@  @@@@@          @@@@@@          @@@@@@  &@@@@@  @@@@@   @@@@@@ @@@@@@   @@@@@  ");
    println!("@@@@@   @@@@@@  @@@@@   @@@@@@ @@@@@@          @@@@@@   @@@@@  @@@@@   @@@@@@ @@@@@@   @@@@@  ");
    println!("@@@@@@@@@@@@@@  @@@@@@@@@@@@@@ @@@@@@          @@@@@@   @@@@@  @@@@@   @@@@@@ @@@@@@@@@@@@@@  ");
    println!("@@@@@@@@@@@@    @@@@@@@@@@@@   @@@             @@@@@@   @@@@@  @@@@@   @@@@@@   @@@@@@@@@@@@   ");
    println!("@@@@@@@@@       @@@@@@@@       @@                  @@   @@@@@  @@@@@@@@@@@@@@       @@@@@@@@    ");
    println!("@@@@@@          @@@@@                                   @@@@@    @@@@@@@@@@            @@@@@   ");
    println!("@@@             @@                                       @@@@       @@@@@                 @@     ");
    println!("@               @                                          @@        @@                    @");
    println!("");
    println!(
        "                               {} version {}",
        env!("CARGO_PKG_NAME"),
        env!("CARGO_PKG_VERSION")
    );
    println!("                          DRACOON Provisioning CLI tool                       ");
    println!(
        "                                Octavio Simone                                      "
    );
    println!("                     https://github.com/unbekanntes-pferd/dcprov         ");
}

fn build_params(
    filter: Option<String>,
    sort: Option<String>,
    offset: Option<u64>,
    limit: Option<u64>,
) -> ListAllParams {
    let params = ListAllParams::builder();

    let params = if let Some(offset) = offset {
        params.with_offset(offset)
    } else {
        params
    };

    let params = if let Some(limit) = limit {
        params.with_limit(limit)
    } else {
        params
    };

    let params = if let Some(filter) = filter {
        params.with_filter(filter)
    } else {
        params
    };

    let params = if let Some(sort) = sort {
        params.with_sort(sort)
    } else {
        params
    };

    params.build()
}
