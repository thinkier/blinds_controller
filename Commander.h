#ifndef SerialIO_H
#define SerialIO_H

#include <Arduino.h>
#include <vector>

struct Command {
  String action;
  std::vector<int32_t> params;
};

class Commander {
  private:
    Stream* inner;

  public:
    Commander(Stream &param) {
      inner = &param;
    }

    void begin();
    bool hasCommand();
    Command readCommand();
    void writeResponse(String resp);
};

#endif