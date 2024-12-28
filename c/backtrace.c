#define MAX_BACKTRACE_ENTRIES 128
#include <stdio.h>
#include <stdlib.h>
#include <string.h>

int backtrace(void **array, int capacity);
char **backtrace_symbols(void **array, int capacity);
#define PAGE_SIZE 4096

#define byte unsigned char
#define u64 unsigned long long
#ifdef __linux__
#include <stdbool.h>
#endif	// __linux__

#ifdef __APPLE__
#include <dlfcn.h>
#include <mach/mach.h>
#endif	// __APPLE__

const char *backtrace_full() {
	void *array[MAX_BACKTRACE_ENTRIES];
	int size = backtrace(array, MAX_BACKTRACE_ENTRIES);
	char **strings = backtrace_symbols(array, size);
	char *ret = malloc(4 * 4096);
	bool term = false;
	int len_sum = 0;
	for (int i = 0; i < size; i++) {
		char address[256];
#ifdef __linux__
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
				strncat(ret, buffer, strlen(buffer));
				i = size;
				break;
			}
			strncat(ret, buffer, strlen(buffer));
		}
		pclose(fp);
#else
		printf("WARN: Unsupported OS: cannot build backtraces\n");
#endif
	}

	if (strings && size) free(strings);
	return ret;
}
