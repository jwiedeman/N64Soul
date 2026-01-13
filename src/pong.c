/**
 * PROJECT NEURON - Pong Game Implementation
 * Game simulation, physics, collision detection
 */

#include <stdlib.h>
#include <string.h>
#include <math.h>
#include "pong.h"

// =============================================================================
// RANDOM NUMBER GENERATION (for ball angle variation)
// =============================================================================

static uint32_t pong_rng_state = 98765;

static uint32_t pong_xorshift32(void) {
    uint32_t x = pong_rng_state;
    x ^= x << 13;
    x ^= x >> 17;
    x ^= x << 5;
    pong_rng_state = x;
    return x;
}

static float pong_randf(void) {
    return (float)(pong_xorshift32() & 0x7FFFFFFF) / (float)0x7FFFFFFF;
}

// =============================================================================
// HELPER FUNCTIONS
// =============================================================================

/**
 * Clamp value to range
 */
static float clampf(float val, float min, float max) {
    if (val < min) return min;
    if (val > max) return max;
    return val;
}

/**
 * Sign function
 */
static float signf(float val) {
    if (val > 0) return 1.0f;
    if (val < 0) return -1.0f;
    return 0.0f;
}

// =============================================================================
// GAME LIFECYCLE
// =============================================================================

void pong_init(PongState* state) {
    memset(state, 0, sizeof(PongState));

    // Center paddles
    state->ai_paddle_y = SCREEN_HEIGHT / 2.0f;
    state->opp_paddle_y = SCREEN_HEIGHT / 2.0f;

    // Serve ball
    pong_serve(state, 1);
}

void pong_reset(PongState* state) {
    state->ai_score = 0;
    state->opp_score = 0;
    state->rally_count = 0;
    state->longest_rally = 0;

    // Center paddles
    state->ai_paddle_y = SCREEN_HEIGHT / 2.0f;
    state->opp_paddle_y = SCREEN_HEIGHT / 2.0f;

    // Serve to whoever lost last point (or AI by default)
    pong_serve(state, 1);
}

void pong_serve(PongState* state, int serve_to_ai) {
    // Center ball
    state->ball_x = SCREEN_WIDTH / 2.0f;
    state->ball_y = SCREEN_HEIGHT / 2.0f;

    // Random angle between -45 and +45 degrees from horizontal
    float angle = (pong_randf() - 0.5f) * 1.57f;  // +/- 45 degrees

    state->ball_vx = BALL_INITIAL_SPEED * cosf(angle);
    state->ball_vy = BALL_INITIAL_SPEED * sinf(angle);

    // Direction based on who we're serving to
    if (serve_to_ai) {
        state->ball_vx = -fabsf(state->ball_vx);  // Go left (toward AI)
    } else {
        state->ball_vx = fabsf(state->ball_vx);   // Go right (toward opponent)
    }

    state->ball_served = 1;
    state->point_scored = 0;
    state->rally_count = 0;
}

// =============================================================================
// GAME SIMULATION
// =============================================================================

