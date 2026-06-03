use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::time::{Duration,Instant};
use std::thread;
use std::sync::mpsc::{channel, TryRecvError};
use macroquad::prelude::*;
use std::collections::HashSet;

mod c64;
use c64::C64;
use c64::c64memory::{C64CharaterRam, C64KeyboadMap};

fn window_conf() -> Conf {
    Conf {
        window_title: "Rusty C64".to_owned(),
        window_resizable: false,
        //fullscreen: true,
        ..Default::default()
    }
}

enum ScreenUpdate{
    Chars(C64CharaterRam),
    CharsRam([u8; 4096]),
}

struct KeysPressed{
    key_codes : HashSet<KeyCode>,
}

#[macroquad::main(window_conf)]
async fn main() {
    let running = Arc::new(AtomicBool::new(true));
    let r = running.clone();
    let r2 = running.clone();

    ctrlc::set_handler(move || {
        r.store(false, Ordering::SeqCst);
    }).expect("Error setting Ctrl-C handler");

    let enable_dbug_at: Option<u16> = None;

    //enable_dbug_at = Some(0xff48);

    let (fromc64_tx,fromc64_rx) = channel();
    let (to64_tx,to64_rx) = channel::<KeysPressed>();

    //let color = color_u8!(0x50,0x45,0x9b,255);
    let color = color_u8!(0x88,0x7e,0xcb,255);
    let mut image = Image::gen_image_color(384, 272, color);
    let texture: Texture2D = Texture2D::from_image(&image);

    //let c64_font = load_ttf_font("fonts/C64_Pro_Mono-STYLE.ttf").await.expect("c64 font");

    let thread_handle = thread::Builder::new().name("C64".to_owned()).spawn(move || {
        let mut cnt = 0;
        let mut c64 = C64::new();

        c64.enable_trace(64);
        c64.reset();

        let mut debug_mode = false;

        let character_set = c64.get_character_rom(true);
        fromc64_tx.send(ScreenUpdate::CharsRam(character_set.unwrap())).expect("send");

        let mut now = Instant::now();
        while running.load(Ordering::SeqCst){
            let r = c64.run_single();

            cnt += 1;

            match r{
                Ok(pc) => {
                    if let Some(debug_at) = enable_dbug_at{
                        if debug_at == pc{
                            debug_mode = true;
                            println!("Entering debug mode at PC={:#06x} F5 to step", pc);
                        }
                    }
                },
                Err(e) => {
                    eprintln!("C64 Cpu error: {}", e);
                    break;
                }
            };

            if debug_mode{
                let state = c64.get_last_state();
                println!("{:?}", state);
                loop{
                    match to64_rx.recv(){
                        Ok(c) => {
                            let mut need_break = false;
                            for i in c.key_codes{
                                if i == KeyCode::F5{
                                    need_break = true;
                                }
                                else if i == KeyCode::F7{
                                    debug_mode = false;
                                    need_break = true;
                                }
                            }
                            if need_break{
                                break;
                            }
                        }
                        Err(e) => {
                            eprintln!("Error c64rx {}", e);
                            return;
                        }
                    }
                }
                //let state = c64.get_last_state();
                //println!("{:?}", state);
            }

            match to64_rx.try_recv(){
                Err(e) if e == TryRecvError::Empty => {},
                Err(e) => {
                    eprintln!("Error c64rx {}", e);
                    break;
                }
                Ok(c) => {
                    let mut keymap = C64KeyboadMap::new();
                    let is_shift = c.key_codes.contains(&KeyCode::LeftShift) || c.key_codes.contains(&KeyCode::RightShift);
                    for i in c.key_codes{
                        // Map PC KeyCodes to C64 Keyboard Matrix (Col/Row)
                        match i{
                            // Row 0: DEL, RET, L/R, F7, F1, F3, F5, U/D
                            KeyCode::Backspace => {keymap.col[0] &= !(1 << 0);},
                            KeyCode::Enter     => {keymap.col[0] &= !(1 << 1);},
                            KeyCode::F1        => {keymap.col[0] &= !(1 << 4);},
                            KeyCode::F3        => {keymap.col[0] &= !(1 << 5);},
                            KeyCode::F5        => {keymap.col[0] &= !(1 << 6);},
                            KeyCode::F7        => {keymap.col[0] &= !(1 << 3);},

                            // Row 1-4: Alphanumeric
                            KeyCode::Key3 => {keymap.col[1] &= !(1 << 0);},
                            KeyCode::W => {keymap.col[1] &= !(1 << 1);},
                            KeyCode::A => {keymap.col[1] &= !(1 << 2);},
                            KeyCode::Key4 => {keymap.col[1] &= !(1 << 3);},
                            KeyCode::Z => {keymap.col[1] &= !(1 << 4);},
                            KeyCode::S => {keymap.col[1] &= !(1 << 5);},
                            KeyCode::E => {keymap.col[1] &= !(1 << 6);},
                            KeyCode::LeftShift => {keymap.col[1] &= !(1 << 7);},
                            KeyCode::RightShift => {keymap.col[6] &= !(1 << 4);},

                            KeyCode::Key5 => {keymap.col[2] &= !(1 << 0);},
                            KeyCode::R => {keymap.col[2] &= !(1 << 1);},
                            KeyCode::D => {keymap.col[2] &= !(1 << 2);},
                            KeyCode::Key6 => {keymap.col[2] &= !(1 << 3);},
                            KeyCode::C => {keymap.col[2] &= !(1 << 4);},
                            KeyCode::F => {keymap.col[2] &= !(1 << 5);},
                            KeyCode::T => {keymap.col[2] &= !(1 << 6);},
                            KeyCode::X => {keymap.col[2] &= !(1 << 7);},

                            KeyCode::Key7 => {keymap.col[3] &= !(1 << 0);},
                            KeyCode::Y => {keymap.col[3] &= !(1 << 1);},
                            KeyCode::G => {keymap.col[3] &= !(1 << 2);},
                            KeyCode::Key8 => {
                                if is_shift { keymap.col[6] &= !(1 << 1); } // PC * (Shift+8) -> C64 *
                                else { keymap.col[3] &= !(1 << 3); }        // PC 8 -> C64 8
                            },
                            KeyCode::B => {keymap.col[3] &= !(1 << 4);},
                            KeyCode::H => {keymap.col[3] &= !(1 << 5);},
                            KeyCode::U => {keymap.col[3] &= !(1 << 6);},
                            KeyCode::V => {keymap.col[3] &= !(1 << 7);},

                            KeyCode::Key9 => {
                                if is_shift { keymap.col[3] &= !(1 << 3); } // PC ( (Shift+9) -> C64 8 (Shift+8 = ()
                                else { keymap.col[4] &= !(1 << 0); }        // PC 9 -> C64 9
                            },
                            KeyCode::I => {keymap.col[4] &= !(1 << 1);},
                            KeyCode::J => {keymap.col[4] &= !(1 << 2);},
                            KeyCode::Key0 => {
                                if is_shift { keymap.col[4] &= !(1 << 0); } // PC ) (Shift+0) -> C64 9 (Shift+9 = ))
                                else { keymap.col[4] &= !(1 << 3); }        // PC 0 -> C64 0
                            },
                            KeyCode::M => {keymap.col[4] &= !(1 << 4);},
                            KeyCode::K => {keymap.col[4] &= !(1 << 5);},
                            KeyCode::O => {keymap.col[4] &= !(1 << 6);},
                            KeyCode::N => {keymap.col[4] &= !(1 << 7);},

                            KeyCode::P => {keymap.col[5] &= !(1 << 1);},
                            KeyCode::L => {keymap.col[5] &= !(1 << 2);},
                            KeyCode::Comma =>  {keymap.col[5] &= !(1 << 7);},
                            KeyCode::Period => {keymap.col[5] &= !(1 << 4);},
                            KeyCode::Minus => {
                                if is_shift { keymap.col[7] &= !(1 << 1); } // PC _ (Shift+Minus) -> C64 _
                                else { keymap.col[5] &= !(1 << 3); }        // PC - -> C64 -
                            },
                            KeyCode::Equal => {
                                if is_shift { keymap.col[5] &= !(1 << 0); } // PC + (Shift+Equal) -> C64 +
                                else { keymap.col[6] &= !(1 << 5); }        // PC = -> C64 =
                            },
                            KeyCode::Slash =>  {keymap.col[6] &= !(1 << 7);},
                            KeyCode::Semicolon => {keymap.col[6] &= !(1 << 2);},
                            KeyCode::Apostrophe => {
                                if is_shift { keymap.col[7] &= !(1 << 3); } // PC " (Shift+') -> C64 2 (Shift+2 = ")
                                else {
                                    keymap.col[3] &= !(1 << 0); // C64 7
                                    keymap.col[1] &= !(1 << 7); // Force Shift for C64 ' (Shift+7)
                                }
                            },

                            KeyCode::Key1 => {keymap.col[7] &= !(1 << 0);},
                            KeyCode::Key2 => {
                                if is_shift { keymap.col[5] &= !(1 << 6); } // PC @ (Shift+2) -> C64 @
                                else { keymap.col[7] &= !(1 << 3); }        // PC 2 -> C64 2
                            },
                            KeyCode::Q => {keymap.col[7] &= !(1 << 6);},
                            KeyCode::Space => {keymap.col[7] &= !(1 << 4);},
                            KeyCode::LeftControl => {keymap.col[7] &= !(1 << 2);},
                            KeyCode::Escape => {keymap.col[7] &= !(1 << 7);}, // Map Stop

                            KeyCode::F6 => {debug_mode = true;},

                            // Emulator System Shortcuts
                            KeyCode::F11 => { c64.interrupt(); },
                            KeyCode::F12 => { c64.reset(); },
                            _ => {},
                        }
                    }
                    c64.set_keyboard_map(keymap);
                },
            }

            match now.elapsed(){
                v if v >= Duration::from_millis(200) => {
                    let charram = c64.get_character_ram();
                    fromc64_tx.send(ScreenUpdate::Chars(charram)).expect("Send");
                    now = Instant::now();
                }
                _ => {}
            }

            if cnt % 150 == 0{
                thread::sleep(Duration::from_micros(10));
            }
        }
        println!("Exiting...");
        c64.show_debug();
        c64.show_screen_ram(true);
    }).expect("thread spawn error");

    let mut charram = C64CharaterRam::new();
    let mut charrom = None;
    let mut need_redraw = false;

    while r2.load(Ordering::SeqCst){
        clear_background(BLACK);
        match fromc64_rx.try_recv(){
            Err(e) if e == TryRecvError::Empty => {},
            Err(e) => {
                eprintln!("Error graphics rx {}", e);
                break;
            }
            Ok(v) => {
                match v{
                    ScreenUpdate::Chars(c) => {
                        charram = c;
                        need_redraw = true;
                    },
                    ScreenUpdate::CharsRam(r) => {
                        charrom = Some(r);
                        need_redraw = true;
                    },
                }
            },
        }

        if charrom.is_none() || !need_redraw{
            next_frame().await;
            continue;
        }

        let image_w = image.width();
        let image_x_off = (image_w - 320) / 2;
        let image_y_off = (image.height() - 200) / 2;
        let image_data = image.get_image_data_mut();

        let bg_color = color_u8!(0x50,0x45,0x9b,255);
        let fg_color = color_u8!(0x88,0x7e,0xcb,255);

        let chars = charram.ram;
        for lnum in 0..25 as usize{
            for i in 0..40 as usize{
                let symbol = chars[lnum*40 + i];
                let symbol_data: [u8; 8] = match charrom.unwrap()[(symbol as usize *8) .. (symbol as usize *8+8)].try_into(){
                    Ok(s) => {
                        s
                    }
                    Err(e) =>{
                        panic!("err {} symbol {:#04x}", e, symbol);
                    }
                };
                for y in 0..8{
                    let symbol_row = symbol_data[y];
                    for x in 0..8{

                        let px = (lnum*8+y+image_y_off)*image_w + i*8 + x + image_x_off;
                        let bit = (symbol_row >> (7-x)) & 0x1;
                        if bit == 1{
                            image_data[px] = fg_color.into();
                        }
                        else{
                            image_data[px] = bg_color.into();
                        }
                    }
                }
            }
        }

        texture.update(&image);
        let tex_params = DrawTextureParams {
            dest_size: Some(vec2(screen_width(), screen_height())),
            ..Default::default()
        };
        draw_texture_ex(&texture, 0., 0., WHITE, tex_params);

        /*for lnum in 0..25 as usize{
            let line = charram.ram
                .iter()
                .skip(lnum*40)
                .take(40)
                .map(|c| C64Memory::screen_code_to_char(*c))
                .fold(String::with_capacity(40), |mut a, c| {a.push(c); a});

            draw_text_ex(&line, 40.0, lnum as f32 * 18.0 + 100.0, TextParams{font_size: 18, font: Some(&c64_font), color: color_u8!(255,255,255,255), ..Default::default()});
        }*/

        let keys_down = get_keys_down();
        let keys_released = get_keys_released();

        if !keys_down.is_empty() || !keys_released.is_empty() {
            let key_pressed = KeysPressed{key_codes: keys_down};
            if let Err(e) = to64_tx.send(key_pressed){
                println!("Send error {e}");
                break;
            }
        }


        next_frame().await;
    }

    match thread_handle.join(){
        Ok(()) => {},
        Err(e) => {
            println!("Error join {:?}", e);
        },
    }

}
