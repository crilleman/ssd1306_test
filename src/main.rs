#![no_main]
#![no_std]

use panic_rtt_target as _;

#[rtic::app(device = stm32l4xx_hal::pac, dispatchers = [ADC1], peripherals = true)]
mod app {

    // Crates used
    use core::fmt::Write;
    use dwt_systick_monotonic::DwtSystick;
    use rtic::time::duration::*;
    use rtt_target::{rprintln, rtt_init_print};
    use stm32l4xx_hal::{
        device::SPI1,
        gpio::{
            gpiob::PB13, gpioc::PC13, Alternate, Edge, ExtiPin, Floating, Input, Output, PushPull,
            PA5, PA6, PA7, PB1,
        },
        hal::spi::{Mode, Phase, Polarity},
        prelude::*,
        serial::{self, Config, Serial},
        spi::*,
    };

    //crates to use
    use display_interface_spi::SPIInterface;
    use embedded_graphics::{
        image::{Image, ImageRaw},
        pixelcolor::BinaryColor,
        prelude::*,
    };
    use ssd1306::{prelude::*, Builder, I2CDIBuilder};

    #[monotonic(binds = SysTick, default = true)]
    type DwtMono = DwtSystick<80_000_000>;

    #[resources]
    struct Resources {
        led: PB13<Output<PushPull>>,
        btn: PC13<Input<Floating>>,
        // spi: stm32l4xx_hal::spi::Spi<
        //     stm32l4xx_hal::pac::SPI1,
        //     (
        //         PA5<
        //             Alternate<
        //                 stm32l4xx_hal::gpio::AF5,
        //                 stm32l4xx_hal::gpio::Input<stm32l4xx_hal::gpio::Floating>,
        //             >,
        //         >,
        //         PA6<
        //             Alternate<
        //                 stm32l4xx_hal::gpio::AF5,
        //                 stm32l4xx_hal::gpio::Input<stm32l4xx_hal::gpio::Floating>,
        //             >,
        //         >,
        //         PA7<
        //             Alternate<
        //                 stm32l4xx_hal::gpio::AF5,
        //                 stm32l4xx_hal::gpio::Input<stm32l4xx_hal::gpio::Floating>,
        //             >,
        //         >,
        //     ),
        // >,

        // dc: PB1<Output<PushPull>>,
    }

    /// SPI mode
    pub const MODE: Mode = Mode {
        phase: Phase::CaptureOnFirstTransition,
        polarity: Polarity::IdleLow,
    };

