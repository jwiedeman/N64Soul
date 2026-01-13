/**
 * PROJECT NEURON - Training Implementation
 * DQN algorithm, experience replay, hyperparameter management
 */

#include <stdlib.h>
#include <string.h>
#include <math.h>
#include "training.h"
#include "neural_net.h"

// =============================================================================
// RANDOM NUMBER GENERATION (for action selection)
// =============================================================================

static uint32_t train_rng_state = 54321;

static uint32_t train_xorshift32(void) {
    uint32_t x = train_rng_state;
    x ^= x << 13;
    x ^= x >> 17;
    x ^= x << 5;
    train_rng_state = x;
    return x;
}

static float train_randf(void) {
    return (float)(train_xorshift32() & 0x7FFFFFFF) / (float)0x7FFFFFFF;
}

static int train_randint(int max) {
    return (int)(train_xorshift32() % (uint32_t)max);
}

// =============================================================================
// REPLAY BUFFER
// =============================================================================

void replay_buffer_init(ReplayBuffer* buf) {
    buf->head = 0;
    buf->count = 0;
    memset(buf->buffer, 0, sizeof(buf->buffer));
}

void replay_buffer_add(ReplayBuffer* buf,
                       const float* state,
                       int action,
                       float reward,
                       const float* next_state,
                       uint8_t done) {
    Transition* t = &buf->buffer[buf->head];

    memcpy(t->state, state, STATE_SIZE * sizeof(float));
    t->action = action;
    t->reward = reward;
    memcpy(t->next_state, next_state, STATE_SIZE * sizeof(float));
    t->done = done;

    buf->head = (buf->head + 1) % REPLAY_BUFFER_SIZE;
    if (buf->count < REPLAY_BUFFER_SIZE) {
        buf->count++;
    }
}

const Transition* replay_buffer_sample(const ReplayBuffer* buf) {
    if (buf->count == 0) return NULL;
    int idx = train_randint(buf->count);
    return &buf->buffer[idx];
}

int replay_buffer_ready(const ReplayBuffer* buf, int batch_size) {
    return buf->count >= batch_size;
}

// =============================================================================
// TRAINING STATE INITIALIZATION
// =============================================================================

void training_init(TrainingState* ts) {
    training_init_custom(ts,
                         DEFAULT_LEARNING_RATE,
                         DEFAULT_GAMMA,
                         DEFAULT_EPSILON_START,
                         DEFAULT_EPSILON_MIN,
                         DEFAULT_EPSILON_DECAY,
                         DEFAULT_BATCH_SIZE);
}

void training_init_custom(TrainingState* ts,
                          float learning_rate,
                          float gamma,
                          float epsilon_start,
                          float epsilon_min,
                          float epsilon_decay,
                          int batch_size) {
    ts->learning_rate = learning_rate;
    ts->gamma = gamma;
    ts->epsilon = epsilon_start;
    ts->epsilon_min = epsilon_min;
    ts->epsilon_decay = epsilon_decay;
    ts->batch_size = batch_size;
    ts->target_update_freq = DEFAULT_TARGET_UPDATE_FREQ;
    ts->use_adam = 1;  // Use Adam by default

    training_reset_stats(ts);
}

void training_reset_stats(TrainingState* ts) {
    ts->total_episodes = 0;
    ts->total_steps = 0;
    ts->steps_since_target_update = 0;

    // Reset loss history
    memset(ts->loss_history, 0, sizeof(ts->loss_history));
    ts->loss_head = 0;
    ts->loss_count = 0;
    ts->loss_sum = 0.0f;
    ts->loss_smoothed = 0.0f;

    // Reset performance metrics
    ts->win_rate = 0.0f;
    ts->avg_rally_length = 0.0f;
    ts->avg_reward_per_episode = 0.0f;

    // Reset episode stats
    ts->current_episode_steps = 0;
    ts->current_episode_reward = 0.0f;
    ts->current_episode_rallies = 0;
}

