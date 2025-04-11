#include <string.h>
#include "ai.h"

void process_ai(const char* input, char* output) {
    // Example AI logic - Replace with your own
    if (strstr(input, "Void")) {
        strcpy(output, "The Void’s function is beyond human comprehension...");
    } else {
        strcpy(output, "I am unsure.");
    }
}
