// Copyright (c) 2024, The MyFamily Developers
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
//     http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

#include <limits.h>
#include <util.h>

void copy_bytes(byte *X, const byte *Y, u64 x) {
	while (x--) *(X)++ = *(Y)++;
}

void set_bytes(byte *X, byte x, u64 y) {
	while (y--) *(X++) = x;
}

u64 cstring_len(const char *X) {
	const char *Y = X;
	while (*X) X++;
	return X - Y;
}

int cstring_compare(const char *X, const char *Y) {
	while (*X == *Y && *X) {
		X++;
		Y++;
	}
	if (*X > *Y) return 1;
	if (*Y > *X) return -1;
	return 0;
}

int cstring_compare_n(const char *X, const char *Y, u64 n) {
	while (*X == *Y && n && *X) {
		n--;
		X++;
		Y++;
	}
	if (n == 0) return 0;
	if (*X > *Y) return 1;
	return -1;
}

const char *cstring_strstr(const char *X, const char *Y) {
	for (; *X; X++) {
		const char *tmpX = X, *tmpY = Y;
		while (*tmpX == *tmpY && *tmpX) {
			tmpX++;
			tmpY++;
		}
		if (!*tmpY) return X;
	}
	return 0;
}

void swap(byte *X, u64 x, u64 y) {
	byte t = X[x];
	X[x] = X[y];
	X[y] = t;
}

void reverse(byte *X, i64 length) {
	i64 start = 0, end = length - 1;
	while (start < end) swap(X, start++, end--);
}

u64 cstring_itoau64(u64 num, char *X, int base, u64 capacity) {
	u64 length = 0, i = 1;
	for (u64 num_copy = num; num_copy; num_copy /= base) length++;
	while (capacity && num) {
		u64 rem = num % base;
		X[length - i++] = (rem > 9) ? (rem - 10) + 'A' : rem + '0';
		num /= base;
		capacity--;
	}
	if (length == 0 && capacity) X[length++] = '0';
	if (length == 0) length++;
	return length;
}

u64 cstring_itoai64(i64 num, char *X, int base, u64 capacity) {
	if (num < 0) {
		if (num == INT64_MIN)
			num = INT64_MAX;
		else
			num *= -1;
		if (capacity) {
			capacity -= 1;
			X[0] = '-';
		}
		return cstring_itoau64(num, X + 1, base, capacity) + 1;
	} else
		return cstring_itoau64(num, X, base, capacity);
}

u64 cstring_strtoull(const char *X, int base) {
	u64 ret = 0, mul = 1, len = cstring_len(X);
	while (len-- && X[len] != 'x') {
		ret += X[len] > '9' ? ((X[len] - 'a') + 10) * mul
				    : (X[len] - '0') * mul;
		mul *= base;
	}
	return ret;
}

void cstring_cat_n(char *X, char *Y, u64 n) {
	X += cstring_len(X);
	while (n-- && *Y) {
		*X = *Y;
		X++;
		Y++;
	}
	*X = 0;
}

int cstring_char_is_alpha_numeric(char ch) {
	if (ch >= 'a' && ch <= 'z') return 1;
	if (ch >= 'A' && ch <= 'Z') return 1;
	if (ch >= '0' && ch <= '9') return 1;
	if (ch == '_' || ch == '\n') return 1;
	return 0;
}
int cstring_is_alpha_numeric(const char *X) {
	if (*X >= '0' && *X <= '9') return 0;
	while (*X)
		if (!cstring_char_is_alpha_numeric(*X++)) return 0;

	return 1;
}