// =============================================================================
// DQN TRAINING
// =============================================================================

float train_batch(NeuralNetwork* nn, ReplayBuffer* buf, TrainingState* ts) {
    if (!replay_buffer_ready(buf, ts->batch_size)) {
        return 0.0f;
    }

    float total_loss = 0.0f;

    // Clear gradients before batch
    nn_clear_gradients(nn);

    // Process each sample in batch
    for (int b = 0; b < ts->batch_size; b++) {
        const Transition* t = replay_buffer_sample(buf);
        if (!t) continue;

        // Forward pass to get current Q-values
        float q_values[NUM_ACTIONS];
        nn_forward(nn, t->state, q_values);

        // Compute target Q-value
        float target;
        if (t->done) {
            // Terminal state: Q = reward
            target = t->reward;
        } else {
            // Non-terminal: Q = reward + gamma * max(Q(s'))
            float next_q[NUM_ACTIONS];
            nn_forward(nn, t->next_state, next_q);

            // Find max Q-value for next state
            float max_next_q = next_q[0];
            for (int i = 1; i < NUM_ACTIONS; i++) {
                if (next_q[i] > max_next_q) {
                    max_next_q = next_q[i];
                }
            }

            target = t->reward + ts->gamma * max_next_q;
        }

        // TD error: how wrong was our prediction?
        float td_error = target - q_values[t->action];
        total_loss += td_error * td_error;

        // Re-run forward pass to set up activations for backprop
        // (since we overwrote them getting next_q)
        nn_forward(nn, t->state, NULL);

        // Backpropagate
        nn_backward(nn, t->action, td_error);
    }

    // Average gradients over batch
    for (int l = 1; l < nn->num_layers; l++) {
        int prev_size = nn->layer_sizes[l - 1];
        int curr_size = nn->layer_sizes[l];
        int weight_count = curr_size * prev_size;

        float inv_batch = 1.0f / (float)ts->batch_size;

        for (int i = 0; i < weight_count; i++) {
            nn->weight_grads[l][i] *= inv_batch;
        }
        for (int i = 0; i < curr_size; i++) {
            nn->bias_grads[l][i] *= inv_batch;
        }
    }

    // Update weights
    nn_update_weights(nn, ts->learning_rate, ts->use_adam);

    // Return average loss
    return total_loss / (float)ts->batch_size;
}

void training_decay_epsilon(TrainingState* ts) {
    if (ts->epsilon > ts->epsilon_min) {
        ts->epsilon *= ts->epsilon_decay;
        if (ts->epsilon < ts->epsilon_min) {
            ts->epsilon = ts->epsilon_min;
        }
    }
}

// =============================================================================
// EPISODE TRACKING
// =============================================================================

void training_record_step(TrainingState* ts, float reward) {
    ts->current_episode_steps++;
    ts->current_episode_reward += reward;
}

void training_end_episode(TrainingState* ts, int ai_score, int opponent_score) {
    ts->total_episodes++;

    // Update win rate (exponential moving average)
    float won = (ai_score > opponent_score) ? 1.0f : 0.0f;
    if (ts->total_episodes == 1) {
        ts->win_rate = won;
    } else {
        ts->win_rate = ts->win_rate * 0.99f + won * 0.01f;
    }

    // Update average reward per episode
    if (ts->total_episodes == 1) {
        ts->avg_reward_per_episode = ts->current_episode_reward;
    } else {
        ts->avg_reward_per_episode = ts->avg_reward_per_episode * 0.99f +
                                     ts->current_episode_reward * 0.01f;
    }

    // Update average rally length
    float rally_estimate = (float)ts->current_episode_steps /
                           (float)(ai_score + opponent_score + 1);
    if (ts->total_episodes == 1) {
        ts->avg_rally_length = rally_estimate;
    } else {
        ts->avg_rally_length = ts->avg_rally_length * 0.99f +
                               rally_estimate * 0.01f;
    }

    // Reset episode counters
    ts->current_episode_steps = 0;
    ts->current_episode_reward = 0.0f;
    ts->current_episode_rallies = 0;
}

