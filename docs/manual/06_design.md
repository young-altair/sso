# Design

## Architecture

| Directory     | Description                                                                                                                                 |
| ------------- | ------------------------------------------------------------------------------------------------------------------------------------------- |
| `migrations`  | SQL migration files for [Postgres][postgresql] driver. [SQLite][sqlite] driver is planned.                                                  |
| `/src/api`    | API request and response types and synchronous handlers.                                                                                    |
| `/src/client` | Synchronous and asynchronous clients using [reqwest][reqwest] crate and `api` module types.                                                 |
| `/src/driver` | Database types and driver for Postgres, SQLite support is planned.                                                                          |
| `/src/notify` | Email notifications actor.                                                                                                                  |
| `/src/server` | Actix-web frontend for API, asynchronous route handlers deserialise data for synchronous `api` calls.                                       |
| `/tests`      | Integration tests which use `ClientSync` to run tests against an instance of `sso`.                                                         |

## OWASP: ASVS

[OWASP ASVS][owasp-asvs]

The OWASP Application Security Verification Standard is being used as a reference to improve this application. These are some development and design notes based on requirements from the 4.0 version of the ASVS standard. This is a self-evaluation and should be viewed skeptically.

### 1.2.1

- Binary sso and postgres/sqlite must be run as unique or special low privelege operating system accounts.

TODO(docs): Systemd unit file examples, sso, postgres, nginx, etc.

### 1.2.2

- HTTP calls (except ping) require service key authentication.

TODO(docs): Mutual TLS using rustls configuration and PKI for communication between sso and services.

### 1.2.3

- Binary sso is designed to provide multiple authentication mechanisms, none of which have been vetted.
- Relies on libraries which may be unvetted, e.g. libreauth, jsonwebtoken, rustls, etc.
- What does strong authentication mean in this context?
- One feature of sso is providing email/password login, which is probably not considered strong authentication.
- Audit logging and monitoring via prometheus.

TODO(feature): Audit logging and prometheus metrics improvements for detecting account abuse and breaches.

### 1.2.4

- All authentication pathways are designed to be as strong as that pathway can be.
- For example, email password resets are supported which are probably not considered strong.

TODO(feature): More opt-ins for pathway branches that may be weak, for example ability to reset passwords by email.

### 1.4.1

- All access controls are enforced at a trusted enforcement point (the server).
- Registered services must implement their own access controls for their own data.

### 1.4.2

- Access controls are designed for many services and many users, where users have access to one or more services.
- All registered services can read all registered users, other services and keys belonging to them are hidden.
- Registered services may implement more complex access controls for their own data.

TODO(test): More tests for data access, is service data masked correctly.

### 1.4.3

- Verify enforcement of principle of least privelege, requires more integration tests.

TODO(test): More tests on preventing spoofing, elevation of privelege.

### 1.4.4

- HTTP calls (except ping) require service key authentication.

TODO(refactor): Service key authentication mechanism code is split across files, cleaner code.

### 1.4.5

- This crate provides user authentication, not access control, is this out of scope?

TODO(feature): Structured data for users, may require access controls.

### 1.5.1


TODO(docs): GDPR and other data protection compliance research.

### 1.5.2

- API for sso is JSON requests over HTTP so serialisation is required.
- Using [serde][serde] and [serde_qs][serde_qs] for serialisation and deserialisation.

TODO(test): Test requests with other/unknown content types are handled correctly.

TODO(feature): Flag(s) to require HTTPS to ensure all requests/responses are encrypted in transmit.

### 1.5.3

- Input validation is enforced at a trusted enforcement point (the server).
- Using [validator][validator] for input validation.

TODO(test): More input tests including unicode passwords, bad strings, etc.

### 1.5.4

- All output encoding is [UTF-8][utf-8].

### 1.6.1

- Key values are used for [JWT][jwt] cryptographic encoding and decoding.
- Key values are only returned to service or user on creation.
- Keys can be disabled and/or revoked to prevent use.

TODO(docs): Check this meets key management standard NIST SP 800-57.

### 1.6.2

- Cannot verify that services protect created key values.

### 1.6.3

- No hard-coded keys or passwords, all keys and passwords can be replaced.

### 1.6.4

- API key support is clear-text equivalent.
- Authentication via API key is probably not considered low risk secret.
- Keys can be disabled/revoked to mitigate breaches, but this is not a solution.

### 1.7.1, 1.7.2

- Audit log format is common and used when making calls via API.
- Stdout/stderr logging is not consistent.
- Audit logs are saved to table, not transmitted to a remote system.
- Stdout/stderr logging is not transmitted to a remote system,

TODO(feature): Option to transmit audit logs, stdout/stderr to external service(s).

### 1.8.1, 1.8.2

- Sensitive data is not identified or classified into protection levels.

TODO(docs): Evaluate data and identify/classify sensitive data.
TODO(refactor): Audit log retention configuration.

### 1.9.1, 1.9.2

- Connection to database, other services must be encrypted.

TODO(docs): Mutual TLS encryption/authentication for postgres connection.