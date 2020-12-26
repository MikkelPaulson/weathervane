use rppal::{gpio, spi};
use std::thread;
use std::time::Duration;

fn main() {
    let mut display = Display::new(DisplayMode::Grayscale);
    println!("display.init();");
    display.init();
    println!("display.clear();");
    display.clear();

    println!("display.checkerboard();");
    display.checkerboard();

    println!("sleeping");
    thread::sleep(Duration::from_secs(10));

    println!("display.sleep();");
    display.init();
    display.clear();
    display.sleep();
}

struct Display {
    mode: DisplayMode,
    spi: spi::Spi,
    pin_dc: gpio::OutputPin,
    pin_rst: gpio::OutputPin,
    pin_busy: gpio::InputPin,
}

impl Display {
    const PIN_DC: u8 = 25; // Data/command pin (high = data, low = command)
    const PIN_RST: u8 = 17; // External reset pin (low = reset)
    const PIN_BUSY: u8 = 24; // Busy output pin (low = busy)

    const DISPLAY_WIDTH: usize = 280;
    const DISPLAY_HEIGHT: usize = 480;

    pub fn new(mode: DisplayMode) -> Self {
        let gpio = gpio::Gpio::new().expect("Unable to connect to GPIO.");

        Self {
            mode,
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
        }
    }

    pub fn reset(&mut self) {
        self.wait_for_busy();
        self.pin_rst.set_high();
        thread::sleep(Duration::from_millis(30));
        self.pin_rst.set_low();
        thread::sleep(Duration::from_millis(3));
        self.pin_rst.set_high();
        thread::sleep(Duration::from_millis(30));
    }

    pub fn wait_for_busy(&mut self) {
        if self.pin_busy.is_high() {
            print!("Waiting for device...");
            while self.pin_busy.is_high() {
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
        self.pin_dc.set_low();
        self.spi
            .write(&[command])
            .expect("Unable to write command.");
    }

    pub fn send_data(&mut self, data: &[u8]) {
        self.pin_dc.set_high();
        for chunk in data[..].chunks(4096) {
            self.spi.write(&chunk).expect("Unable to write data.");
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
            match self.mode {
                DisplayMode::BlackAndWhite => {
                    &[0x00, 0xFF, 0xFF, 0xFF, 0xFF, 0x4F, 0xFF, 0xFF, 0xFF, 0xFF]
                }
                DisplayMode::Grayscale => {
                    &[0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00]
                }
            },
        );

        // setting X direction start/end position of RAM
        self.send(0x44, &[0x00, 0x00, 0x17, 0x01]);

        // setting Y direction start/end position of RAM
        self.send(0x45, &[0x00, 0x00, 0xDF, 0x01]);

        // Display Update Control 2
        self.send(0x22, &[0xCF]);
    }

    pub fn clear(&mut self) {
        if self.mode == DisplayMode::Grayscale {
            self.send(0x49, &[0x00]);
        }

        self.send(0x4E, &[0x00, 0x00]);
        self.send(0x4F, &[0x00, 0x00]);
        self.send(
            0x24,
            &[0xFF].repeat(Self::DISPLAY_WIDTH / 8 * Self::DISPLAY_HEIGHT),
        );

        if self.mode == DisplayMode::Grayscale {
            self.send(0x4E, &[0x00, 0x00]);
            self.send(0x4F, &[0x00, 0x00]);
            self.send(
                0x26,
                &[0xFF].repeat(Self::DISPLAY_WIDTH / 8 * Self::DISPLAY_HEIGHT),
            );
        }

        self.load_look_up_table();

        if self.mode == DisplayMode::Grayscale {
            self.send(0x22, &[0xC7]);
        }

        self.send(0x20, &[]);
        self.wait_for_busy();
    }

    pub fn sleep(&mut self) {
        self.send(0x50, &[0xF7]);
        self.send(0x02, &[]);
        self.send(0x07, &[0xA5]);
    }

    pub fn checkerboard(&mut self) {
        self.send(0x4E, &[0x00, 0x00]);
        self.send(0x4F, &[0x00, 0x00]);

        for y in 0..Self::DISPLAY_HEIGHT {
            self.send(
                0x24,
                &[0xFF, 0x00].repeat(Self::DISPLAY_WIDTH / 16 + 1)[0..Self::DISPLAY_WIDTH / 8],
            );
            self.send(
                0x26,
                &[0xFF, 0x00].repeat(Self::DISPLAY_WIDTH / 16 + 1)[0..Self::DISPLAY_WIDTH / 8],
            );
        }

        self.load_look_up_table();

        self.send(0x20, &[]);
        self.wait_for_busy();
    }

    fn load_look_up_table(&mut self) {
        self.send(
            0x32,
            match self.mode {
                DisplayMode::BlackAndWhite => &[
                    0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, //1
                    0x01, 0x2A, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, //2
                    0x0A, 0x55, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, //3
                    0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, //4
                    0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, //5
                    0x00, 0x00, 0x05, 0x05, 0x00, 0x05, 0x03, 0x05, 0x05, 0x00, //6
                    0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, //7
                    0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, //8
                    0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, //9
                    0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, //10
                    0x22, 0x22, 0x22, 0x22, 0x22,
                ],
                DisplayMode::Grayscale => &[
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
            },
        );
    }
}

#[derive(PartialEq)]
enum DisplayMode {
    BlackAndWhite,
    Grayscale,
}
