#include "zmem.h"

/*
 * Print the history of values in a line graph.
 * @param data MemoryStats struct containing the history
 * @param graph_height The height of the graph in characters
 * */

void draw_line_graph(BoundedQueue* data, int graph_height) {
    double max = 0;
    double min = 0;
    int history_size = data->capacity;
    // Find the maximum and minimum values in the history array
    for (int i = 0; i < history_size; ++i) {
        max = data->array[i] > max ? data->array[i] : max;
        min = data->array[i] < min ? data->array[i] : min;
    }

    double range = max - min;
    double scale = range / graph_height;
    double offset = min;
    double center = (max + min) / 2;

    // Print colored max, min, range, scale, offset, center, keys should be blue
    // and values should be green in the format "key: value"
    printf(BLUE "max: %.0lf  " BLUE "min: %.0lf  " BLUE "range: %.0lf  " BLUE
                "scale: %.0lf  " BLUE "offset: %.0lf  " BLUE
                "center: %.0lf\n" RESET,
           max, min, range, scale, offset, center);

    // Print the top and bottom borders of the graph
    char border[history_size + 1];
    memset(border, '-', history_size);
    border[history_size] = '\0';

    // Print the top border
    printf("           +%s\n", border);
    // This loop iterates graph_height times, once for each line of the graph
    for (int i = 0; i < graph_height; i++) {
        // Print the y-axis value for this line of the graph. The value is
        // calculated as the center value plus or minus an offset, depending on
        // the iteration number. The offset is calculated by multiplying the
        // scale value by the difference between the half the height of the
        // graph and the current iteration number. The scale value determines
        // the scaling of the y-axis, with each character on the y-axis
        // representing a certain number of units.
        printf(GREEN "%10.0lf " RESET "|",
               center + ((double)graph_height / 2 - i) * scale);

        // This loop iterates over the values in the history array
        for (int j = 0; j < history_size; j++) {
            double current = get_element(data, j);

            // Determine the range represented by the current y-axis value
            double range_min = (current > center) ? max - (i * scale)
                                                  : max - ((i + 1) * scale);
            double range_max = (current > center) ? max - ((i - 1) * scale)
                                                  : max - (i * scale);

            // If the value falls within the range, print a blue asterisk
            // character. If it doesn't, print a space character.
            if (current >= range_min && current < range_max) {
                printf(BLUE "*" RESET);
            } else {
                printf(" ");
            }
        }
        // After the inner loop completes, print a newline character to move to
        // the next line on the graph.
        printf("\n" RESET);
    }

    printf("           +%s\n", border);
}

/*
 * Function to parse memory info from /proc/meminfo
 * @param memory_stats Pointer to MemoryStats structure to store parsed memory
 * info
 */
void parse_memory_info(MemoryStats* memory_stats) {
    // Parse the values from /proc/meminfo
    FILE* file = fopen("/proc/meminfo", "r");
    if (file == NULL) {
        return;
    }

    char line[256];
    while (fgets(line, sizeof(line), file) != NULL) {
        sscanf(line, "MemTotal: %lf kB", &memory_stats->memory_total);
        sscanf(line, "MemFree: %lf kB", &memory_stats->free);
        sscanf(line, "MemAvailable: %lf kB", &memory_stats->available);
        sscanf(line, "Shmem: %lf kB", &memory_stats->shared);
        sscanf(line, "Buffers: %lf kB", &memory_stats->buffers);
        sscanf(line, "Cached: %lf kB", &memory_stats->cache);
        sscanf(line, "Zswap: %lf kB", &memory_stats->zswap);
        sscanf(line, "Zswapped: %lf kB", &memory_stats->zswapped);
        sscanf(line, "SwapCached: %lf kB", &memory_stats->swapcached);
        sscanf(line, "SwapTotal: %lf kB", &memory_stats->swap_total);
        sscanf(line, "SwapFree: %lf kB", &memory_stats->swap_free);
    }
    memory_stats->used = memory_stats->memory_total - memory_stats->free -
                         memory_stats->buffers - memory_stats->cache;
    // convert to MB
    memory_stats->memory_total = memory_stats->memory_total / 1024;
    memory_stats->free = memory_stats->free / 1024;
    memory_stats->available = memory_stats->available / 1024;
    memory_stats->used = memory_stats->used / 1024;
    memory_stats->shared = memory_stats->shared / 1024;
    memory_stats->buffers = memory_stats->buffers / 1024;
    memory_stats->cache = memory_stats->cache / 1024;
    memory_stats->zswap = memory_stats->zswap / 1024;
    memory_stats->zswapped = memory_stats->zswapped / 1024;
    memory_stats->swapcached = memory_stats->swapcached / 1024;
    memory_stats->swap_total = memory_stats->swap_total / 1024;
    memory_stats->swap_free = memory_stats->swap_free / 1024;

    fclose(file);

    // Calculate the compression value in MB
    memory_stats->compression = memory_stats->zswapped / memory_stats->zswap;
}

/*
 * Function to print memory info
 * @param memory_stats Pointer to MemoryStats structure to store parsed memory
 * info
 */
void print_memory_info(const MemoryStats* memory_stats) {
    // Print the memory info in the format specified in the comments above
    printf(
        "             total       used       free     shared buff/cache  "
        "available\n");
    printf(CYAN "Mem:    " RESET
                "%10.0lf %10.0lf %10.0lf %10.0lf %10.0lf %10.0lf\n",
           memory_stats->memory_total, memory_stats->used, memory_stats->free,
           memory_stats->shared, memory_stats->buffers + memory_stats->cache,
           memory_stats->available);
    printf(PURPLE "Swap:   " RESET
                  "%10.0lf %10.0lf %10.0lf %10.0lf %10.0lf %10.0lf\n",
           memory_stats->swap_total,
           memory_stats->swap_total - memory_stats->swap_free -
               memory_stats->swapcached,
           memory_stats->swap_total - memory_stats->zswapped, 0.0,
           memory_stats->swapcached,
           memory_stats->swap_total - memory_stats->zswapped +
               memory_stats->swapcached);
}

void draw_history(BoundedQueueMemoryStats* data) {
    // Print header
    printf(
        "             total       used       free     shared buff/cache  "
        "available\n");
    // Print the memory history graph
    for (int i = 0; i < data->size; i++) {
        MemoryStats* current = get_element_memory_stats(data, i);
        printf("Memory: %10.0lf %10.0lf %10.0lf %10.0lf %10.0lf %10.0lf\n",
               current->memory_total, current->used, current->free,
               current->shared, current->buffers + current->cache,
               current->available);
        printf("Swap:   %10.0lf %10.0lf %10.0lf %10.0lf %10.0lf %10.0lf\n",
               current->swap_total,
               current->swap_total - current->swap_free - current->swapcached,
               current->swap_total - current->zswapped, 0.0,
               current->swapcached,
               current->swap_total - current->zswapped + current->swapcached);
    }
}