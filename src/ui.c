/**
 * PROJECT NEURON - UI Implementation
 * Menu system, state machine, input handling
 */

#include <libdragon.h>
#include <stdlib.h>
#include <string.h>
#include <stdio.h>
#include "ui.h"
#include "render.h"

// =============================================================================
// STATIC MENU DEFINITIONS
// =============================================================================

// Main menu items
static MenuItem main_menu_items[] = {
    {"NEW TRAINING", MENU_ACTION, NULL, NULL, 0, 0, 0, 0, NULL, NULL, 0, STATE_TRAINING_SETUP},
    {"LOAD CHECKPOINT", MENU_ACTION, NULL, NULL, 0, 0, 0, 0, NULL, NULL, 0, STATE_MENU_LOAD},
    {"SETTINGS", MENU_ACTION, NULL, NULL, 0, 0, 0, 0, NULL, NULL, 0, STATE_MENU_SETTINGS},
    {"HOW IT WORKS", MENU_ACTION, NULL, NULL, 0, 0, 0, 0, NULL, NULL, 0, STATE_TUTORIAL},
};

static Menu main_menu = {
    "MAIN MENU",
    main_menu_items,
    4,
    0
};

// Pause menu items
static MenuItem pause_menu_items[] = {
    {"RESUME", MENU_ACTION, NULL, NULL, 0, 0, 0, 0, NULL, NULL, 0, STATE_SIM_TRAINING},
    {"SETTINGS", MENU_ACTION, NULL, NULL, 0, 0, 0, 0, NULL, NULL, 0, STATE_MENU_SETTINGS},
    {"RESET WEIGHTS", MENU_ACTION, NULL, NULL, 0, 0, 0, 0, NULL, NULL, 0, STATE_SIM_TRAINING},
    {"EXIT TO MENU", MENU_ACTION, NULL, NULL, 0, 0, 0, 0, NULL, NULL, 0, STATE_MENU_MAIN},
};

static Menu pause_menu = {
    "PAUSED",
    pause_menu_items,
    4,
    0
};

// Load menu items
static const char* checkpoint_labels[] = {"RANDOM", "NOVICE", "COMPETENT", "EXPERT"};
static int selected_checkpoint = 0;

static MenuItem load_menu_items[] = {
    {"RANDOM", MENU_ACTION, NULL, NULL, 0, 0, 0, 0, NULL, NULL, 0, STATE_SIM_TRAINING},
    {"NOVICE", MENU_ACTION, NULL, NULL, 0, 0, 0, 0, NULL, NULL, 0, STATE_SIM_TRAINING},
    {"COMPETENT", MENU_ACTION, NULL, NULL, 0, 0, 0, 0, NULL, NULL, 0, STATE_SIM_TRAINING},
    {"EXPERT", MENU_ACTION, NULL, NULL, 0, 0, 0, 0, NULL, NULL, 0, STATE_SIM_TRAINING},
    {"BACK", MENU_ACTION, NULL, NULL, 0, 0, 0, 0, NULL, NULL, 0, STATE_MENU_MAIN},
};

static Menu load_menu = {
    "LOAD CHECKPOINT",
    load_menu_items,
    5,
    0
};

// Tier labels
static const char* tier_labels[] = {
    "MINIMAL (6-16-3)",
    "LIGHT (6-32-32-3)",
    "MEDIUM (6-64-64-32-3)",
    "HEAVY (6-128-128-64-32-3)",
    "SUPERHEAVY (6-256-256-128-64-3)"
};

// =============================================================================
// INITIALIZATION
// =============================================================================

void ui_init(UIState* ui) {
    ui->current_state = STATE_BOOT;
    ui->previous_state = STATE_BOOT;
    ui->current_menu = NULL;
    ui->menu_cursor = 0;
    ui->transition_timer = 0;
    ui->cursor_blink_timer = 0;
    ui->tutorial_page = 0;
    ui->tutorial_page_count = 5;
    ui->boot_timer = 0;
    ui->speed_multiplier = 1;
    ui->selected_tier = TIER_LIGHT;
}

