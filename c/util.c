typedef unsigned long size_t;
int snprintf(char *s, size_t n, const char *format, ...);

int f64_to_str(double d, char *buf, unsigned long long capacity) {
	return snprintf(buf, capacity, "%.5f", d);
}

void ptr_add(void **p, long long v) { *p = (void *)((char *)*p + v); }
