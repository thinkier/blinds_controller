#include "SerialIO.h"

void SerialIO::begin() {
  Serial1.begin(115200);
  Serial1.setTimeout(50);
}

bool SerialIO::hasCommand() {
  return Serial1.available() > 0;
}

Command SerialIO::readCommand() {
  String action = Serial1.readStringUntil(' ');
  action.toUpperCase();
  std::vector<int32_t> params;
  
  uint8_t i = 0;
  while (true) {
    switch (Serial1.peek()) {
      case '\r':
      case '\n': {
        Serial1.readStringUntil('\n');

        Command cmd = {
          .action = action,
          .params = params
        };

        return cmd;
      }
      case ' ': break;
      default: params.push_back(Serial1.parseInt(SKIP_NONE));
    }
  }
}

void SerialIO::writeResponse(String resp) {
  Serial1.println(resp);
}