// =============================================================================
// STATE MANAGEMENT
// =============================================================================

void ui_transition(UIState* ui, AppState new_state) {
    ui->previous_state = ui->current_state;
    ui->current_state = new_state;
    ui->transition_timer = 0;
    ui->menu_cursor = 0;

    // Set up menu pointers
    switch (new_state) {
        case STATE_MENU_MAIN:
            ui->current_menu = &main_menu;
            break;
        case STATE_MENU_LOAD:
            ui->current_menu = &load_menu;
            break;
        case STATE_SIM_PAUSED:
            ui->current_menu = &pause_menu;
            break;
        default:
            ui->current_menu = NULL;
            break;
    }
}

void ui_go_back(UIState* ui) {
    ui_transition(ui, ui->previous_state);
}

// =============================================================================
// INPUT HANDLING
// =============================================================================

// Track previous buttons to detect edges
static uint16_t prev_buttons = 0;

// Track previous stick state for debouncing
static int8_t prev_stick_y = 0;
static int stick_repeat_delay = 0;

static int button_pressed(uint16_t buttons, uint16_t button) {
    return (buttons & button) && !(prev_buttons & button);
}

// Check if analog stick just crossed threshold (edge detection)
static int stick_up_pressed(int8_t stick_y) {
    if (stick_y > 50 && prev_stick_y <= 50) return 1;
    // Allow repeat after delay
    if (stick_y > 50 && stick_repeat_delay == 0) {
        stick_repeat_delay = 10; // 10 frames between repeats
        return 1;
    }
    return 0;
}

static int stick_down_pressed(int8_t stick_y) {
    if (stick_y < -50 && prev_stick_y >= -50) return 1;
    // Allow repeat after delay
    if (stick_y < -50 && stick_repeat_delay == 0) {
        stick_repeat_delay = 10;
        return 1;
    }
    return 0;
}

