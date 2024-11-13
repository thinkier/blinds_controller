#ifndef SerialIO_H
#define SerialIO_H

#include <Arduino.h>
#include <vector>

struct Command {
  String action;
  std::vector<int32_t> params;
};

class SerialIO {
  public:
    static void begin();
    static bool hasCommand();
    static Command readCommand();
    static void writeResponse(String resp);
};

#endif