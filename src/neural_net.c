/**
 * PROJECT NEURON - Neural Network Implementation
 * Forward pass, backpropagation, weight updates
 */

#include <stdlib.h>
#include <string.h>
#include <math.h>
#include "neural_net.h"

// =============================================================================
// TIER CONFIGURATIONS
// =============================================================================

// Layer sizes for each tier
static const int TIER_CONFIGS[][MAX_LAYERS + 1] = {
    // TIER_MINIMAL: 6 -> 16 -> 3
    {3, 6, 16, 3, 0, 0, 0},
    // TIER_LIGHT: 6 -> 32 -> 32 -> 3
    {4, 6, 32, 32, 3, 0, 0},
    // TIER_MEDIUM: 6 -> 64 -> 64 -> 32 -> 3
    {5, 6, 64, 64, 32, 3, 0},
    // TIER_HEAVY: 6 -> 128 -> 128 -> 64 -> 32 -> 3
    {6, 6, 128, 128, 64, 32, 3},
    // TIER_SUPERHEAVY: 6 -> 256 -> 256 -> 128 -> 64 -> 3
    {6, 6, 256, 256, 128, 64, 3},
};

// =============================================================================
// HELPER FUNCTIONS
// =============================================================================

/**
 * Simple pseudo-random number generator (xorshift32)
 * Used for weight initialization - doesn't need crypto quality
 */
static uint32_t rng_state = 12345;

static uint32_t xorshift32(void) {
    uint32_t x = rng_state;
    x ^= x << 13;
    x ^= x >> 17;
    x ^= x << 5;
    rng_state = x;
    return x;
}

/**
 * Random float in range [0, 1)
 */
static float randf(void) {
    return (float)(xorshift32() & 0x7FFFFFFF) / (float)0x7FFFFFFF;
}

/**
 * Random float from standard normal distribution (Box-Muller)
 */
static float randn(void) {
    float u1 = randf();
    float u2 = randf();
    if (u1 < 1e-10f) u1 = 1e-10f;  // Avoid log(0)
    return sqrtf(-2.0f * logf(u1)) * cosf(6.28318530718f * u2);
}

/**
 * ReLU activation function
 */
static inline float relu(float x) {
    return x > 0.0f ? x : 0.0f;
}

/**
 * ReLU derivative
 */
static inline float relu_derivative(float x) {
    return x > 0.0f ? 1.0f : 0.0f;
}

// =============================================================================
// MEMORY ALLOCATION
// =============================================================================

/**
 * Allocate array of floats, zeroed
 */
static float* alloc_floats(int count) {
    float* ptr = (float*)malloc(count * sizeof(float));
    if (ptr) {
        memset(ptr, 0, count * sizeof(float));
    }
    return ptr;
}

/**
 * Allocate array of uint8_t, zeroed
 */
static uint8_t* alloc_bytes(int count) {
    uint8_t* ptr = (uint8_t*)malloc(count);
    if (ptr) {
        memset(ptr, 0, count);
    }
    return ptr;
}

// =============================================================================
// NETWORK LIFECYCLE
// =============================================================================

