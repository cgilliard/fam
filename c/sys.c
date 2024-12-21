#include <time.h>

void *malloc(unsigned long);
void free(void *);
long long __alloc_count = 0;

void *alloc(unsigned long size) {
	void *ret = malloc(size);
	__alloc_count++;
	return ret;
}

void release(void *ptr) {
	__alloc_count--;
	free(ptr);
}

unsigned long long getmicros() {
	struct timespec now;
	clock_gettime(CLOCK_REALTIME, &now);
	return (unsigned long long)((__int128_t)now.tv_sec * 1000000) +
	       (unsigned long long)(now.tv_nsec / 1000);
}

long long getalloccount() { return __alloc_count; }
