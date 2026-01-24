/// Stock EV3 Color Sensor
pub mod color_sensor;
/// Stock EV3 Gyro Sensor
pub mod gyro_sensor;
/// Stock EV3 Infrared Sensor
pub mod infrared_sensor;
/// Stock EV3 Large or Medium Motor
pub mod motor;
/// Stock EV3 Touch Sensor
pub mod touch_sensor;
/// Stock EV3 Ultrasonic Sensor
pub mod ultrasonic_sensor;

pub use color_sensor::ColorSensor;
pub use gyro_sensor::GyroSensor;
pub use infrared_sensor::InfraredSensor;
pub use motor::Motor;
pub use touch_sensor::TouchSensor;
pub use ultrasonic_sensor::UltrasonicSensor;
