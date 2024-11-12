#ifndef SerialIO_H
#define SerialIO_H

#include <Arduino.h>
#include <vector>

#define FromPi SerialIO()

struct Command {
  String action;
  std::vector<int32_t> params;
};

// Uses singleton pattern from https://stackoverflow.com/a/1008289/8835688
class SerialIO {
  public:
    void begin();
    bool hasCommand();
    Command readCommand();
    void writeResponse(String resp);
};

#endif