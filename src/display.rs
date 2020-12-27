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

    pub fn reset(&mut self) {
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

    pub fn wait_for_busy(&mut self) {
        if self.hardware_interface.get_level(GpioInputPin::Busy) == gpio::Level::High {
            print!("Waiting for device...");
            while self.hardware_interface.get_level(GpioInputPin::Busy) == gpio::Level::High {
                thread::sleep(Duration::from_millis(200));
            }
            println!("Done");
        }
    }

    pub fn send(&mut self, command: u8, data: &[u8]) {
        self.send_command(command);
        if !data.is_empty() {
            self.send_data(data);
        }
    }

    pub fn send_command(&mut self, command: u8) {
        self.hardware_interface
            .set_level(GpioOutputPin::DataCommand, gpio::Level::Low);
        self.hardware_interface
            .write_to_spi(&[command])
            .expect("Unable to write command.");
    }

    pub fn send_data(&mut self, data: &[u8]) {
        self.hardware_interface
            .set_level(GpioOutputPin::DataCommand, gpio::Level::High);
        for chunk in data[..].chunks(4096) {
            self.hardware_interface
                .write_to_spi(&chunk)
                .expect("Unable to write data.");
        }
    }

    pub fn init(&mut self) {
        self.reset();

        self.send(0x12, &[]);
        thread::sleep(Duration::from_millis(300));

        self.send(0x46, &[0xF7]);
        self.wait_for_busy();
        self.send(0x47, &[0xF7]);
        self.wait_for_busy();

        // setting gate number
        self.send(0x01, &[0xDF, 0x01, 0x00]);

        // set gate voltage
        self.send(0x03, &[0x00]);

        // set source voltage
        self.send(0x04, &[0x41, 0xA8, 0x32]);

        // set data entry sequence
        self.send(0x11, &[0x03]);

        // set border
        self.send(0x3C, &[0x00]);

        // set booster strength
        self.send(0x0C, &[0xAE, 0xC7, 0xC3, 0xC0, 0xC0]);

        // set internal sensor on
        self.send(0x18, &[0x80]);

        // set vcom value
        self.send(0x2C, &[0x44]);

        // set display option, these setting turn on previous function
        self.send(
            0x37,
            &[0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00],
        );

        // setting X direction start/end position of RAM
        self.send(0x44, &[0x00, 0x00, 0x17, 0x01]);

        // setting Y direction start/end position of RAM
        self.send(0x45, &[0x00, 0x00, 0xDF, 0x01]);

        // Display Update Control 2
        self.send(0x22, &[0xCF]);
    }

    pub fn clear(&mut self) {
        self.send(0x49, &[0x00]);

        self.draw(
            &[0xFF].repeat(Self::DISPLAY_WIDTH / 8 * Self::DISPLAY_HEIGHT),
            &[0xFF].repeat(Self::DISPLAY_WIDTH / 8 * Self::DISPLAY_HEIGHT),
        );
    }

    pub fn sleep(&mut self) {
        self.send(0x50, &[0xF7]);
        self.send(0x02, &[]);
        self.send(0x07, &[0xA5]);

        self.hardware_interface
            .set_level(GpioOutputPin::DataCommand, gpio::Level::Low);
        self.hardware_interface
            .set_level(GpioOutputPin::Reset, gpio::Level::Low);
    }

    pub fn draw(&mut self, register1: &[u8], register2: &[u8]) {
        self.send(0x4E, &[0x00, 0x00]);
        self.send(0x4F, &[0x00, 0x00]);

        self.send(0x24, &register1);
        self.send(0x26, &register2);

        self.load_look_up_table();
        self.send(0x22, &[0xC7]);
        self.send(0x20, &[]);
        self.wait_for_busy();
    }

    pub fn checkerboard(&mut self) {
        let mut register = Vec::with_capacity(Self::DISPLAY_HEIGHT * Self::DISPLAY_WIDTH / 8);

        for y in 0..Self::DISPLAY_HEIGHT {
            let sequence = if y / 8 % 2 == 0 {
                [0xFF, 0x00]
            } else {
                [0x00, 0xFF]
            };

            register.extend_from_slice(
                &sequence.repeat(Self::DISPLAY_WIDTH / 16 + 1)[0..Self::DISPLAY_WIDTH / 8],
            );
        }

        self.draw(&register[..], &register[..]);
    }

    fn load_look_up_table(&mut self) {
        self.send(
            0x32,
            &[
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
            ],
        );
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

    /// 0x21 "Display Update Control 1" in documentation
    SetRegisterMode,

    /// 0x22 "Display Update Control 2" in documentation
    UpdateSequence,

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
