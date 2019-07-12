/*
 * Hacky include file so our library compiles.
 */

#ifndef ARDUINO_H

#define ARDUINO_H

#include <stdint.h>
#include <stdlib.h>
#include <math.h>
#include <assert.h>

#define HIGH 1
#define LOW 0

#define OUTPUT 0

typedef bool boolean;

unsigned long MICROS = 0;

unsigned long micros() {
    return MICROS;
}

void digitalWrite(int pin, int state) {
    assert(false);
}

void pinMode(int pin, int state) {
    assert(false);
}

void delayMicroseconds(unsigned long micros) {
    assert(false);
}

float constrain(float value, float min, float max) {
    return value < min ? min : value > max ? max : value;
}

float max(float a, float b) {
    return fmax(a, b);
}


#endif // ARDUINO_H