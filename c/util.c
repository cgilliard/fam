#include <stdio.h>

int f64_to_str(double d, char *buf, unsigned long long capacity) {
	return snprintf(buf, capacity, "%.5f", d);
}

void ptr_add(void **p, long long v) { *p = (void *)((char *)*p + v); }
