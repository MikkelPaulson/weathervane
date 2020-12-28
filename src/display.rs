use rppal::{gpio, spi};
use std::thread;
use std::time::Duration;

pub struct Display {
    hardware_interface: Box<dyn HardwareInterface>,
}

impl Display {
    const PIN_DC: u8 = 25; // Data/command pin (high = data, low = command)
    const PIN_RST: u8 = 17; // External reset pin (low = reset)
    const PIN_BUSY: u8 = 24; // Busy output pin (low = busy)

    pub const DISPLAY_WIDTH: usize = 280;
    pub const DISPLAY_HEIGHT: usize = 480;

    pub fn new() -> Self {
        let gpio = gpio::Gpio::new().expect("Unable to connect to GPIO.");

        Self {
            hardware_interface: Box::new(DisplayHardwareInterface {
                spi: spi::Spi::new(
                    spi::Bus::Spi0,
                    spi::SlaveSelect::Ss0,
                    10_000_000, // 10 MHz = 100 ns
                    spi::Mode::Mode0,
                )
                .expect("Unable to initialize SPI connection."),
                pin_dc: gpio
                    .get(Self::PIN_DC)
                    .expect("Unable to acquire data/command pin.")
                    .into_output(),
                pin_rst: gpio
                    .get(Self::PIN_RST)
                    .expect("Unable to acquire reset pin.")
                    .into_output(),
                pin_busy: gpio
                    .get(Self::PIN_BUSY)
                    .expect("Unable to acquire busy pin.")
                    .into_input(),
            }),
        }
    }

    pub fn init(&mut self) -> Result<(), &'static str> {
        self.reset();

        self.run(Command::Unknown0x12)?;
        thread::sleep(Duration::from_millis(300));

        self.run(Command::Unknown0x46)?;
        self.run(Command::Unknown0x47)?;

        self.run(Command::SetGateNumber)?;

        self.run(Command::SetGateVoltage)?;

        self.run(Command::SetSourceVoltage)?;

        self.run(Command::SetDataEntrySequence)?;
        self.run(Command::SetBorder)?;
        self.run(Command::SetBoosterStrength)?;
        self.run(Command::SetInternalSensorOn)?;
        self.run(Command::SetVComValue)?;
        self.run(Command::SetDisplayOption(&[
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
        ]))?;

        self.run(Command::SetXRamPosition(&[0x00, 0x00, 0x17, 0x01]))?;
        self.run(Command::SetYRamPosition(&[0x00, 0x00, 0xDF, 0x01]))?;

        self.run(Command::UpdateSequence(&[0xCF]))?;