int nn_init(NeuralNetwork* nn, int tier) {
    if (tier < 0 || tier > TIER_SUPERHEAVY) {
        return -1;
    }

    // Get configuration for this tier
    const int* config = TIER_CONFIGS[tier];
    nn->num_layers = config[0];

    for (int i = 0; i < nn->num_layers; i++) {
        nn->layer_sizes[i] = config[i + 1];
    }

    // Initialize all pointers to NULL
    for (int i = 0; i < MAX_LAYERS; i++) {
        nn->weights[i] = NULL;
        nn->biases[i] = NULL;
        nn->weight_grads[i] = NULL;
        nn->bias_grads[i] = NULL;
        nn->weight_m[i] = NULL;
        nn->weight_v[i] = NULL;
        nn->bias_m[i] = NULL;
        nn->bias_v[i] = NULL;
        nn->activations[i] = NULL;
        nn->pre_activations[i] = NULL;
        nn->prev_weights[i] = NULL;
        nn->weight_flash[i] = NULL;
    }
    nn->adam_t = 0;

    // Allocate memory for each layer
    for (int l = 0; l < nn->num_layers; l++) {
        int size = nn->layer_sizes[l];

        // Activations for all layers
        nn->activations[l] = alloc_floats(size);
        nn->pre_activations[l] = alloc_floats(size);
        if (!nn->activations[l] || !nn->pre_activations[l]) {
            nn_free(nn);
            return -1;
        }

        // Weights and biases for layers 1+
        if (l > 0) {
            int prev_size = nn->layer_sizes[l - 1];
            int weight_count = size * prev_size;

            nn->weights[l] = alloc_floats(weight_count);
            nn->biases[l] = alloc_floats(size);
            nn->weight_grads[l] = alloc_floats(weight_count);
            nn->bias_grads[l] = alloc_floats(size);

            // Adam optimizer state
            nn->weight_m[l] = alloc_floats(weight_count);
            nn->weight_v[l] = alloc_floats(weight_count);
            nn->bias_m[l] = alloc_floats(size);
            nn->bias_v[l] = alloc_floats(size);

            // Visualization state
            nn->prev_weights[l] = alloc_floats(weight_count);
            nn->weight_flash[l] = alloc_bytes(weight_count);

            if (!nn->weights[l] || !nn->biases[l] ||
                !nn->weight_grads[l] || !nn->bias_grads[l] ||
                !nn->weight_m[l] || !nn->weight_v[l] ||
                !nn->bias_m[l] || !nn->bias_v[l] ||
                !nn->prev_weights[l] || !nn->weight_flash[l]) {
                nn_free(nn);
                return -1;
            }
        }
    }

    // Initialize weights
    nn_reset_weights(nn);

    return 0;
}

void nn_free(NeuralNetwork* nn) {
    for (int l = 0; l < MAX_LAYERS; l++) {
        if (nn->weights[l]) free(nn->weights[l]);
        if (nn->biases[l]) free(nn->biases[l]);
        if (nn->weight_grads[l]) free(nn->weight_grads[l]);
        if (nn->bias_grads[l]) free(nn->bias_grads[l]);
        if (nn->weight_m[l]) free(nn->weight_m[l]);
        if (nn->weight_v[l]) free(nn->weight_v[l]);
        if (nn->bias_m[l]) free(nn->bias_m[l]);
        if (nn->bias_v[l]) free(nn->bias_v[l]);
        if (nn->activations[l]) free(nn->activations[l]);
        if (nn->pre_activations[l]) free(nn->pre_activations[l]);
        if (nn->prev_weights[l]) free(nn->prev_weights[l]);
        if (nn->weight_flash[l]) free(nn->weight_flash[l]);

        nn->weights[l] = NULL;
        nn->biases[l] = NULL;
        nn->weight_grads[l] = NULL;
        nn->bias_grads[l] = NULL;
        nn->weight_m[l] = NULL;
        nn->weight_v[l] = NULL;
        nn->bias_m[l] = NULL;
        nn->bias_v[l] = NULL;
        nn->activations[l] = NULL;
        nn->pre_activations[l] = NULL;
        nn->prev_weights[l] = NULL;
        nn->weight_flash[l] = NULL;
    }
    nn->num_layers = 0;
}

