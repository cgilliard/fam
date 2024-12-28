#include <pthread.h>

#define ERROR_CAPACITY_EXCEEDED -1

void _exit(int);
int perror(const char *msg);
int printf(const char *fmt, ...);
void release(void *);
void *memcpy(void *, const void *, unsigned long);

typedef struct Message {
	struct Message *next;
	unsigned char buffer[];
} Message;

typedef struct Channel {
	pthread_mutex_t lock;
	pthread_cond_t cond;
	Message *head;
	Message *tail;
} Channel;

_Bool channel_pending(Channel *handle) { return handle->head; }

int channel_init(Channel *handle) {
	// printf("channel init %p\n", handle);
	if (pthread_mutex_init(&handle->lock, NULL)) return -1;
	if (pthread_cond_init(&handle->cond, NULL)) return -1;
	handle->head = handle->tail = NULL;
	return 0;
}
int channel_send(Channel *handle, Message *msg) {
	int ret = 0;
	if (pthread_mutex_lock(&handle->lock)) {
		perror("pthread_mutex_lock");
		_exit(-1);
	}

	msg->next = NULL;
	if (handle->tail)
		handle->tail->next = msg;
	else
		handle->head = msg;
	handle->tail = msg;

	if (pthread_mutex_unlock(&handle->lock)) {
		perror("pthread_mutex_unlock");
		_exit(-1);
	}

	if (ret == 0 && pthread_cond_signal(&handle->cond)) {
		perror("pthread_cond_signal");
		_exit(-1);
	}

	return ret;
}

Message *channel_recv(Channel *handle, Message *buffer) {
	if (pthread_mutex_lock(&handle->lock)) {
		perror("pthread_mutex_lock");
		_exit(1);
	}
	Message *ret = NULL;

	while (!handle->head) pthread_cond_wait(&handle->cond, &handle->lock);

	ret = handle->head;
	handle->head = handle->head->next;
	if (!handle->head) handle->tail = NULL;

	if (pthread_mutex_unlock(&handle->lock)) {
		perror("pthread_mutex_unlock");
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
