/**
 * PROJECT NEURON - Main Entry Point
 * N64 Neural Network Visualization System
 * Codename: DEEP PADDLE
 */

#include <libdragon.h>
#include <stdlib.h>
#include <string.h>
#include <math.h>

#include "../include/config.h"
#include "neural_net.h"
#include "training.h"
#include "pong.h"
#include "render.h"
#include "ui.h"
#include "save.h"

// =============================================================================
// GLOBAL STATE
// =============================================================================

static NeuralNetwork network;
static TrainingState training;
static ReplayBuffer replay_buffer;
static PongState pong;
static RenderSettings render_settings;
static UIState ui;
static int network_initialized = 0;  // Track if network allocation succeeded

// =============================================================================
// INITIALIZATION
// =============================================================================

static void init_all(void) {
    // Zero all global state first - real hardware may have garbage in memory
    memset(&network, 0, sizeof(network));
    memset(&training, 0, sizeof(training));
    memset(&replay_buffer, 0, sizeof(replay_buffer));
    memset(&pong, 0, sizeof(pong));
    memset(&render_settings, 0, sizeof(render_settings));
    memset(&ui, 0, sizeof(ui));

    // Initialize libdragon subsystems
    display_init(RESOLUTION_320x240, DEPTH_16_BPP, 2, GAMMA_NONE, ANTIALIAS_RESAMPLE);
    rdpq_init();
    joypad_init();  // Use modern joypad API instead of deprecated controller_init
    timer_init();

    // Initialize our systems
    ui_init(&ui);
    render_init(&render_settings);
    training_init(&training);
    replay_buffer_init(&replay_buffer);
    pong_init(&pong);

    // Initialize network with default tier
    if (nn_init(&network, DEFAULT_TIER) == 0) {
        network_initialized = 1;
    } else {
        // Network allocation failed - will skip training features
        network_initialized = 0;
    }
}

// =============================================================================
// MAIN SIMULATION STEP
// =============================================================================

static void simulation_step(void) {
    // Save previous ball X for reward calculation
    float prev_ball_x = pong.ball_x;

    // 1. Get normalized state
    float state[STATE_SIZE];
    pong_get_normalized_state(&pong, state);

    // 2. Select action (epsilon-greedy during training)
    int action;
    if (ui.current_state == STATE_SIM_TRAINING) {
        action = select_action_epsilon_greedy(&network, state, training.epsilon);
    } else {
        // During play/watch mode, always use best action
        action = nn_get_best_action(&network, state);
    }

    // 3. Execute action
    pong_execute_ai_action(&pong, action);

    // 4. Update opponent
    pong_update_opponent(&pong);

    // 5. Step game simulation
    pong_step(&pong);

    // 6. Get reward and new state
    float reward = pong_calculate_reward(&pong, prev_ball_x);
    float next_state[STATE_SIZE];
    pong_get_normalized_state(&pong, next_state);
    uint8_t done = !pong.ball_served;

    // 7. During training, store transition and learn
    if (ui.current_state == STATE_SIM_TRAINING) {
        // Store in replay buffer
        replay_buffer_add(&replay_buffer, state, action, reward, next_state, done);

        // Train on batch if buffer is ready
        if (replay_buffer_ready(&replay_buffer, training.batch_size)) {
            float loss = train_batch(&network, &replay_buffer, &training);
            loss_history_add(&training, loss);
        }

        // Decay epsilon
        training_decay_epsilon(&training);

        // Record step
        training_record_step(&training, reward);

        training.total_steps++;
    }

    // 8. Handle point scored / episode end
    if (done) {
        if (ui.current_state == STATE_SIM_TRAINING) {
            training_end_episode(&training, pong.ai_score, pong.opp_score);
        }

        // Check if game is over
        if (pong_is_done(&pong, 11)) {
            pong_reset(&pong);
        } else {
            // Serve new ball
            pong_serve(&pong, pong.last_scorer == 0);
        }
    }
}

// =============================================================================
// MAIN LOOP
// =============================================================================