void pong_step(PongState* state) {
    if (!state->ball_served) {
        return;
    }

    state->point_scored = 0;

    // Store old position for collision detection
    float old_ball_x = state->ball_x;
    float old_ball_y = state->ball_y;

    // Move ball
    state->ball_x += state->ball_vx;
    state->ball_y += state->ball_vy;

    // Top/bottom wall collision
    float ball_half = BALL_SIZE / 2.0f;
    if (state->ball_y - ball_half < PLAYFIELD_TOP) {
        state->ball_y = PLAYFIELD_TOP + ball_half;
        state->ball_vy = -state->ball_vy;
    }
    if (state->ball_y + ball_half > PLAYFIELD_BOTTOM) {
        state->ball_y = PLAYFIELD_BOTTOM - ball_half;
        state->ball_vy = -state->ball_vy;
    }

    // AI paddle collision (left side)
    if (state->ball_vx < 0) {  // Ball moving left
        float paddle_left = AI_PADDLE_X;
        float paddle_right = AI_PADDLE_X + PADDLE_WIDTH;
        float paddle_top = state->ai_paddle_y - PADDLE_HEIGHT / 2.0f;
        float paddle_bottom = state->ai_paddle_y + PADDLE_HEIGHT / 2.0f;

        if (state->ball_x - ball_half <= paddle_right &&
            old_ball_x - ball_half > paddle_right) {
            // Ball crossed paddle x-line
            if (state->ball_y + ball_half >= paddle_top &&
                state->ball_y - ball_half <= paddle_bottom) {
                // Hit paddle!
                state->ball_x = paddle_right + ball_half;
                state->ball_vx = -state->ball_vx;

                // Add angle based on where ball hit paddle
                float hit_pos = (state->ball_y - state->ai_paddle_y) /
                               (PADDLE_HEIGHT / 2.0f);
                hit_pos = clampf(hit_pos, -1.0f, 1.0f);
                state->ball_vy += hit_pos * 2.0f;

                // Speed up slightly
                float speed = sqrtf(state->ball_vx * state->ball_vx +
                                   state->ball_vy * state->ball_vy);
                if (speed < BALL_MAX_SPEED) {
                    float factor = 1.05f;
                    state->ball_vx *= factor;
                    state->ball_vy *= factor;
                }

                // Clamp vertical speed
                state->ball_vy = clampf(state->ball_vy, -BALL_MAX_SPEED, BALL_MAX_SPEED);

                state->rally_count++;
                if (state->rally_count > state->longest_rally) {
                    state->longest_rally = state->rally_count;
                }
            }
        }
    }

    // Opponent paddle collision (right side)
    if (state->ball_vx > 0) {  // Ball moving right
        float paddle_left = OPP_PADDLE_X;
        float paddle_right = OPP_PADDLE_X + PADDLE_WIDTH;
        float paddle_top = state->opp_paddle_y - PADDLE_HEIGHT / 2.0f;
        float paddle_bottom = state->opp_paddle_y + PADDLE_HEIGHT / 2.0f;

        if (state->ball_x + ball_half >= paddle_left &&
            old_ball_x + ball_half < paddle_left) {
            // Ball crossed paddle x-line
            if (state->ball_y + ball_half >= paddle_top &&
                state->ball_y - ball_half <= paddle_bottom) {
                // Hit paddle!
                state->ball_x = paddle_left - ball_half;
                state->ball_vx = -state->ball_vx;

                // Add angle based on where ball hit paddle
                float hit_pos = (state->ball_y - state->opp_paddle_y) /
                               (PADDLE_HEIGHT / 2.0f);
                hit_pos = clampf(hit_pos, -1.0f, 1.0f);
                state->ball_vy += hit_pos * 2.0f;

                // Speed up slightly
                float speed = sqrtf(state->ball_vx * state->ball_vx +
                                   state->ball_vy * state->ball_vy);
                if (speed < BALL_MAX_SPEED) {
                    float factor = 1.05f;
                    state->ball_vx *= factor;
                    state->ball_vy *= factor;
                }

                // Clamp vertical speed
                state->ball_vy = clampf(state->ball_vy, -BALL_MAX_SPEED, BALL_MAX_SPEED);

                state->rally_count++;
            }
        }
    }

    // Check for scoring
    if (state->ball_x - ball_half < PLAYFIELD_LEFT) {
        // Ball passed AI paddle - opponent scores
        state->opp_score++;
        state->ball_served = 0;
        state->point_scored = 1;
        state->last_scorer = 1;
    } else if (state->ball_x + ball_half > PLAYFIELD_RIGHT) {
        // Ball passed opponent paddle - AI scores
        state->ai_score++;
        state->ball_served = 0;
        state->point_scored = 1;
        state->last_scorer = 0;
    }
}

// =============================================================================
// PADDLE CONTROL
// =============================================================================

void pong_execute_ai_action(PongState* state, int action) {
    float paddle_half = PADDLE_HEIGHT / 2.0f;

    switch (action) {
        case ACTION_UP:
            state->ai_paddle_y -= PADDLE_SPEED;
            break;
        case ACTION_DOWN:
            state->ai_paddle_y += PADDLE_SPEED;
            break;
        case ACTION_STAY:
        default:
            // Do nothing
            break;
    }

    // Clamp to playfield
    state->ai_paddle_y = clampf(state->ai_paddle_y,
                                PLAYFIELD_TOP + paddle_half,
                                PLAYFIELD_BOTTOM - paddle_half);
}

void pong_update_opponent(PongState* state) {
    // Simple tracking AI: follow ball with some delay/imperfection
    float paddle_half = PADDLE_HEIGHT / 2.0f;

    // Only track when ball is coming toward opponent
    if (state->ball_vx > 0) {
        float target = state->ball_y;

        // Add some prediction based on ball velocity
        float time_to_reach = (OPP_PADDLE_X - state->ball_x) / state->ball_vx;
        target += state->ball_vy * time_to_reach * 0.5f;  // Partial prediction

        // Move toward target with speed limit
        float diff = target - state->opp_paddle_y;
        float max_move = PADDLE_SPEED * 0.9f;  // Slightly slower than AI can be

        if (diff > max_move) {
            state->opp_paddle_y += max_move;
        } else if (diff < -max_move) {
            state->opp_paddle_y -= max_move;
        } else {
            state->opp_paddle_y += diff;
        }
    }

    // Clamp to playfield
    state->opp_paddle_y = clampf(state->opp_paddle_y,
                                 PLAYFIELD_TOP + paddle_half,
                                 PLAYFIELD_BOTTOM - paddle_half);
}

