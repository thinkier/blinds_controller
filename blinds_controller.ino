#include <Arduino.h>
#include "SerialIO.h"
#include "Registers.h"

// X Z Y E
uint8_t PIN_INT[4] =  { 4, 25, 3, 16};
uint8_t PIN_ENA[4] =  {12,  2, 7, 15};
uint8_t PIN_STP[4] =  {11, 19, 6, 14};
uint8_t PIN_DIR[4] =  {10, 28, 5, 13};

uint8_t PIN_NEOPIXEL = 24;

// Initializing communications
void setup() {
  SerialIO::begin();
}

// Initialize peripherals
void setup1() {
  Serial.begin(115200);
}

// Communications loop managing incomings & outgoings
void loop() {
  if (SerialIO::hasCommand()) {
    Command cmd = SerialIO::readCommand();

    if (cmd.action == "CLEAR") {
      for (uint8_t i = 0; i < 4; i++) {
        Registers.interrupt[i] = false;
        Registers.steps[i] = 0;
      }
      SerialIO::writeResponse("OK");
    } else {
      SerialIO::writeResponse("UNKNOWN");
    }
  }
}

// Timings loop managing realtime signals such as bit-banged PWM
void loop1() {
}
