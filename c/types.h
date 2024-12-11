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

#ifndef _BASE_TYPES__
#define _BASE_TYPES__

// primitives
typedef signed long long i64;
typedef unsigned long long u64;
typedef __uint128_t u128;
typedef __int128_t i128;
typedef int i32;
typedef unsigned int u32;
typedef unsigned char byte;
typedef double f64;
typedef float f32;
typedef _Bool bool;

typedef unsigned long size_t;
#ifdef __linux__
typedef long int off_t;
#else
typedef i64 off_t;
#endif

#ifdef __APPLE__
typedef long ssize_t;
#else
typedef long int ssize_t;
#endif

typedef int pid_t;

// booleans
#define bool _Bool
#define true (_Bool)1
#define false (_Bool)0

// NULL
#ifndef NULL
#define NULL ((void *)0)
#endif	// NULL

#endif	// _BASE_TYPES__
