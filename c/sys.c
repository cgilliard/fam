#include <sys/mman.h>

int getpagesize();

void *map(unsigned long long pages) {
	void *ret = mmap(0, getpagesize() * pages, PROT_READ | PROT_WRITE,
			 MAP_PRIVATE | MAP_ANONYMOUS, -1, 0);
	if (ret == MAP_FAILED) return 0;
	return ret;
}

void unmap(void *ptr, unsigned long long pages) {
	munmap(ptr, getpagesize() * pages);
}
