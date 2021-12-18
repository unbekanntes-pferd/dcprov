

# DCPROV – DRACOON Provisioning CLI

## What is this?

This is a command line tool for the DRACOON provisioning API built in Rust with: 

* reqwest
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
- [Linux x86](https://github.com/unbekanntes-pferd/dcprov/releases/download/v0.1.0/dcprov-linux-x86-64-0.1.0.zip)
- [Windows x86](https://github.com/unbekanntes-pferd/dcprov/releases/download/v0.1.0/dcprov-win64-0.1.0.zip) 
- [MacOS M1 (arm64)](https://github.com/unbekanntes-pferd/dcprov/releases/download/v0.1.0/dcprov-darwin-arm64.zip)

2. Compile from source:
To compile from source, git clone this repo and run

```bash
cargo build --release
```

### Basic commands

The tool is fairly simple to use via commandline and comes with the following commands:

* list – list all available customers
* get – get a single customer by id
* update – update a single customer by id
* delete – delete a single customer by id
* config – configure (set, get or remove) token (secure storage: keytar bindings)

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

You can pass any API parameters like
- offset
- limit
- filter
- sort

To do so, use either the short or long version, example for a filter:

```bash
./dcprov list https://dracoon.team -f companyName:cn:DRACOON
```

#### Get a single customer

To list the info of a single customer, use the get command with the corresponding id:

```bash
./dcprov get https://dracoon.team 999
```

If you don't know the id, search for the id with the list command and filter e.g. via company name (see example above for filter).

#### Create a new customer

To create a new customer, there are two supported ways:
- Create from file (JSON)
- Interactive prompt (local users only)

To create a customer from a file, use the following command:

```bash
./dcprov create from-file https://dracoon.team ./test.json
```

To create a customer from the prompt, use the following command:

```bash
./dcprov create prompt https://dracoon.team
```

#### Update a customer

To update a customer, specify the supported update command (command in parenthesis):
- maximum quota (quota-max)
- maximum users (user-max)
- company name (company-name)

Use the following command to update (example updating user max to 1000):

```bash
./dcprov update https://dracoon.team 999 user-max 1000
```

Example to update quota max (in bytes!):
```bash
./dcprov update https://dracoon.team 999 quota-max 1000000000
```

Example to update the company name:

```bash
./dcprov update https://dracoon.team 999 company-name "DRACOON TEST"
```

#### Delete a single customer

To delete a single customer, provide the id with the following command:

```bash
./dcprov delete https://dracoon.team 999 
```

#### Configure the token 

In order to perform any requests, you will need to enter the X-SDS-Service-Token. 

##### Setting (securely storing) a token
To store a token, use the set command:

```bash
./dcprov config set https://dracoon.team your-very-secret-token
```
The token will be stored securely based on your OS (keytar bindings).

##### Getting a securely stored token
To print a token to screen, use the get command:

```bash
./dcprov config get https://dracoon.team
```
The token will be fetched from the secure storage and printed to screen – the command can be used as well to check, if a token is stored for a given domain.

##### Deleting a securely stored token
To delete, use the delete command:

```bash
./dcprov config delete https://dracoon.team
```
The token will be removed from the secure storage.



