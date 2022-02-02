mod rover;
pub mod gamepad_serializer_model {
    include!("gamepad_serializer.model.rs");
}

use anyhow::Result;
use my_esp_idf::{
    l298_motor_controller::L298MotorController,
    mcpwm_pin, output_pin,
    servo::{Angle, Servo},
    steam_controller::{Axis, Button, SteamControllerEvent},
};

#[no_mangle]
extern "C" fn rust_main() {
    main().unwrap();
}

pub fn main() -> Result<()> {
    my_esp_idf::init();

    let peripherals = esp_idf_hal::prelude::Peripherals::take().unwrap();

    let motor_controller_1 = L298MotorController::new(
        output_pin!(peripherals.pins.gpio17),
        output_pin!(peripherals.pins.gpio16),
        output_pin!(peripherals.pins.gpio18),
        output_pin!(peripherals.pins.gpio19),
        mcpwm_pin!(peripherals.pins.gpio0, peripherals.pins.mcpwm_unit_0_gen_b),
        mcpwm_pin!(peripherals.pins.gpio2, peripherals.pins.mcpwm_unit_0_gen_a),
    )?;

    let gamepad_rx = connect_steam_controller()?;

    let mut rover = rover::Rover::new(
        output_pin!(peripherals.pins.gpio13),
        output_pin!(peripherals.pins.gpio33),
        Servo::new(
            mcpwm_pin!(peripherals.pins.gpio32, peripherals.pins.mcpwm_unit_1_gen_a),
            Angle(180.),
            std::time::Duration::from_micros(700),
            std::time::Duration::from_micros(2200),
        ),
        motor_controller_1,
        gamepad_rx,
    );

    rover.run()?;

    Ok(())
}

// Waits for unencrypted gamepad_serializer_model::GamePadState
// protobuf messages via UDP on port 1234.
pub fn connect_udp_controller(
) -> Result<std::sync::mpsc::Receiver<gamepad_serializer_model::GamePadState>> {
    let (gamepad_chan_tx, gamepad_chan_rx) = std::sync::mpsc::channel();
    my_esp_idf::start_udp_listener(1234, move |_src, buffer| {
        if let Ok(state) =
            <gamepad_serializer_model::GamePadState as prost::Message>::decode(buffer)
        {
            gamepad_chan_tx.send(state).ok();
        } else {
            log::info!("UDP: invalid packet: {:?}", buffer);
        }
    })?;
    Ok(gamepad_chan_rx)
}

// Starts a thread to scan and connect to a Steam Controller. This call doesn't
// block.
pub fn connect_steam_controller(
) -> Result<std::sync::mpsc::Receiver<gamepad_serializer_model::GamePadState>> {
    let (_gamepad_chan_tx, gamepad_chan_rx) = std::sync::mpsc::channel();
    let mut state = gamepad_serializer_model::GamePadState {
        gamepad_id: 0,
        left_y: 0.,
        left_x: 0.,
        right_x: 0.,
        right_y: 0.,
        north: false,
        south: false,
        west: false,
        east: false,
        left_trigger_2: 0.,
        right_trigger_2: 0.,
    };
    my_esp_idf::steam_controller::connect(my_esp_idf::connect_ble()?, move |event| {
        match event {
            SteamControllerEvent::ButtonChanged(button, value) => match button {
                Button::South => state.south = value == 1.0,
                Button::North => state.north = value == 1.0,
                Button::East => state.east = value == 1.0,
                Button::West => state.west = value == 1.0,
                _ => {}
            },
            SteamControllerEvent::AxisChanged(axis, value) => match axis {
                Axis::LeftStickX => state.left_x = value,
                Axis::LeftStickY => state.left_y = value,
                Axis::RightPadX => state.right_x = value,
                Axis::RightPadY => state.right_y = value,
                _ => {}
            },
            SteamControllerEvent::Connected => {
                log::info!("Steam controller connected!");
            }
            SteamControllerEvent::Disconnected => {
                log::info!("Steam controller disconnected!");
                state = gamepad_serializer_model::GamePadState {
                    gamepad_id: 0,
                    left_y: 0.,
                    left_x: 0.,
                    right_x: 0.,
                    right_y: 0.,
                    north: false,
                    south: false,
                    west: false,
                    east: false,
                    left_trigger_2: 0.,
                    right_trigger_2: 0.,
                }
            }
        };
        _gamepad_chan_tx.send(state.clone()).unwrap();
    })?;
    Ok(gamepad_chan_rx)
}