void ui_handle_input(UIState* ui, uint16_t buttons,
                     int8_t stick_x, int8_t stick_y) {
    // Decrement stick repeat delay
    if (stick_repeat_delay > 0) stick_repeat_delay--;

    switch (ui->current_state) {
        case STATE_BOOT:
            // Any button skips boot
            if (buttons & (BTN_A | BTN_B | BTN_START)) {
                ui_transition(ui, STATE_TITLE);
            }
            break;

        case STATE_TITLE:
            if (button_pressed(buttons, BTN_START) ||
                button_pressed(buttons, BTN_A)) {
                ui_transition(ui, STATE_MENU_MAIN);
            }
            break;

        case STATE_MENU_MAIN:
        case STATE_MENU_LOAD:
        case STATE_SIM_PAUSED:
            // Navigate menu (with debounced analog stick)
            if (button_pressed(buttons, BTN_DU) || stick_up_pressed(stick_y)) {
                if (ui->menu_cursor > 0) {
                    ui->menu_cursor--;
                }
            }
            if (button_pressed(buttons, BTN_DD) || stick_down_pressed(stick_y)) {
                if (ui->current_menu && ui->menu_cursor < ui->current_menu->item_count - 1) {
                    ui->menu_cursor++;
                }
            }

            // Select
            if (button_pressed(buttons, BTN_A) || button_pressed(buttons, BTN_START)) {
                if (ui->current_menu && ui->menu_cursor < ui->current_menu->item_count) {
                    MenuItem* item = &ui->current_menu->items[ui->menu_cursor];
                    if (item->type == MENU_ACTION) {
                        ui_transition(ui, item->action_state);
                    }
                }
            }

            // Back
            if (button_pressed(buttons, BTN_B)) {
                if (ui->current_state == STATE_SIM_PAUSED) {
                    ui_transition(ui, STATE_SIM_TRAINING);
                } else if (ui->current_state != STATE_MENU_MAIN) {
                    ui_transition(ui, STATE_MENU_MAIN);
                }
            }
            break;

        case STATE_MENU_SETTINGS:
        case STATE_TRAINING_SETUP:
            // Tier selection with D-pad left/right
            if (button_pressed(buttons, BTN_DL)) {
                if (ui->selected_tier > 0) {
                    ui->selected_tier--;
                }
            }
            if (button_pressed(buttons, BTN_DR)) {
                if (ui->selected_tier < TIER_SUPERHEAVY) {
                    ui->selected_tier++;
                }
            }

            // Start training
            if (button_pressed(buttons, BTN_START) || button_pressed(buttons, BTN_A)) {
                ui_transition(ui, STATE_SIM_TRAINING);
            }

            // Back
            if (button_pressed(buttons, BTN_B)) {
                ui_transition(ui, STATE_MENU_MAIN);
            }
            break;

        case STATE_SIM_TRAINING:
        case STATE_SIM_WATCH:
            // Pause
            if (button_pressed(buttons, BTN_START)) {
                ui_transition(ui, STATE_SIM_PAUSED);
            }

            // Speed control
            if (button_pressed(buttons, BTN_A)) {
                ui->speed_multiplier *= 2;
                if (ui->speed_multiplier > 16) ui->speed_multiplier = 16;
            }
            if (button_pressed(buttons, BTN_B)) {
                ui->speed_multiplier /= 2;
                if (ui->speed_multiplier < 1) ui->speed_multiplier = 1;
            }

            // Switch to play mode
            if (button_pressed(buttons, BTN_Z)) {
                ui_transition(ui, STATE_SIM_PLAY);
            }
            break;

        case STATE_SIM_PLAY:
            // Back to training
            if (button_pressed(buttons, BTN_Z)) {
                ui_transition(ui, STATE_SIM_TRAINING);
            }
            if (button_pressed(buttons, BTN_START)) {
                ui_transition(ui, STATE_SIM_PAUSED);
            }
            break;

        case STATE_TUTORIAL:
            // Page navigation
            if (button_pressed(buttons, BTN_DR) || button_pressed(buttons, BTN_A)) {
                if (ui->tutorial_page < ui->tutorial_page_count - 1) {
                    ui->tutorial_page++;
                }
            }
            if (button_pressed(buttons, BTN_DL)) {
                if (ui->tutorial_page > 0) {
                    ui->tutorial_page--;
                }
            }
            if (button_pressed(buttons, BTN_B)) {
                ui_transition(ui, STATE_MENU_MAIN);
            }
            break;

        default:
            break;
    }

    prev_buttons = buttons;
    prev_stick_y = stick_y;
}

// =============================================================================
// UI RENDERING
// =============================================================================

void ui_render(const UIState* ui,
               const TrainingState* training,
               const RenderSettings* render) {
    switch (ui->current_state) {
        case STATE_BOOT:
            ui_render_boot(ui->boot_timer);
            break;
        case STATE_TITLE:
            ui_render_title(ui->transition_timer);
            break;
        case STATE_MENU_MAIN:
        case STATE_MENU_LOAD:
            ui_render_menu(ui->current_menu, ui->menu_cursor);
            break;
        case STATE_MENU_SETTINGS:
        case STATE_TRAINING_SETUP:
            ui_render_settings(training, render, ui->selected_tier, ui->menu_cursor);
            break;
        case STATE_SIM_PAUSED:
            ui_render_pause(ui->menu_cursor);
            break;
        case STATE_TUTORIAL:
            ui_render_tutorial(ui->tutorial_page);
            break;
        default:
            break;
    }
}

