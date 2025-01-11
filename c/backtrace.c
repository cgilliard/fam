#include <stdio.h>
#include <stdlib.h>
#ifdef __APPLE__
#include <dlfcn.h>
#include <mach/mach.h>
#endif	// __APPLE__

extern long long __alloc_count;
int backtrace(void **array, int capacity);
char **backtrace_symbols(void **array, int capacity);

#define u64 unsigned long long
#define MAX_BACKTRACE_ENTRIES 128

int getpagesize();
#ifndef PAGE_SIZE
#define PAGE_SIZE (getpagesize())
#endif	// PAGE_SIZE

unsigned long long cstring_len(const char *X) {
	const char *Y = X;
	while (*X) X++;
	return X - Y;
}

void cstring_cat_n(char *X, char *Y, unsigned long long n) {
	X += cstring_len(X);
	while (n-- && *Y) {
		*X = *Y;
		X++;
		Y++;
	}
	*X = 0;
}

const char *backtrace_full() {
	void *array[MAX_BACKTRACE_ENTRIES];
	int size = backtrace(array, MAX_BACKTRACE_ENTRIES);
	char **strings = backtrace_symbols(array, size);
	char *ret = malloc(1024 * 1024);
	if (ret == NULL) return NULL;
#ifdef TEST
	__alloc_count++;
#endif	// TEST
	bool term = false;
	int len_sum = 0;
	for (int i = 0; i < size; i++) {
		char address[256];
#ifdef __linux__
		int len = strlen(strings[i]);
		int last_plus = -1;

		while (len > 0) {
			if (strings[i][len] == '+') {
				last_plus = len;
				break;
			}
			len--;
		}
		if (last_plus > 0) {
			byte *addr = strings[i] + last_plus + 1;
			int itt = 0;
			while (addr[itt]) {
				if (addr[itt] == ')') {
					addr[itt] = 0;
					break;
				}
				itt++;
			}
			u64 address = strtoull(addr, 16);
			address -= 8;

			char command[256];
			snprintf(command, sizeof(command),
				 "addr2line -f -e ./bin/test_fam %llx",
				 address);

			void *fp = popen(command, "r");
			char buffer[128];
			while (fgets(buffer, sizeof(buffer), fp) != NULL) {
				int len = strlen(buffer);
				if (strstr(buffer, ".c:")) {
					len_sum += len;
					if (len_sum >= 4 * PAGE_SIZE) break;
					if (term) {
						if (buffer[len - 1] == '\n')
							buffer[len - 1] = 0;
						cstring_cat_n(ret, buffer,
							      strlen(buffer));
						i = size;
						break;
					}
					cstring_cat_n(ret, buffer,
						      strlen(buffer));
				} else if (cstring_is_alpha_numeric(buffer)) {
					if (len && buffer[len - 1] == '\n') {
						len--;
						buffer[len] = ' ';
					}

					len_sum += len;
					if (len_sum >= 4 * PAGE_SIZE) break;
					cstring_cat_n(ret, buffer,
						      strlen(buffer));
					if (!cstring_compare(buffer, "main ")) {
						term = true;
					}
				}
			}

			pclose(fp);
		}
#elif defined(__APPLE__)
		Dl_info info;
		dladdr(array[i], &info);
		u64 addr = 0x0000000100000000 + info.dli_saddr - info.dli_fbase;
		u64 offset = (u64)array[i] - (u64)info.dli_saddr;
		addr += offset;
		addr -= 4;
		snprintf(address, sizeof(address), "0x%llx", addr);
		char command[256];
		snprintf(command, sizeof(command),
			 "atos -fullPath -o ./bin/test_fam -l 0x100000000 %s",
			 address);
		void *fp = popen(command, "r");
		char buffer[128];

		while (fgets(buffer, sizeof(buffer), fp) != NULL) {
			int len = strlen(buffer);
			len_sum += len;
			if (len_sum >= 4 * PAGE_SIZE) break;
			if (strstr(buffer, "main ") == buffer) {
				if (len && buffer[len - 1] == '\n')
					buffer[len - 1] = 0;
				cstring_cat_n(ret, buffer, strlen(buffer));
				i = size;
				break;
			}
			cstring_cat_n(ret, buffer, strlen(buffer));
		}
		pclose(fp);
#else
		println("WARN: Unsupported OS: cannot build backtraces");
#endif
	}

	if (strings && size) free(strings);
	return ret;
}

