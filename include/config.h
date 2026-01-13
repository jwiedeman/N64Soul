/**
 * PROJECT NEURON - Configuration Header
 * Build-time configuration for neural network and training parameters
 */

#ifndef CONFIG_H
#define CONFIG_H

// =============================================================================
// NETWORK ARCHITECTURE
// =============================================================================

#define MAX_LAYERS 6
#define MAX_NEURONS_PER_LAYER 256

// Network tiers (select one at build time or runtime)
#define TIER_MINIMAL    0   // 6 -> 16 -> 3
#define TIER_LIGHT      1   // 6 -> 32 -> 32 -> 3
#define TIER_MEDIUM     2   // 6 -> 64 -> 64 -> 32 -> 3
#define TIER_HEAVY      3   // 6 -> 128 -> 128 -> 64 -> 32 -> 3
#define TIER_SUPERHEAVY 4   // 6 -> 256 -> 256 -> 128 -> 64 -> 3

#define DEFAULT_TIER TIER_LIGHT

// =============================================================================
// TRAINING PARAMETERS
// =============================================================================

#define REPLAY_BUFFER_SIZE 2000
#define LOSS_HISTORY_SIZE 1000

// Default hyperparameters
#define DEFAULT_LEARNING_RATE 0.001f
#define DEFAULT_GAMMA 0.99f
#define DEFAULT_EPSILON_START 1.0f
#define DEFAULT_EPSILON_MIN 0.05f
#define DEFAULT_EPSILON_DECAY 0.9995f
#define DEFAULT_BATCH_SIZE 32
#define DEFAULT_TARGET_UPDATE_FREQ 100

// =============================================================================
// GAME PARAMETERS
// =============================================================================

#define SCREEN_WIDTH 320
#define SCREEN_HEIGHT 240

#define PADDLE_WIDTH 8
#define PADDLE_HEIGHT 40
#define PADDLE_SPEED 4.0f

#define BALL_SIZE 8
#define BALL_INITIAL_SPEED 4.0f
#define BALL_MAX_SPEED 8.0f

// =============================================================================
// REWARD SHAPING
// =============================================================================

#define REWARD_SCORE 1.0f
#define REWARD_OPPONENT_SCORE -1.0f
#define REWARD_BALL_TOWARD_OPPONENT 0.01f
#define REWARD_TIME_PENALTY -0.001f

// =============================================================================
// VISUALIZATION
// =============================================================================

#define WEIGHT_FLASH_THRESHOLD 0.1f
#define WEIGHT_FLASH_DURATION 10

#define PULSE_THRESHOLD 0.5f

// =============================================================================
// INPUT STATE INDICES
// =============================================================================

#define STATE_BALL_X 0
#define STATE_BALL_Y 1
#define STATE_BALL_VX 2
#define STATE_BALL_VY 3
#define STATE_PADDLE_Y 4
#define STATE_OPPONENT_Y 5
#define STATE_SIZE 6

// =============================================================================
// ACTION INDICES
// =============================================================================

#define ACTION_UP 0
#define ACTION_STAY 1
#define ACTION_DOWN 2
#define NUM_ACTIONS 3

// =============================================================================
// COLOR PALETTE (N64 RGB)
// =============================================================================

#define COLOR_VOID           0x000000FF
#define COLOR_TERMINAL_GREEN 0x00FF41FF
#define COLOR_AMBER_WARN     0xFFB000FF
#define COLOR_PHOSPHOR_DIM   0x005A19FF
#define COLOR_HOT_WHITE      0xFFFFFFFF
#define COLOR_NEGATIVE_RED   0xFF2D2DFF
#define COLOR_COOL_BLUE      0x2D91FFFF

#endif // CONFIG_H
