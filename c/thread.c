#include <pthread.h>

int printf(const char *, ...);
int thread_create(void *(*start_routine)(void *), void *arg) {
	pthread_t th;
	pthread_attr_t attr;
	pthread_attr_init(&attr);
	pthread_attr_setdetachstate(&attr, PTHREAD_CREATE_DETACHED);
	int ret = pthread_create(&th, &attr, start_routine, arg);
	pthread_attr_destroy(&attr);
	return ret;
}

