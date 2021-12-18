use crate::credentials::{get_dracoon_env, set_dracoon_env};
use crate::provisioning::{
    self, Customer, DRACOONProvisioning, DRACOONProvisioningError, FirstAdminUser,
    GetCustomersResponse, NewCustomerRequest, NewCustomerResponse, UpdateCustomerRequest,
    UpdateCustomerResponse, UserAuthData,
};
use std::fs;

const CSV_HEADER: &str = "companyName,contractType,userUsed,userMax,quotaUsed,quotaMax,id,createdAt";
pub enum UpdateType {
    CompanyName(String),
    QuotaMax(i64),
    UserMax(i64),
}
#[derive(Clone, Copy)]
pub enum PrintType {
    Pretty,
    Csv
}

pub async fn init_provisioning(url: String) -> DRACOONProvisioning {
    let mut from_creds: bool = false;
    let token = match get_dracoon_env(&url) {
        Ok(pwd) => {
            from_creds = true;
            pwd
        }
        Err(e) => {
            println!("Please enter X-SDS-Service-Token: ");
            let mut service_token = String::new();
            std::io::stdin()
                .read_line(&mut service_token)
                .expect("Error parsing user input (service token).");

            service_token
        }
    };

    let provider: DRACOONProvisioning;

    match provisioning::DRACOONProvisioning::new(
        url.trim_end().to_string(),
        token.trim_end().to_string(),
    )
    .await
    {
        Ok(prov) => provider = prov,
        Err(err) => {
            println!("An error ocurred – could not create provider");
            println!("{:?}", err);
            std::process::exit(1)
        }
    };

    if from_creds == false {
        loop {
            println!("Store service token for {} securely? (Y/N)", url);

            let mut store = String::new();
            std::io::stdin()
                .read_line(&mut store)
                .expect("Error parsing user input (store credentials response).");

            match store.as_str().trim() {
                "Y" => {
                    set_dracoon_env(&url, &token);
                    break;
                }
                "y" => {
                    set_dracoon_env(&url, &token);
                    break;
                }
                "N" => break,
                "n" => break,
                _ => (),
            };
        }
    }

    provider
}

fn customer_to_string(customer: Customer, print_type: PrintType) -> String {

    match print_type {
        PrintType::Csv => {
            let cus_line = format!("{},{},{},{},{},{},{},{}\n", customer.company_name, customer.customer_contract_type, customer.user_used, customer.user_max, customer.quota_used, customer.quota_max, customer.id, customer.created_at);
            cus_line
        },
        PrintType::Pretty => {

            let cus_line = format!("company: {} | contract: {} | users used: {} | users max: {} | quota used: {} | quota max: {} | id: {} | created_at: {}", customer.company_name, customer.customer_contract_type, customer.user_used, customer.user_max, customer.quota_used, customer.quota_max, customer.id, customer.created_at);
            cus_line
        }
    }

}

pub async fn list_customers(
    provider: DRACOONProvisioning,
    filter: Option<String>,
    sort: Option<String>,
    offset: Option<i64>,
    limit: Option<i64>,
    print_type: PrintType,
) -> () {
    let customer_res: Option<GetCustomersResponse>;

    match provider
        .get_customers(filter, sort, limit, offset, None)
        .await
    {
        Ok(res) => customer_res = Some(res),
        Err(e) => {
            customer_res = None;
            println!("Could not fetch customers: \n");
            println!("{:?}", e);
            std::process::exit(1)
        }
    };

    if let Some(customer_res) = customer_res {

        match print_type {
            PrintType::Csv => {
                println!("{}", CSV_HEADER);
            },
            PrintType::Pretty => {
                println!(
                    "total customers: {} | offset: {} | limit: {}",
                    customer_res.range.total, customer_res.range.offset, customer_res.range.limit
                );
            }
        };

        for customer in customer_res.items {
            let cus_line = customer_to_string(customer, print_type);
            println!("{}", cus_line);
        }
    };
}

pub async fn get_customer(provider: DRACOONProvisioning, id: u32, print_type: PrintType) -> () {
    let customer: Option<Customer>;

    match provider.get_customer(id.into(), None).await {
        Ok(res) => customer = Some(res),
        Err(e) => {
            customer = None;
            println!("Could not fetch customers: \n");
            println!("{:?}", e);
            std::process::exit(1)
        }
    };

    if let Some(customer) = customer {

        let cus_line = customer_to_string(customer, print_type);
        println!("{}", cus_line);
    };
}

fn create_update_request(update_type: UpdateType) -> UpdateCustomerRequest {
    let update_customer: UpdateCustomerRequest = match update_type {
        UpdateType::CompanyName(name) => UpdateCustomerRequest {
            company_name: Some(name),
            customer_contract_type: None,
            quota_max: None,
            user_max: None,
            is_locked: None,
            provider_customer_id: None,
            webhooks_max: None,
        },
        UpdateType::QuotaMax(quota_max) => UpdateCustomerRequest {
            company_name: None,
            customer_contract_type: None,
            quota_max: Some(quota_max),
            user_max: None,
            is_locked: None,
            provider_customer_id: None,
            webhooks_max: None,
        },
        UpdateType::UserMax(user_max) => UpdateCustomerRequest {
            company_name: None,
            customer_contract_type: None,
            quota_max: None,
            user_max: Some(user_max),
            is_locked: None,
            provider_customer_id: None,
            webhooks_max: None,
        },
    };

    update_customer
}

