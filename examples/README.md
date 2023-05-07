# Examples for the rp2040 `WIZnet W5500-EVB-Pico` board

Examples are written for the [`WIZnet W5500-EVB-Pico`](https://www.wiznet.io/product-item/w5500-evb-pico/) board.

## Prerequisites
```bash
cargo install probe-rs-cli
```

## Running the TCP server example
```bash
cargo run --bin tcp --release
```
This example implements a TCP echo server on port 1234 and using DHCP.
Send it some data, you should see it echoed back and printed in the console.
