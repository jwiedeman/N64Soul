#include <stdio.h>
#include "n64.h"
#include "ai.h"  // Include the AI function header

int main(void)   // Change `void main(void)` to `int main(void)`
{
    // N64 Graphics Setup
    console_init();
    console_clear();

    // Display Welcome Message
    printf("N64 AI Booting...\n");

    // Run AI Function
    char response[256];
    process_ai("What is the Void’s true function?", response); // Now properly declared

    // Print AI Response
    printf("%s\n", response);

    // Loop forever (N64 needs a main loop)
    while (1);
}
