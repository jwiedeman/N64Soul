/**
 * PROJECT NEURON - Neural Network Header
 * Core neural network data structures and function declarations
 */

#ifndef NEURAL_NET_H
#define NEURAL_NET_H

#include <stdint.h>
#include "../include/config.h"

// =============================================================================
// DATA STRUCTURES
// =============================================================================

typedef struct {
    int num_layers;
    int layer_sizes[MAX_LAYERS];

    // Weights: weights[layer][to * prev_size + from]
    float* weights[MAX_LAYERS];
    float* biases[MAX_LAYERS];

    // Gradients for backpropagation
    float* weight_grads[MAX_LAYERS];
    float* bias_grads[MAX_LAYERS];

    // Adam optimizer state
    float* weight_m[MAX_LAYERS];  // First moment estimate
    float* weight_v[MAX_LAYERS];  // Second moment estimate
    float* bias_m[MAX_LAYERS];
    float* bias_v[MAX_LAYERS];
    int adam_t;                    // Timestep

    // Activations cache (for backprop and visualization)
    float* activations[MAX_LAYERS];
    float* pre_activations[MAX_LAYERS];  // Before activation function

    // Visualization state
    float* prev_weights[MAX_LAYERS];     // For smooth animation
    uint8_t* weight_flash[MAX_LAYERS];   // Flash timers for gradient viz

} NeuralNetwork;

// =============================================================================
// NETWORK LIFECYCLE
// =============================================================================

/**
 * Initialize a neural network with the given tier configuration
 * @param nn Pointer to network structure
 * @param tier Network complexity tier (TIER_MINIMAL to TIER_SUPERHEAVY)
 * @return 0 on success, -1 on allocation failure
 */
int nn_init(NeuralNetwork* nn, int tier);

/**
 * Free all memory allocated for the network
 * @param nn Pointer to network structure
 */
void nn_free(NeuralNetwork* nn);

/**
 * Reset network weights to Xavier/He initialization
 * @param nn Pointer to network structure
 */
void nn_reset_weights(NeuralNetwork* nn);

// =============================================================================
// FORWARD PASS
// =============================================================================

/**
 * Perform forward pass through the network
 * @param nn Pointer to network structure
 * @param input Input array (size = layer_sizes[0])
 * @param output Output array (size = layer_sizes[num_layers-1])
 */
void nn_forward(NeuralNetwork* nn, const float* input, float* output);

/**
 * Get the best action (argmax of Q-values)
 * @param nn Pointer to network structure
 * @param state Normalized state input
 * @return Action index (ACTION_UP, ACTION_STAY, or ACTION_DOWN)
 */
int nn_get_best_action(NeuralNetwork* nn, const float* state);

/**
 * Get Q-values for all actions
 * @param nn Pointer to network structure
 * @param state Normalized state input
 * @param q_values Output array for Q-values (size = NUM_ACTIONS)
 */
void nn_get_q_values(NeuralNetwork* nn, const float* state, float* q_values);

// =============================================================================
// BACKWARD PASS (TRAINING)
// =============================================================================

/**
 * Perform backpropagation for a single sample
 * @param nn Pointer to network structure
 * @param action Action that was taken
 * @param td_error Temporal difference error (target - predicted)
 */
void nn_backward(NeuralNetwork* nn, int action, float td_error);

/**
 * Update weights using accumulated gradients (SGD or Adam)
 * @param nn Pointer to network structure
 * @param learning_rate Learning rate
 * @param use_adam Whether to use Adam optimizer (vs vanilla SGD)
 */
void nn_update_weights(NeuralNetwork* nn, float learning_rate, int use_adam);

/**
 * Clear accumulated gradients
 * @param nn Pointer to network structure
 */
void nn_clear_gradients(NeuralNetwork* nn);

// =============================================================================
// VISUALIZATION HELPERS
// =============================================================================

/**
 * Update visualization state after weight changes
 * @param nn Pointer to network structure
 */
void nn_update_vis_state(NeuralNetwork* nn);

/**
 * Get weight value at specific position
 * @param nn Pointer to network structure
 * @param layer Layer index (1 to num_layers-1)
 * @param to Destination neuron index
 * @param from Source neuron index
 * @return Weight value
 */
float nn_get_weight(const NeuralNetwork* nn, int layer, int to, int from);

/**
 * Get activation value at specific position
 * @param nn Pointer to network structure
 * @param layer Layer index
 * @param neuron Neuron index
 * @return Activation value
 */
float nn_get_activation(const NeuralNetwork* nn, int layer, int neuron);

// =============================================================================
// SERIALIZATION
// =============================================================================

/**
 * Get total size needed to serialize network weights
 * @param nn Pointer to network structure
 * @return Size in bytes
 */
int nn_get_serialized_size(const NeuralNetwork* nn);

/**
 * Serialize network weights to buffer
 * @param nn Pointer to network structure
 * @param buffer Output buffer
 * @param buffer_size Size of output buffer
 * @return Bytes written, or -1 on error
 */
int nn_serialize(const NeuralNetwork* nn, uint8_t* buffer, int buffer_size);

/**
 * Deserialize network weights from buffer
 * @param nn Pointer to network structure
 * @param buffer Input buffer
 * @param buffer_size Size of input buffer
 * @return 0 on success, -1 on error
 */
int nn_deserialize(NeuralNetwork* nn, const uint8_t* buffer, int buffer_size);

#endif // NEURAL_NET_H
