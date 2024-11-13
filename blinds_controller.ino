#include <Arduino.h>
#include "SerialIO.h"

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
  }
}

// Timings loop managing realtime signals such as bit-banged PWM
void loop1() {
}
