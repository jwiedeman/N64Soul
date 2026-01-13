/**
 * PROJECT NEURON - Training Header
 * DQN training algorithm and experience replay
 */

#ifndef TRAINING_H
#define TRAINING_H

#include <stdint.h>
#include "../include/config.h"
#include "neural_net.h"

// =============================================================================
// DATA STRUCTURES
// =============================================================================

/**
 * Single experience transition for replay buffer
 */
typedef struct {
    float state[STATE_SIZE];
    int action;
    float reward;
    float next_state[STATE_SIZE];
    uint8_t done;  // Episode ended after this transition
} Transition;

/**
 * Ring buffer for experience replay
 */
typedef struct {
    Transition buffer[REPLAY_BUFFER_SIZE];
    int head;
    int count;
} ReplayBuffer;

/**
 * Training state and hyperparameters
 */
typedef struct {
    // Hyperparameters
    float learning_rate;
    float gamma;           // Discount factor
    float epsilon;         // Current exploration rate
    float epsilon_min;
    float epsilon_decay;
    int batch_size;
    int target_update_freq;
    int use_adam;          // Use Adam optimizer vs SGD

    // Statistics
    int total_episodes;
    int total_steps;
    int steps_since_target_update;

    // Loss tracking
    float loss_history[LOSS_HISTORY_SIZE];
    int loss_head;
    int loss_count;
    float loss_sum;        // Running sum for average
    float loss_smoothed;   // EMA smoothed loss

    // Performance metrics
    float win_rate;
    float avg_rally_length;
    float avg_reward_per_episode;

    // Current episode stats
    int current_episode_steps;
    float current_episode_reward;
    int current_episode_rallies;

} TrainingState;

// =============================================================================
// REPLAY BUFFER
// =============================================================================

/**
 * Initialize replay buffer
 * @param buf Pointer to replay buffer
 */
void replay_buffer_init(ReplayBuffer* buf);

/**
 * Add transition to replay buffer
 * @param buf Pointer to replay buffer
 * @param state Current state
 * @param action Action taken
 * @param reward Reward received
 * @param next_state Resulting state
 * @param done Whether episode ended
 */
void replay_buffer_add(ReplayBuffer* buf,
                       const float* state,
                       int action,
                       float reward,
                       const float* next_state,
                       uint8_t done);

/**
 * Sample random transition from buffer
 * @param buf Pointer to replay buffer
 * @return Pointer to sampled transition
 */
const Transition* replay_buffer_sample(const ReplayBuffer* buf);

/**
 * Check if buffer has enough samples for training
 * @param buf Pointer to replay buffer
 * @param batch_size Minimum batch size needed
 * @return 1 if enough samples, 0 otherwise
 */
int replay_buffer_ready(const ReplayBuffer* buf, int batch_size);

// =============================================================================
// TRAINING STATE
// =============================================================================

/**
 * Initialize training state with default hyperparameters
 * @param ts Pointer to training state
 */
void training_init(TrainingState* ts);

/**
 * Initialize training state with specific hyperparameters
 * @param ts Pointer to training state
 * @param learning_rate Learning rate
 * @param gamma Discount factor
 * @param epsilon_start Initial exploration rate
 * @param epsilon_min Minimum exploration rate
 * @param epsilon_decay Exploration decay rate
 * @param batch_size Training batch size
 */
void training_init_custom(TrainingState* ts,
                          float learning_rate,
                          float gamma,
                          float epsilon_start,
                          float epsilon_min,
                          float epsilon_decay,
                          int batch_size);

/**
 * Reset training statistics (but keep hyperparameters)
 * @param ts Pointer to training state
 */
void training_reset_stats(TrainingState* ts);

// =============================================================================
// TRAINING LOOP
// =============================================================================

/**
 * Train network on a batch of experiences
 * @param nn Pointer to neural network
 * @param buf Pointer to replay buffer
 * @param ts Pointer to training state
 * @return Average loss for this batch
 */
float train_batch(NeuralNetwork* nn, ReplayBuffer* buf, TrainingState* ts);

/**
 * Decay epsilon after each step
 * @param ts Pointer to training state
 */
void training_decay_epsilon(TrainingState* ts);

/**
 * Record end of episode and update statistics
 * @param ts Pointer to training state
 * @param ai_score AI's final score
 * @param opponent_score Opponent's final score
 */
void training_end_episode(TrainingState* ts, int ai_score, int opponent_score);

/**
 * Record a single step's reward
 * @param ts Pointer to training state
 * @param reward Reward received
 */
void training_record_step(TrainingState* ts, float reward);

// =============================================================================
// LOSS TRACKING
// =============================================================================

/**
 * Add loss value to history
 * @param ts Pointer to training state
 * @param loss Loss value to record
 */
void loss_history_add(TrainingState* ts, float loss);

/**
 * Get loss value at index
 * @param ts Pointer to training state
 * @param index Index (0 = oldest in buffer)
 * @return Loss value
 */
float loss_history_get(const TrainingState* ts, int index);

/**
 * Get current smoothed loss
 * @param ts Pointer to training state
 * @return EMA smoothed loss
 */
float loss_get_smoothed(const TrainingState* ts);

/**
 * Get min/max loss in history for auto-scaling
 * @param ts Pointer to training state
 * @param min_out Pointer to store minimum
 * @param max_out Pointer to store maximum
 */
void loss_get_range(const TrainingState* ts, float* min_out, float* max_out);

// =============================================================================
// ACTION SELECTION
// =============================================================================

/**
 * Select action using epsilon-greedy policy
 * @param nn Pointer to neural network
 * @param state Normalized state input
 * @param epsilon Current exploration rate
 * @return Selected action
 */
int select_action_epsilon_greedy(NeuralNetwork* nn,
                                  const float* state,
                                  float epsilon);

// =============================================================================
// HYPERPARAMETER PRESETS
// =============================================================================

/**
 * Apply "Fast Learner" preset
 * High learning rate, fast epsilon decay
 */
void training_preset_fast(TrainingState* ts);

/**
 * Apply "Balanced" preset (default settings)
 */
void training_preset_balanced(TrainingState* ts);

/**
 * Apply "Careful" preset
 * Low learning rate, slow epsilon decay
 */
void training_preset_careful(TrainingState* ts);

#endif // TRAINING_H