// =============================================================================
// LOSS HISTORY
// =============================================================================

void loss_history_add(TrainingState* ts, float loss) {
    // Remove old value from sum if buffer is full
    if (ts->loss_count == LOSS_HISTORY_SIZE) {
        ts->loss_sum -= ts->loss_history[ts->loss_head];
    }

    // Add new value
    ts->loss_history[ts->loss_head] = loss;
    ts->loss_sum += loss;

    // Update head pointer
    ts->loss_head = (ts->loss_head + 1) % LOSS_HISTORY_SIZE;
    if (ts->loss_count < LOSS_HISTORY_SIZE) {
        ts->loss_count++;
    }

    // Update smoothed loss (EMA)
    if (ts->loss_count == 1) {
        ts->loss_smoothed = loss;
    } else {
        ts->loss_smoothed = ts->loss_smoothed * 0.99f + loss * 0.01f;
    }
}

float loss_history_get(const TrainingState* ts, int index) {
    if (index < 0 || index >= ts->loss_count) {
        return 0.0f;
    }

    // Convert index to ring buffer position
    // Index 0 = oldest entry
    int pos = (ts->loss_head - ts->loss_count + index + LOSS_HISTORY_SIZE)
              % LOSS_HISTORY_SIZE;
    return ts->loss_history[pos];
}

float loss_get_smoothed(const TrainingState* ts) {
    return ts->loss_smoothed;
}

void loss_get_range(const TrainingState* ts, float* min_out, float* max_out) {
    if (ts->loss_count == 0) {
        *min_out = 0.0f;
        *max_out = 1.0f;
        return;
    }

    float min_val = ts->loss_history[0];
    float max_val = ts->loss_history[0];

    for (int i = 0; i < ts->loss_count; i++) {
        int pos = (ts->loss_head - ts->loss_count + i + LOSS_HISTORY_SIZE)
                  % LOSS_HISTORY_SIZE;
        float val = ts->loss_history[pos];

        if (val < min_val) min_val = val;
        if (val > max_val) max_val = val;
    }

    *min_out = min_val;
    *max_out = max_val;

    // Ensure minimum range
    if (*max_out - *min_out < 0.1f) {
        *max_out = *min_out + 0.1f;
    }
}

// =============================================================================
// ACTION SELECTION
// =============================================================================

int select_action_epsilon_greedy(NeuralNetwork* nn,
                                  const float* state,
                                  float epsilon) {
    // With probability epsilon, choose random action
    if (train_randf() < epsilon) {
        return train_randint(NUM_ACTIONS);
    }

    // Otherwise, choose best action
    return nn_get_best_action(nn, state);
}

// =============================================================================
// HYPERPARAMETER PRESETS
// =============================================================================

void training_preset_fast(TrainingState* ts) {
    ts->learning_rate = 0.003f;
    ts->gamma = 0.95f;
    ts->epsilon = 1.0f;
    ts->epsilon_min = 0.1f;
    ts->epsilon_decay = 0.999f;
    ts->batch_size = 16;
}

void training_preset_balanced(TrainingState* ts) {
    ts->learning_rate = DEFAULT_LEARNING_RATE;
    ts->gamma = DEFAULT_GAMMA;
    ts->epsilon = DEFAULT_EPSILON_START;
    ts->epsilon_min = DEFAULT_EPSILON_MIN;
    ts->epsilon_decay = DEFAULT_EPSILON_DECAY;
    ts->batch_size = DEFAULT_BATCH_SIZE;
}

void training_preset_careful(TrainingState* ts) {
    ts->learning_rate = 0.0003f;
    ts->gamma = 0.99f;
    ts->epsilon = 1.0f;
    ts->epsilon_min = 0.02f;
    ts->epsilon_decay = 0.99995f;
    ts->batch_size = 64;
}
