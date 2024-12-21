#include <pthread.h>

int printf(const char *, ...);
int thread_create(pthread_t *th, void *(*start_routine)(void *), void *arg,
		  _Bool detached) {
	pthread_attr_t attr;

	if (detached) {
		pthread_attr_init(&attr);
		pthread_attr_setdetachstate(&attr, PTHREAD_CREATE_DETACHED);
		int ret = pthread_create(th, &attr, start_routine, arg);
		pthread_attr_destroy(&attr);
		return ret;
	} else {
		return pthread_create(th, NULL, start_routine, arg);
	}
}

int thread_join(pthread_t *th) { return pthread_join(*th, NULL); }

size_t thread_handle_size() { return sizeof(pthread_t); }

int thread_detach(pthread_t *th) { return pthread_detach(*th); }
