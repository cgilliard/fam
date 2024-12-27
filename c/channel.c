#include <pthread.h>

#define ERROR_CAPACITY_EXCEEDED -1

void _exit(int);
int perror(const char *msg);
int printf(const char *fmt, ...);
void release(void *);
void *memcpy(void *, const void *, unsigned long);

typedef struct Message {
	union data {
		struct {
			unsigned long long len;
			unsigned char buffer[];
		} bounded;
		struct {
			struct Message *next;
			void *payload;
		} unbounded;
	} data;
} Message;

typedef enum ChannelType {
	BOUNDED = 0,
	UNBOUNDED = 1,
} ChannelType;

typedef struct Channel {
	pthread_mutex_t lock;
	pthread_cond_t cond;
	ChannelType type;
	union state {
		struct {
			unsigned long long capacity;
			unsigned long long head;
			unsigned long long tail;
			unsigned long long msg_size;
		} bounded;
		struct {
			Message *head;
			Message *tail;
		} unbounded;
	} state;
	unsigned char buffer[];
} Channel;

_Bool channel_pending(Channel *handle) { return handle->state.unbounded.head; }

int channel_unbounded_init(Channel *handle) {
	// printf("channel init %p\n", handle);
	handle->type = UNBOUNDED;
	if (pthread_mutex_init(&handle->lock, NULL)) return -1;
	if (pthread_cond_init(&handle->cond, NULL)) return -1;
	handle->state.unbounded.head = handle->state.unbounded.tail = NULL;
	return 0;
}
int channel_bounded_init(Channel *handle, unsigned long long capacity,
			 unsigned long long msg_size) {
	handle->type = BOUNDED;
	if (pthread_mutex_init(&handle->lock, NULL)) return -1;
	if (pthread_cond_init(&handle->cond, NULL)) return -1;
	handle->state.bounded.head = handle->state.bounded.tail = 0;
	handle->state.bounded.capacity = capacity;
	handle->state.bounded.msg_size = msg_size;
	return 0;
}
int channel_send(Channel *handle, Message *msg) {
	int ret = 0;
	if (pthread_mutex_lock(&handle->lock)) {
		perror("pthread_mutex_lock");
		_exit(-1);
	}

	if (handle->type == UNBOUNDED) {
		msg->data.unbounded.next = NULL;
		if (handle->state.unbounded.tail)
			handle->state.unbounded.tail->data.unbounded.next = msg;
		else
			handle->state.unbounded.head = msg;
		handle->state.unbounded.tail = msg;
	} else {
		unsigned long long next_tail =
		    (handle->state.bounded.tail + 1) %
		    handle->state.bounded.capacity;
		if (next_tail != handle->state.bounded.head) {
			unsigned long long msg_size = msg->data.bounded.len;
			memcpy(handle->buffer +
				   handle->state.bounded.tail * msg_size,
			       msg->data.bounded.buffer, msg_size);

			handle->state.bounded.tail = next_tail;
		} else
			ret = ERROR_CAPACITY_EXCEEDED;
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

Message *channel_recv(Channel *handle, Message *buffer) {
	if (pthread_mutex_lock(&handle->lock)) {
		perror("pthread_mutex_lock");
		_exit(1);
	}
	Message *ret = NULL;

	if (handle->type == UNBOUNDED) {
		while (!handle->state.unbounded.head)
			pthread_cond_wait(&handle->cond, &handle->lock);

		ret = handle->state.unbounded.head;
		handle->state.unbounded.head =
		    handle->state.unbounded.head->data.unbounded.next;
		if (!handle->state.unbounded.head)
			handle->state.unbounded.tail = NULL;
	} else {
		while (handle->state.bounded.head == handle->state.bounded.tail)
			pthread_cond_wait(&handle->cond, &handle->lock);

		unsigned long long msg_size = handle->state.bounded.msg_size;

		memcpy(buffer->data.bounded.buffer,
		       handle->buffer + handle->state.bounded.head * msg_size,
		       msg_size);
		ret = buffer;

		handle->state.bounded.head = (handle->state.bounded.head + 1) %
					     handle->state.bounded.capacity;
	}

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
