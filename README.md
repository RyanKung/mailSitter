Mail Sitter
------------------

Mail Sitter is a command-line tool designed to simplify email management tasks. With Mail Sitter, you can easily initialize email configurations, fetch emails, and even generate new privacy email addresses through DuckDuckGo's email protection service.

## Features

* Configuration Initialization: Easily set up your email configuration including SMTP server details and credentials.

* Email Fetching: Fetch unread emails from your configured email account.

* DuckDuckGo Email Protection Integration: Generate new privacy email addresses for enhanced security and privacy.

## Installation

```
cargo install mail-sitter
```

## Usage

### 1. Initialize Configuration
To initialize the configuration, run:

```
mail-sitter init --email <your_email> --pwd <your_password> --smtp <smtp_server_address>
```

Optionally, if you are using DuckDuckGo's email protection service, you can provide your username with the --username flag.


### 2. Fetch Emails

To fetch unread emails, simply run:

```
mail-sitter fetch
```

### 3. Request Alias (DuckDuckGo Email Protection)

If you're using DuckDuckGo's email protection service, you can generate a new privacy email address by running:

```
mail-sitter address
```

## Additional Notes

* Google App Password: If you're using Gmail, you might need to create an app password. You can find instructions on how to do this [here](https://support.google.com/accounts/answer/185833?hl=en).

License
This project is licensed under the MIT License - see the LICENSE file for details.
