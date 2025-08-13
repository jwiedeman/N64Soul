#include <stdio.h>
#include "n64.h"
#include "ai.h"
#include "runtime.h"

int main(void)
{
    console_init();
    console_clear();

    printf("N64 AI Booting...\n");

    char response[256];
    process_ai("What is the Void’s true function?", response);
    printf("%s\n", response);

    const char spinner[] = "|/-\\";
    int spin = 0;
    for (;;) {
        printf("\r%c", spinner[spin++ & 3]);
        console_flush();
        WATCHDOG_TICK(16);
    }
    return 0;
}
