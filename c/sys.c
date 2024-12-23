#include <time.h>

int printf(const char *, ...);
void *malloc(unsigned long);
void free(void *);
long long __alloc_count = 0;
void _exit(int);

void *alloc(unsigned long size) {
	void *ptr = malloc(size);
	// printf("malloc %p (%lu)\n", ptr, size);
#ifdef TEST
	__atomic_fetch_add(&__alloc_count, 1, __ATOMIC_SEQ_CST);
#endif	// TEST
	return ptr;
}

void release(void *ptr) {
	// printf("free %p\n", ptr);
#ifdef TEST
	__atomic_fetch_sub(&__alloc_count, 1, __ATOMIC_SEQ_CST);
#endif	// TEST
	free(ptr);
}

unsigned long long getmicros() {
	struct timespec now;
	clock_gettime(CLOCK_REALTIME, &now);
	return (unsigned long long)((__int128_t)now.tv_sec * 1000000) +
	       (unsigned long long)(now.tv_nsec / 1000);
}

int sleep_millis(unsigned long long millis) {
	struct timespec ts;
	ts.tv_sec = millis / 1000;
	ts.tv_nsec = (millis % 1000) * 1000000;
	int ret = nanosleep(&ts, 0);
	return ret;
}

long long getalloccount() { return __alloc_count; }