int main(void) {
    init_all();

    while (1) {
        // Poll joypad (modern API - reads controllers asynchronously)
        joypad_poll();

        // Read input from port 1 if connected
        joypad_buttons_t buttons_pressed = {0};
        joypad_inputs_t inputs = {0};

        if (joypad_is_connected(JOYPAD_PORT_1)) {
            buttons_pressed = joypad_get_buttons_pressed(JOYPAD_PORT_1);
            inputs = joypad_get_inputs(JOYPAD_PORT_1);
        }

        // Handle input based on current state
        // Convert joypad_buttons_t to a simple bitmask for ui_handle_input
        uint16_t btn_mask = 0;
        if (buttons_pressed.a) btn_mask |= 0x8000;      // A
        if (buttons_pressed.b) btn_mask |= 0x4000;      // B
        if (buttons_pressed.z) btn_mask |= 0x2000;      // Z
        if (buttons_pressed.start) btn_mask |= 0x1000;  // Start
        if (buttons_pressed.d_up) btn_mask |= 0x0800;   // D-Up
        if (buttons_pressed.d_down) btn_mask |= 0x0400; // D-Down
        if (buttons_pressed.d_left) btn_mask |= 0x0200; // D-Left
        if (buttons_pressed.d_right) btn_mask |= 0x0100;// D-Right
        if (buttons_pressed.l) btn_mask |= 0x0020;      // L
        if (buttons_pressed.r) btn_mask |= 0x0010;      // R

        ui_handle_input(&ui, btn_mask, inputs.stick_x, inputs.stick_y);

        // Run simulation steps based on speed multiplier (only if network is initialized)
        if (network_initialized &&
            (ui.current_state == STATE_SIM_TRAINING ||
             ui.current_state == STATE_SIM_PLAY ||
             ui.current_state == STATE_SIM_WATCH)) {

            for (int i = 0; i < ui.speed_multiplier; i++) {
                simulation_step();
            }

            // Update network visualization state
            nn_update_vis_state(&network);
        }

        // Get display context
        display_context_t disp = display_lock();
        if (disp) {
            // Set render context for all render functions
            render_set_context(disp);

            // Clear screen to dark blue (Skunkworks terminal style)
            graphics_fill_screen(disp, graphics_make_color(0, 0, 32, 255));

            // Render based on state
            switch (ui.current_state) {
                case STATE_BOOT:
                    ui_render_boot(ui.boot_timer);
                    break;

                case STATE_TITLE:
                    ui_render_title(ui.transition_timer);
                    break;

                case STATE_MENU_MAIN:
                case STATE_MENU_LOAD:
                    ui_render_menu(ui.current_menu, ui.menu_cursor);
                    break;

                case STATE_MENU_SETTINGS:
                case STATE_TRAINING_SETUP:
                    ui_render_settings(&training, &render_settings,
                                      ui.selected_tier, ui.menu_cursor);
                    break;

                case STATE_SIM_TRAINING:
                case STATE_SIM_PLAY:
                case STATE_SIM_WATCH:
                    render_frame(&pong, &network, &training, &render_settings);
                    break;

                case STATE_SIM_PAUSED:
                    // Render game in background, then pause overlay
                    render_frame(&pong, &network, &training, &render_settings);
                    ui_render_pause(ui.menu_cursor);
                    break;

                case STATE_TUTORIAL:
                    ui_render_tutorial(ui.tutorial_page);
                    break;

                default:
                    break;
            }

            // Apply scanlines if enabled
            if (render_settings.scanlines_enabled) {
                render_scanlines();
            }

            display_show(disp);

            // Update animation timers INSIDE vsync block (once per frame)
            ui.transition_timer++;
            ui.cursor_blink_timer++;
            if (ui.current_state == STATE_BOOT) {
                ui.boot_timer++;
                if (ui.boot_timer > 120) {  // 2 seconds at 60fps
                    ui_transition(&ui, STATE_TITLE);
                }
            }
        }
    }

    return 0;
}
