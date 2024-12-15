typedef long long i64;
typedef unsigned long long u64;

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
	return __atomic_compare_exchange_n(ptr, expect, desired, 0,
					   __ATOMIC_RELEASE, __ATOMIC_RELAXED);
}

int ctzl(u64 v) { return __builtin_ctzl(v); }
