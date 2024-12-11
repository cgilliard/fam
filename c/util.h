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

#ifndef _BASE_UTIL__
#define _BASE_UTIL__

#include <types.h>

void copy_bytes(byte *dest, const byte *src, u64 n);
void set_bytes(byte *dst, byte b, u64 n);
u64 cstring_len(const char *S);
int cstring_compare(const char *s1, const char *s2);
int cstring_compare_n(const char *s1, const char *s2, u64 n);
void cstring_cat_n(char *s1, char *s2, u64 n);
const char *cstring_strstr(const char *X, const char *Y);
void reverse(byte *str, i64 end);
u64 cstring_itoau64(u64 num, char *str, int base, u64 capacity);
u64 cstring_itoai64(i64 num, char *str, int base, u64 capacity);
u64 cstring_strtoull(const char *str, int base);
int cstring_is_alpha_numeric(const char *str);

#endif	// _BASE_UTIL__
