## API Key

Create service with key and start server.

```bash
sso create-service-with-key $service_name $service_url
sso start-server
```

Service creates a user without password.

```bash
curl --header "Content-Type: application/json" \
  --header "Authorization: $service_key" \
  --request POST \
  --data '{"is_enabled":true,"name":"$user_name","email":"$user_email"}' \
  $server_url/v1/user
```

Service creates a key for user.

```bash
curl --header "Content-Type: application/json" \
  --header "Authorization: $service_key" \
  --request POST \
  --data '{"is_enabled":true,"type":"Key","name":"$key_name","user_id":"$user_id"}' \
  $server_url/v1/key
```

User makes requests to service with key value, key can be verified to authenticate requests.

```bash
curl --header "Content-Type: application/json" \
  --header "Authorization: $service_key" \
  --request POST \
  --data '{"key":"$user_key"}' \
  $server_url/v1/auth/key/verify
```

Key can be revoked, this will disable and revoke the key created earlier and prevent verification.

```bash
curl --header "Content-Type: application/json" \
  --header "Authorization: $service_key" \
  --request POST \
  --data '{"key":"$user_key"}' \
  $server_url/v1/auth/key/revoke
```