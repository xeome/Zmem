#include "zmem.h"

#include <stdio.h>
#include <unistd.h>
#include <signal.h>
#include <memory.h>
#include <stdlib.h>

#define DELAY_MS 800  // Delay between each update in milliseconds
#define HIST_SIZE 61  // Number of values in the history

// Global flag to indicate if the program should continue running
sig_atomic_t keep_running = 1;

// Signal handler to set the keep_running flag to 0 when Ctrl+C is pressed
void sigint_handler(int sig);

int main() {
    // Register the signal handler
    signal(SIGINT, sigint_handler);

    // Initialize the MemoryStats struct
    MemoryStats* memory_stats = initialize_memory_stats();
    // Initialize history queue for used memory
    BoundedQueue* used_history = create_bounded_queue(HIST_SIZE);

    BoundedQueueMemoryStats *memory_stats_history = create_bounded_queue_memory_stats(5);

    while (keep_running) {
        // Clear the screen
        printf("\033[H\033[2J");
        // Parse the values from /proc/meminfo
        parse_memory_info(memory_stats);
        // Print the current values of the memory stats
        print_memory_info(memory_stats);

        // Update the history queue with used memory
        double used = memory_stats->memory_total - memory_stats->free -
                      memory_stats->buffers - memory_stats->cache;
        enqueue(used_history, used);

        // Update the history of memory stats
        MemoryStats* current = initialize_memory_stats();
        memcpy(current, memory_stats, sizeof(MemoryStats));
        enqueue_memory_stats(memory_stats_history, current);

        // Graph separator
        printf(
            "------------------------------------------------------------------"
            "-------\n");

        draw_history(memory_stats_history);

        // Graph separator
        printf(
            "------------------------------------------------------------------"
            "-------\n");

        // Print the graph of the history of Zswapped values
        draw_line_graph(used_history, 15);


        // Sleep for the specified delay
        usleep(DELAY_MS * 1000);
    }

    return 0;
}

void sigint_handler(int sig) {
    keep_running = 0;
}
