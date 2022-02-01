use anyhow::Result;
use my_esp_idf::get_time_millis;
use my_esp_idf::l298_motor_controller::L298MotorControllerI;
use my_esp_idf::servo::Angle;
use my_esp_idf::servo::ServoI;
use my_esp_idf::OutputPin;
use my_esp_idf::PinState;
use std::sync::mpsc::Receiver;

use crate::gamepad_serializer_model::GamePadState;

pub struct Rover<S: ServoI, BP: OutputPin, CML: OutputPin, MC1: L298MotorControllerI> {
    buzzer_pin: BP,
    camera_light_pin: CML,
    camera_x_servo: S,
    camera_servos_on: bool,
    camera_servos_sleep_time_ms: i64,
    camera_servos_pos: (f32, f32),
    motor_controller_1: MC1,
    now: i64,
    gamepad_state: GamePadState,
    gamepad_rx: Receiver<GamePadState>,
}

impl<S: ServoI, BP: OutputPin, CML: OutputPin, MC1: L298MotorControllerI> Rover<S, BP, CML, MC1> {
    pub fn new(
        buzzer_pin: BP,
        camera_light_pin: CML,
        camera_x_servo: S,
        motor_controller_1: MC1,
        gamepad_rx: Receiver<GamePadState>,
    ) -> Self {
        Self {
            buzzer_pin,
            camera_light_pin,
            camera_x_servo,
            camera_servos_on: true,
            camera_servos_sleep_time_ms: 0,
            camera_servos_pos: (0., 0.),
            motor_controller_1,
            now: 0,
            gamepad_rx,
            gamepad_state: GamePadState {
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
            },
        }
    }
    pub fn run(&mut self) -> Result<()> {
        self.on_run_starting();
        loop {
            self.now = get_time_millis();
            if let Err(e) = self.run_tick() {
                self.on_shutdown();
                return Err(e);
            }
        }
    }

    fn on_run_starting(&mut self) {
        self.camera_light_pin.set_high().unwrap();
        self.buzzer_pin.set_high().unwrap();
        my_esp_idf::delay_ms(100);
        self.camera_light_pin.set_low().unwrap();
        self.buzzer_pin.set_low().unwrap();
    }

    fn on_shutdown(&mut self) {
        // Avoid panics to try to shutdown everything.
        self.motor_controller_1
            .set_motors_speed_and_direction(0., 0.)
            .ok();
        self.buzzer_pin.set_low().ok();
        self.camera_light_pin.set_low().ok();
        self.camera_x_servo.disable().ok();
    }

    #[inline(always)]
    pub fn run_tick(&mut self) -> Result<()> {
        loop {
            match self.gamepad_rx.try_recv() {
                Ok(s) => self.gamepad_state = s,
                Err(e) => match e {
                    std::sync::mpsc::TryRecvError::Disconnected => {
                        return Err(anyhow::Error::new(e))
                    }
                    std::sync::mpsc::TryRecvError::Empty => break,
                },
            }
        }

        if self.gamepad_state.east {
            self.on_shutdown();
            return my_esp_idf::run_https_ota();
        }

        self.buzzer_pin.set_state(match self.gamepad_state.north {
            true => PinState::High,
            false => PinState::Low,
        })?;

        self.camera_light_pin
            .set_state(match self.gamepad_state.west {
                true => PinState::High,
                false => PinState::Low,
            })?;

        let (motor1_vector, motor2_vector) =
            my_rover_lib::calculate_motors_direction_velocity_vector(
                self.gamepad_state.left_x,
                self.gamepad_state.left_y,
            );
        self.motor_controller_1
            .set_motors_speed_and_direction(motor1_vector, motor2_vector)?;

        self.update_camera_servos()?;

        my_esp_idf::delay_ms(10);

        Ok(())
    }

    fn update_camera_servos(&mut self) -> Result<()> {
        let rounder = 50.;
        let rounded_pos = (
            (self.gamepad_state.right_x * rounder).round() / rounder,
            (self.gamepad_state.right_y * rounder).round() / rounder,
        );
        if rounded_pos != self.camera_servos_pos {
            self.camera_servos_pos = rounded_pos;
            self.camera_servos_sleep_time_ms = self.now + 500;
            if !self.camera_servos_on {
                self.camera_servos_on = true;
                self.camera_x_servo.enable()?;
            }
        } else if self.camera_servos_on && self.camera_servos_sleep_time_ms <= self.now {
            self.camera_servos_on = false;
            self.camera_x_servo.disable()?;
        }
        self.camera_x_servo.set_angle(Angle(my_rover_lib::lerp(
            self.gamepad_state.right_x,
            -1.0,
            1.0,
            180.,
            0.,
        )))?;
        Ok(())
    }
}
