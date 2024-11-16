#include "Commander.h"

void Commander::begin() {
  this->inner->setTimeout(50);
}

bool Commander::hasCommand() {
  return this->inner->available() > 0;
}

Command Commander::readCommand() {
  String action = this->inner->readStringUntil(' ');
  action.toUpperCase();
  std::vector<int32_t> params;
  
  uint8_t i = 0;
  while (true) {
    switch (this->inner->peek()) {
      case '\r':
      case '\n': {
        this->inner->readStringUntil('\n');

        Command cmd = {
          .action = action,
          .params = params
        };

        return cmd;
      }
      case ' ': break;
      default: params.push_back(this->inner->parseInt(SKIP_NONE));
    }
  }
}

void Commander::writeResponse(String resp) {
  this->inner->println(resp);
}
