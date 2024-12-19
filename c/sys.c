#include <stdio.h>
#include <sys/mman.h>
#include <time.h>

int getpagesize();
void _exit(int);

void *map(unsigned long long pages) {
	void *ret = mmap(0, getpagesize() * pages, PROT_READ | PROT_WRITE,
			 MAP_PRIVATE | MAP_ANONYMOUS, -1, 0);
	if (ret == MAP_FAILED) return 0;
	// fprintf(stderr, "map %llu %p\n", pages, ret);
	return ret;
}

void unmap(void *ptr, unsigned long long pages) {
	// fprintf(stderr, "unmap %llu %p\n", pages, ptr);
	if (munmap(ptr, getpagesize() * pages)) {
		fprintf(stderr, "Could not unmap address %p [pages=%llu]\n",
			ptr, pages);
		_exit(-1);
	}
}

unsigned long long getmicros() {
	struct timespec now;
	clock_gettime(CLOCK_REALTIME, &now);
	return (unsigned long long)((__int128_t)now.tv_sec * 1000000) +
	       (unsigned long long)(now.tv_nsec / 1000);
}
