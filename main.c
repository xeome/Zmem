#include <stdio.h>
#include <unistd.h>
#include <signal.h>
#include <memory.h>

#define HIST_SIZE 50  // Number of values in the history
#define DELAY_MS 200  // Delay between each update in milliseconds
#define WHITE "\033[37m"  // White text
#define BLUE "\033[34m"  // Blue text
#define BLACK "\033[30m"  // Black text
#define GREEN "\033[32m"  // Green text
#define RESET "\033[0m"  // Reset the color

// Global flag to indicate if the program should continue running
sig_atomic_t keep_running = 1;

// MemoryInfo struct definition
typedef struct {
    double zswap;
    double zswapped;
    double compression;
    double active;
    double inactive;
    double free;
    double history[HIST_SIZE];
} MemoryInfo;

// Function templates
void parse_memory_info(MemoryInfo *memory_info);

void draw_line_graph(const MemoryInfo *memory_info, int graph_height);

void sigint_handler(int sig);

void print_memory_info(const MemoryInfo *memory_info);

int main() {
    // Register the signal handler
    signal(SIGINT, sigint_handler);

    // Initialize the MemoryInfo struct
    MemoryInfo memory_info = {0};
    while (keep_running) {
        // Clear the screen
        printf("\033[H\033[2J");
        // Parse the values from /proc/meminfo
        parse_memory_info(&memory_info);
        // Print the current values of the memory, keys should be blue and values should be green in the format "key: value" in MB
        print_memory_info(&memory_info);

        // Print the graph of the history of Zswapped values
        draw_line_graph(&memory_info, 25);

        // Update the history queue with free value
        for (int i = 0; i < HIST_SIZE - 1; i++) {
            memory_info.history[i] = memory_info.history[i + 1];
        }
        memory_info.history[HIST_SIZE - 1] = memory_info.free;

        // Sleep for the specified delay
        usleep(DELAY_MS * 1000);
    }

    return 0;
}

/*
 * Print the history of values in a line graph.
 * graph should be centered around average value in history
 * @param history The history of values to print
 * @param size The number of values in the history
 * @param graph_width The width of the graph in characters
 * @param graph_height The height of the graph in characters
 * */
void draw_line_graph(const MemoryInfo *memory_info, int graph_height) {
    double max = 0;
    double min = 0;

    // Find the maximum and minimum values in the history array
    for (int i = 0; i < HIST_SIZE; ++i) {
        max = (memory_info->history[i] > max) ? memory_info->history[i] : max;
        min = (memory_info->history[i] < min) ? memory_info->history[i] : min;
    }

    double range = max - min;
    double scale = range / graph_height;
    double offset = min;
    double center = (max + min) / 2;

    // Print colored max, min, range, scale, offset, center, keys should be blue and values should be green in the format "key: value"
    printf(BLUE "max: " GREEN "%.0lf  " BLUE "min: " GREEN "%.0lf  " BLUE "range: " GREEN "%.0lf  " BLUE "scale: " GREEN "%.0lf  " BLUE "offset: " GREEN "%.0lf  " BLUE "center: " GREEN "%.0lf\n" RESET,
           max, min, range, scale, offset, center);

    // Print the top and bottom borders of the graph
    char border[HIST_SIZE + 1];
    memset(border, '-', HIST_SIZE);
    border[HIST_SIZE] = '\0';

    printf("           +%s\n", border);
    // This loop iterates graph_height times, once for each line of the graph
    for (int i = 0; i < graph_height; i++) {
        // Print the y-axis value for this line of the graph. The value is calculated as the center value plus or minus an offset, depending on the iteration number. The offset is calculated by multiplying the scale value by the difference between the half the height of the graph and the current iteration number. The scale value determines the scaling of the y-axis, with each character on the y-axis representing a certain number of units.
        printf(GREEN "%10.0lf "RESET"|", center + ((double) graph_height / 2 - i) * scale);

        // This loop iterates over the values in the history array
        for (int j = 0; j < HIST_SIZE; j++) {
            // Determine the range represented by the current y-axis value
            double range_min = (memory_info->history[j] > center) ? max - (i * scale) : max - ((i + 1) * scale);
            double range_max = (memory_info->history[j] > center) ? max - ((i - 1) * scale) : max - (i * scale);

            // If the value falls within the range, print a blue asterisk character. If it doesn't, print a space character.
            if (memory_info->history[j] >= range_min && memory_info->history[j] < range_max) {
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
void parse_memory_info(MemoryInfo *memory_info) {
    // Parse the values from /proc/meminfo
    FILE *file = fopen("/proc/meminfo", "r");
    if (file == NULL) {
        return;
    }

    char line[256];
    while (fgets(line, sizeof(line), file) != NULL) {
        sscanf(line, "Zswap: %lf", &memory_info->zswap);
        sscanf(line, "Zswapped: %lf", &memory_info->zswapped);
        sscanf(line, "Active: %lf", &memory_info->active);
        sscanf(line, "Inactive: %lf", &memory_info->inactive);
        sscanf(line, "MemFree: %lf", &memory_info->free);
    }
    // convert to MB
    memory_info->zswap /= 1024;
    memory_info->zswapped /= 1024;
    memory_info->active /= 1024;
    memory_info->inactive /= 1024;
    memory_info->free /= 1024;

    fclose(file);

    // Calculate the compression value in MB
    memory_info->compression = memory_info->zswapped / memory_info->zswap;
}

// Signal handler to set the keep_running flag to 0 when Ctrl+C is pressed
void sigint_handler(int sig) {
    keep_running = 0;
}

// Function to print the memory info in the format "key: value" in MB with the keys colored blue and the values colored green, each key value pair should be on a new line
void print_memory_info(const MemoryInfo *memory_info) {
    printf(BLUE "Zswap: " GREEN "%.0lf MB\n" RESET, memory_info->zswap);
    printf(BLUE "Zswapped: " GREEN "%.0lf MB\n" RESET, memory_info->zswapped);
    printf(BLUE "Compression: " GREEN "%.2lf\n" RESET, memory_info->compression);
    printf(BLUE "Active: " GREEN "%.0lf MB\n" RESET, memory_info->active);
    printf(BLUE "Inactive: " GREEN "%.0lf MB\n" RESET, memory_info->inactive);
    printf(BLUE "Free: " GREEN "%.0lf MB\n" RESET, memory_info->free);
}