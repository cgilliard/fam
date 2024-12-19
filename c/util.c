#include <pthread.h>

#ifndef NULL
#define NULL (0(void *))
#endif	// NULL

typedef long long i64;
typedef unsigned long long u64;
typedef unsigned int u32;

int printf(const char *, ...);

u64 cstring_len(const char *X) {
	const char *Y = X;
	while (*X) X++;
	return X - Y;
}

void atomic_store_u64(u64 *ptr, u64 value) {
	__atomic_store_n(ptr, value, __ATOMIC_RELEASE);
}
u64 atomic_load_u64(u64 *ptr) { return __atomic_load_n(ptr, __ATOMIC_ACQUIRE); }
u64 atomic_fetch_add_u64(u64 *ptr, u64 value) {
	return __atomic_fetch_add(ptr, value, __ATOMIC_SEQ_CST);
}
u64 atomic_fetch_sub_u64(u64 *ptr, u64 value) {
	return __atomic_fetch_sub(ptr, value, __ATOMIC_SEQ_CST);
}

u64 cas_release(u64 *ptr, u64 *expect, u64 desired) {
	u64 ret = __atomic_compare_exchange_n(
	    ptr, expect, desired, 0, __ATOMIC_RELEASE, __ATOMIC_RELAXED);

	return ret;
}

u64 cas_seq(u64 *ptr, u64 *expect, u64 desired) {
	u64 ret = __atomic_compare_exchange_n(
	    ptr, expect, desired, 0, __ATOMIC_SEQ_CST, __ATOMIC_RELAXED);

	return ret;
}

int ctzl(u64 v) { return __builtin_ctzl(v); }

int ctz(u32 v) { return __builtin_ctz(v); }

int thread_create(pthread_t *th, void *(*start_routine)(void *), void *arg) {
	return pthread_create(th, NULL, start_routine, arg);
}
int thread_join(pthread_t *th) { return pthread_join(*th, NULL); }

size_t thread_handle_size() { return sizeof(pthread_t); }

int thread_detach(pthread_t *th) { return pthread_detach(*th); }
