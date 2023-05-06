#![no_std]
#![no_main]
#![feature(type_alias_impl_trait)]

use defmt::*;
use embassy_executor::Spawner;
use embassy_futures::yield_now;
use embassy_net::{Stack, StackResources};
use embassy_net_w5500::*;
use embassy_rp::clocks::RoscRng;
use embassy_rp::gpio::{Input, Level, Output, Pull};
use embassy_rp::peripherals::{PIN_17, PIN_20, PIN_21, SPI0};
use embassy_rp::spi::{Async, Config as SpiConfig, Spi};
use embedded_hal_async::spi::ExclusiveDevice;
use embedded_io::asynch::Write;
use rand::RngCore;
use static_cell::StaticCell;
use {defmt_rtt as _, panic_probe as _};

macro_rules! singleton {
    ($val:expr) => {{
        type T = impl Sized;
        static STATIC_CELL: StaticCell<T> = StaticCell::new();
        let (x,) = STATIC_CELL.init(($val,));
        x
    }};
}

#[embassy_executor::task]
async fn ethernet_task(
    runner: Runner<
        'static,
        ExclusiveDevice<Spi<'static, SPI0, Async>, Output<'static, PIN_17>>,
        Input<'static, PIN_21>,
        Output<'static, PIN_20>,
    >,
) -> ! {
    runner.run().await
}

#[embassy_executor::task]
async fn net_task(stack: &'static Stack<Device<'static>>) -> ! {
    stack.run().await
}

#[embassy_executor::main]
async fn main(spawner: Spawner) {
    let p = embassy_rp::init(Default::default());
    let mut rng = RoscRng;
    let mut led = Output::new(p.PIN_25, Level::Low);

    let mut spi_cfg = SpiConfig::default();
    spi_cfg.frequency = 50_000_000;
    let (miso, mosi, clk) = (p.PIN_16, p.PIN_19, p.PIN_18);
    let spi = Spi::new(p.SPI0, clk, mosi, miso, p.DMA_CH0, p.DMA_CH1, spi_cfg);
    let cs = Output::new(p.PIN_17, Level::High);
    let w5500_int = Input::new(p.PIN_21, Pull::Up);
    let w5500_reset = Output::new(p.PIN_20, Level::High);

    let mac_addr = [0x02, 0x00, 0x00, 0x00, 0x00, 0x00];
    let state = singleton!(State::<8, 8>::new());
    let (device, runner) = embassy_net_w5500::new(
        mac_addr,
        state,
        ExclusiveDevice::new(spi, cs),
        w5500_int,
        w5500_reset,
    )
    .await;
    unwrap!(spawner.spawn(ethernet_task(runner)));

    // Generate random seed
    let seed = rng.next_u64();

    // Init network stack
    let stack = &*singleton!(Stack::new(
        device,
        embassy_net::Config::Dhcp(Default::default()),
        singleton!(StackResources::<2>::new()),
        seed
    ));

    // Launch network task
    unwrap!(spawner.spawn(net_task(&stack)));

    info!("Waiting for DHCP...");
    let cfg = wait_for_config(stack).await;
    let local_addr = cfg.address.address();
    info!("IP address: {:?}", local_addr);

    let mut rx_buffer = [0; 4096];
    let mut tx_buffer = [0; 4096];
    let mut buf = [0; 4096];

    loop {
        let mut socket = embassy_net::tcp::TcpSocket::new(stack, &mut rx_buffer, &mut tx_buffer);
        socket.set_timeout(Some(embassy_net::SmolDuration::from_secs(10)));

        led.set_low();
        info!("Listening on TCP:1234...");
        if let Err(e) = socket.accept(1234).await {
            warn!("accept error: {:?}", e);
            continue;
        }

        info!("Received connection from {:?}", socket.remote_endpoint());
        led.set_high();

        loop {
            let n = match socket.read(&mut buf).await {
                Ok(0) => {
                    warn!("read EOF");
                    break;
                }
                Ok(n) => n,
                Err(e) => {
                    warn!("{:?}", e);
                    break;
                }
            };

            info!("rxd {}", core::str::from_utf8(&buf[..n]).unwrap());

            match socket.write_all(&buf[..n]).await {
                Ok(()) => {}
                Err(e) => {
                    warn!("write error: {:?}", e);
                    break;
                }
            };
        }
    }
}

async fn wait_for_config(stack: &'static Stack<Device<'static>>) -> embassy_net::StaticConfig {
    loop {
        if let Some(config) = stack.config() {
            return config.clone();
        }
        yield_now().await;
    }
}
