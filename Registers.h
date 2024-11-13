#ifndef States_H
#define States_H

#include <atomic>

struct States {
  atomic_bool[4] enable;
  atomic_bool[4] interrupt;
  atomic_int32_t[4] steps;
};

#endif