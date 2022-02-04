# Notes for Indy Rust Demo

Below are the notes I used for the [February Indy.rs] demo.  I modified
the Axum "todos" example to use a PostgreSQL database backend and added
light instrumentation with [tracing] to show traces in [Jaeger].

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

## Start Jaeger for Visualization

Copied verbatim from the [Jaeger getting started page]:

    docker run --rm --name jaeger \
      -e COLLECTOR_ZIPKIN_HOST_PORT=:9411 \                                                                                                       -p 5775:5775/udp \
      -p 6831:6831/udp \
      -p 6832:6832/udp \
      -p 5778:5778 \
      -p 16686:16686 \
      -p 14250:14250 \
      -p 14268:14268 \
      -p 14269:14269 \
      -p 9411:9411 \
      jaegertracing/all-in-one:1.30


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

## View Traces in Jaeger

The Jaeger UI running in the docker container above is listening on port `:16686`.  Review
your traces at by opening <http://localhost:16686/>.


[curl]: https://curl.haxx.se/
[curlie]: https://curlie.io/
[february indy.rs]: https://www.meetup.com/indyrs/events/qwtdjsydcdbdb/
[httpie]: https://httpie.org/
[jaeger]: https://jaegertracing.io/
[jaeger getting started page]: https://www.jaegertracing.io/docs/1.30/getting-started/
[postgresql]: https://postgresql.org/
[tracing]: https://tokio.rs/#tk-lib-tracing