pub async fn update_customer(
    provider: DRACOONProvisioning,
    id: u32,
    update_type: UpdateType,
) -> () {
    let customer: Option<UpdateCustomerResponse>;

    let update_customer = create_update_request(update_type);

    match provider.update_customer(id.into(), update_customer).await {
        Ok(res) => customer = Some(res),
        Err(e) => {
            customer = None;
            println!("Could not update customer: \n");
            println!("{:?}", e);
            std::process::exit(1);
        }
    };

    if let Some(customer) = customer {
        println!("Updated customer successfully: \n");

        let cus_line = format!(
            "company: {} | contract: {} | users max: {} | quota max: {} | id: {}",
            customer.company_name,
            customer.customer_contract_type,
            customer.user_max,
            customer.quota_max,
            customer.id
        );
        println!("{}", cus_line);
    };
}

pub async fn delete_customer(provider: DRACOONProvisioning, id: u32) -> () {
    let customer: Result<(), DRACOONProvisioningError>;

    match provider.delete_customer(id.into()).await {
        Ok(res) => {
            println!("Successfully deleted customer with id {}", id);
            std::process::exit(0)
        }
        Err(e) => {
            println!("Could not delete customer with id: {}\n", id);
            println!("{:?}", e);
            std::process::exit(1);
        }
    };
}

pub fn parse_customer_json_from_file(path: &str) -> NewCustomerRequest {
    let raw_json = fs::read_to_string(path);

    let raw_json = match raw_json {
        Ok(res) => res,
        Err(e) => {
            println!(
                "Error opening specified file from path '{}' \n {:?}",
                path, e
            );
            std::process::exit(1)
        }
    };

    let new_customer = match serde_json::from_str::<NewCustomerRequest>(&raw_json) {
        Ok(customer) => customer,
        Err(e) => {
            println!(
                "Error parsing JSON (invalid format?) from file '{}' \n {:?}",
                path, e
            );
            std::process::exit(1)
        }
    };

    new_customer
}

pub fn prompt_new_customer() -> NewCustomerRequest {
    let first_name: String;
    let last_name: String;
    let final_quota_max: i64;
    let final_user_max: i64;

    // first admin user
    println!("Step 1: Enter first admin user");

    // full name
    loop {
        println!("Please enter full name (first & last name separated by SPACE): ");
        let mut full_name = String::new();
        std::io::stdin()
            .read_line(&mut full_name)
            .expect("Error parsing user input (full name).");

        let names: Vec<&str> = full_name.split(" ").collect();

        match names.len() {
            2 => {
                first_name = names[0].to_string();
                last_name = names[1].trim().to_string();
                break;
            }
            _ => {
                println!("Please use correct format: firstname lastname.");
            }
        };
    }

    // email
    println!("Please enter email address: ");
    let mut email = String::new();
    std::io::stdin()
        .read_line(&mut email)
        .expect("Error parsing user input (email).");

    // optional username
    println!("Please enter username (optional – default: email): ");
    let mut username = String::new();
    std::io::stdin()
        .read_line(&mut username)
        .expect("Error parsing user input (email).");

    let username = match username.trim() {
        "" => email.clone(),
        _ => username,
    };

    // customer
    println!("Step 2: Configure customer");

    println!("Please enter company name: ");
    let mut company_name = String::new();
    std::io::stdin()
        .read_line(&mut company_name)
        .expect("Error parsing user input (company name).");

    // users max
    loop {
        println!("Please enter maxium users: ");
        let mut user_max = String::new();
        std::io::stdin()
            .read_line(&mut user_max)
            .expect("Error parsing user input (user).");

        match user_max.trim().parse::<i64>() {
            Ok(num) => {
                if num > 0 {
                    final_user_max = num;
                    break;
                }
            }
            Err(e) => {
                println!("Please enter a valid positive number.")
            }
        };
    }

    // quota max
    loop {
        println!("Please enter maxium quota (in bytes): ");
        let mut quota_max = String::new();
        std::io::stdin()
            .read_line(&mut quota_max)
            .expect("Error parsing user input (quota).");

        match quota_max.trim().parse::<i64>() {
            Ok(num) => {
                if num > 0 {
                    final_quota_max = num;
                    break;
                }
            }
            Err(e) => {
                println!("Please enter a valid positive number.")
            }
        };
    }

    let local_auth = UserAuthData {
        method: "basic".to_string(),
        must_change_password: Some(true),
        login: None,
        ad_config_id: None,
        oid_config_id: None,
        password: None,
    };

    let first_admin = FirstAdminUser {
        first_name: first_name.trim().to_string(),
        last_name: last_name.trim().to_string(),
        user_name: Some(username.trim().to_string()),
        auth_data: Some(local_auth),
        receiver_language: None,
        notify_user: Some(true),
        email: Some(email.trim().to_string()),
        phone: None,
    };

    NewCustomerRequest {
        customer_contract_type: "pay".to_string(),
        quota_max: final_quota_max,
        user_max: final_user_max,
        first_admin_user: first_admin,
        company_name: Some(company_name.trim().to_string()),
        trial_days: None,
        is_locked: None,
        customer_attributes: None,
        provider_customer_id: None,
        webhooks_max: None,
    }
}

pub async fn create_customer(
    provider: DRACOONProvisioning,
    new_customer: NewCustomerRequest,
) -> () {
    let customer_res: Option<NewCustomerResponse>;

    match provider.create_customer(new_customer).await {
        Ok(res) => customer_res = Some(res),
        Err(e) => {
            customer_res = None;
            println!("Could not create customer: \n");
            println!("{:?}", e);
            std::process::exit(1)
        }
    };

    if let Some(customer_res) = customer_res {
        println!(
            "Created customer: {} | user max: {} | quota max: {}",
            customer_res.company_name, customer_res.user_max, customer_res.quota_max
        );
    };
}
