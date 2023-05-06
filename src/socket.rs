use crate::device::RegisterBlock;
use crate::spi::SpiInterface;
use embedded_hal_async::spi::SpiDevice;

pub const MODE: u16 = 0x00;
pub const COMMAND: u16 = 0x01;
pub const RXBUF_SIZE: u16 = 0x1E;
pub const TXBUF_SIZE: u16 = 0x1F;
pub const TX_FREE_SIZE: u16 = 0x20;
pub const TX_DATA_WRITE_PTR: u16 = 0x24;
pub const RECVD_SIZE: u16 = 0x26;
pub const RX_DATA_READ_PTR: u16 = 0x28;
pub const SOCKET_INTR_MASK: u16 = 0x2C;

#[repr(u8)]
pub enum Command {
    Open = 0x01,
    Send = 0x20,
    Receive = 0x40,
}

pub const INTR: u16 = 0x02;
#[repr(u8)]
pub enum Interrupt {
    SendOk = 0b010000_u8,
    Receive = 0b00100_u8,
}

pub async fn reset_interrupt<SPI: SpiDevice>(
    bus: &mut SpiInterface<SPI>,
    code: Interrupt,
) -> Result<(), SPI::Error> {
    let data = [code as u8];
    bus.write_frame(RegisterBlock::Socket0, INTR, &data).await
}

pub async fn is_interrupt<SPI: SpiDevice>(
    bus: &mut SpiInterface<SPI>,
    code: Interrupt,
) -> Result<bool, SPI::Error> {
    let mut data = [0u8];
    bus.read_frame(RegisterBlock::Socket0, INTR, &mut data)
        .await?;
    Ok(data[0] & code as u8 != 0)
}

pub async fn get_tx_write_ptr<SPI: SpiDevice>(
    bus: &mut SpiInterface<SPI>,
) -> Result<u16, SPI::Error> {
    let mut data = [0u8; 2];
    bus.read_frame(RegisterBlock::Socket0, TX_DATA_WRITE_PTR, &mut data)
        .await?;
    Ok(u16::from_be_bytes(data))
}

pub async fn set_tx_write_ptr<SPI: SpiDevice>(
    bus: &mut SpiInterface<SPI>,
    ptr: u16,
) -> Result<(), SPI::Error> {
    let data = ptr.to_be_bytes();
    bus.write_frame(RegisterBlock::Socket0, TX_DATA_WRITE_PTR, &data)
        .await
}

pub async fn get_rx_read_ptr<SPI: SpiDevice>(
    bus: &mut SpiInterface<SPI>,
) -> Result<u16, SPI::Error> {
    let mut data = [0u8; 2];
    bus.read_frame(RegisterBlock::Socket0, RX_DATA_READ_PTR, &mut data)
        .await?;
    Ok(u16::from_be_bytes(data))
}

pub async fn set_rx_read_ptr<SPI: SpiDevice>(
    bus: &mut SpiInterface<SPI>,
    ptr: u16,
) -> Result<(), SPI::Error> {
    let data = ptr.to_be_bytes();
    bus.write_frame(RegisterBlock::Socket0, RX_DATA_READ_PTR, &data)
        .await
}

pub async fn command<SPI: SpiDevice>(
    bus: &mut SpiInterface<SPI>,
    command: Command,
) -> Result<(), SPI::Error> {
    let data = [command as u8];
    bus.write_frame(RegisterBlock::Socket0, COMMAND, &data)
        .await
}

pub async fn get_recv_size<SPI: SpiDevice>(bus: &mut SpiInterface<SPI>) -> Result<u16, SPI::Error> {
    loop {
        // Section 4.2 of datasheet, Sn_TX_FSR address docs indicate that read must be repeated until two sequential reads are stable
        let mut sample_0 = [0u8; 2];
        bus.read_frame(RegisterBlock::Socket0, RECVD_SIZE, &mut sample_0)
            .await?;
        let mut sample_1 = [0u8; 2];
        bus.read_frame(RegisterBlock::Socket0, RECVD_SIZE, &mut sample_1)
            .await?;
        if sample_0 == sample_1 {
            break Ok(u16::from_be_bytes(sample_0));
        }
    }
}

pub async fn get_tx_free_size<SPI: SpiDevice>(
    bus: &mut SpiInterface<SPI>,
) -> Result<u16, SPI::Error> {
    let mut data = [0; 2];
    bus.read_frame(RegisterBlock::Socket0, TX_FREE_SIZE, &mut data)
        .await?;
    Ok(u16::from_be_bytes(data))
}
