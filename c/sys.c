#include <sys/mman.h>
#include <sys/time.h>
#include <time.h>
#include <util.h>

int getpagesize();
void _exit(int code);
int write(int fd, const char *buf, unsigned long long len);

void *map(unsigned long long pages) {
	return mmap(0, getpagesize() * pages, PROT_READ | PROT_WRITE,
		    MAP_PRIVATE | MAP_ANONYMOUS, -1, 0);
}

void unmap(void *ptr, unsigned long long pages) {
	munmap(ptr, getpagesize() * pages);
}

int os_sleep(unsigned long long millis) {
	struct timespec ts;
	ts.tv_sec = millis / 1000;
	ts.tv_nsec = (millis % 1000) * 1000000;
	int ret = nanosleep(&ts, 0);
	return ret;
}

struct Nano {
	unsigned long long high;
	unsigned long long low;
};

struct Nano getnanos() {
	struct timespec now;
	clock_gettime(CLOCK_REALTIME, &now);

	__uint128_t nanos =
	    (__uint128_t)now.tv_sec * 1000000000 + (__uint128_t)now.tv_nsec;

	struct Nano result;
	result.high = (unsigned long long)(nanos >> 64);  // High 64 bits
	result.low = (unsigned long long)(nanos);	  // Low 64 bits

	return result;
}

static void check_arch(char *type, int actual, int expected) {
	char buf[30] = {};
	if (actual != expected) {
		write(2, "'", 1);
		write(2, type, cstring_len(type));
		write(2, "' must be ", 10);
		cstring_itoau64(expected, buf, 10, 30);
		write(2, buf, cstring_len(buf));
		write(2, " bytes. It is ", 14);
		cstring_itoau64(actual, buf, 10, 30);
		write(2, buf, cstring_len(buf));
		write(2, " bytes. Arch invalid!\n", 23);
		_exit(-1);
	}
}

#define arch(type, expected) check_arch(#type, sizeof(type), expected)

void __attribute__((constructor)) __check_sizes() {
	char buf[30] = {};
	arch(int, 4);
	arch(long long, 8);
	arch(unsigned long long, 8);
	arch(unsigned long, 8);
	arch(__uint128_t, 16);
	arch(char, 1);
	arch(unsigned char, 1);
	arch(float, 4);
	arch(double, 8);
	if (__SIZEOF_SIZE_T__ != 8) {
		write(2, "size_t must be 8 bytes. It is ", 30);
		cstring_itoau64(__SIZEOF_SIZE_T__, buf, 10, 30);
		write(2, buf, cstring_len(buf));
		write(2, " bytes. Arch invalid.\n", 22);
		_exit(-1);
	}

	// little endian check
	int test = 0x1;
	if (*(unsigned char *)&test != 0x1) {
		write(2, "Big endian is not supported!\n", 29);
		_exit(-1);
	}
}

