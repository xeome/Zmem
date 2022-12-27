#include "zmem.h"

#include <stdio.h>
#include <memory.h>

/*
 * Print the history of values in a line graph.
 * graph should be centered around average value in history
 * @param history The history of values to print
 * @param size The number of values in the history
 * @param graph_width The width of the graph in characters
 * @param graph_height The height of the graph in characters
 * */
void draw_line_graph(const MemoryStats *memory_stats, int graph_height) {
    double max = 0;
    double min = 0;
    int history_size = memory_stats->history_size;
    // Find the maximum and minimum values in the history array
    for (int i = 0; i < history_size; ++i) {
        max = (memory_stats->history[i] > max) ? memory_stats->history[i] : max;
        min = (memory_stats->history[i] < min) ? memory_stats->history[i] : min;
    }

    double range = max - min;
    double scale = range / graph_height;
    double offset = min;
    double center = (max + min) / 2;

    // Print colored max, min, range, scale, offset, center, keys should be blue and values should be green in the format "key: value"
    printf(BLUE "max: " GREEN "%.0lf  " BLUE "min: " GREEN "%.0lf  " BLUE "range: " GREEN "%.0lf  " BLUE "scale: " GREEN "%.0lf  " BLUE "offset: " GREEN "%.0lf  " BLUE "center: " GREEN "%.0lf\n" RESET,
           max, min, range, scale, offset, center);

    // Print the top and bottom borders of the graph
    char border[history_size + 1];
    memset(border, '-', history_size);
    border[history_size] = '\0';

    printf("           +%s\n", border);
    // This loop iterates graph_height times, once for each line of the graph
    for (int i = 0; i < graph_height; i++) {
        // Print the y-axis value for this line of the graph. The value is calculated as the center value plus or minus an offset, depending on the iteration number. The offset is calculated by multiplying the scale value by the difference between the half the height of the graph and the current iteration number. The scale value determines the scaling of the y-axis, with each character on the y-axis representing a certain number of units.
        printf(GREEN "%10.0lf "RESET"|", center + ((double) graph_height / 2 - i) * scale);

        // This loop iterates over the values in the history array
        for (int j = 0; j < history_size; j++) {
            // Determine the range represented by the current y-axis value
            double range_min = (memory_stats->history[j] > center) ? max - (i * scale) : max - ((i + 1) * scale);
            double range_max = (memory_stats->history[j] > center) ? max - ((i - 1) * scale) : max - (i * scale);

            // If the value falls within the range, print a blue asterisk character. If it doesn't, print a space character.
            if (memory_stats->history[j] >= range_min && memory_stats->history[j] < range_max) {
                printf(BLUE "*" RESET);
            } else {
                printf(" ");
            }
        }
        // After the inner loop completes, print a newline character to move to the next line on the graph.
        printf("\n" RESET);
    }

    printf("           +%s\n", border);
}

// Function to parse memory info from /proc/meminfo zswap, zswapped, compression, active, inactive, free
void parse_memory_info(MemoryStats *memory_stats) {
    // Parse the values from /proc/meminfo
    FILE *file = fopen("/proc/meminfo", "r");
    if (file == NULL) {
        return;
    }

    char line[256];
    while (fgets(line, sizeof(line), file) != NULL) {
        sscanf(line, "Zswap: %lf", &memory_stats->zswap);
        sscanf(line, "Zswapped: %lf", &memory_stats->zswapped);
        sscanf(line, "Active: %lf", &memory_stats->active);
        sscanf(line, "Inactive: %lf", &memory_stats->inactive);
        sscanf(line, "MemFree: %lf", &memory_stats->free);
    }
    // convert to MB
    memory_stats->zswap /= 1024;
    memory_stats->zswapped /= 1024;
    memory_stats->active /= 1024;
    memory_stats->inactive /= 1024;
    memory_stats->free /= 1024;

    fclose(file);

    // Calculate the compression value in MB
    memory_stats->compression = memory_stats->zswapped / memory_stats->zswap;
}


// Function to print the memory info in the format "key: value" in MB with the keys colored blue and the values colored green, each key value pair should be on a new line
void print_memory_info(const MemoryStats *memory_stats) {
    printf(BLUE "Zswap: " GREEN "%.0lf MB\n" RESET, memory_stats->zswap);
    printf(BLUE "Zswapped: " GREEN "%.0lf MB\n" RESET, memory_stats->zswapped);
    printf(BLUE "Compression: " GREEN "%.2lf\n" RESET, memory_stats->compression);
    printf(BLUE "Active: " GREEN "%.0lf MB\n" RESET, memory_stats->active);
    printf(BLUE "Inactive: " GREEN "%.0lf MB\n" RESET, memory_stats->inactive);
    printf(BLUE "Free: " GREEN "%.0lf MB\n" RESET, memory_stats->free);
}