        Ok(())
    }

    pub fn clear(&mut self) -> Result<(), &'static str> {
        self.draw(
            &[0xFF].repeat(Self::DISPLAY_WIDTH / 8 * Self::DISPLAY_HEIGHT),
            &[0xFF].repeat(Self::DISPLAY_WIDTH / 8 * Self::DISPLAY_HEIGHT),
        )
    }

    pub fn sleep(&mut self) -> Result<(), &'static str> {
        self.run(Command::Unknown0x50)?;
        self.run(Command::PowerOff)?;
        self.run(Command::Sleep)?;

        self.hardware_interface
            .set_level(GpioOutputPin::DataCommand, gpio::Level::Low);
        self.hardware_interface
            .set_level(GpioOutputPin::Reset, gpio::Level::Low);

        Ok(())
    }

    pub fn render<F: FnOnce(&mut piet_cairo::CairoRenderContext)>(&mut self, f: F) {
        let mut device = piet_common::Device::new().unwrap();
        let mut bitmap_target = device
            .bitmap_target(Self::DISPLAY_WIDTH, Self::DISPLAY_HEIGHT, 1.)
            .unwrap();

        let mut render_context = bitmap_target.render_context();
        f(&mut render_context);

        let (channel1, channel2): (Vec<u8>, Vec<u8>) = bitmap_target
            .to_image_buf(piet_common::ImageFormat::RgbaPremul)
            .unwrap()
            .raw_pixels()
            .chunks(8 * 4) // 8 pixels @ RGBA
            .map(|chunk: &[u8]| {
                chunk
                    .chunks_exact(4) // chunk by pixel (RGBA)
                    .enumerate()
                    .map(|(index, pixel)| {
                        // For now, we just render the alpha channel.
                        let color = (pixel[3] as f64 / 255. * 3.).round() as u8;
                        let result = (
                            if color & 0x01 == 0x01 {
                                0
                            } else {
                                0x80 >> index
                            },
                            if color & 0x02 == 0x02 {
                                0
                            } else {
                                0x80 >> index
                            },
                        );
                        result
                    })
                    .fold((0, 0), |a, b| (a.0 | b.0, a.1 | b.1))
            })
            .unzip();

        self.draw(&channel1, &channel2).unwrap();
    }

    pub fn draw(&mut self, register1: &[u8], register2: &[u8]) -> Result<(), &'static str> {
        self.run(Command::Unknown0x49)?;

        self.run(Command::Unknown0x4E)?;
        self.run(Command::Unknown0x4F)?;
        self.run(Command::WriteRegister1(&register1))?;

        self.run(Command::Unknown0x4E)?;
        self.run(Command::Unknown0x4F)?;
        self.run(Command::WriteRegister2(&register2))?;

        self.load_look_up_table()?;

        self.run(Command::UpdateSequence(&[0xCF]))?;
        self.run(Command::Display)?;

        Ok(())
    }

    fn reset(&mut self) {
        self.hardware_interface
            .set_level(GpioOutputPin::Reset, gpio::Level::High);
        thread::sleep(Duration::from_millis(30));
        self.hardware_interface
            .set_level(GpioOutputPin::Reset, gpio::Level::Low);
        thread::sleep(Duration::from_millis(3));
        self.hardware_interface
            .set_level(GpioOutputPin::Reset, gpio::Level::High);
        thread::sleep(Duration::from_millis(30));
    }

    fn load_look_up_table(&mut self) -> Result<(), &'static str> {
        self.run(Command::WriteLookUpTableRegister(&[
            0x2A, 0x06, 0x15, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, //1
            0x28, 0x06, 0x14, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, //2
            0x20, 0x06, 0x10, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, //3
            0x14, 0x06, 0x28, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, //4
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, //5
            0x00, 0x02, 0x02, 0x0A, 0x00, 0x00, 0x00, 0x08, 0x08, 0x02, //6
            0x00, 0x02, 0x02, 0x0A, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, //7
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, //8
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, //9
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, //10
            0x22, 0x22, 0x22, 0x22, 0x22,
        ]))
    }

    fn run(&mut self, command: Command) -> Result<(), &'static str> {
        let (command_byte, data_bytes) = command.get_bytes();

        self.hardware_interface
            .set_level(GpioOutputPin::DataCommand, gpio::Level::Low);
        self.hardware_interface
            .write_to_spi(&[command_byte])
            .map_err(|_| "Unable to write command.")?;

        if !data_bytes.is_empty() {
            self.hardware_interface
                .set_level(GpioOutputPin::DataCommand, gpio::Level::High);
            for chunk in data_bytes[..].chunks(4096) {
                self.hardware_interface
                    .write_to_spi(&chunk)
                    .map_err(|_| "Unable to write data.")?;
            }
        }

        if command.is_blocking()
            && self.hardware_interface.get_level(GpioInputPin::Busy) == gpio::Level::High
        {
            print!("Waiting for device...");
            while self.hardware_interface.get_level(GpioInputPin::Busy) == gpio::Level::High {
                thread::sleep(Duration::from_millis(200));
            }
            println!("Done");
        }

        Ok(())
    }
}

