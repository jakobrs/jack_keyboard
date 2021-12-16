use std::{
    any::Any,
    collections::HashSet,
    sync::mpsc::{self, Receiver, Sender},
};

use jack::{Client, ClientOptions, ClosureProcessHandler, ProcessScope, RawMidi};
use winit::{
    event::{ElementState, Event, KeyboardInput, ScanCode, VirtualKeyCode, WindowEvent},
    event_loop::{ControlFlow, EventLoop},
    window::{Window, WindowBuilder},
};

fn main() {
    let event_loop = EventLoop::new();
    let window = WindowBuilder::new().build(&event_loop).unwrap();

    let (tx, rx) = mpsc::channel();

    // JACK
    let _async_client = handle_jack(rx);

    // Window, blocking
    run_window(event_loop, window, tx);
}

fn handle_jack(rx: Receiver<KeyboardMsg>) -> impl Any {
    let (client, _client_status) =
        Client::new("jack_keyboard", ClientOptions::NO_START_SERVER).unwrap();

    let mut out = client
        .register_port("out", jack::MidiOut::default())
        .unwrap();

    let process = move |_client: &Client, process_scope: &ProcessScope| -> jack::Control {
        let mut writer = out.writer(process_scope);

        while let Ok(msg) = rx.try_recv() {
            let KeyboardMsg { note, pressed } = msg;

            match writer.write(&RawMidi {
                time: 0,
                bytes: &[
                    if pressed { 0x91 } else { 0x81 }, // Command
                    note.to_midi_value(),              // Note
                    0x70,                              // Velocity
                ],
            }) {
                Ok(_) => (),
                Err(err) => eprintln!("{:?}", err),
            }
        }

        jack::Control::Continue
    };

    client
        .activate_async((), ClosureProcessHandler::new(process))
        .unwrap()
}

fn run_window(event_loop: EventLoop<()>, window: Window, tx: Sender<KeyboardMsg>) {
    let mut active_keys = HashSet::new();

    event_loop.run(move |event, _, control_flow| {
        *control_flow = ControlFlow::Wait;

        match event {
            Event::WindowEvent {
                event:
                    WindowEvent::KeyboardInput {
                        input:
                            KeyboardInput {
                                scancode,
                                state,
                                virtual_keycode,
                                ..
                            },
                        ..
                    },
                window_id,
                ..
            } if window_id == window.id() => {
                if virtual_keycode == Some(VirtualKeyCode::Escape) {
                    *control_flow = ControlFlow::Exit;
                    return;
                }

                if state == ElementState::Pressed && active_keys.contains(&scancode) {
                    // Ignore repeated keys
                    return;
                }

                match state {
                    ElementState::Pressed => active_keys.insert(scancode),
                    ElementState::Released => active_keys.remove(&scancode),
                };

                if let Some(note) = Note::from_scancode(scancode) {
                    tx.send(KeyboardMsg {
                        note,
                        pressed: state == ElementState::Pressed,
                    })
                    .unwrap();
                }
            }
            Event::WindowEvent {
                event: WindowEvent::CloseRequested,
                window_id,
                ..
            } if window_id == window.id() => *control_flow = ControlFlow::Exit,
            _ => (),
        }
    });
}

#[derive(Debug)]
struct KeyboardMsg {
    note: Note,
    pressed: bool,
}

#[derive(Debug, Clone, Copy)]
enum Note {
    C4,
    CSharp4,
    D4,
    DSharp4,
    E4,
    F4,
    FSharp4,
    G4,
    GSharp4,
    A4,
    ASharp4,
    B4,
    C5,
}

impl Note {
    fn from_scancode(scancode: ScanCode) -> Option<Self> {
        Some(match scancode {
            30 => Note::C4,
            31 => Note::D4,
            32 => Note::E4,
            33 => Note::F4,
            34 => Note::G4,
            35 => Note::A4,
            36 => Note::B4,
            37 => Note::C5,

            17 => Note::CSharp4,
            18 => Note::DSharp4,
            20 => Note::FSharp4,
            21 => Note::GSharp4,
            22 => Note::ASharp4,

            _ => return None,
        })
    }

    fn to_midi_value(self) -> u8 {
        match self {
            Note::C4 => 60,
            Note::CSharp4 => 61,
            Note::D4 => 62,
            Note::DSharp4 => 63,
            Note::E4 => 64,
            Note::F4 => 65,
            Note::FSharp4 => 66,
            Note::G4 => 67,
            Note::GSharp4 => 68,
            Note::A4 => 69,
            Note::ASharp4 => 70,
            Note::B4 => 71,
            Note::C5 => 72,
        }
    }
}
