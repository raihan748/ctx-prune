// Real C Sample File for ctx-prune verification
#include <stdio.h>
#include <stdlib.h>

/*
 * Multi-line header block comment
 * Author: SMK Telkom Developer
 */
int calculate_factorial(int n) {
    // Base case check
    if (n <= 1) {
        return 1;
    }
    return n * calculate_factorial(n - 1);
}

int main(int argc, char **argv) {
    /* Print a string that contains comment characters inside quotes */
    const char *url = "https://smktelkom-mks.sch.id//api/v1/*test*/";
    printf("Connecting to %s\n", url);
    int res = calculate_factorial(5); // Trailing comment
    printf("Result: %d\n", res);
    return 0;
}
