/* vim: set sw=2 expandtab tw=80: */

#include <string.h>
#include <stdio.h>
#include <stdlib.h>
#include <unistd.h>

#include <tock.h>
#include <firestorm.h>

void main() {
  int err;
  int16_t temperature;
  char buf[64];

  putstr("Welcome to Tock in C (with libc)\r\n\
Initializing Accelerometer... ");

  err = accel_enable();
  if (err < 0) {
    snprintf(buf, 64, "Error(%d): Failed to enable accelerometer.\r\n", err);
    putstr(buf);
    return;
  } else {
    snprintf(buf, 64, "Initialized accelerometer! val %d", err);
    putstr(buf);
    return;

  }

}

