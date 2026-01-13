/**
 * PROJECT NEURON - Save System Header
 * Controller Pak I/O and checkpoint management
 */

#ifndef SAVE_H
#define SAVE_H

#include <stdint.h>
#include "neural_net.h"
#include "training.h"

// =============================================================================
// SAVE FILE STRUCTURE
// =============================================================================

#define SAVE_MAGIC "NRNN"
#define SAVE_VERSION 0x0100  // 1.0

/**
 * Save file header (fits in first 256 bytes)
 */
typedef struct {
    char magic[4];           // "NRNN"
    uint16_t version;        // Save format version
    uint8_t network_tier;    // Network complexity tier
    uint8_t reserved1;

    uint32_t episode_count;  // Total episodes trained
    uint32_t total_steps;    // Total training steps
    uint32_t training_time;  // Seconds of training

    float best_win_rate;     // Best win rate achieved
    float current_epsilon;   // Current exploration rate
    float learning_rate;     // Current learning rate

    // Hyperparameters snapshot
    float gamma;
    float epsilon_min;
    float epsilon_decay;
    uint16_t batch_size;
    uint16_t reserved2;

    uint32_t checksum;       // CRC32 of weights

    uint8_t padding[256 - 52];  // Pad to 256 bytes

} SaveHeader;

// =============================================================================
// CONTROLLER PAK INTERFACE
// =============================================================================

/**
 * Check if Controller Pak is present
 * @param controller Controller port (0-3)
 * @return 1 if present, 0 otherwise
 */
int save_pak_present(int controller);

/**
 * Get available space on Controller Pak
 * @param controller Controller port
 * @return Available bytes, or -1 on error
 */
int save_pak_free_space(int controller);

/**
 * Check if a save file exists
 * @param controller Controller port
 * @return 1 if exists, 0 otherwise
 */
int save_exists(int controller);

// =============================================================================
// SAVE/LOAD OPERATIONS
// =============================================================================

/**
 * Save network state to Controller Pak
 * @param controller Controller port
 * @param nn Pointer to neural network
 * @param training Pointer to training state
 * @return 0 on success, negative on error
 */
int save_to_pak(int controller,
                const NeuralNetwork* nn,
                const TrainingState* training);

/**
 * Load network state from Controller Pak
 * @param controller Controller port
 * @param nn Pointer to neural network
 * @param training Pointer to training state
 * @return 0 on success, negative on error
 */
int load_from_pak(int controller,
                  NeuralNetwork* nn,
                  TrainingState* training);

/**
 * Delete save file from Controller Pak
 * @param controller Controller port
 * @return 0 on success, negative on error
 */
int save_delete(int controller);

// =============================================================================
// ROM CHECKPOINTS
// =============================================================================

/**
 * Number of built-in checkpoints in ROM
 */
#define NUM_ROM_CHECKPOINTS 4

/**
 * Checkpoint names
 */
extern const char* ROM_CHECKPOINT_NAMES[NUM_ROM_CHECKPOINTS];

/**
 * Load built-in checkpoint from ROM
 * @param checkpoint_id Checkpoint index (0-3)
 * @param nn Pointer to neural network
 * @return 0 on success, negative on error
 */
int load_rom_checkpoint(int checkpoint_id, NeuralNetwork* nn);

/**
 * Get info about ROM checkpoint
 * @param checkpoint_id Checkpoint index
 * @param win_rate Output: expected win rate
 * @param episodes Output: episodes trained
 * @return 0 on success, negative on error
 */
int get_rom_checkpoint_info(int checkpoint_id,
                            float* win_rate,
                            int* episodes);

// =============================================================================
// SAVE INFO DISPLAY
// =============================================================================

/**
 * Read save header without loading full state
 * @param controller Controller port
 * @param header Output: save header
 * @return 0 on success, negative on error
 */
int save_read_header(int controller, SaveHeader* header);

/**
 * Format training time for display
 * @param seconds Training time in seconds
 * @param buffer Output buffer (at least 16 chars)
 */
void save_format_time(uint32_t seconds, char* buffer);

// =============================================================================
// ERROR CODES
// =============================================================================

#define SAVE_OK              0
#define SAVE_ERR_NO_PAK     -1
#define SAVE_ERR_NO_SPACE   -2
#define SAVE_ERR_CORRUPT    -3
#define SAVE_ERR_VERSION    -4
#define SAVE_ERR_TIER       -5
#define SAVE_ERR_IO         -6
#define SAVE_ERR_NOT_FOUND  -7

#endif // SAVE_H
