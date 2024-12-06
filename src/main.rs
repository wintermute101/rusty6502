use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::time::{Duration,Instant};
use std::thread;
use std::sync::mpsc::{channel, TryRecvError};
use macroquad::prelude::*;

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
    let (to64_tx,to64_rx) = channel();

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
                            for i in c{
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
                    /*let mut keymap = C64KeyboadMap::new();
                    for i in c{
                        match i{
                            KeyCode::A => {keymap.col[1] &= !(1 << 2);},
                            KeyCode::D => {keymap.col[2] &= !(1 << 2);},
                            KeyCode::G => {keymap.col[3] &= !(1 << 2);},
                            KeyCode::J => {keymap.col[4] &= !(1 << 2);},
                            KeyCode::L => {keymap.col[5] &= !(1 << 2);},
                            KeyCode::Semicolon => {keymap.col[6] &= !(1 << 2);},

                            KeyCode::W => {keymap.col[1] &= !(1 << 1);},
                            KeyCode::R => {keymap.col[2] &= !(1 << 1);},
                            KeyCode::Y => {keymap.col[3] &= !(1 << 1);},
                            KeyCode::I => {keymap.col[4] &= !(1 << 1);},
                            KeyCode::P => {keymap.col[5] &= !(1 << 1);},

                            KeyCode::F6 => {debug_mode = true;},

                            _ => {/*println!("Not supported key {:?}", i)*/},
                        }
                    }
                    //keymap.col[1] &= !(1 << 2);
                    c64.set_keyboard_map(keymap);*/
                    for i in c{
                        println!("Keystroke {:?}", i);
                        match i{

                            KeyCode::Enter => c64.add_key_stroke(0x0d),

                            KeyCode::Space => c64.add_key_stroke(0x20),
                            KeyCode::LeftBracket => c64.add_key_stroke(0x21),
                            KeyCode::RightBracket => c64.add_key_stroke(0x22),

                            KeyCode::Key0 => c64.add_key_stroke(0x30),
                            KeyCode::Key1 => c64.add_key_stroke(0x31),
                            KeyCode::Key2 => c64.add_key_stroke(0x32),
                            KeyCode::Key3 => c64.add_key_stroke(0x33),
                            KeyCode::Key4 => c64.add_key_stroke(0x34),
                            KeyCode::Key5 => c64.add_key_stroke(0x35),
                            KeyCode::Key6 => c64.add_key_stroke(0x36),
                            KeyCode::Key7 => c64.add_key_stroke(0x37),
                            KeyCode::Key8 => c64.add_key_stroke(0x38),
                            KeyCode::Key9 => c64.add_key_stroke(0x39),

                            KeyCode::Semicolon => c64.add_key_stroke(0x3b),

                            KeyCode::A => c64.add_key_stroke(0x41),
                            KeyCode::B => c64.add_key_stroke(0x42),
                            KeyCode::C => c64.add_key_stroke(0x43),
                            KeyCode::D => c64.add_key_stroke(0x44),
                            KeyCode::E => c64.add_key_stroke(0x45),
                            KeyCode::F => c64.add_key_stroke(0x46),
                            KeyCode::G => c64.add_key_stroke(0x47),
                            KeyCode::H => c64.add_key_stroke(0x48),
                            KeyCode::I => c64.add_key_stroke(0x49),
                            KeyCode::J => c64.add_key_stroke(0x4a),
                            KeyCode::K => c64.add_key_stroke(0x4b),
                            KeyCode::L => c64.add_key_stroke(0x4c),
                            KeyCode::M => c64.add_key_stroke(0x4d),
                            KeyCode::N => c64.add_key_stroke(0x4e),
                            KeyCode::O => c64.add_key_stroke(0x4f),
                            KeyCode::P => c64.add_key_stroke(0x50),
                            KeyCode::Q => c64.add_key_stroke(0x51),
                            KeyCode::R => c64.add_key_stroke(0x52),
                            KeyCode::S => c64.add_key_stroke(0x53),
                            KeyCode::T => c64.add_key_stroke(0x54),
                            KeyCode::U => c64.add_key_stroke(0x55),
                            KeyCode::V => c64.add_key_stroke(0x56),
                            KeyCode::W => c64.add_key_stroke(0x57),
                            KeyCode::X => c64.add_key_stroke(0x58),
                            KeyCode::Y => c64.add_key_stroke(0x59),
                            KeyCode::Z => c64.add_key_stroke(0x5a),

                            KeyCode::F1 => {c64.interrupt();}
                            KeyCode::F6 => {debug_mode = true;},
                            KeyCode::Escape => {c64.reset();}
                            _ => {println!("Not supported key {:?}", i)},
                        }
                    }
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

        let keys = get_keys_pressed();
        if !keys.is_empty(){
            to64_tx.send(keys).unwrap();//TODO fix unwrap
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
