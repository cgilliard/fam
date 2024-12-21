#include <pthread.h>

void _exit(int);
int perror(const char *msg);
int printf(const char *fmt, ...);

typedef struct Message {
	struct Message *next;
	void *payload;
} Message;

typedef struct Channel {
	pthread_mutex_t lock;
	pthread_cond_t cond;
	Message *pending;
} Channel;

int channel_init(Channel *handle) {
	if (pthread_mutex_init(&handle->lock, NULL)) return -1;
	if (pthread_cond_init(&handle->cond, NULL)) return -1;
	handle->pending = NULL;
	return 0;
}
int channel_send(Channel *handle, Message *msg) {
	if (pthread_mutex_lock(&handle->lock)) {
		perror("pthread_mutex_lock");
		_exit(-1);
	}

	handle->pending = msg;
	if (pthread_cond_signal(&handle->cond)) {
		perror("pthread_cond_signal");
		_exit(-1);
	}

	if (pthread_mutex_unlock(&handle->lock)) {
		perror("pthread_mutex_unlock");
		_exit(-1);
	}

	return 0;
}
Message *channel_recv(Channel *handle) {
	if (pthread_mutex_lock(&handle->lock)) {
		perror("pthread_mutex_lock");
		_exit(1);
	}

	while (!handle->pending)
		pthread_cond_wait(&handle->cond, &handle->lock);

	Message *ret = handle->pending;
	handle->pending = NULL;

	if (pthread_mutex_unlock(&handle->lock)) {
		perror("pthread_mutex_lock");
		_exit(1);
	}

	return ret;
}
unsigned long long channel_handle_size() { return sizeof(Channel); }
int channel_destroy(Channel *handle) {
	if (pthread_mutex_destroy(&handle->lock)) {
		perror("pthread_mutex_destroy");
		_exit(-1);
	}
	if (pthread_cond_destroy(&handle->cond)) {
		perror("pthread_cond_destroy");
		_exit(-1);
	}
	return 0;
}
