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
    controller_init();
    timer_init();

    // Initialize our systems
    ui_init(&ui);
    render_init(&render_settings);
    training_init(&training);
    replay_buffer_init(&replay_buffer);
    pong_init(&pong);

    // Initialize network with default tier
    if (nn_init(&network, DEFAULT_TIER) != 0) {
        // Fatal error - can't allocate network
        // In real implementation, display error screen
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

    // Initialize controller data to zero to avoid garbage on real hardware
    struct controller_data keys_pressed;
    struct controller_data keys_held;
    memset(&keys_pressed, 0, sizeof(keys_pressed));
    memset(&keys_held, 0, sizeof(keys_held));

    while (1) {
        // Scan controllers
        controller_scan();

        // Only read controller if one is connected
        int controllers = get_controllers_present();
        if (controllers & CONTROLLER_1_INSERTED) {
            keys_pressed = get_keys_pressed();
            keys_held = get_keys_down();
        } else {
            // No controller - zero out to prevent garbage input
            memset(&keys_pressed, 0, sizeof(keys_pressed));
            memset(&keys_held, 0, sizeof(keys_held));
        }

        // Handle input based on current state
        ui_handle_input(&ui, keys_pressed.c[0].data,
                       keys_held.c[0].x, keys_held.c[0].y);

        // Run simulation steps based on speed multiplier
        if (ui.current_state == STATE_SIM_TRAINING ||
            ui.current_state == STATE_SIM_PLAY ||
            ui.current_state == STATE_SIM_WATCH) {

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
