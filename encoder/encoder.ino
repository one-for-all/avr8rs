#include <AS5600.h>
#include <Wire.h>

// Create an instance of the AS5600 class
AMS_5600 ams5600;

void setup() {
    Serial.begin(115200); // Start the serial communication
    Wire.begin();         // Start the I2C communication

    while (!ams5600.detectMagnet()) {
        Serial.println("[AS5600] Waiting for magnet...");
        delay(1000); // Wait for the magnet to be detected
    }

    // Print the current magnitude of the magnet
    Serial.println("[AS5600] Current magnitude: ");
    Serial.println(ams5600.getMagnitude());

}

void loop() {
    // while (!ams5600.detectMagnet())
    // {
    //     Serial.println("[AS5600] Waiting for magnet...");
    //     delay(1000); // Wait for the magnet to be detected
    // }
}
