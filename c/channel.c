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
	unsigned long long head;
	unsigned long long tail;
	unsigned long long capacity;
	unsigned long long msg_size;
	unsigned char buffer[];
} Channel;

int channel_init(Channel *handle, unsigned long long msg_size,
		 unsigned long long capacity) {
	if (pthread_mutex_init(&handle->lock, NULL)) return -1;
	if (pthread_cond_init(&handle->cond, NULL)) return -1;
	handle->head = handle->tail = 0;
	handle->msg_size = msg_size;
	handle->capacity = capacity;
	return 0;
}
int channel_send(Channel *handle, Message *msg) {
	int ret = 0;
	if (pthread_mutex_lock(&handle->lock)) {
		perror("pthread_mutex_lock");
		_exit(-1);
	}

	unsigned long long next_tail = (handle->tail + 1) % handle->capacity;
	if (next_tail == handle->head) {
		ret = -1;
	} else {
		unsigned char *dest =
		    handle->buffer + (handle->tail * handle->msg_size);
		memcpy(dest, msg->buffer, handle->msg_size);

		handle->tail = next_tail;
	}

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

Message *channel_recv(Channel *handle, Message *msg) {
	if (pthread_mutex_lock(&handle->lock)) {
		perror("pthread_mutex_lock");
		_exit(1);
	}

	Message *ret = NULL;

	while (handle->head == handle->tail) {
		if (pthread_cond_wait(&handle->cond, &handle->lock)) {
			perror("pthread_cond_wait");
			_exit(-1);
		}
	}

	unsigned char *src = handle->buffer + (handle->head * handle->msg_size);
	memcpy(msg->buffer, src, handle->msg_size);
	handle->head = (handle->head + 1) % handle->capacity;

	if (pthread_mutex_unlock(&handle->lock)) {
		perror("pthread_mutex_unlock");
		_exit(1);
	}

	return ret;
}
unsigned long long channel_handle_size() { return sizeof(Channel); }

void channel_destroy(Channel *handle) {
	if (pthread_mutex_destroy(&handle->lock)) {
		perror("pthread_mutex_destroy");
		_exit(-1);
	}
	if (pthread_cond_destroy(&handle->cond)) {
		perror("pthread_cond_destroy");
		_exit(-1);
	}
}
