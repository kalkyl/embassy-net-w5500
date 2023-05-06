# Examples for the rp2040 `WIZnet W5500-EVB-Pico` board

## Running the example

- `$ cargo install probe-rs-cli`
- `$ cargo run --release`

The example implements a TCP echo server on port 1234 and using DHCP.
Send it some data, you should see it echoed back and printed in the console.
