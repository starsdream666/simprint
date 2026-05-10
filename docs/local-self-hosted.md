# Local Self-Hosted Setup

This setup runs the Simprint server stack locally with Docker Desktop:

- `simprint-server`
- `postgres`
- `redis`
- `mailpit` for local email verification on `mailpit:1025`

## Ports

- `40041`: Simprint server
- `8025`: Mailpit web inbox
- `1025`: Mailpit SMTP

## Start the stack

```powershell
cd deploy/local-self-hosted
docker compose up -d
docker compose ps
```

## Client configuration

Development builds should point to:

```toml
[server]
base_url = "http://127.0.0.1:40041/api/"
version = "v1"
secret_key = "Nuexz9Y2hRc5Z6HK7Atb"
```

The repository already uses that value in `src-tauri/config.development.toml`.

## Register the first user

1. Start the desktop client in development mode.
2. Open the register page in the client.
3. Request a verification code.
4. Open `http://127.0.0.1:8025` in a browser.
5. Copy the verification code from Mailpit and finish registration.

## Health checks

```powershell
docker compose logs -f simprint-server
docker compose logs -f postgres
docker compose logs -f redis
curl http://127.0.0.1:40041/api/v1/time/now
```

## Stop the stack

```powershell
cd deploy/local-self-hosted
docker compose down
```

Add `-v` if you also want to remove the local database and Redis data.
