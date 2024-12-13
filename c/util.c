typedef long long i64;
typedef unsigned long long u64;

u64 cstring_len(const char *X) {
	const char *Y = X;
	while (*X) X++;
	return X - Y;
}

void atomic_store_i64(i64 *ptr, i64 value) {
	__atomic_store_n(ptr, value, __ATOMIC_RELEASE);
}
i64 atomic_load_i64(i64 *ptr) { return __atomic_load_n(ptr, __ATOMIC_ACQUIRE); }
i64 atomic_fetch_add_i64(i64 *ptr, i64 value) {
	return __atomic_fetch_add(ptr, value, __ATOMIC_SEQ_CST);
}
i64 atomic_fetch_sub_i64(i64 *ptr, i64 value) {
	return __atomic_fetch_sub(ptr, value, __ATOMIC_SEQ_CST);
}
