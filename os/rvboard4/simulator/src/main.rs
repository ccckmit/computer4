use sdl2::pixels::Color;
use sdl2::rect::Rect;
use sdl2::event::Event;
use sdl2::keyboard::Keycode;
use std::time::Duration;

struct LedSimulator {
    led_on: bool,
}

impl LedSimulator {
    fn new() -> Self {
        LedSimulator { led_on: false }
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

    fn draw_breadboard(&self, canvas: &mut sdl2::render::Canvas<sdl2::video::Window>) {
        let bx = 50i32;
        let by = 180i32;
        let bw = 540u32;
        let bh = 160u32;

        canvas.set_draw_color(Color::RGB(240, 240, 240));
        canvas.clear();

        canvas.set_draw_color(Color::RGB(220, 220, 220));
        let _ = canvas.fill_rect(Rect::new(bx - 5, by - 5, bw + 10, bh + 10));

        canvas.set_draw_color(Color::RGB(255, 255, 255));
        let _ = canvas.fill_rect(Rect::new(bx, by, bw, bh));

        canvas.set_draw_color(Color::RGB(0, 102, 204));
        let _ = canvas.fill_rect(Rect::new(bx, by, bw, 25));

        let hole_color = Color::RGB(80, 80, 80);
        let row_spacing = 10i32;
        let col_spacing = 10i32;

        for row in 0..15i32 {
            let y = by + 35 + row * row_spacing;
            for col in 0..60i32 {
                let x = bx + 10 + col * col_spacing;
                canvas.set_draw_color(hole_color);
                let _ = canvas.fill_rect(Rect::new(x, y, 3, 3));
            }
        }

        for col in 0..60i32 {
            let x = bx + 10 + col * col_spacing;
            canvas.set_draw_color(Color::RGB(220, 220, 220));
            let _ = canvas.fill_rect(Rect::new(x - 1, by + 35 - 8, 5, 5));
            let _ = canvas.fill_rect(Rect::new(x - 1, by + 35 + 15 * row_spacing, 5, 5));
        }

        canvas.set_draw_color(Color::RGB(255, 100, 100));
        for i in 0..15i32 {
            let y = by + 40 + i * row_spacing;
            if i % 5 != 0 && i != 0 && i != 14 {
                let _ = canvas.fill_rect(Rect::new(bx + 8, y, 3, 3));
            }
        }

        canvas.set_draw_color(Color::RGB(0, 0, 0));
        for i in 0..15i32 {
            let y = by + 40 + i * row_spacing;
            if i % 5 != 0 && i != 0 && i != 14 {
                let _ = canvas.fill_rect(Rect::new(bx + bw as i32 - 11, y, 3, 3));
            }
        }

        canvas.set_draw_color(Color::RGB(100, 100, 100));
        let _ = canvas.fill_rect(Rect::new(bx + 15, by + 35 + 3 * row_spacing, (bw - 30) as u32, 3));
        let _ = canvas.fill_rect(Rect::new(bx + 15, by + 35 + 8 * row_spacing, (bw - 30) as u32, 3));
        let _ = canvas.fill_rect(Rect::new(bx + 15, by + 35 + 11 * row_spacing, (bw - 30) as u32, 3));
    }

    fn draw_arduino(&self, canvas: &mut sdl2::render::Canvas<sdl2::video::Window>) {
        let ax = 80i32;
        let ay = 30i32;
        let aw = 480u32;
        let ah = 130u32;

        canvas.set_draw_color(Color::RGB(0, 117, 178));
        let _ = canvas.fill_rect(Rect::new(ax - 3, ay - 3, aw + 6, ah + 6));

        canvas.set_draw_color(Color::RGB(0, 102, 153));
        let _ = canvas.fill_rect(Rect::new(ax, ay, aw, ah));

        canvas.set_draw_color(Color::RGB(0, 69, 107));
        let _ = canvas.fill_rect(Rect::new(ax, ay, aw, 25));

        let chip_x = ax + 180;
        let chip_y = ay + 50;
        canvas.set_draw_color(Color::RGB(30, 30, 30));
        let _ = canvas.fill_rect(Rect::new(chip_x, chip_y, 120, 70));
        canvas.set_draw_color(Color::RGB(20, 20, 20));
        let _ = canvas.fill_rect(Rect::new(chip_x + 5, chip_y + 5, 110, 60));

        for i in 0..8 {
            let notch_x = chip_x + 10 + i * 15;
            canvas.set_draw_color(Color::RGB(50, 50, 50));
            let _ = canvas.fill_rect(Rect::new(notch_x, chip_y - 4, 6, 6));
            let _ = canvas.fill_rect(Rect::new(notch_x, chip_y + 68, 6, 6));
        }

        canvas.set_draw_color(Color::RGB(200, 200, 200));
        for i in 0..8i32 {
            let pin_x = ax + 55 + i * 15;
            let pin_y = ay + 90;
            let _ = canvas.fill_rect(Rect::new(pin_x, pin_y, 8, 25));
            let _ = canvas.fill_rect(Rect::new(pin_x, pin_y + 30, 8, 25));
        }

        let reset_x = ax + 15;
        let reset_y = ay + 95;
        canvas.set_draw_color(Color::RGB(100, 100, 100));
        let _ = canvas.fill_rect(Rect::new(reset_x, reset_y, 20, 12));
        canvas.set_draw_color(Color::RGB(80, 80, 80));
        let _ = canvas.fill_rect(Rect::new(reset_x + 2, reset_y + 2, 16, 8));

        canvas.set_draw_color(Color::RGB(200, 50, 50));
        let tx_x = ax + 380;
        let tx_y = ay + 70;
        let _ = canvas.fill_rect(Rect::new(tx_x, tx_y, 15, 12));
        canvas.set_draw_color(Color::RGB(150, 40, 40));
        let _ = canvas.fill_rect(Rect::new(tx_x + 2, tx_y + 2, 11, 8));

        canvas.set_draw_color(Color::RGB(50, 200, 50));
        let rx_x = ax + 410;
        let rx_y = ay + 70;
        let _ = canvas.fill_rect(Rect::new(rx_x, rx_y, 15, 12));
        canvas.set_draw_color(Color::RGB(40, 150, 40));
        let _ = canvas.fill_rect(Rect::new(rx_x + 2, rx_y + 2, 11, 8));
    }

    fn draw_led(&self, canvas: &mut sdl2::render::Canvas<sdl2::video::Window>) {
        let led_x = 400i32;
        let led_y = 270i32;
        let led_size = 20i32;

        let (r, g, b) = if self.led_on { (255, 0, 0) } else { (80, 0, 0) };

        canvas.set_draw_color(Color::RGB(150, 150, 150));
        let _ = canvas.fill_rect(Rect::new(led_x, led_y, 15, 50));
        let _ = canvas.fill_rect(Rect::new(led_x + led_size + 5, led_y, 15, 50));

        canvas.set_draw_color(Color::RGB(r / 3, g / 3, b / 3));
        let _ = canvas.fill_rect(Rect::new(led_x - 5, led_y + 15, 20, 25));

        canvas.set_draw_color(Color::RGB(r, g, b));
        let _ = canvas.fill_rect(Rect::new(led_x + led_size / 2 - 10, led_y - 15, 20, 20));

        if self.led_on {
            canvas.set_draw_color(Color::RGB(255, 150, 150));
            let _ = canvas.fill_rect(Rect::new(led_x + led_size / 2 - 15, led_y - 20, 30, 30));
        }
    }

    fn draw_wires(&self, canvas: &mut sdl2::render::Canvas<sdl2::video::Window>) {
        canvas.set_draw_color(Color::RGB(0, 0, 0));
        let _ = canvas.fill_rect(Rect::new(620, 290, 30, 3));
        let _ = canvas.fill_rect(Rect::new(620, 295, 30, 3));
        let _ = canvas.fill_rect(Rect::new(645, 293, 3, 50));

        canvas.set_draw_color(Color::RGB(0, 0, 0));
        let _ = canvas.fill_rect(Rect::new(500, 250, 3, 30));
        let _ = canvas.fill_rect(Rect::new(500, 275, 60, 3));

        canvas.set_draw_color(Color::RGB(255, 0, 0));
        let _ = canvas.fill_rect(Rect::new(550, 275, 50, 3));
        let _ = canvas.fill_rect(Rect::new(595, 275, 3, 20));
    }

    fn draw_resistor(&self, canvas: &mut sdl2::render::Canvas<sdl2::video::Window>) {
        let rx = 300i32;
        let ry = 250i32;

        canvas.set_draw_color(Color::RGB(150, 100, 50));
        let _ = canvas.fill_rect(Rect::new(rx, ry + 3, 10, 4));
        canvas.set_draw_color(Color::RGB(220, 180, 100));
        let _ = canvas.fill_rect(Rect::new(rx + 10, ry, 15, 10));
        canvas.set_draw_color(Color::RGB(150, 100, 50));
        let _ = canvas.fill_rect(Rect::new(rx + 25, ry + 3, 10, 4));

        canvas.set_draw_color(Color::RGB(0, 0, 0));
        let _ = canvas.fill_rect(Rect::new(rx - 30, ry + 4, 30, 2));
        let _ = canvas.fill_rect(Rect::new(rx + 35, ry + 4, 40, 2));
        let _ = canvas.fill_rect(Rect::new(rx + 70, ry + 4, 3, 50));
    }

    fn run(&mut self) {
        let sdl_context = sdl2::init().unwrap();
        let video_subsystem = sdl_context.video().unwrap();

        let window = video_subsystem
            .window("RVBoard4 LED Simulator (Fritzing Style)", 700, 420)
            .position_centered()
            .build()
            .unwrap();

        let mut canvas = window.into_canvas().build().unwrap();

        println!("RVBoard4 LED Simulator - Fritzing Style");
        println!("Press SPACE to toggle LED mode, Q to quit");

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
                            println!("Manual mode: Press LEFT/RIGHT arrows to control LED");
                        } else {
                            println!("Auto mode: Running LED blink demo");
                        }
                    }
                    Event::KeyDown {
                        keycode: Some(Keycode::Left),
                        ..
                    } => {
                        self.led_off();
                        println!("LED OFF");
                    }
                    Event::KeyDown {
                        keycode: Some(Keycode::Right),
                        ..
                    } => {
                        self.led_on();
                        println!("LED ON ");
                    }
                    _ => {}
                }
            }

            if !manual_mode {
                println!("LED ON ");
                self.led_on();
                canvas.set_draw_color(Color::RGB(240, 240, 240));
                canvas.clear();
                self.draw_arduino(&mut canvas);
                self.draw_breadboard(&mut canvas);
                self.draw_wires(&mut canvas);
                self.draw_resistor(&mut canvas);
                self.draw_led(&mut canvas);
                canvas.present();
                self.delay_ms(500);

                println!("LED OFF");
                self.led_off();
                canvas.set_draw_color(Color::RGB(240, 240, 240));
                canvas.clear();
                self.draw_arduino(&mut canvas);
                self.draw_breadboard(&mut canvas);
                self.draw_wires(&mut canvas);
                self.draw_resistor(&mut canvas);
                self.draw_led(&mut canvas);
                canvas.present();
                self.delay_ms(500);
            } else {
                canvas.set_draw_color(Color::RGB(240, 240, 240));
                canvas.clear();
                self.draw_arduino(&mut canvas);
                self.draw_breadboard(&mut canvas);
                self.draw_wires(&mut canvas);
                self.draw_resistor(&mut canvas);
                self.draw_led(&mut canvas);
                canvas.present();
                std::thread::sleep(Duration::from_millis(16));
            }
        }
    }
}

fn main() {
    let mut sim = LedSimulator::new();
    sim.run();
}