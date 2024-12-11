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

#ifndef _BASE_ATOMIC__
#define _BASE_ATOMIC__

#define CAS(...)                                                  \
	__atomic_compare_exchange_n(__VA_ARGS__, 0, __ATOMIC_RELAXED, \
								__ATOMIC_RELAXED)
#define CAS_SEQ(...)                                              \
	__atomic_compare_exchange_n(__VA_ARGS__, 0, __ATOMIC_SEQ_CST, \
								__ATOMIC_SEQ_CST)
#define CAS_ACQUIRE(...)                                          \
	__atomic_compare_exchange_n(__VA_ARGS__, 0, __ATOMIC_ACQUIRE, \
								__ATOMIC_ACQUIRE)
#define CAS_RELEASE(...)                                          \
	__atomic_compare_exchange_n(__VA_ARGS__, 0, __ATOMIC_RELEASE, \
								__ATOMIC_RELAXED)
#define ALOAD(ptr) __atomic_load_n(ptr, __ATOMIC_ACQUIRE)
#define ASTORE(ptr, value) __atomic_store_n(ptr, value, __ATOMIC_RELEASE)
#define AADD(ptr, value) __atomic_fetch_add(ptr, value, __ATOMIC_SEQ_CST)
#define ASUB(ptr, value) __atomic_fetch_sub(ptr, value, __ATOMIC_SEQ_CST)

#endif	// _BASE_ATOMIC__
