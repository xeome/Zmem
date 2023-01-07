//
// Created by xeome on 28.12.2022.
//

#ifndef ZMEM_ZMEM_H
#define ZMEM_ZMEM_H
#include <stdio.h>
#include <memory.h>

#define WHITE "\033[37m"   // White text
#define BLUE "\033[34m"    // Blue text
#define BLACK "\033[30m"   // Black text
#define GREEN "\033[32m"   // Green text
#define RED "\033[31m"     // Red text
#define CYAN "\033[36m"    // Cyan text
#define PURPLE "\033[35m"  // Purple text
#define RESET "\033[0m"    // Reset the color

// MemoryStats struct definition
typedef struct {
    double memory_total;
    double swap_total;
    double free;
    double available;
    double used;
    double shared;
    double buffers;
    double cache;
    double zswap;
    double zswapped;
    double swapcached;
    double swap_free;
    double compression;
} MemoryStats;

// Bounded queue for double values
typedef struct bounded_queue {
    int size;
    int capacity;
    int front;
    int rear;
    double* array;
} BoundedQueue;

// Bounded queue for memory stats
typedef struct bounded_queue_memory_stats {
    int size;
    int capacity;
    int front;
    int rear;
    MemoryStats** array;
} BoundedQueueMemoryStats;

// Function prototypes
void parse_memory_info(MemoryStats* memory_stats);
void draw_line_graph(BoundedQueue* data, int graph_height);
void print_memory_info(const MemoryStats* memory_stats);
void enqueue(BoundedQueue* queue, double data);
double dequeue(BoundedQueue* queue);
double get_element(BoundedQueue* queue, int index);
int empty(BoundedQueue* queue);
int full(BoundedQueue* queue);
BoundedQueue* create_bounded_queue(int capacity);
MemoryStats* initialize_memory_stats();
void enqueue_memory_stats(BoundedQueueMemoryStats* queue, MemoryStats* data);
MemoryStats* dequeue_memory_stats(BoundedQueueMemoryStats* queue);
MemoryStats* get_element_memory_stats(BoundedQueueMemoryStats* queue, int index);
int empty_memory_stats(BoundedQueueMemoryStats* queue);
int full_memory_stats(BoundedQueueMemoryStats* queue);
BoundedQueueMemoryStats* create_bounded_queue_memory_stats(int capacity);
void draw_history(BoundedQueueMemoryStats* data);

#endif  // ZMEM_ZMEM_H