void pong_set_opponent_position(PongState* state, float y) {
    float paddle_half = PADDLE_HEIGHT / 2.0f;
    state->opp_paddle_y = clampf(y,
                                 PLAYFIELD_TOP + paddle_half,
                                 PLAYFIELD_BOTTOM - paddle_half);
}

// =============================================================================
// STATE QUERIES
// =============================================================================

void pong_get_normalized_state(const PongState* state, float* out) {
    // Normalize all values to approximately [-1, 1] range

    // Ball X: 0 to SCREEN_WIDTH -> -1 to 1
    out[STATE_BALL_X] = (state->ball_x - SCREEN_WIDTH / 2.0f) /
                        (SCREEN_WIDTH / 2.0f);

    // Ball Y: 0 to SCREEN_HEIGHT -> -1 to 1
    out[STATE_BALL_Y] = (state->ball_y - SCREEN_HEIGHT / 2.0f) /
                        (SCREEN_HEIGHT / 2.0f);

    // Ball velocity X: -MAX_SPEED to MAX_SPEED -> -1 to 1
    out[STATE_BALL_VX] = state->ball_vx / BALL_MAX_SPEED;

    // Ball velocity Y: -MAX_SPEED to MAX_SPEED -> -1 to 1
    out[STATE_BALL_VY] = state->ball_vy / BALL_MAX_SPEED;

    // AI paddle Y: 0 to SCREEN_HEIGHT -> -1 to 1
    out[STATE_PADDLE_Y] = (state->ai_paddle_y - SCREEN_HEIGHT / 2.0f) /
                          (SCREEN_HEIGHT / 2.0f);

    // Opponent paddle Y: 0 to SCREEN_HEIGHT -> -1 to 1
    out[STATE_OPPONENT_Y] = (state->opp_paddle_y - SCREEN_HEIGHT / 2.0f) /
                            (SCREEN_HEIGHT / 2.0f);
}

float pong_calculate_reward(const PongState* state, float prev_ball_x) {
    float reward = 0.0f;

    // Major rewards for scoring
    if (state->point_scored) {
        if (state->last_scorer == 0) {
            // AI scored
            reward += REWARD_SCORE;
        } else {
            // Opponent scored
            reward += REWARD_OPPONENT_SCORE;
        }
    }

    // Small reward for ball moving toward opponent (shaping)
    if (state->ball_vx > 0) {
        reward += REWARD_BALL_TOWARD_OPPONENT;
    }

    // Tiny time penalty to encourage efficient play
    reward += REWARD_TIME_PENALTY;

    return reward;
}

int pong_is_done(const PongState* state, int max_score) {
    return (state->ai_score >= max_score || state->opp_score >= max_score);
}

int pong_point_just_scored(const PongState* state) {
    return state->point_scored;
}

// =============================================================================
// COLLISION HELPERS
// =============================================================================

int pong_check_ai_paddle_collision(const PongState* state) {
    if (state->ball_vx >= 0) return 0;  // Ball moving away

    float ball_half = BALL_SIZE / 2.0f;
    float paddle_right = AI_PADDLE_X + PADDLE_WIDTH;
    float paddle_top = state->ai_paddle_y - PADDLE_HEIGHT / 2.0f;
    float paddle_bottom = state->ai_paddle_y + PADDLE_HEIGHT / 2.0f;

    // Check if ball will reach paddle this frame
    float next_ball_x = state->ball_x + state->ball_vx;
    if (next_ball_x - ball_half <= paddle_right) {
        float next_ball_y = state->ball_y + state->ball_vy;
        if (next_ball_y + ball_half >= paddle_top &&
            next_ball_y - ball_half <= paddle_bottom) {
            return 1;
        }
    }

    return 0;
}

int pong_check_opp_paddle_collision(const PongState* state) {
    if (state->ball_vx <= 0) return 0;  // Ball moving away

    float ball_half = BALL_SIZE / 2.0f;
    float paddle_left = OPP_PADDLE_X;
    float paddle_top = state->opp_paddle_y - PADDLE_HEIGHT / 2.0f;
    float paddle_bottom = state->opp_paddle_y + PADDLE_HEIGHT / 2.0f;

    // Check if ball will reach paddle this frame
    float next_ball_x = state->ball_x + state->ball_vx;
    if (next_ball_x + ball_half >= paddle_left) {
        float next_ball_y = state->ball_y + state->ball_vy;
        if (next_ball_y + ball_half >= paddle_top &&
            next_ball_y - ball_half <= paddle_bottom) {
            return 1;
        }
    }

    return 0;
}
