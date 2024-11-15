#ifndef States_H
#define States_H

#include <atomic>

using namespace std;

struct States {
  atomic_bool interrupt[4];
  atomic_bool enable[4];
  atomic_int32_t steps[4];
};

static States Registers;

#endif