enum Command<'a> {
    /// 0x01 "setting gaet number" in example code
    SetGateNumber,

    /// 0x02 "power off" in example code
    PowerOff,

    /// 0x03 "Gate Driving voltage Control" in documentation
    SetGateVoltage,

    /// 0x04 "Source Driving voltage Control" in documentation
    SetSourceVoltage,

    /// 0x07 "deep sleep" in example code
    Sleep,

    /// 0x0C "set booster strength" in example code
    SetBoosterStrength,

    /// 0x11 "set data entry sequence" in example code
    SetDataEntrySequence,

    /// 0x12 Not documented
    Unknown0x12,

    /// 0x18 "set internal sensor on" in example code
    SetInternalSensorOn,

    /// 0x20 "Master Activation" in documentation (activate display update sequence)
    Display,

    /// 0x22 "Display Update Control 2" in documentation
    UpdateSequence(&'a [u8]),

    /// 0x24 Write RAM (register 1)
    WriteRegister1(&'a [u8]),

    /// 0x26 Write RAM (register 2)
    WriteRegister2(&'a [u8]),

    /// 0x2C "set vcom value" in example code
    SetVComValue,

    /// 0x32 "Write LUT register" in documentation
    WriteLookUpTableRegister(&'a [u8]),

    /// 0x37 "set display option, these setting turn on previous function" in example code
    SetDisplayOption(&'a [u8]),

    /// 0x3C "set border" in example code
    SetBorder,

    /// 0x44 "set X direction start/end position of RAM" in example code
    SetXRamPosition(&'a [u8; 4]),

    /// 0x45 "set Y direction start/end position of RAM" in example code
    SetYRamPosition(&'a [u8; 4]),

    /// 0x46 Not documented
    Unknown0x46,

    /// 0x47 Not documented
    Unknown0x47,

    /// 0x49 Not documented
    Unknown0x49,

    /// 0x4E Not documented
    Unknown0x4E,

    /// 0x4F Not documented
    Unknown0x4F,

    /// 0x50 Not documented
    Unknown0x50,
}

impl<'a> Command<'a> {
    fn get_bytes(&self) -> (u8, &[u8]) {
        match self {
            Self::SetGateNumber => (0x01, &[0xDF, 0x01, 0x00]),
            Self::PowerOff => (0x02, &[]),
            Self::SetGateVoltage => (0x03, &[0x00]),
            Self::SetSourceVoltage => (0x04, &[0x41, 0xA8, 0x32]),
            Self::Sleep => (0x07, &[0xA5]),
            Self::SetBoosterStrength => (0x0C, &[0xAE, 0xC7, 0xC3, 0xC0, 0xC0]),
            Self::SetDataEntrySequence => (0x11, &[0x03]),
            Self::Unknown0x12 => (0x12, &[]),
            Self::SetInternalSensorOn => (0x18, &[0x80]),
            Self::Display => (0x20, &[]),
            Self::UpdateSequence(data) => (0x22, data),
            Self::WriteRegister1(data) => (0x24, data),
            Self::WriteRegister2(data) => (0x26, data),
            Self::SetVComValue => (0x2C, &[0x44]),
            Self::WriteLookUpTableRegister(data) => (0x32, data),
            Self::SetDisplayOption(data) => (0x37, data),
            Self::SetBorder => (0x3C, &[0x00]),
            Self::SetXRamPosition(data) => (0x44, &data[..]),
            Self::SetYRamPosition(data) => (0x45, &data[..]),
            Self::Unknown0x46 => (0x46, &[0xF7]),
            Self::Unknown0x47 => (0x47, &[0xF7]),
            Self::Unknown0x49 => (0x49, &[0x00]),
            Self::Unknown0x4E => (0x4E, &[0x00, 0x00]),
            Self::Unknown0x4F => (0x4F, &[0x00, 0x00]),
            Self::Unknown0x50 => (0x50, &[0xF7]),
        }
    }

    fn is_blocking(&self) -> bool {
        match self {
            Self::Unknown0x46 | Self::Unknown0x47 | Self::Display => true,
            _ => false,
        }
    }
}

struct DisplayHardwareInterface {
    spi: spi::Spi,
    pin_dc: gpio::OutputPin,
    pin_rst: gpio::OutputPin,
    pin_busy: gpio::InputPin,
}

impl HardwareInterface for DisplayHardwareInterface {
    fn set_level(&mut self, pin: GpioOutputPin, level: gpio::Level) {
        match pin {
            GpioOutputPin::Reset => &mut self.pin_rst,
            GpioOutputPin::DataCommand => &mut self.pin_dc,
        }
        .write(level)
    }

    fn get_level(&self, pin: GpioInputPin) -> gpio::Level {
        match pin {
            GpioInputPin::Busy => &self.pin_busy,
        }
        .read()
    }

    fn write_to_spi(&mut self, buffer: &[u8]) -> Result<usize, spi::Error> {
        self.spi.write(buffer)
    }
}

trait HardwareInterface {
    fn set_level(&mut self, pin: GpioOutputPin, level: gpio::Level);

    fn get_level(&self, pin: GpioInputPin) -> gpio::Level;

    fn write_to_spi(&mut self, data: &[u8]) -> Result<usize, spi::Error>;
}

#[derive(Copy, Clone, Debug, PartialEq)]
enum GpioOutputPin {
    Reset,
    DataCommand,
}

#[derive(Copy, Clone, Debug, PartialEq)]
enum GpioInputPin {
    Busy,
}