void nn_reset_weights(NeuralNetwork* nn) {
    // Seed RNG
    rng_state = 12345;

    for (int l = 1; l < nn->num_layers; l++) {
        int prev_size = nn->layer_sizes[l - 1];
        int curr_size = nn->layer_sizes[l];
        int weight_count = curr_size * prev_size;

        // He initialization: weights ~ N(0, sqrt(2/fan_in))
        float std = sqrtf(2.0f / (float)prev_size);

        for (int i = 0; i < weight_count; i++) {
            nn->weights[l][i] = randn() * std;
            nn->prev_weights[l][i] = nn->weights[l][i];
        }

        // Biases start at zero
        for (int i = 0; i < curr_size; i++) {
            nn->biases[l][i] = 0.0f;
        }

        // Clear gradients and optimizer state
        memset(nn->weight_grads[l], 0, weight_count * sizeof(float));
        memset(nn->bias_grads[l], 0, curr_size * sizeof(float));
        memset(nn->weight_m[l], 0, weight_count * sizeof(float));
        memset(nn->weight_v[l], 0, weight_count * sizeof(float));
        memset(nn->bias_m[l], 0, curr_size * sizeof(float));
        memset(nn->bias_v[l], 0, curr_size * sizeof(float));
        memset(nn->weight_flash[l], 0, weight_count);
    }

    nn->adam_t = 0;
}

// =============================================================================
// FORWARD PASS
// =============================================================================

void nn_forward(NeuralNetwork* nn, const float* input, float* output) {
    // Copy input to first layer activations
    memcpy(nn->activations[0], input, nn->layer_sizes[0] * sizeof(float));

    // Forward through each layer
    for (int l = 1; l < nn->num_layers; l++) {
        int prev_size = nn->layer_sizes[l - 1];
        int curr_size = nn->layer_sizes[l];

        for (int j = 0; j < curr_size; j++) {
            float sum = nn->biases[l][j];

            // Dot product with previous layer
            for (int i = 0; i < prev_size; i++) {
                sum += nn->weights[l][j * prev_size + i] * nn->activations[l - 1][i];
            }

            nn->pre_activations[l][j] = sum;

            // ReLU for hidden layers, linear for output
            if (l < nn->num_layers - 1) {
                nn->activations[l][j] = relu(sum);
            } else {
                nn->activations[l][j] = sum;  // Linear output (Q-values)
            }
        }
    }

    // Copy output
    if (output) {
        memcpy(output, nn->activations[nn->num_layers - 1],
               nn->layer_sizes[nn->num_layers - 1] * sizeof(float));
    }
}

int nn_get_best_action(NeuralNetwork* nn, const float* state) {
    float q_values[NUM_ACTIONS];
    nn_forward(nn, state, q_values);

    // Argmax
    int best = 0;
    for (int i = 1; i < NUM_ACTIONS; i++) {
        if (q_values[i] > q_values[best]) {
            best = i;
        }
    }
    return best;
}

void nn_get_q_values(NeuralNetwork* nn, const float* state, float* q_values) {
    nn_forward(nn, state, q_values);
}

// =============================================================================
// BACKWARD PASS
// =============================================================================

void nn_backward(NeuralNetwork* nn, int action, float td_error) {
    int output_layer = nn->num_layers - 1;
    int output_size = nn->layer_sizes[output_layer];

    // Allocate delta arrays on stack (small enough for N64)
    float deltas[MAX_LAYERS][MAX_NEURONS_PER_LAYER];

    // Output layer delta: only for the action taken
    // dL/dQ = -(target - Q) = -td_error for the taken action, 0 for others
    for (int i = 0; i < output_size; i++) {
        deltas[output_layer][i] = (i == action) ? -td_error : 0.0f;
    }

    // Backpropagate through hidden layers
    for (int l = output_layer; l >= 1; l--) {
        int curr_size = nn->layer_sizes[l];
        int prev_size = nn->layer_sizes[l - 1];

        // Accumulate gradients for this layer
        for (int j = 0; j < curr_size; j++) {
            // Apply ReLU derivative for hidden layers
            float delta = deltas[l][j];
            if (l < output_layer) {
                delta *= relu_derivative(nn->pre_activations[l][j]);
            }

            // Bias gradient
            nn->bias_grads[l][j] += delta;

            // Weight gradients
            for (int i = 0; i < prev_size; i++) {
                nn->weight_grads[l][j * prev_size + i] += delta * nn->activations[l - 1][i];
            }
        }

        // Propagate delta to previous layer (if not input layer)
        if (l > 1) {
            int next_prev_size = nn->layer_sizes[l - 1];
            for (int i = 0; i < next_prev_size; i++) {
                deltas[l - 1][i] = 0.0f;
                for (int j = 0; j < curr_size; j++) {
                    float delta = deltas[l][j];
                    if (l < output_layer) {
                        delta *= relu_derivative(nn->pre_activations[l][j]);
                    }
                    deltas[l - 1][i] += nn->weights[l][j * prev_size + i] * delta;
                }
            }
        }
    }
}

