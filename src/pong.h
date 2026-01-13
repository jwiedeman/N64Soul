/**
 * PROJECT NEURON - Pong Game Header
 * Pong game simulation and state management
 */

#ifndef PONG_H
#define PONG_H

#include <stdint.h>
#include "../include/config.h"

// =============================================================================
// DATA STRUCTURES
// =============================================================================

/**
 * Complete pong game state
 */
typedef struct {
    // Ball state
    float ball_x;
    float ball_y;
    float ball_vx;
    float ball_vy;

    // Paddle positions (center Y coordinate)
    float ai_paddle_y;
    float opp_paddle_y;

    // Score
    int ai_score;
    int opp_score;

    // Game state flags
    uint8_t ball_served;      // Ball is in play
    uint8_t point_scored;     // Point was just scored (for effects)
    uint8_t last_scorer;      // 0 = AI, 1 = opponent

    // Rally tracking
    int rally_count;          // Current rally length
    int longest_rally;        // Longest rally this episode

} PongState;

// =============================================================================
// GAME LIFECYCLE
// =============================================================================

/**
 * Initialize pong game state
 * @param state Pointer to pong state
 */
void pong_init(PongState* state);

/**
 * Reset game for new episode (scores to 0)
 * @param state Pointer to pong state
 */
void pong_reset(PongState* state);

/**
 * Reset ball position and serve
 * @param state Pointer to pong state
 * @param serve_to_ai Ball served toward AI paddle (1) or opponent (0)
 */
void pong_serve(PongState* state, int serve_to_ai);

// =============================================================================
// GAME SIMULATION
// =============================================================================

/**
 * Advance game by one step
 * Updates ball position, handles collisions, scoring
 * @param state Pointer to pong state
 */
void pong_step(PongState* state);

/**
 * Move AI paddle based on action
 * @param state Pointer to pong state
 * @param action ACTION_UP, ACTION_STAY, or ACTION_DOWN
 */
void pong_execute_ai_action(PongState* state, int action);

/**
 * Update opponent paddle (simple tracking AI)
 * @param state Pointer to pong state
 */
void pong_update_opponent(PongState* state);

/**
 * Set opponent paddle position directly (for human play)
 * @param state Pointer to pong state
 * @param y Y position
 */
void pong_set_opponent_position(PongState* state, float y);

// =============================================================================
// STATE QUERIES
// =============================================================================

/**
 * Get normalized state vector for neural network input
 * @param state Pointer to pong state
 * @param out Output array (size = STATE_SIZE)
 */
void pong_get_normalized_state(const PongState* state, float* out);

/**
 * Calculate reward for current step
 * @param state Pointer to pong state
 * @param prev_ball_x Previous ball X position (for direction reward)
 * @return Reward value
 */
float pong_calculate_reward(const PongState* state, float prev_ball_x);

/**
 * Check if episode is done (game over)
 * @param state Pointer to pong state
 * @param max_score Score needed to win
 * @return 1 if episode done, 0 otherwise
 */
int pong_is_done(const PongState* state, int max_score);

/**
 * Check if point was just scored
 * @param state Pointer to pong state
 * @return 1 if point just scored, 0 otherwise
 */
int pong_point_just_scored(const PongState* state);

// =============================================================================
// COLLISION HELPERS
// =============================================================================

/**
 * Check if ball will hit AI paddle this frame
 * @param state Pointer to pong state
 * @return 1 if collision, 0 otherwise
 */
int pong_check_ai_paddle_collision(const PongState* state);

/**
 * Check if ball will hit opponent paddle this frame
 * @param state Pointer to pong state
 * @return 1 if collision, 0 otherwise
 */
int pong_check_opp_paddle_collision(const PongState* state);

// =============================================================================
// GEOMETRY CONSTANTS
// =============================================================================

// Paddle positions (X coordinates)
#define AI_PADDLE_X 20
#define OPP_PADDLE_X (SCREEN_WIDTH - 20 - PADDLE_WIDTH)

// Playfield bounds
#define PLAYFIELD_TOP 20
#define PLAYFIELD_BOTTOM (SCREEN_HEIGHT - 20)
#define PLAYFIELD_LEFT 10
#define PLAYFIELD_RIGHT (SCREEN_WIDTH - 10)

#endif // PONG_H
