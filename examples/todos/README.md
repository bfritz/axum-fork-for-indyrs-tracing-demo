## Run PostgreSQL Server

Start an ephemeral instance of [PostgreSQL] in Docker.  Specifying the
command as `postgres -c log_statement=all` will tell the database
server to log all queries to the terminal.

    docker run --name pg --rm -it \
        -e POSTGRES_HOST_AUTH_METHOD=trust \
        -p 7432:5432 \
        postgres:14.1-alpine postgres -c log_statement=all

## Initialize Database

    # our local database server running in docker
    export DATABASE_URL="postgres://postgres@localhost:7432/todos"

    cargo install sqlx-cli
    $HOME/.cargo/bin/sqlx db create
    $HOME/.cargo/bin/sqlx migrate run

## Start Axum Web Server

    cargo run

## Make REST Calls

    URL=http://localhost:3000/todos
    curl -X POST -H 'Content-Type: application/json' -d '{"text": "Do something."}' $URL
    curl -X POST -H 'Content-Type: application/json' -d '{"text": "Do something else."}' $URL

    # or with curlie
    curlie post $URL text="Get stuff done."

    curl -s $URL | jq -r

    # or with curlie
    curlie $URL

[curlie] is an http client with the power of [curl], the ease of use of [httpie].

[curl]: https://curl.haxx.se/
[curlie]: https://curlie.io/
[httpie]: https://httpie.org/
[postgresql]: https://postgresql.org/
