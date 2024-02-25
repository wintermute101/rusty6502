use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::time::{Duration,Instant};
use std::thread;
use std::sync::mpsc::{channel, TryRecvError};
use macroquad::prelude::*;

mod c64;
use c64::C64;
use c64::c64memory::{C64CharaterRam, C64Memory, C64KeyboadMap};

fn window_conf() -> Conf {
    Conf {
        window_title: "Rusty C64".to_owned(),
        window_resizable: false,
        //fullscreen: true,
        ..Default::default()
    }
}

#[macroquad::main(window_conf)]
async fn main() {
    let running = Arc::new(AtomicBool::new(true));
    let r = running.clone();
    let r2 = running.clone();

    ctrlc::set_handler(move || {
        r.store(false, Ordering::SeqCst);
    }).expect("Error setting Ctrl-C handler");

    let (fromc64_tx,fromc64_rx) = channel();
    let (to64_tx,to64_rx) = channel();

    let color = color_u8!(255,0,0,255);
    let mut image = Image::gen_image_color(400, 400, color);
    //let texture: Texture2D = Texture2D::from_image(&image);

    let c64_font = load_ttf_font("fonts/C64_Pro_Mono-STYLE.ttf").await.expect("c64 font");

    let mut cnt = 0;

    let thread_handle = thread::Builder::new().name("C64".to_owned()).spawn(move || {
        let mut c64 = C64::new();

        c64.enable_trace(64);
        c64.reset();

        let mut cnt = 0;

        let mut now = Instant::now();
        while running.load(Ordering::SeqCst){
            let r = c64.run_single();

            cnt += 1;

            match r{
                Ok(()) => {},
                Err(e) => {
                    eprintln!("C64 Cpu error: {}", e);
                    break;
                }
            }

            match to64_rx.try_recv(){
                Err(e) if e == TryRecvError::Empty => {},
                Err(e) => {
                    eprintln!("Error c64rx {}", e);
                    break;
                }
                Ok(c) => {
                    let mut keymap = C64KeyboadMap::new();
                    for i in c{
                        match i{
                            KeyCode::A => {keymap.col[1] &= !(1 << 2);},
                            _ => {},
                        }
                    }
                    //keymap.col[7] &= !(1 << 4);
                    c64.set_keyboard_map(keymap);
                },
            }

            match now.elapsed(){
                //v if v >= Duration::from_micros(16666) => {
                v if v >= Duration::from_millis(100) => {
                    //println!("Intterrupt! {}", cnt);
                    let charram = c64.get_character_ram();
                    c64.interrupt();
                    now = Instant::now();
                    fromc64_tx.send(charram).unwrap(); //TODO fix unwrap
                }
                _ => {}
             }

             if cnt % 1000 == 0{
                thread::sleep(Duration::from_millis(1));
             }
        }
        println!("Exiting...");
        c64.show_debug();
        c64.show_screen_ram(true);
    }).expect("thread spawn error");

    let mut charram = C64CharaterRam::new();

    while r2.load(Ordering::SeqCst){
        clear_background(BLACK);
        //draw_texture(&texture, 0., 0., WHITE);
        match fromc64_rx.try_recv(){
            Err(e) if e == TryRecvError::Empty => {},
            Err(e) => {
                eprintln!("Error graphics rx {}", e);
                break;
            }
            Ok(v) => {
                charram = v;
            },
        }

        let data = image.get_image_data_mut();

        data[cnt] = color_u8!(0,255,0,255).into();

        cnt += 1;

        //texture.update(&image);

        for lnum in 0..25 as usize{
            let line = charram.ram
                .iter()
                .skip(lnum*40)
                .take(40)
                .map(|c| C64Memory::screen_code_to_char(*c))
                .fold(String::with_capacity(40), |mut a, c| {a.push(c); a});

            draw_text_ex(&line, 40.0, lnum as f32 * 18.0 + 100.0, TextParams{font_size: 18, font: Some(&c64_font), color: color_u8!(255,255,255,255), ..Default::default()});
        }

        to64_tx.send(get_keys_down()).unwrap();//TODO fix unwrap

        next_frame().await;
    }

    match thread_handle.join(){
        Ok(()) => {},
        Err(e) => {
            println!("Error join {:?}", e);
        },
    }

}
