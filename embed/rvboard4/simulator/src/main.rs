use sdl2::pixels::Color;
use sdl2::rect::Rect;
use sdl2::event::Event;
use sdl2::keyboard::Keycode;
use std::time::Duration;

struct LedSimulator {
    led_on: bool,
    window_width: u32,
    window_height: u32,
}

impl LedSimulator {
    fn new() -> Self {
        LedSimulator {
            led_on: false,
            window_width: 640,
            window_height: 480,
        }
    }

    fn led_on(&mut self) {
        self.led_on = true;
    }

    fn led_off(&mut self) {
        self.led_on = false;
    }

    fn delay_ms(&self, ms: u32) {
        std::thread::sleep(Duration::from_millis(ms as u64));
    }

    fn draw_led(&self, canvas: &mut sdl2::render::Canvas<sdl2::video::Window>) {
        canvas.set_draw_color(Color::RGB(40, 40, 40));
        canvas.clear();

        let center_x = self.window_width as i32 / 2;
        let center_y = self.window_height as i32 / 2;
        let led_radius = 60i32;

        let (r, g, b) = if self.led_on {
            (255, 255, 0)
        } else {
            (80, 80, 80)
        };

        let outer_color = Color::RGB(r / 3, g / 3, b / 3);
        let inner_color = Color::RGB(r, g, b);

        for i in (10..=led_radius).rev() {
            let alpha = ((i as f32 / led_radius as f32) * 100.0) as u8;
            let color = if self.led_on {
                Color::RGB(
                    (r as f32 * (1.0 - alpha as f32 / 200.0)) as u8,
                    (g as f32 * (1.0 - alpha as f32 / 200.0)) as u8,
                    (b as f32 * (1.0 - alpha as f32 / 200.0)) as u8,
                )
            } else {
                Color::RGB(60, 60, 60)
            };

            let diameter = (i * 2) as u32;
            let offset = (led_radius - i) as i32;
            canvas.set_draw_color(color);
            let _ = canvas.fill_rect(Rect::new(
                center_x - led_radius + offset,
                center_y - led_radius + offset,
                diameter,
                diameter,
            ));
        }

        canvas.set_draw_color(outer_color);
        let _ = canvas.fill_rect(Rect::new(
            center_x - led_radius - 10,
            center_y - led_radius - 10,
            (led_radius + 10) as u32 * 2,
            (led_radius + 10) as u32 * 2,
        ));

        canvas.set_draw_color(inner_color);
        let _ = canvas.fill_rect(Rect::new(
            center_x - led_radius,
            center_y - led_radius,
            (led_radius * 2) as u32,
            (led_radius * 2) as u32,
        ));

        canvas.present();
    }

    fn run(&mut self) {
        let sdl_context = sdl2::init().unwrap();
        let video_subsystem = sdl_context.video().unwrap();

        let window = video_subsystem
            .window("RVBoard4 LED Simulator", self.window_width, self.window_height)
            .position_centered()
            .build()
            .unwrap();

        let mut canvas = window.into_canvas().build().unwrap();

        println!("RVBoard4 LED Simulator");
        println!("Press SPACE to toggle LED, Q to quit");
        println!("Running LED blink demo...\n");

        let mut event_pump = sdl_context.event_pump().unwrap();
        let mut manual_mode = false;

        loop {
            for event in event_pump.poll_iter() {
                match event {
                    Event::Quit { .. }
                    | Event::KeyDown {
                        keycode: Some(Keycode::Q),
                        ..
                    } => return,
                    Event::KeyDown {
                        keycode: Some(Keycode::Space),
                        ..
                    } => {
                        manual_mode = !manual_mode;
                        if manual_mode {
                            println!("Manual mode: Press SPACE to toggle LED");
                        } else {
                            println!("Auto mode: Running LED blink demo");
                        }
                    }
                    Event::KeyDown {
                        keycode: Some(Keycode::Left),
                        ..
                    } => {
                        self.led_off();
                        self.draw_led(&mut canvas);
                        println!("LED -> OFF");
                    }
                    Event::KeyDown {
                        keycode: Some(Keycode::Right),
                        ..
                    } => {
                        self.led_on();
                        self.draw_led(&mut canvas);
                        println!("LED -> ON");
                    }
                    _ => {}
                }
            }

            if !manual_mode {
                println!("LED ON");
                self.led_on();
                self.draw_led(&mut canvas);
                self.delay_ms(500);

                println!("LED OFF");
                self.led_off();
                self.draw_led(&mut canvas);
                self.delay_ms(500);
            } else {
                std::thread::sleep(Duration::from_millis(16));
            }
        }
    }
}

fn main() {
    let mut sim = LedSimulator::new();
    sim.run();
}