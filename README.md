# Restaurant Order Service
Created and tested on Ubuntu 22.04

## Installation
`cargo build --release`

## Config
Check config/restaurant.toml

## Required dependency
This app uses SQLite3. So you need to install it on your system first.
For Debian
`apt install libsqlite3-dev`

## Try it out
1. In an instance of a shell. `cargo run --release`
    * OR simply `docker-compose up` for those have docker and docker-compose setup.
2. In *another* instance of a shell. `cargo run --release -p client -- [url] [order string]`

### Example
`cargo run --release -p client -- http://localhost:3000/order "new order for table 1: yakisoba * 2"`

other order string examples:
* `cancel for table 1: yakisoba * 1`
* `check for table 1`
* `check for table 1: yakisoba`

See [TORO](toro/README.md) for order string format.
