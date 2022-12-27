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

// Function templates
void parse_memory_info(double *, double *, double *, double *, double *, double *);

void draw_line_graph(const double *, int, int);

void sigint_handler(int);

void print_memory_info(const double *, const double *, const double *, const double *, const double *, const double *);

int main() {
    // Register the signal handler
    signal(SIGINT, sigint_handler);

    // Initialize the history queue with 0s
    double history[HIST_SIZE] = {0};


    while (keep_running) {
        // Parse the values from /proc/meminfo
        double zswap = 0, zswapped = 0, compression = 0, active = 0, inactive = 0, free = 0;
        parse_memory_info(&zswap, &zswapped, &compression, &active, &inactive, &free);

        // Clear the screen
        printf("\033[H\033[2J");

        // Print the current values of the memory, keys should be blue and values should be green in the format "key: value" in MB
        print_memory_info(&zswap, &zswapped, &compression, &active, &inactive, &free);

        // Print the graph of the history of Zswapped values
        draw_line_graph(history, HIST_SIZE, 20);

        // Update the history queue with free value
        for (int i = 0; i < HIST_SIZE - 1; i++) {
            history[i] = history[i + 1];
        }
        history[HIST_SIZE - 1] = free;


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
void draw_line_graph(const double *history, int size, int graph_height) {
    double max = 0;
    double min = 0;

    // Find the maximum and minimum values in the history array
    for (int i = 0; i < size; ++i) {
        max = (history[i] > max) ? history[i] : max;
        min = (history[i] < min) ? history[i] : min;
    }

    double range = max - min;
    double scale = range / graph_height;
    double offset = min;
    double center = (max + min) / 2;

    // Print colored max, min, range, scale, offset, center, keys should be blue and values should be green in the format "key: value"
    printf(BLUE "max: " GREEN "%.0lf  " BLUE "min: " GREEN "%.0lf  " BLUE "range: " GREEN "%.0lf  " BLUE "scale: " GREEN "%.0lf  " BLUE "offset: " GREEN "%.0lf  " BLUE "center: " GREEN "%.0lf\n" RESET,
           max, min, range, scale, offset, center);

    // Print the top and bottom borders of the graph
    char border[size + 1];
    memset(border, '-', size);
    border[size] = '\0';

    printf("           +%s\n", border);
    // This loop iterates graph_height times, once for each line of the graph
    for (int i = 0; i < graph_height; i++) {
        // Print the y-axis value for this line of the graph. The value is calculated as the center value plus or minus an offset, depending on the iteration number. The offset is calculated by multiplying the scale value by the difference between the half the height of the graph and the current iteration number. The scale value determines the scaling of the y-axis, with each character on the y-axis representing a certain number of units.
        printf(GREEN "%10.0lf "RESET"|", center + ((double) graph_height / 2 - i) * scale);

        // This loop iterates over the values in the history array
        for (int j = 0; j < size; j++) {
            // Determine the range represented by the current y-axis value
            double range_min = (history[j] > center) ? max - (i * scale) : max - ((i + 1) * scale);
            double range_max = (history[j] > center) ? max - ((i - 1) * scale) : max - (i * scale);

            // If the value falls within the range, print a blue asterisk character. If it doesn't, print a space character.
            if (history[j] >= range_min && history[j] < range_max) {
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
void parse_memory_info(double *zswap, double *zswapped, double *compression, double *active, double *inactive,
                       double *free) {
    // Parse the values from /proc/meminfo
    FILE *file = fopen("/proc/meminfo", "r");
    if (file == NULL) {
        return;
    }

    char line[256];
    while (fgets(line, sizeof(line), file) != NULL) {
        sscanf(line, "Zswap: %lf", zswap);
        sscanf(line, "Zswapped: %lf", zswapped);
        sscanf(line, "Active(anon): %lf", active);
        sscanf(line, "Inactive(anon): %lf", inactive);
        sscanf(line, "MemFree: %lf", free);
    }
    fclose(file);

    // Calculate the compression value
    *compression = *zswapped / *zswap;
}

// Signal handler to set the keep_running flag to 0 when Ctrl+C is pressed
void sigint_handler(int sig) {
    keep_running = 0;
}

// Function to print the memory info in the format "key: value" in MB with the keys colored blue and the values colored green, each key value pair should be on a new line
void print_memory_info(const double *zswap, const double *zswapped, const double *compression, const double *active,
                       const double *inactive, const double *free) {
    printf(BLUE "Zswap: " GREEN "%.0lf MB\n" RESET, *zswap / 1024);
    printf(BLUE "Zswapped: " GREEN "%.0lf MB\n" RESET, *zswapped / 1024);
    printf(BLUE "Compression: " GREEN "%.2lf\n" RESET, *compression);
    printf(BLUE "Active(anon): " GREEN "%.0lf MB\n" RESET, *active / 1024);
    printf(BLUE "Inactive(anon): " GREEN "%.0lf MB\n" RESET, *inactive / 1024);
    printf(BLUE "MemFree: " GREEN "%.0lf MB\n" RESET, *free / 1024);
}