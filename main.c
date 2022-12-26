#include <stdio.h>
#include <unistd.h>
#include <signal.h>

#define HIST_SIZE 50  // Number of values in the history
#define DELAY_SECONDS 1  // Number of seconds to sleep between updates
#define WHITE "\033[37m"  // White text
#define BLUE "\033[34m"  // Blue text
#define BLACK "\033[30m"  // Black text
#define GREEN "\033[32m"  // Green text
#define RESET "\033[0m"  // Reset the color

// Global flag to indicate if the program should continue running
sig_atomic_t keep_running = 1;

// Signal handler to set the keep_running flag to 0 when Ctrl+C is pressed
void sigint_handler(int sig) {
    keep_running = 0;
}

int main() {
    // Register the signal handler
    signal(SIGINT, sigint_handler);

    // Initialize the history buffer with all zeros
    double history[HIST_SIZE] = {0};
    int history_index = 0;

    while (keep_running) {
        // Parse the values from /proc/meminfo
        double zswap = 0, zswapped = 0, compression = 0;
        FILE *file = fopen("/proc/meminfo", "r");
        if (file != NULL) {
            char line[256];
            while (fgets(line, sizeof(line), file) != NULL) {
                if (sscanf(line, "Zswap: %lf", &zswap) == 1) {
                    // Zswap value found
                } else if (sscanf(line, "Zswapped: %lf", &zswapped) == 1) {
                    // Zswapped value found
                }
            }
            fclose(file);

            // Calculate the compression value
            compression = zswapped / zswap;
        }

        // Clear the screen
        printf("\033[H\033[2J");

        // Print the current values of Zswap and Zswapped in blue
        printf(BLUE "Zswap: %.0lf  Zswapped: %.0lf\n" RESET, zswap, zswapped);

        // Check if the compression value is valid (greater than zero)
        if (compression > 0) {
            // Print the graph
            int hist_height = 20;
            int hist_width = HIST_SIZE;

            // Print the top border in white
            printf(WHITE "+");
            for (int i = 0; i < hist_width; i++) {
                putchar('-');
            }
            printf("+\n" RESET);

            // Print each row of the graph
            for (int row = 0; row < hist_height; row++) {
                // Calculate the minimum value for this row
                double min_value = 1.0 - (double)row / (double)hist_height;

                // Print the left border in white
                printf(WHITE "|");

                // Print each column of the row
                for (int col = 0; col < hist_width; col++) {
                    int value_index = (history_index - col - 1 + HIST_SIZE) % HIST_SIZE;
                    double value = history[value_index];
                    if (value >= min_value) {
                        // This column is part of the graph
                        printf(GREEN "*" RESET);
                    } else {
                        // This column is not part of the graph
                        printf(BLACK " " RESET);
                    }
                }

                // Print the right border in white
                printf(WHITE "|\n" RESET);
            }

            // Print the bottom border in white
            printf(WHITE "+");
            for (int i = 0; i < hist_width; i++) {
                putchar('-');
            }
            printf("+\n" RESET);
        } else {
            // Print an error message in red
            printf("\033[31mError: Invalid compression value\033[0m\n");
        }

        // Update the circular buffer
        history[history_index] = compression;
        history_index = (history_index + 1) % HIST_SIZE;

        // Sleep for a brief period before updating again
        sleep(DELAY_SECONDS);
    }

    return 0;
}

