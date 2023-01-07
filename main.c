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
    MemoryStats memory_stats = {0};
    memory_stats.history_size = HIST_SIZE;
    memory_stats.history = malloc(HIST_SIZE * sizeof(double));
    if (memory_stats.history == NULL) {
        // Error allocating memory
        exit(1);
    }

    while (keep_running) {
        // Clear the screen
        printf("\033[H\033[2J");
        // Parse the values from /proc/meminfo
        parse_memory_info(&memory_stats);
        // Print the current values of the memory stats
        print_memory_info(&memory_stats);

        // Graph separator
        printf("-------------------------------------------------------------------------\n");

        // Print the graph of the history of Zswapped values
        draw_line_graph(&memory_stats, 15);

        // Update the history queue with used memory
        for (int i = 0; i < HIST_SIZE - 1; i++) {
            memory_stats.history[i] = memory_stats.history[i + 1];
        }
        memory_stats.history[HIST_SIZE - 1] =
            memory_stats.memory_total - memory_stats.free -
            memory_stats.buffers - memory_stats.cache;

        // Sleep for the specified delay
        usleep(DELAY_MS * 1000);
    }

    return 0;
}

void sigint_handler(int sig) {
    keep_running = 0;
}
