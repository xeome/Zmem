#include "zmem.h"
#include <malloc.h>

// BoundedQueue that holds the history of memory usage values, if full the
// oldest value is removed

void enqueue(BoundedQueue* queue, double data) {
    if (full(queue)) {
        dequeue(queue);
    }
    queue->array[queue->rear] = data;
    queue->rear = (queue->rear + 1) % queue->capacity;
    queue->size = queue->size + 1;
}

double dequeue(BoundedQueue* queue) {
    if (empty(queue)) {
        return -1;
    }
    double data = queue->array[queue->front];
    queue->front = (queue->front + 1) % queue->capacity;
    queue->size = queue->size - 1;
    return data;
}

double get_element(BoundedQueue* queue, int index) {
    if (index >= queue->size) {
        return -1;
    }
    return queue->array[(queue->front + index) % queue->capacity];
}

int empty(BoundedQueue* queue) {
    return queue->size == 0;
}

int full(BoundedQueue* queue) {
    return queue->size == queue->capacity;
}

BoundedQueue* create_bounded_queue(int capacity) {
    BoundedQueue* queue = malloc(sizeof(BoundedQueue));
    queue->size = 0;
    queue->capacity = capacity;
    queue->front = 0;
    queue->rear = 0;
    queue->array = malloc(sizeof(double) * queue->capacity);
    memset(queue->array, 0, queue->capacity);
    return queue;
}

MemoryStats* initialize_memory_stats() {
    MemoryStats* memory_stats = malloc(sizeof(MemoryStats));
    memory_stats->memory_total = 0;
    memory_stats->swap_total = 0;
    memory_stats->free = 0;
    memory_stats->available = 0;
    memory_stats->used = 0;
    memory_stats->shared = 0;
    memory_stats->buffers = 0;
    memory_stats->cache = 0;
    memory_stats->zswap = 0;
    memory_stats->zswapped = 0;
    memory_stats->swapcached = 0;
    memory_stats->swap_free = 0;
    memory_stats->compression = 0;
    return memory_stats;
}

void enqueue_memory_stats(BoundedQueueMemoryStats* queue, MemoryStats* data) {
    if (full_memory_stats(queue)) {
        dequeue_memory_stats(queue);
    }
    queue->array[queue->rear] = data;
    queue->rear = (queue->rear + 1) % queue->capacity;
    queue->size = queue->size + 1;
}

MemoryStats* dequeue_memory_stats(BoundedQueueMemoryStats* queue) {
    if (empty_memory_stats(queue)) {
        return NULL;
    }
    MemoryStats* data = queue->array[queue->front];
    queue->front = (queue->front + 1) % queue->capacity;
    queue->size = queue->size - 1;
    return data;
}

MemoryStats* get_element_memory_stats(BoundedQueueMemoryStats* queue,
                                      int index) {
    if (index >= queue->size) {
        return NULL;
    }
    return queue->array[(queue->front + index) % queue->capacity];
}

int empty_memory_stats(BoundedQueueMemoryStats* queue) {
    return queue->size == 0;
}

int full_memory_stats(BoundedQueueMemoryStats* queue) {
    return queue->size == queue->capacity;
}

BoundedQueueMemoryStats* create_bounded_queue_memory_stats(int capacity) {
    BoundedQueueMemoryStats* queue = malloc(sizeof(BoundedQueueMemoryStats));
    queue->size = 0;
    queue->capacity = capacity;
    queue->front = 0;
    queue->rear = 0;
    queue->array = malloc(sizeof(MemoryStats*) * queue->capacity);
    // Initialize the array
    for (int i = 0; i < queue->capacity; i++) {
        queue->array[i] = initialize_memory_stats();
    }

    return queue;
}