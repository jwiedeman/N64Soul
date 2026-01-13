/**
 * PROJECT NEURON - UI Header
 * Menu system and settings panel
 */

#ifndef UI_H
#define UI_H

#include <stdint.h>
#include "../include/config.h"
#include "training.h"
#include "render.h"

// =============================================================================
// APPLICATION STATES
// =============================================================================

typedef enum {
    STATE_BOOT,           // Logo, init hardware
    STATE_TITLE,          // Title screen, press start
    STATE_MENU_MAIN,      // New/Load/Settings/About
    STATE_MENU_SETTINGS,  // Network tier, speed, display options
    STATE_MENU_LOAD,      // Load from Controller Pak or built-in checkpoints
    STATE_TRAINING_SETUP, // Confirm settings before starting
    STATE_SIM_TRAINING,   // Auto-play, learning enabled
    STATE_SIM_PAUSED,     // Frozen, menu overlay
    STATE_SIM_PLAY,       // Human vs AI, learning disabled
    STATE_SIM_WATCH,      // AI vs AI, learning disabled (demo mode)
    STATE_SAVING,         // Writing to Controller Pak
    STATE_LOADING,        // Reading from Controller Pak
    STATE_TUTORIAL,       // How it works screens
    STATE_ABOUT,          // Credits/info
} AppState;

// =============================================================================
// MENU ITEM TYPES
// =============================================================================

typedef enum {
    MENU_ACTION,          // Triggers an action on select
    MENU_TOGGLE,          // On/off toggle
    MENU_SLIDER,          // Numeric slider
    MENU_CHOICE,          // Multiple choice (tier selection, etc.)
} MenuItemType;

typedef struct {
    const char* label;
    MenuItemType type;

    // For toggles
    uint8_t* toggle_value;

    // For sliders
    float* slider_value;
    float slider_min;
    float slider_max;
    float slider_step;
    int slider_decimals;

    // For choices
    int* choice_value;
    const char** choice_labels;
    int choice_count;

    // For actions
    AppState action_state;  // State to transition to

} MenuItem;

typedef struct {
    const char* title;
    MenuItem* items;
    int item_count;
    int selected_index;
} Menu;

// =============================================================================
// UI STATE
// =============================================================================

typedef struct {
    AppState current_state;
    AppState previous_state;

    // Menu navigation
    Menu* current_menu;
    int menu_cursor;

    // Animation timers
    int transition_timer;
    int cursor_blink_timer;

    // Tutorial page
    int tutorial_page;
    int tutorial_page_count;

    // Boot sequence
    int boot_timer;

    // Simulation speed multiplier
    int speed_multiplier;

    // Selected network tier
    int selected_tier;

} UIState;

// =============================================================================
// INITIALIZATION
// =============================================================================

/**
 * Initialize UI state
 * @param ui Pointer to UI state
 */
void ui_init(UIState* ui);

// =============================================================================
// STATE MANAGEMENT
// =============================================================================

/**
 * Transition to new state
 * @param ui Pointer to UI state
 * @param new_state State to transition to
 */
void ui_transition(UIState* ui, AppState new_state);

/**
 * Go back to previous state
 * @param ui Pointer to UI state
 */
void ui_go_back(UIState* ui);

// =============================================================================
// INPUT HANDLING
// =============================================================================

/**
 * Handle input for current state
 * @param ui Pointer to UI state
 * @param buttons Button state (from controller)
 * @param stick_x Analog stick X (-128 to 127)
 * @param stick_y Analog stick Y (-128 to 127)
 */
void ui_handle_input(UIState* ui, uint16_t buttons,
                     int8_t stick_x, int8_t stick_y);

// =============================================================================
// RENDERING
// =============================================================================

/**
 * Render current UI state
 * @param ui Pointer to UI state
 * @param training Pointer to training state (for settings display)
 * @param render Pointer to render settings
 */
void ui_render(const UIState* ui,
               const TrainingState* training,
               const RenderSettings* render);

/**
 * Render boot screen
 * @param timer Boot animation timer
 */
void ui_render_boot(int timer);

/**
 * Render title screen
 * @param timer Animation timer
 */
void ui_render_title(int timer);

/**
 * Render main menu
 * @param menu Pointer to menu
 * @param cursor Current selection
 */
void ui_render_menu(const Menu* menu, int cursor);

/**
 * Render settings panel
 * @param training Pointer to training state
 * @param render Pointer to render settings
 * @param tier Selected network tier
 * @param cursor Current selection
 */
void ui_render_settings(const TrainingState* training,
                        const RenderSettings* render,
                        int tier, int cursor);

/**
 * Render pause overlay
 * @param cursor Current selection
 */
void ui_render_pause(int cursor);

/**
 * Render tutorial page
 * @param page Page number
 */
void ui_render_tutorial(int page);

// =============================================================================
// MENU DEFINITIONS
// =============================================================================

/**
 * Get main menu definition
 * @return Pointer to main menu
 */
Menu* ui_get_main_menu(void);

/**
 * Get pause menu definition
 * @return Pointer to pause menu
 */
Menu* ui_get_pause_menu(void);

/**
 * Get load menu definition
 * @return Pointer to load menu
 */
Menu* ui_get_load_menu(void);

// =============================================================================
// BUTTON DEFINITIONS (N64 Controller)
// =============================================================================

#define BTN_A       0x8000
#define BTN_B       0x4000
#define BTN_Z       0x2000
#define BTN_START   0x1000
#define BTN_DU      0x0800  // D-pad up
#define BTN_DD      0x0400  // D-pad down
#define BTN_DL      0x0200  // D-pad left
#define BTN_DR      0x0100  // D-pad right
#define BTN_L       0x0020
#define BTN_R       0x0010
#define BTN_CU      0x0008  // C-up
#define BTN_CD      0x0004  // C-down
#define BTN_CL      0x0002  // C-left
#define BTN_CR      0x0001  // C-right

#endif // UI_H
