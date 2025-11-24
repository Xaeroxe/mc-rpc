# schema_fetch

run this small rust app to fetch a new schema.json from a running minecraft server.  

## Example `server.properties`

How the `server.properties` could look like when using a local minecraft server just to get the schema from it.  

```sh
management-server-allowed-origins=
management-server-enabled=true
management-server-host=localhost
management-server-port=7218
management-server-secret=a325DpwUnti8XdqpAR0c9q3wHjdVj8wdBag7ibhU
management-server-tls-enabled=false
management-server-tls-keystore=
management-server-tls-keystore-password=
```

## Example `.env`

How the `.env` file in this app could look like with the above given properties.  

```sh
management_server_secret = "a325DpwUnti8XdqpAR0c9q3wHjdVj8wdBag7ibhU"
management_server_url = "ws://localhost:7218"
```

## Running
```sh
cd schema_fetch
cargo r
```