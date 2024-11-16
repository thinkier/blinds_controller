#include <Arduino.h>
#include "Commander.h"
#include "Registers.h"

// X Z Y E
uint8_t PIN_INT[4] =  { 4, 25, 3, 16};
uint8_t PIN_ENA[4] =  {12,  2, 7, 15};
uint8_t PIN_STP[4] =  {11, 19, 6, 14};
uint8_t PIN_DIR[4] =  {10, 28, 5, 13};

uint8_t PIN_NEOPIXEL = 24;

Commander Controller(Serial);

// Initializing communications
void setup() {
  Serial.begin(115200);
  Controller.begin();
}

// Initialize peripherals
void setup1() {
}

// Communications loop managing incomings & outgoings
void loop() {
  if (Controller.hasCommand()) {
    Command cmd = Controller.readCommand();

    if (cmd.action == "CLEAR") {
      for (uint8_t i = 0; i < 4; i++) {
        Registers.interrupt[i] = false;
        Registers.steps[i] = 0;
      }
      Controller.writeResponse("OK");
    } else {
      Controller.writeResponse("UNKNOWN");
    }
  }
}

// Timings loop managing realtime signals such as bit-banged PWM
void loop1() {
  for (uint8_t i = 0; i < 4; i++) {
    unsigned long time = micros();

    bool enable = Registers.enable[i];
    bool interrupted = digitalRead(PIN_INT[i]);
    int32_t steps = Registers.steps[i];

    digitalWrite(PIN_ENA[i], !enable);
    if (steps == 0) {
    } else if (interrupted) {
      Registers.interrupt[i] = true;
    } else if (enable) {
      digitalWrite(PIN_DIR[i], steps < 0);
      if (steps > 0) {
        Registers.steps[i]--;
      } else {
        Registers.steps[i]++;
      }
      digitalWrite(PIN_STP[i], HIGH);
    }

    delayMicroseconds(time + 50 - micros()); // Synthetic cap of 5000 oscillations per channel-second
    digitalWrite(PIN_STP[i], LOW);
  }
}
