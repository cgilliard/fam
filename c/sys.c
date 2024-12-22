#include <time.h>

int printf(const char *, ...);
void *malloc(unsigned long);
void free(void *);
long long __alloc_count = 0;
void _exit(int);

void *alloc(unsigned long size) {
	void *ptr = malloc(size);
	//	printf("malloc %p (%lu)\n", ptr, size);
	__alloc_count++;
	return ptr;
}

void release(void *ptr) {
	//	printf("free %p\n", ptr);
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
