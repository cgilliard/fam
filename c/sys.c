#include <sys/mman.h>
#include <time.h>

int getpagesize();
void _exit(int);
int printf(const char *fmt, ...);

void *map(unsigned long long pages) {
	void *ret = mmap(0, getpagesize() * pages, PROT_READ | PROT_WRITE,
			 MAP_PRIVATE | MAP_ANONYMOUS, -1, 0);
	if (ret == MAP_FAILED) return 0;
	return ret;
}

void unmap(void *ptr, unsigned long long pages) {
	if (munmap(ptr, getpagesize() * pages)) {
		printf("Could not unmap address %p [pages=%llu]\n", ptr, pages);
		_exit(-1);
	}
}

unsigned long long getmicros() {
	struct timespec now;
	clock_gettime(CLOCK_REALTIME, &now);
	return (unsigned long long)now.tv_sec * (__int128_t)1e9 +
	       (unsigned long long)(now.tv_nsec / 1000);
}
