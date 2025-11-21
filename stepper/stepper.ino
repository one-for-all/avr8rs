// #include <AccelStepper.h>

// // Define the stepper motor connections
// #define dirPin 2  // Direction
// #define stepPin 3 // Step

// // Create an instance of the AccelStepper class
// AccelStepper stepper(AccelStepper::DRIVER, stepPin, dirPin, 0, 0, false);

// #define stepsPerRevolution 200 * 8 // 200 steps per revolution * 8 microsteps

// float _speed = 1.0;
// float _acceleration = 1.0;

// float _cn = 1.0;

void setup()
{
    DDRB |= 0b00001000;

    float n = 3.14 * 2;
    Serial.begin(9600);
    Serial.print("world, ");
    Serial.print("hello!: ");
    Serial.print(n);
    Serial.flush();
    // DDRD |= 0b00001000;

    // // Set the maximum speed and acceleration
    // stepper.setMaxSpeed(20000);
    // stepper.setAcceleration(10000);
    // // stepper.setMaxSpeed(20000);
    // // stepper.setAcceleration(200);

    // // Set the enable pin for the stepper motor driver and
    // // invert it because we are using a DRV8825 board with an
    // // active-low enable signal (LOW = enabled, HIGH = disabled)
    // stepper.setEnablePin(5);
    // stepper.setPinsInverted(false, false, true);

    // // Set the initial position
    // stepper.setCurrentPosition(0);

    // // Enable the motor outputs
    // stepper.enableOutputs();

    // stepper.runToNewPosition(40);

    // float result = (2.0 - _cn);

    // // DDRB = 0b11111111;

    // // float result = 1.0;
    // uint32_t* bits = (uint32_t*)&result;
    // DDRC = (*bits) >> 24;
    // PINC = (*bits) >> 16;
    // DDRD = (*bits) >> 8;
    // PIND = (*bits);

    // uint8_t x = result;
    // if (x == 1) {
    //     DDRB |= 0b00001000;
    //     PORTB |= 0b00001000;
    //     PORTB &= ~0b00001000;
    // }
    // _cn += 0.0;
}

void loop()
{
}