void ui_render_boot(int timer) {
    // Simple boot animation
    int y_offset = timer < 30 ? (30 - timer) * 4 : 0;

    render_text(SCREEN_WIDTH/2 - 40, 80 - y_offset, "PROJECT", COLOR_TERMINAL_GREEN);
    render_text(SCREEN_WIDTH/2 - 40, 100 - y_offset, "NEURON", COLOR_HOT_WHITE);

    if (timer > 60) {
        render_text(SCREEN_WIDTH/2 - 60, 160, "INITIALIZING...", COLOR_PHOSPHOR_DIM);

        // Progress bar
        int progress = (timer - 60) * 3;
        if (progress > 100) progress = 100;
        render_rect(60, 180, progress * 2, 8, COLOR_TERMINAL_GREEN);
        render_panel(58, 178, 204, 12, NULL, 0);
    }
}

void ui_render_title(int timer) {
    // Title text
    render_text(SCREEN_WIDTH/2 - 48, 40, "N E U R O N", COLOR_TERMINAL_GREEN);
    render_text(SCREEN_WIDTH/2 - 80, 60, "NEURAL NETWORK LABORATORY", COLOR_PHOSPHOR_DIM);

    // Animated subtitle
    render_text(SCREEN_WIDTH/2 - 60, 90, "CODENAME: DEEP PADDLE", COLOR_AMBER_WARN);

    // Blinking "press start" - 45 frames on, 45 off (1.5 second cycle at 60fps)
    if ((timer / 45) % 2 == 0) {
        render_text(SCREEN_WIDTH/2 - 48, 160, "- PRESS START -", COLOR_TERMINAL_GREEN);
    }

    // Footer
    render_hline(20, 210, SCREEN_WIDTH - 40, COLOR_PHOSPHOR_DIM);
    render_text(20, 220, "BLUE FROG ANALYTICS // v1.0", COLOR_PHOSPHOR_DIM);
}

void ui_render_menu(const Menu* menu, int cursor) {
    if (!menu) return;

    // Title
    render_text(SCREEN_WIDTH/2 - 40, 30, menu->title, COLOR_TERMINAL_GREEN);
    render_hline(SCREEN_WIDTH/2 - 60, 42, 120, COLOR_PHOSPHOR_DIM);

    // Menu items
    int y = 60;
    for (int i = 0; i < menu->item_count; i++) {
        uint32_t color = (i == cursor) ? COLOR_HOT_WHITE : COLOR_TERMINAL_GREEN;

        // Selection indicator
        if (i == cursor) {
            render_text(40, y, ">", COLOR_TERMINAL_GREEN);
            render_panel(50, y - 2, 220, 14, NULL, 1);
        }

        render_text(60, y, menu->items[i].label, color);
        y += 24;
    }

    // Controls hint
    render_hline(20, 200, SCREEN_WIDTH - 40, COLOR_PHOSPHOR_DIM);
    render_text(20, 210, "[A] SELECT  [B] BACK", COLOR_PHOSPHOR_DIM);
}

void ui_render_settings(const TrainingState* training,
                        const RenderSettings* render,
                        int tier, int cursor) {
    render_text(SCREEN_WIDTH/2 - 40, 20, "SETTINGS", COLOR_TERMINAL_GREEN);

    // Network tier selection
    render_text(20, 50, "NETWORK TIER:", COLOR_TERMINAL_GREEN);
    render_panel(18, 62, 284, 20, NULL, 1);

    // Tier display with arrows
    render_text(30, 68, "<", tier > 0 ? COLOR_TERMINAL_GREEN : COLOR_PHOSPHOR_DIM);
    render_text(140 - strlen(tier_labels[tier]) * 3, 68, tier_labels[tier], COLOR_HOT_WHITE);
    render_text(280, 68, ">", tier < TIER_SUPERHEAVY ? COLOR_TERMINAL_GREEN : COLOR_PHOSPHOR_DIM);

    // Tier info
    char buf[64];
    int param_counts[] = {163, 1379, 6819, 27171, 107395};
    snprintf(buf, sizeof(buf), "Parameters: %d", param_counts[tier]);
    render_text(20, 90, buf, COLOR_PHOSPHOR_DIM);

    // Hyperparameters display (read-only for now)
    render_text(20, 120, "HYPERPARAMETERS:", COLOR_TERMINAL_GREEN);

    snprintf(buf, sizeof(buf), "Learning Rate: %.4f", training->learning_rate);
    render_text(30, 135, buf, COLOR_PHOSPHOR_DIM);

    snprintf(buf, sizeof(buf), "Gamma: %.2f", training->gamma);
    render_text(30, 148, buf, COLOR_PHOSPHOR_DIM);

    snprintf(buf, sizeof(buf), "Batch Size: %d", training->batch_size);
    render_text(30, 161, buf, COLOR_PHOSPHOR_DIM);

    // Controls
    render_hline(20, 200, SCREEN_WIDTH - 40, COLOR_PHOSPHOR_DIM);
    render_text(20, 210, "[L/R] TIER  [START] BEGIN  [B] BACK", COLOR_PHOSPHOR_DIM);
}

