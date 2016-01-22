/* vim: set sw=2 expandtab tw=80: */

#include <string.h>
#include <stdio.h>
#include <stdlib.h>
#include <unistd.h>

#include <tock.h>
#include <firestorm.h>

void read_accelerometer_once() {
  int err;
  char buf[64];
  // read from the accelerometer
  err = FXOS8700CQ_accel_enable();
  if (err < 0) {
    snprintf(buf, 64, "Error(%d): Failed to enable accelerometer.\r\n", err);
    putstr(buf);
    return;
  }
  snprintf(buf, 64, "Initialized accelerometer!\r\n");
  putstr(buf);
  putstr("Reading from accelerometer...\n");

  accel_result_t accel;
  err = FXOS8700CQ_accel_read(&accel);
  if (err < 0) {
    snprintf(buf, 64, "Error(%d) reading from accelerometer.\r\n", err);
    putstr(buf);
    return;
  }
  sprintf(buf, "accel -> x %d y %d z %d\r\n", accel.x, accel.y, accel.z);
  putstr(buf);
}

void read_magnetometer_once() {
  int err;
  char buf[64];
  // read from the magnetometer
  err = FXOS8700CQ_magnet_enable();
  if (err < 0) {
    snprintf(buf, 64, "Error(%d): Failed to enable magnetometer.\r\n", err);
    putstr(buf);
    return;
  }
  snprintf(buf, 64, "Initialized magnetometer!\r\n");
  putstr(buf);
  putstr("Reading from magnetometer...\n");

  magnet_result_t magnet_vals;
  err = FXOS8700CQ_magnet_read(&magnet_vals);
  if (err < 0) {
    snprintf(buf, 64, "Error(%d) reading from magnetometer.\r\n", err);
    putstr(buf);
    return;
  }
  sprintf(buf, "magnetometer -> x %d y %d z %d\r\n", magnet_vals.x, magnet_vals.y, magnet_vals.z);
  putstr(buf);
}

void main() {
  putstr("Welcome to Tock's FXOS8700CQ test app in C\n");
  // currently only supports one of the two
  //read_accelerometer_once();
  read_magnetometer_once();

}

