//
// Created by xeome on 28.12.2022.
//

#ifndef ZMEM_ZMEM_H
#define ZMEM_ZMEM_H


#define WHITE "\033[37m"  // White text
#define BLUE "\033[34m"  // Blue text
#define BLACK "\033[30m"  // Black text
#define GREEN "\033[32m"  // Green text
#define RESET "\033[0m"  // Reset the color


// MemoryStats struct definition
typedef struct {
    double zswap;
    double zswapped;
    double compression;
    double active;
    double inactive;
    double free;
    double *history;
    int history_size;
} MemoryStats;

// Function prototypes
void parse_memory_info(MemoryStats *memory_stats);

void draw_line_graph(const MemoryStats *memory_stats, int graph_height);

void print_memory_info(const MemoryStats *memory_stats);

#endif //ZMEM_ZMEM_H