void nn_update_weights(NeuralNetwork* nn, float learning_rate, int use_adam) {
    const float beta1 = 0.9f;
    const float beta2 = 0.999f;
    const float epsilon = 1e-8f;

    if (use_adam) {
        nn->adam_t++;
    }

    for (int l = 1; l < nn->num_layers; l++) {
        int prev_size = nn->layer_sizes[l - 1];
        int curr_size = nn->layer_sizes[l];
        int weight_count = curr_size * prev_size;

        if (use_adam) {
            // Bias correction factors
            float bc1 = 1.0f - powf(beta1, (float)nn->adam_t);
            float bc2 = 1.0f - powf(beta2, (float)nn->adam_t);

            // Update weights with Adam
            for (int i = 0; i < weight_count; i++) {
                float g = nn->weight_grads[l][i];

                // Update moments
                nn->weight_m[l][i] = beta1 * nn->weight_m[l][i] + (1.0f - beta1) * g;
                nn->weight_v[l][i] = beta2 * nn->weight_v[l][i] + (1.0f - beta2) * g * g;

                // Bias-corrected moments
                float m_hat = nn->weight_m[l][i] / bc1;
                float v_hat = nn->weight_v[l][i] / bc2;

                // Update weight
                float update = learning_rate * m_hat / (sqrtf(v_hat) + epsilon);
                nn->weights[l][i] -= update;

                // Track significant updates for visualization
                if (fabsf(update) > WEIGHT_FLASH_THRESHOLD) {
                    nn->weight_flash[l][i] = WEIGHT_FLASH_DURATION;
                }
            }

            // Update biases with Adam
            for (int i = 0; i < curr_size; i++) {
                float g = nn->bias_grads[l][i];

                nn->bias_m[l][i] = beta1 * nn->bias_m[l][i] + (1.0f - beta1) * g;
                nn->bias_v[l][i] = beta2 * nn->bias_v[l][i] + (1.0f - beta2) * g * g;

                float m_hat = nn->bias_m[l][i] / bc1;
                float v_hat = nn->bias_v[l][i] / bc2;

                nn->biases[l][i] -= learning_rate * m_hat / (sqrtf(v_hat) + epsilon);
            }
        } else {
            // Vanilla SGD
            for (int i = 0; i < weight_count; i++) {
                float update = learning_rate * nn->weight_grads[l][i];
                nn->weights[l][i] -= update;

                if (fabsf(update) > WEIGHT_FLASH_THRESHOLD) {
                    nn->weight_flash[l][i] = WEIGHT_FLASH_DURATION;
                }
            }

            for (int i = 0; i < curr_size; i++) {
                nn->biases[l][i] -= learning_rate * nn->bias_grads[l][i];
            }
        }
    }
}

void nn_clear_gradients(NeuralNetwork* nn) {
    for (int l = 1; l < nn->num_layers; l++) {
        int prev_size = nn->layer_sizes[l - 1];
        int curr_size = nn->layer_sizes[l];
        int weight_count = curr_size * prev_size;

        memset(nn->weight_grads[l], 0, weight_count * sizeof(float));
        memset(nn->bias_grads[l], 0, curr_size * sizeof(float));
    }
}

// =============================================================================
// VISUALIZATION HELPERS
// =============================================================================

