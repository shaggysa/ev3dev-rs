# ev3dev_rs

### High level pybricks-like rust bindings for e3dev.

# Usage

```rust
extern crate ev3dev_rs;
extern crate tokio;

use ev3dev_rs::Ev3Result;
use ev3dev_rs::pupdevices::{GyroSensor, Motor, ColorSensor};
use ev3dev_rs::robotics::DriveBase;
use ev3dev_rs::parameters::{MotorPort, SensorPort, Direction};

#[tokio::main]
async fn main() -> Ev3Result<()> {
    let left_motor = Motor::new(MotorPort::OutA, Direction::Clockwise)?;
    let right_motor = Motor::new(MotorPort::OutD, Direction::Clockwise)?;

    let gyro_sensor = GyroSensor::new(SensorPort::In1)?;
    let color_sensor = ColorSensor::new(SensorPort::In4)?;

    println!("Detected color: {}", color_sensor.color()?);

    let drive = DriveBase::new(&left_motor, &right_motor, 62.4, 130.5)?.with_gyro(&gyro_sensor)?;

    drive.use_gyro(true)?;

    drive.straight(500).await?;
    drive.turn(90).await?;
    drive.curve(600, 90).await?;
    drive.veer(600, 500).await?;

    Ok(())
}
```

Please see [ev3dev-rs-template] for an example project and detailed usage instructions.

[ev3dev-rs-template]: https://github.com/shaggysa/ev3dev-rs-template

# Supported features

### Motors:

- EV3 and NXT Motors

### Sensors

* EV3 Color Sensor
* EV3 Gyro Sensor
* EV3 Infrared Sensor
* EV3 Touch Sensor
* EV3 Ultrasonic Sensor

# Unsupported features

Hitechnic sensors, NXT sensors, and hub functions (Buttons, Leds, Screen, Sound) are not currently supported.

If you want support for a specific feature, please open an Issue.