void ui_render_pause(int cursor) {
    // Darken background (simulated with rectangles)
    for (int y = 0; y < SCREEN_HEIGHT; y += 4) {
        render_hline(0, y, SCREEN_WIDTH, 0x00000080);
    }

    // Pause panel
    int panel_x = 60;
    int panel_y = 50;
    int panel_w = 200;
    int panel_h = 140;

    // Panel background
    render_rect(panel_x + 2, panel_y + 2, panel_w - 4, panel_h - 4, COLOR_VOID);
    render_panel(panel_x, panel_y, panel_w, panel_h, NULL, 1);

    // Title
    render_text(panel_x + 70, panel_y + 10, "PAUSED", COLOR_AMBER_WARN);

    // Menu items
    const char* items[] = {"RESUME", "SETTINGS", "RESET WEIGHTS", "EXIT TO MENU"};
    int y = panel_y + 40;
    for (int i = 0; i < 4; i++) {
        uint32_t color = (i == cursor) ? COLOR_HOT_WHITE : COLOR_TERMINAL_GREEN;
        if (i == cursor) {
            render_text(panel_x + 20, y, ">", COLOR_TERMINAL_GREEN);
        }
        render_text(panel_x + 35, y, items[i], color);
        y += 20;
    }
}

void ui_render_tutorial(int page) {
    render_text(SCREEN_WIDTH/2 - 50, 20, "HOW IT WORKS", COLOR_TERMINAL_GREEN);

    char page_buf[32];
    snprintf(page_buf, sizeof(page_buf), "PAGE %d/5", page + 1);
    render_text(SCREEN_WIDTH - 70, 20, page_buf, COLOR_PHOSPHOR_DIM);

    int y = 50;

    switch (page) {
        case 0:  // Forward Pass
            render_text(20, y, "THE FORWARD PASS", COLOR_AMBER_WARN);
            y += 20;
            render_text(20, y, "Input -> Hidden Layers -> Output", COLOR_TERMINAL_GREEN);
            y += 15;
            render_text(20, y, "The network sees 6 numbers:", COLOR_PHOSPHOR_DIM);
            y += 12;
            render_text(30, y, "Ball X, Y position", COLOR_PHOSPHOR_DIM);
            y += 12;
            render_text(30, y, "Ball X, Y velocity", COLOR_PHOSPHOR_DIM);
            y += 12;
            render_text(30, y, "AI paddle Y position", COLOR_PHOSPHOR_DIM);
            y += 12;
            render_text(30, y, "Opponent paddle Y position", COLOR_PHOSPHOR_DIM);
            y += 20;
            render_text(20, y, "Output: UP, STAY, or DOWN", COLOR_TERMINAL_GREEN);
            break;

        case 1:  // Weights
            render_text(20, y, "WEIGHTS & CONNECTIONS", COLOR_AMBER_WARN);
            y += 20;
            render_text(20, y, "Each connection has a 'weight'", COLOR_TERMINAL_GREEN);
            y += 15;
            render_text(20, y, "Positive weights (green):", COLOR_TERMINAL_GREEN);
            render_text(30, y + 12, "Strengthen the signal", COLOR_PHOSPHOR_DIM);
            y += 30;
            render_text(20, y, "Negative weights (red):", COLOR_NEGATIVE_RED);
            render_text(30, y + 12, "Invert the signal", COLOR_PHOSPHOR_DIM);
            y += 30;
            render_text(20, y, "Line thickness = weight magnitude", COLOR_PHOSPHOR_DIM);
            break;

        case 2:  // Loss
            render_text(20, y, "THE LOSS FUNCTION", COLOR_AMBER_WARN);
            y += 20;
            render_text(20, y, "Loss = How wrong was the prediction?", COLOR_TERMINAL_GREEN);
            y += 20;
            render_text(20, y, "High loss = bad predictions", COLOR_NEGATIVE_RED);
            y += 15;
            render_text(20, y, "Low loss = good predictions", COLOR_TERMINAL_GREEN);
            y += 20;
            render_text(20, y, "Watch the loss curve descend", COLOR_PHOSPHOR_DIM);
            render_text(20, y + 12, "as the network learns!", COLOR_PHOSPHOR_DIM);
            break;

        case 3:  // Backprop
            render_text(20, y, "BACKPROPAGATION", COLOR_AMBER_WARN);
            y += 20;
            render_text(20, y, "After each mistake:", COLOR_TERMINAL_GREEN);
            y += 15;
            render_text(30, y, "1. Calculate how wrong we were", COLOR_PHOSPHOR_DIM);
            y += 12;
            render_text(30, y, "2. Trace back through network", COLOR_PHOSPHOR_DIM);
            y += 12;
            render_text(30, y, "3. Adjust weights to reduce error", COLOR_PHOSPHOR_DIM);
            y += 20;
            render_text(20, y, "Weights change to make better", COLOR_TERMINAL_GREEN);
            render_text(20, y + 12, "predictions next time!", COLOR_TERMINAL_GREEN);
            break;

        case 4:  // Exploration
            render_text(20, y, "EXPLORATION vs EXPLOITATION", COLOR_AMBER_WARN);
            y += 20;
            render_text(20, y, "Epsilon (e) controls randomness", COLOR_TERMINAL_GREEN);
            y += 20;
            render_text(20, y, "High e = More random moves", COLOR_AMBER_WARN);
            render_text(30, y + 12, "Tries new things (exploration)", COLOR_PHOSPHOR_DIM);
            y += 30;
            render_text(20, y, "Low e = Use what we learned", COLOR_TERMINAL_GREEN);
            render_text(30, y + 12, "Best known strategy (exploitation)", COLOR_PHOSPHOR_DIM);
            y += 25;
            render_text(20, y, "e starts at 1.0, decays to 0.05", COLOR_PHOSPHOR_DIM);
            break;
    }

    // Navigation
    render_hline(20, 200, SCREEN_WIDTH - 40, COLOR_PHOSPHOR_DIM);
    render_text(20, 210, "[< >] PAGE  [B] BACK", COLOR_PHOSPHOR_DIM);

    // Page dots
    int dot_x = SCREEN_WIDTH / 2 - 20;
    for (int i = 0; i < 5; i++) {
        uint32_t color = (i == page) ? COLOR_HOT_WHITE : COLOR_PHOSPHOR_DIM;
        render_circle_filled(dot_x + i * 10, 215, 2, color);
    }
}

// =============================================================================
// MENU GETTERS
// =============================================================================

Menu* ui_get_main_menu(void) {
    return &main_menu;
}

Menu* ui_get_pause_menu(void) {
    return &pause_menu;
}

Menu* ui_get_load_menu(void) {
    return &load_menu;
}