void nn_update_vis_state(NeuralNetwork* nn) {
    for (int l = 1; l < nn->num_layers; l++) {
        int prev_size = nn->layer_sizes[l - 1];
        int curr_size = nn->layer_sizes[l];
        int weight_count = curr_size * prev_size;

        // Smoothly interpolate previous weights toward current
        for (int i = 0; i < weight_count; i++) {
            nn->prev_weights[l][i] = nn->prev_weights[l][i] * 0.9f +
                                     nn->weights[l][i] * 0.1f;

            // Decay flash timers
            if (nn->weight_flash[l][i] > 0) {
                nn->weight_flash[l][i]--;
            }
        }
    }
}

float nn_get_weight(const NeuralNetwork* nn, int layer, int to, int from) {
    if (layer < 1 || layer >= nn->num_layers) return 0.0f;
    int prev_size = nn->layer_sizes[layer - 1];
    return nn->weights[layer][to * prev_size + from];
}

float nn_get_activation(const NeuralNetwork* nn, int layer, int neuron) {
    if (layer < 0 || layer >= nn->num_layers) return 0.0f;
    if (neuron < 0 || neuron >= nn->layer_sizes[layer]) return 0.0f;
    return nn->activations[layer][neuron];
}

// =============================================================================
// SERIALIZATION
// =============================================================================

int nn_get_serialized_size(const NeuralNetwork* nn) {
    int size = 0;

    // Header: num_layers + layer_sizes
    size += sizeof(int) * (1 + nn->num_layers);

    // Weights and biases
    for (int l = 1; l < nn->num_layers; l++) {
        int prev_size = nn->layer_sizes[l - 1];
        int curr_size = nn->layer_sizes[l];
        size += (curr_size * prev_size + curr_size) * sizeof(float);
    }

    return size;
}

int nn_serialize(const NeuralNetwork* nn, uint8_t* buffer, int buffer_size) {
    int required = nn_get_serialized_size(nn);
    if (buffer_size < required) return -1;

    uint8_t* ptr = buffer;

    // Write num_layers
    memcpy(ptr, &nn->num_layers, sizeof(int));
    ptr += sizeof(int);

    // Write layer sizes
    for (int l = 0; l < nn->num_layers; l++) {
        memcpy(ptr, &nn->layer_sizes[l], sizeof(int));
        ptr += sizeof(int);
    }

    // Write weights and biases
    for (int l = 1; l < nn->num_layers; l++) {
        int prev_size = nn->layer_sizes[l - 1];
        int curr_size = nn->layer_sizes[l];
        int weight_count = curr_size * prev_size;

        memcpy(ptr, nn->weights[l], weight_count * sizeof(float));
        ptr += weight_count * sizeof(float);

        memcpy(ptr, nn->biases[l], curr_size * sizeof(float));
        ptr += curr_size * sizeof(float);
    }

    return (int)(ptr - buffer);
}

int nn_deserialize(NeuralNetwork* nn, const uint8_t* buffer, int buffer_size) {
    const uint8_t* ptr = buffer;

    // Read num_layers
    int num_layers;
    memcpy(&num_layers, ptr, sizeof(int));
    ptr += sizeof(int);

    if (num_layers != nn->num_layers) {
        return -1;  // Architecture mismatch
    }

    // Read and verify layer sizes
    for (int l = 0; l < num_layers; l++) {
        int size;
        memcpy(&size, ptr, sizeof(int));
        ptr += sizeof(int);

        if (size != nn->layer_sizes[l]) {
            return -1;  // Architecture mismatch
        }
    }

    // Read weights and biases
    for (int l = 1; l < nn->num_layers; l++) {
        int prev_size = nn->layer_sizes[l - 1];
        int curr_size = nn->layer_sizes[l];
        int weight_count = curr_size * prev_size;

        memcpy(nn->weights[l], ptr, weight_count * sizeof(float));
        ptr += weight_count * sizeof(float);

        memcpy(nn->biases[l], ptr, curr_size * sizeof(float));
        ptr += curr_size * sizeof(float);

        // Copy to prev_weights for visualization
        memcpy(nn->prev_weights[l], nn->weights[l], weight_count * sizeof(float));
    }

    return 0;
}
