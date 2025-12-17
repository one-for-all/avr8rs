# Arduino simulator (in Rust)

Simulates the Arduino (cpu along with peripherals), so that I can execute Arduino code without having an Arduino.

## Getting Started

### Testing

Run all tests in the lib: 

`cargo test --lib -- --nocapture`

### Run examples

Run the stepper motor example:

First, update the `stepper/stepper.ino` file as you wish, and then:

`./build_and_run stepper`

## References

This project (apart from the motor and encoder simulation) is largely a Rust rewrite of the AVR8js project: [AVR8js](https://github.com/wokwi/avr8js)
