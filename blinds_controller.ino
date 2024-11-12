#include <Arduino.h>
#include "SerialIO.h"

// Initializing communications
void setup() {
  FromPi.begin();
}

// Initialize peripherals
void setup1() {
  Serial.begin(115200);
}

// Communications loop managing incomings & outgoings
void loop() {
  Serial.print("Thread 0\n");

  if (FromPi.hasCommand()) {

  }
}

// Timings loop managing realtime signals such as bit-banged PWM
void loop1() {
  Serial.print("Thread 1\n");
}