    #[init]
    fn init(cx: init::Context) -> (init::LateResources, init::Monotonics) {
        //inits...

        let mut flash = cx.device.FLASH.constrain();
        let mut rcc = cx.device.RCC.constrain();
        let mut pwr = cx.device.PWR.constrain(&mut rcc.apb1r1);
        let mut dcb = cx.core.DCB;
        let dwt = cx.core.DWT;
        let systick = cx.core.SYST;

        rtt_init_print!(NoBlockSkip, 4096);

        let mut gpiob = cx.device.GPIOB.split(&mut rcc.ahb2);
        //LED, PB13
        let mut led = gpiob
            .pb13
            .into_push_pull_output(&mut gpiob.moder, &mut gpiob.otyper);

        led.set_high().ok();

        rprintln!("pre init");

        // Initialize the clocks
        let clocks = rcc.cfgr.sysclk(80.mhz()).freeze(&mut flash.acr, &mut pwr);

        // Setup the monotonic timer
        let mono2 = DwtSystick::new(&mut dcb, dwt, systick, clocks.sysclk().0);

        //______________________________GPIO________________________________//

        let mut gpioa = cx.device.GPIOA.split(&mut rcc.ahb2);

        let mut gpioc = cx.device.GPIOC.split(&mut rcc.ahb2);

        //BTN, PC13
        let mut syscfg = cx.device.SYSCFG;
        let mut exti = cx.device.EXTI;

        let mut btn = gpioc
            .pc13
            .into_floating_input(&mut gpioc.moder, &mut gpioc.pupdr);
        btn.enable_interrupt(&mut exti);
        btn.make_interrupt_source(&mut syscfg, &mut rcc.apb2);
        btn.trigger_on_edge(&mut exti, Edge::Rising);
        btn.clear_interrupt_pending_bit();

        //______________________________SPI_________________________________//

        let sck = gpioa.pa5.into_af5(&mut gpioa.moder, &mut gpioa.afrl);
        let miso = gpioa.pa6.into_af5(&mut gpioa.moder, &mut gpioa.afrl);
        let mosi = gpioa.pa7.into_af5(&mut gpioa.moder, &mut gpioa.afrl);

        let mut cs = gpiob
            .pb1
            .into_push_pull_output(&mut gpiob.moder, &mut gpiob.otyper);
        cs.set_high().ok();

        let mut dc = gpiob
            .pb2
            .into_push_pull_output(&mut gpiob.moder, &mut gpiob.otyper);
        dc.set_low().ok();

        // let mut spi = Spi::spi1(
        //     cx.device.SPI1,
        //     (sck, miso, mosi),
        //     MODE,
        //     100.khz(),
        //     clocks,
        //     &mut rcc.apb2,
        // );

        // let interface = display_interface_spi::SPIInterface::new(spi, dc, cs);

        // // let mut disp: GraphicsMode<_, _> = Builder::new().connect(interface).into();
        // let mut disp: TerminalMode<_, _> = Builder::new().connect(interface).into();

        // disp.init().unwrap();
        // let _ = disp.clear();

        // for c in 65..91 {
        //     let _ = disp.write_str(unsafe { core::str::from_utf8_unchecked(&[c]) });
        // }

        // let raw: ImageRaw<BinaryColor> = ImageRaw::new(include_bytes!("./dvd.bmp"), 64, 64);

        // let im = Image::new(&raw, Point::new(32, 0));

        // im.draw(&mut disp).unwrap();

        //__________________________________________________________________//

        (init::LateResources { led, btn }, init::Monotonics(mono2))
    }

    // #[task(resources = [spi, dc])]
    // fn init_display(cx: init_display::Context) {
    //     let mut spi = cx.resources.spi;
    //     let mut dc = cx.resources.dc;

    //     (spi, dc).lock(|spi, dc| {
    //         // Init OLED
    //         let interface = display_interface_spi::SPIInterfaceNoCS::new(spi, dc);

    //         //let mut disp: GraphicsMode<_, _> = Builder::new().connect(interface).into();

    //         //disp.init().unwrap();
    //     });
    // }

    // Interrupt funtion on external button.
    #[task(binds = EXTI15_10,
           resources = [btn, led])]
    fn button_event(cx: button_event::Context) {
        //Extract resources
        let mut button = cx.resources.btn; //Button resource

        let cnt = unsafe {
            static mut CNT: u32 = 1; //Cannot be dereferenced

            &mut CNT
        };
        rprintln!("External interrupt {}!", *cnt);
        *cnt += 1;

        //blink LED
        toggle_led::spawn().ok();

        // Remeber to clear the interrupt!
        // RTIC does not do this for you.
        button.lock(|button| button.clear_interrupt_pending_bit());
    }

    // Function to toggle the LED.
    #[task(resources = [led])]
    fn toggle_led(cx: toggle_led::Context) {
        let mut led = cx.resources.led; //LED resource

        unsafe {
            static mut FLAG: u32 = 1; //LED HIGH/LOW flag
            if (FLAG % 2) == 0 {
                led.lock(|led| led.set_high().ok());
            } else {
                led.lock(|led| led.set_low().ok());
            }
            //rprintln!("Toggle led {}!", FLAG);
            FLAG = FLAG + 1;
        }

        // Can not spawn itself indefinitly AND be called from button_event, use timer funtion instead.
        // toggle_led::spawn_after(Seconds(3_u32)).unwrap();
    }

    #[idle]
    fn idle(_: idle::Context) -> ! {
        rprintln!("");
        rprintln!("idle");

        loop {
            cortex_m::asm::nop();
        }
    }
}
