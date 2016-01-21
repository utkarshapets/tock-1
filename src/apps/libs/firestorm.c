#include <stdlib.h>
#include <string.h>
#include <unistd.h>
#include <firestorm.h>
#include <tock.h>

int gpio_enable(unsigned int pin) {
  return command(1, 0, pin);
}

int gpio_set(unsigned int pin) {
  return command(1, 2, pin);
}

int gpio_clear(unsigned int pin) {
  return command(1, 3, pin);
}

static CB_TYPE putstr_cb(int _x, int _y, int _z, void* str) {
  free(str);
  return PUTSTR;
}

void putnstr(const char *str, size_t len) {
  char* buf = (char*)malloc(len * sizeof(char));
  strncpy(buf, str, len);
  putnstr_async(buf, len, putstr_cb, buf);
  wait_for(PUTSTR);
}

void putnstr_async(const char *str, size_t len, subscribe_cb cb, void* userdata) {
  allow(0, 1, (void*)str, len);
  subscribe(0, 1, cb, userdata);
}

void putstr(const char *str) {
  putnstr(str, strlen(str));
}

static CB_TYPE read_tmp006_cb(int r0, int r1, int r2, void* ud) {
  int16_t *res = (int16_t*)ud;
  *res = (int16_t)r0 / 32;

  return READTMP;
}


int tmp006_read(int16_t *temperature) {
  int error = tmp006_read_async(read_tmp006_cb, (void*)temperature);
  if (error < 0) {
    return error;
  }
  wait_for(READTMP);
  return 0;
}

int tmp006_read_async(subscribe_cb cb, void* userdata) {
  return subscribe(2, 0, cb, userdata);
}

int tmp006_enable() {
  return command(2, 0, 0);
}

/** FXOS8700CQ: magnetometer, accelerometer **/

int accel_enable() {
  return command(3, 1, 0);
}

int magnet_enable() {
  return command(3, 2, 0);
}

static CB_TYPE read_accel_cb(int r0, int r1, int r2, void* ud) {
  accel_result_t *res = (accel_result_t*)ud;
  res->x = r0;
  res->y = r1;
  res->z = r2;
  return READACCEL;
}

int accel_read(accel_result_t *res) {
  int error = accel_read_async(read_accel_cb, (void*)res);
  if (error < 0) {
    return error;
  }
  wait_for(READACCEL);
  return 0;
}

int accel_read_async(subscribe_cb cb, void* userdata) {
    return subscribe(3, 1, cb, userdata);
}

static CB_TYPE read_magnet_cb(int r0, int r1, int r2, void* ud) {
  magnet_result_t *res = (magnet_result_t*)ud;
  res->x = r0;
  res->y = r1;
  res->z = r2;
  return READMAGNET;
}

int magnet_read(magnet_result_t *res) {
  int error = magnet_read_async(read_magnet_cb, (void*)res);
  if (error < 0) {
    return error;
  }
  wait_for(READMAGNET);
  return 0;
}

int magnet_read_async(subscribe_cb cb, void* userdata) {
    return subscribe(3, 2, cb, userdata);
}
