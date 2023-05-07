# WIZnet W5500 embassy-net integration

[`embassy-net`](https://crates.io/crates/embassy-net) integration for the WIZnet W5500 SPI ethernet chip, operating in MACRAW mode.

Supports any SPI driver implementing [`embedded-hal-async`](https://crates.io/crates/embedded-hal-async)

See [`examples`](https://github.com/kalkyl/embassy-net-w5500/tree/main/examples) directory for a simple TCP echo server example made for the rp2040 `WIZnet W5500-EVB-Pico` evaluation board.