#include <AccelStepper.h>

// Define the stepper motor connections
#define dirPin 2  // Direction
#define stepPin 3 // Step

// Create an instance of the AccelStepper class
// AccelStepper stepper(AccelStepper::DRIVER, stepPin, dirPin, 0, 0, false);

#define stepsPerRevolution 200 * 8 // 200 steps per revolution * 8 microsteps

long _targetPos = 3;

float _speed = 0.0;
unsigned long _lastStepTime = 0;
long _currentPos = 0;
unsigned long _stepInterval = 1;

long _n = 0; // the step counter for speed calculations


void step() {
    digitalWrite(stepPin, HIGH);
    delayMicroseconds(100);
    digitalWrite(stepPin, LOW);
}

bool run() {
    if (runSpeed())
        computeNewSpeed();
    return _speed != 0.0 || distanceToGo() != 0;
}

bool runSpeed() {
    if (!_stepInterval)
        return false;

    unsigned long time = micros();
    if (time - _lastStepTime >= _stepInterval) {
        _currentPos += 1;
        step();
        _lastStepTime = time;
        return true;
    }
    return false;
}

void computeNewSpeed() {
    // long distanceTo = distanceToGo(); // +ve is clockwise from current location
    // long stepsToStop = (long)((_speed * _speed) / (2.0 * _acceleration));

    // if (distanceTo == 0 && stepsToStop <= 1)
    // {
    //     _stepInterval = 0;
    //     _speed = 0.0;
    //     _n = 0;
    // }

    // if (_n > 0) {
    //     if ((stepsToStop >= distanceTo))
    //         _n = -stepsToStop; // start deceleration
    // } else if (_n < 0) {
    //     if ((stepsToStop < distanceTo))
    //         _n = -_n; // start acceleration
    // }


}

long distanceToGo() {
    return _targetPos - _currentPos;
}

void setup()
{
    pinMode(stepPin, OUTPUT);

    while(run())
        ;
}

void loop()
{

}
