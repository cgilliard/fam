#include <fcntl.h>
#include <stdio.h>
#include <sys/mman.h>

#define i64 long long
#define byte unsigned char
#define aload(ptr) __atomic_load_n(ptr, __ATOMIC_ACQUIRE)
#define astore(ptr, value) __atomic_store_n(ptr, value, __ATOMIC_RELEASE)
#define cas(ptr, expect, desired)                                              \
	__atomic_compare_exchange_n(ptr, expect, desired, 0, __ATOMIC_RELEASE, \
				    __ATOMIC_RELAXED)

// for macos
#ifndef O_DIRECT
#define O_DIRECT 0
#endif	// O_DIRECT

i64 getpagesize();
#define PAGE_SIZE (getpagesize())
int ftruncate(int fd, off_t size);
int fsync(int fd);
int close(int fd);
off_t lseek(int fd, off_t offset, int whence);
void _exit(int);
void sched_yield();

int _gfd = -1;
i64 cur_file_size = -1;
int fupdate = 0;

int check_size(i64 id) {
	int ret = 0;
	int gfd = aload(&_gfd);
	if (gfd == -1 || id < 0) return -1;

	int target = 0;
	do {
		int v = aload(&fupdate);
		if (v) sched_yield();
	} while (!cas(&fupdate, &target, 1));

	i64 file_size = aload(&cur_file_size);
	if (file_size == -1) {
		astore(&fupdate, 0);
		return -1;
	}

	if ((1 + id) * PAGE_SIZE > file_size) {
		ret = ftruncate(gfd, (1 + id) * PAGE_SIZE);
		if (!ret) astore(&cur_file_size, (1 + id) * PAGE_SIZE);
	}

	astore(&fupdate, 0);
	return ret;
}

void *fmap(i64 id, long long blocks) {
	if (blocks < 1 || check_size(id + (blocks - 1))) return NULL;
	void *ret = mmap(NULL, PAGE_SIZE * blocks, PROT_READ | PROT_WRITE,
			 MAP_SHARED, aload(&_gfd), id * PAGE_SIZE);
	if (ret == MAP_FAILED) return NULL;

	// trigger page fault
	for (int i = 0; i < blocks; i++) {
		volatile byte *p = (byte *)(ret + PAGE_SIZE * i);
		*p = *p;
	}

	return ret;
}
void unmap(void *addr, long long pages) {
	if (munmap(addr, PAGE_SIZE * pages)) {
		perror("munmap failed");
		_exit(-1);
	}
}
int flush() {
	int gfd = aload(&_gfd);
	if (gfd == -1) return -1;
	return fsync(gfd);
}
i64 fsize() { return aload(&cur_file_size); }
void init(const char *path) {
	if (aload(&_gfd) != -1) {
		fprintf(stderr, "Already initialized!\n");
		_exit(-1);
	}
	int gfd = open(path, O_DIRECT | O_RDWR);
	int create = 0;
	if (gfd == -1) {
		create = 1;
		gfd = open(path, O_DIRECT | O_RDWR | O_CREAT, 0600);
	}
	if (gfd == -1) {
		fprintf(stderr, "Could not open file [%s]\n", path);
		_exit(-1);
	}

#ifdef __APPLE__
	if (fcntl(gfd, F_NOCACHE, 1)) {
		fprintf(stderr, "Could not disable cache for file [%s]", path);
		_exit(-1);
	}
#endif	// __APPLE__

	if (create)
		astore(&cur_file_size, 0);
	else
		astore(&cur_file_size, lseek(gfd, 0, SEEK_END));
	astore(&_gfd, gfd);
}
void shutdown(const char *opt_rem_file) {
	int gfd = aload(&_gfd);
	if (gfd != -1) {
		if (close(gfd)) perror("close");
		astore(&_gfd, -1);
		if (opt_rem_file) {
			remove(opt_rem_file);
		}
	}
}
