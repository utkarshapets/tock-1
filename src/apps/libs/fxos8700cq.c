#include <firestorm.h>
#include <tock.h>
#include <fxos8700cq.h>

driver_t FXOS8700CQ = { num: 4};

int FXOS8700CQ_accel_enable() {
  return command(FXOS8700CQ, 1, 0);
}

int FXOS8700CQ_magnet_enable() {
  return command(FXOS8700CQ, 2, 0);
}

static CB_TYPE FXOS8700CQ_read_accel_cb(int r0, int r1, int r2, void* ud) {
  accel_result_t *res = (accel_result_t*)ud;
  res->x = r0;
  res->y = r1;
  res->z = r2;
  return READACCEL;
}

int FXOS8700CQ_accel_read_sync(accel_result_t *res) {
  int error = FXOS8700CQ_accel_read_async(FXOS8700CQ_read_accel_cb, (void*)res);
  if (error < 0) {
    return error;
  }
  wait_for(READACCEL);
  return 0;
}

int FXOS8700CQ_accel_read_async(subscribe_cb cb, void* userdata) {
    return subscribe(FXOS8700CQ, 1, cb, userdata);
}

static CB_TYPE FXOS8700CQ_read_magnet_cb(int r0, int r1, int r2, void* ud) {
  magnet_result_t *res = (magnet_result_t*)ud;
  res->x = r0;
  res->y = r1;
  res->z = r2;
  return READMAGNET;
}

int FXOS8700CQ_magnet_read_sync(magnet_result_t *res) {
  int error = FXOS8700CQ_magnet_read_async(FXOS8700CQ_read_magnet_cb, (void*)res);
  if (error < 0) {
    return error;
  }
  wait_for(READMAGNET);
  return 0;
}

int FXOS8700CQ_magnet_read_async(subscribe_cb cb, void* userdata) {
    return subscribe(FXOS8700CQ, 2, cb, userdata);
}

