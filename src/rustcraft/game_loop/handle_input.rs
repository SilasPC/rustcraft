
use crate::prelude::*;
use super::*;

#[derive(Default)]
pub struct Return {
    pub do_chunk_load: bool,
    pub do_quit: bool,
}

impl<'a> GameLoop<'a> {
    pub fn handle_input(&mut self) -> Return {

        let mut ret = Return::default();

        self.data.input.start_new_frame();
        self.data.display.video.text_input().start();
        self.data.input.update_flags(self.data.display.sdl.keyboard().mod_state());
        for event in self.data.event_pump.poll_iter() {
            use sdl2::event::Event::*;
            use sdl2::keyboard::Keycode::*;
            use sdl2::event::WindowEvent::*;
            self.data.input.update(&event);
            match event {
                TextInput { ref text, .. } => {
                    let input_text = text;
                    match &mut self.state {
                        GameState::Chat { text, start_frame } if *start_frame != self.rdata.frame_time => {
                            let mut txt = text.text().to_owned();
                            txt.push_str(&input_text);
                            text.set_text(txt);
                        },
                        _ => {}
                    }
                },
                Quit {..} => ret.do_quit = true,
                KeyDown {keycode: Some(Escape), repeat: false, ..} => {
                    self.state = match self.state {
                        GameState::Paused => {
                            self.data.display.set_mouse_capture(true);
                            GameState::Playing { breaking: std::option::Option::None }
                        },
                        GameState::Chat { .. } => {
                            self.data.display.set_mouse_capture(true);
                            GameState::Playing { breaking: std::option::Option::None }
                        },
                        _ => {
                            self.data.display.set_mouse_capture(false);
                            GameState::Paused
                        }
                    };
                },
                KeyDown {keycode: Some(F5), repeat: false, .. } => self.data.settings.third_person ^= true,
                KeyDown {keycode: Some(F11), repeat: false, ..} => self.data.display.set_fullscreen(!self.data.display.state.fullscreen),
                KeyDown {keycode: Some(R), ..} => ret.do_chunk_load = true,
                KeyDown {keycode: Some(E), repeat: false, ..} => {
                    use crate::rustcraft::inventory::InventoryShell;
                    match self.state {
                        GameState::Playing {..} => {
                            self.data.display.set_mouse_capture(false);
                            self.state = GameState::Inventory {
                                picked_item: Option::None,
                                inventory: self.pgui.inventory.dyn_clone(),
                            }
                        },
                        GameState::Inventory { .. } => {
                            self.data.display.set_mouse_capture(true);
                            self.state = GameState::Playing { breaking: std::option::Option::None }
                        },
                        _ => {}
                    };
                },
                KeyDown {keycode: Some(Return), repeat: false, ..} => {
                    match &self.state {
                        GameState::Chat { text, .. } => {
                            let cmd: Option<Cmd> = text.text().parse().ok();
                            println!("{}\n => {:?}",text.text(),cmd);
                            if let Some(cmd) = cmd {
                                cmd.exec(&mut self.world, &self.idata);
                            }
                            self.data.display.set_mouse_capture(true);
                            self.state = GameState::Playing { breaking: std::option::Option::None }
                        },
                        _ => {}
                    };
                },
                KeyDown {keycode: Some(T), ..} => {
                    match self.state {
                        GameState::Playing {..} => {
                            self.data.display.set_mouse_capture(false);
                            self.state = GameState::Chat { text: self.idata.font.build_text("".into()), start_frame: self.rdata.frame_time }
                        },
                        _ => {}
                    };
                },
                KeyDown {keycode: Some(Backspace), ..} => {
                    match &mut self.state {
                        GameState::Chat { ref mut text, start_frame } => {
                            let mut txt = text.text().to_owned();
                            txt.pop();
                            text.set_text(txt);
                        },
                        _ => {}
                    };
                },
                Window { win_event: Resized(..), .. } => self.data.display.refresh(),
                _ => {}
            }
        }

        self.pgui.scroll(-self.data.input.scroll());

        ret

    }

}