

# dcprov – DRACOON Provisioning CLI

## What is this?

This is a command line tool for the DRACOON provisioning API built in Rust with: 

* reqwest
* clap
* serde
* tokio (async runtime)
* keytar (bindings)

DRACOON is a cloud storage product / service (SaaS) by DRACOON GmbH (https://dracoon.com). 
DRACOON API documentation can be found here (Swagger UI):

https://dracoon.team/api/

The provisioning API allows to create customers within a tenant. 

## How does it work?

To get going, either get a compiled binary for your OS or compile it from source.

1. Compiled binaries:

[dcprov releases](https://github.com/unbekanntes-pferd/dcprov/releases)

2. Compile from source:
To compile from source, git clone this repo and run

```bash
cargo build --release
```
You need to install Rust in order to build dcprov.

### Installation
There's no installation - just move the binary into your standard path for binaries e.g. `/usr/local/bin` or add the 
path where you have the binary to Path (e.g. Windows).

### Basic commands

The tool is fairly simple to use via commandline and comes with the following commands:

* list – list all available customers
* get – get a single customer by id
* update – update a single customer by id
* delete – delete a single customer by id
* config – configure (set, get or remove) token (secure storage: keytar bindings)
* get-users - get all users for a customer by id
* get-attributes - get all attributes for a customer by id
* set-attributes - set attribute(s) for a customer by id

#### List all customers

Example usage in Linux / MacOS:

```bash
./dcprov list https://dracoon.team 
```
Example usage in Windows:
```bash
dcprov.exe list https://dracoon.team 
```
If no token has been stored, the tool will provide a prompt to store credentials securely:
* Windows: Windows credentials vault
* MacOS: Keychain
* Linux: lib-secret (needs to be installed!)

If you use a system that does not allow storage (e.g. headless Linux), you can pass the token via parameter:

```bash
# can be any command
dcprov list https://dracoon.team --token $yourSecretToken
```

You can pass any API parameters like
- offset
- limit
- filter
- sort

To do so, use either the short or long version, example for a filter:

```bash
# short version for filter 
dcprov list https://dracoon.team -f companyName:cn:DRACOON
```

```bash
# long version for sort (sorts by company name in alphabetical order)
dcprov list https://dracoon.team --sort companyName:asc
```
By default, the output is "pretty printed" to stdout.
If required, the output can be formatted as CSV by passing the csv flag:

```bash
# example exporting the output in csv format to a file
dcprov list https://dracoon.team --csv > ./customers.csv

If you need all customers at once, use the the all flag:

dcprov list https://dracoon.team --all --csv > ./customers.csv

```

#### Get a single customer

To list the info of a single customer, use the get command with the corresponding id:

```bash
dcprov get https://dracoon.team 999
```

If you don't know the id, search for the id with the list command and filter e.g. via company name (see example above for filter).

#### Create a new customer

To create a new customer, there are two supported ways:
- Create from file (JSON)
- Interactive prompt (local users only)

To create a customer from a file, use the following command:

```bash
dcprov create https://dracoon.team from-file ./test.json
```

To create a customer from the prompt, use the following command:

```bash
dcprov create https://dracoon.team prompt
```

#### Update a customer

To update a customer, specify the supported update command (command in parenthesis):
- maximum quota (quota-max)
- maximum users (user-max)
- company name (company-name)

Use the following command to update (example updating user max to 1000):

```bash
dcprov update https://dracoon.team 999 user-max 1000
```

Example to update quota max (in bytes!):
```bash
dcprov update https://dracoon.team 999 quota-max 1000000000
```

Example to update the company name:

```bash
dcprov update https://dracoon.team 999 company-name "DRACOON TEST"
```

#### Delete a single customer

To delete a single customer, provide the id with the following command:

```bash
dcprov delete https://dracoon.team 999 
```

#### Configure the token 

In order to perform any requests, you will need to enter the X-SDS-Service-Token. 

##### Setting (securely storing) a token
To store a token, use the set command:

```bash
dcprov config https://dracoon.team set your-very-secret-token
```
The token will be stored securely based on your OS (keytar bindings).

##### Getting a securely stored token
To print a token to screen, use the get command:

```bash
dcprov config https://dracoon.team get
```
The token will be fetched from the secure storage and printed to screen – the command can be used as well to check, if a token is stored for a given domain.

##### Deleting a securely stored token
To delete, use the delete command:

```bash
dcprov config https://dracoon.team delete
```
The token will be removed from the secure storage.


#### Getting customer users 

As with listing customers, you can pass any parameters (filter, sort, offset, limit) and can select which output should be 
generated for the user list.

Example filtering for a specific user with a login "dracoonhero":

```bash
dcprov get-users https://dracoon.team 999 --filter userName:cn:dracoonhero
```

Example storing the user list in a CSV:
```bash
dcprov get-users https://dracoon.team 999 --csv > customer_999_users.csv
```

#### Getting customer attributes

You can also list all customer attributes with the get-attributes command.

Example filtering for a specific key with name "dracoon_id":

```bash
dcprov get-attributes https://dracoon.team 999 --filter key:eq:dracoon_id
```

Example storing the attributes list in a CSV:
```bash
dcprov get-attributes https://dracoon.team 999 --csv > customer_999_attribs.csv
```

#### Setting customer attributes

You can set multiple attributes with the set-attributes command.

Example setting multiple key-value pairs (-a required for each attribute!):

```bash
dcprov set-attributes https://dracoon.team 999 --csv -a key1=value1 -a key2=value2 -a key3=value3
```


