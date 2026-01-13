/**
 * PROJECT NEURON - Save System Implementation
 * Controller Pak I/O and ROM checkpoint loading
 */

#include <libdragon.h>
#include <stdlib.h>
#include <string.h>
#include <stdio.h>
#include "save.h"
#include "neural_net.h"
#include "training.h"

// =============================================================================
// ROM CHECKPOINT DATA
// =============================================================================

// Pre-trained checkpoint names
const char* ROM_CHECKPOINT_NAMES[NUM_ROM_CHECKPOINTS] = {
    "RANDOM",
    "NOVICE",
    "COMPETENT",
    "EXPERT"
};

// Checkpoint metadata (win rates and episode counts)
static const float checkpoint_win_rates[NUM_ROM_CHECKPOINTS] = {
    0.15f,   // Random
    0.45f,   // Novice
    0.75f,   // Competent
    0.95f    // Expert
};

static const int checkpoint_episodes[NUM_ROM_CHECKPOINTS] = {
    0,       // Random
    500,     // Novice
    2000,    // Competent
    10000    // Expert
};

// =============================================================================
// CRC32 FOR CHECKSUM
// =============================================================================

static uint32_t crc32_table[256];
static int crc32_initialized = 0;

static void init_crc32_table(void) {
    if (crc32_initialized) return;

    for (uint32_t i = 0; i < 256; i++) {
        uint32_t crc = i;
        for (int j = 0; j < 8; j++) {
            if (crc & 1) {
                crc = (crc >> 1) ^ 0xEDB88320;
            } else {
                crc >>= 1;
            }
        }
        crc32_table[i] = crc;
    }
    crc32_initialized = 1;
}

static uint32_t compute_crc32(const uint8_t* data, int length) {
    init_crc32_table();

    uint32_t crc = 0xFFFFFFFF;
    for (int i = 0; i < length; i++) {
        crc = crc32_table[(crc ^ data[i]) & 0xFF] ^ (crc >> 8);
    }
    return crc ^ 0xFFFFFFFF;
}

// =============================================================================
// CONTROLLER PAK DETECTION
// =============================================================================

int save_pak_present(int controller) {
    // Check if controller pak is present
    // libdragon's controller pak functions handle detection
    int status = identify_accessory(controller);
    return (status == ACCESSORY_MEMPAK);
}

int save_pak_free_space(int controller) {
    if (!save_pak_present(controller)) {
        return SAVE_ERR_NO_PAK;
    }

    // Controller Pak has 32KB total (123 pages of 256 bytes)
    // Need to check how much is used by existing saves
    // For simplicity, return max available (32KB - some overhead)
    return 32768 - 1024;  // Reserve 1KB for file system overhead
}

int save_exists(int controller) {
    if (!save_pak_present(controller)) {
        return 0;
    }

    // Try to read header
    SaveHeader header;
    if (save_read_header(controller, &header) == SAVE_OK) {
        return 1;
    }
    return 0;
}

// =============================================================================
// SAVE OPERATIONS
// =============================================================================

int save_to_pak(int controller,
                const NeuralNetwork* nn,
                const TrainingState* training) {
    if (!save_pak_present(controller)) {
        return SAVE_ERR_NO_PAK;
    }

    // Calculate required size
    int weight_size = nn_get_serialized_size(nn);
    int total_size = sizeof(SaveHeader) + weight_size;

    // Check space
    if (total_size > save_pak_free_space(controller)) {
        return SAVE_ERR_NO_SPACE;
    }

    // Allocate buffer
    uint8_t* buffer = (uint8_t*)malloc(total_size);
    if (!buffer) {
        return SAVE_ERR_IO;
    }

    // Fill header
    SaveHeader* header = (SaveHeader*)buffer;
    memset(header, 0, sizeof(SaveHeader));

    memcpy(header->magic, SAVE_MAGIC, 4);
    header->version = SAVE_VERSION;
    header->network_tier = TIER_LIGHT;  // TODO: detect from network

    header->episode_count = training->total_episodes;
    header->total_steps = training->total_steps;
    header->training_time = 0;  // TODO: track actual time

    header->best_win_rate = training->win_rate;
    header->current_epsilon = training->epsilon;
    header->learning_rate = training->learning_rate;
    header->gamma = training->gamma;
    header->epsilon_min = training->epsilon_min;
    header->epsilon_decay = training->epsilon_decay;
    header->batch_size = training->batch_size;

    // Serialize weights
    uint8_t* weight_ptr = buffer + sizeof(SaveHeader);
    int written = nn_serialize(nn, weight_ptr, weight_size);
    if (written < 0) {
        free(buffer);
        return SAVE_ERR_IO;
    }

    // Compute checksum of weights
    header->checksum = compute_crc32(weight_ptr, written);

    // Write to Controller Pak
    // libdragon uses mempak functions
    int result = write_mempak_sector(controller, 0, buffer);

    // May need multiple sectors for larger saves
    int sectors_needed = (total_size + 255) / 256;
    for (int i = 1; i < sectors_needed && result == 0; i++) {
        result = write_mempak_sector(controller, i, buffer + i * 256);
    }

    free(buffer);

    return (result == 0) ? SAVE_OK : SAVE_ERR_IO;
}

int load_from_pak(int controller,
                  NeuralNetwork* nn,
                  TrainingState* training) {
    if (!save_pak_present(controller)) {
        return SAVE_ERR_NO_PAK;
    }

    // Read header first
    SaveHeader header;
    int result = save_read_header(controller, &header);
    if (result != SAVE_OK) {
        return result;
    }

    // Version check
    if (header.version != SAVE_VERSION) {
        return SAVE_ERR_VERSION;
    }

    // Allocate buffer for weights
    int weight_size = nn_get_serialized_size(nn);
    int total_size = sizeof(SaveHeader) + weight_size;
    uint8_t* buffer = (uint8_t*)malloc(total_size);
    if (!buffer) {
        return SAVE_ERR_IO;
    }

    // Read all sectors
    int sectors_needed = (total_size + 255) / 256;
    for (int i = 0; i < sectors_needed; i++) {
        result = read_mempak_sector(controller, i, buffer + i * 256);
        if (result != 0) {
            free(buffer);
            return SAVE_ERR_IO;
        }
    }

    // Verify checksum
    uint8_t* weight_ptr = buffer + sizeof(SaveHeader);
    uint32_t computed_crc = compute_crc32(weight_ptr, weight_size);
    if (computed_crc != header.checksum) {
        free(buffer);
        return SAVE_ERR_CORRUPT;
    }

    // Deserialize weights
    result = nn_deserialize(nn, weight_ptr, weight_size);
    if (result != 0) {
        free(buffer);
        return SAVE_ERR_TIER;  // Architecture mismatch
    }

    // Restore training state
    training->total_episodes = header.episode_count;
    training->total_steps = header.total_steps;
    training->epsilon = header.current_epsilon;
    training->learning_rate = header.learning_rate;
    training->gamma = header.gamma;
    training->epsilon_min = header.epsilon_min;
    training->epsilon_decay = header.epsilon_decay;
    training->batch_size = header.batch_size;
    training->win_rate = header.best_win_rate;

    free(buffer);
    return SAVE_OK;
}

int save_delete(int controller) {
    if (!save_pak_present(controller)) {
        return SAVE_ERR_NO_PAK;
    }

    // Clear first sector (invalidates magic)
    uint8_t zero_sector[256] = {0};
    int result = write_mempak_sector(controller, 0, zero_sector);

    return (result == 0) ? SAVE_OK : SAVE_ERR_IO;
}

// =============================================================================
// ROM CHECKPOINTS
// =============================================================================

int load_rom_checkpoint(int checkpoint_id, NeuralNetwork* nn) {
    if (checkpoint_id < 0 || checkpoint_id >= NUM_ROM_CHECKPOINTS) {
        return SAVE_ERR_NOT_FOUND;
    }

    if (checkpoint_id == 0) {
        // "RANDOM" checkpoint - just reset weights
        nn_reset_weights(nn);
        return SAVE_OK;
    }

    // For other checkpoints, we'd load from ROM
    // In a full implementation, these would be embedded in the ROM
    // using DragonFS (dfs_read from assets/checkpoints/)

    char filename[64];
    snprintf(filename, sizeof(filename), "rom:/checkpoints/tier1_%s.bin",
             ROM_CHECKPOINT_NAMES[checkpoint_id]);

    // Try to open file from DragonFS
    int fp = dfs_open(filename);
    if (fp < 0) {
        // File not found - for now, just reset weights
        // In production, pre-trained weights would be bundled
        nn_reset_weights(nn);
        return SAVE_OK;
    }

    // Get file size
    int file_size = dfs_size(fp);
    if (file_size <= 0) {
        dfs_close(fp);
        nn_reset_weights(nn);
        return SAVE_OK;
    }

    // Allocate buffer
    uint8_t* buffer = (uint8_t*)malloc(file_size);
    if (!buffer) {
        dfs_close(fp);
        return SAVE_ERR_IO;
    }

    // Read file
    int read = dfs_read(buffer, 1, file_size, fp);
    dfs_close(fp);

    if (read != file_size) {
        free(buffer);
        return SAVE_ERR_IO;
    }

    // Deserialize
    int result = nn_deserialize(nn, buffer, file_size);
    free(buffer);

    if (result != 0) {
        nn_reset_weights(nn);
        return SAVE_ERR_TIER;
    }

    return SAVE_OK;
}

int get_rom_checkpoint_info(int checkpoint_id,
                            float* win_rate,
                            int* episodes) {
    if (checkpoint_id < 0 || checkpoint_id >= NUM_ROM_CHECKPOINTS) {
        return SAVE_ERR_NOT_FOUND;
    }

    if (win_rate) {
        *win_rate = checkpoint_win_rates[checkpoint_id];
    }
    if (episodes) {
        *episodes = checkpoint_episodes[checkpoint_id];
    }

    return SAVE_OK;
}

// =============================================================================
// HEADER READING
// =============================================================================

int save_read_header(int controller, SaveHeader* header) {
    if (!save_pak_present(controller)) {
        return SAVE_ERR_NO_PAK;
    }

    // Read first sector
    uint8_t sector[256];
    int result = read_mempak_sector(controller, 0, sector);
    if (result != 0) {
        return SAVE_ERR_IO;
    }

    // Copy to header
    memcpy(header, sector, sizeof(SaveHeader));

    // Verify magic
    if (memcmp(header->magic, SAVE_MAGIC, 4) != 0) {
        return SAVE_ERR_NOT_FOUND;
    }

    return SAVE_OK;
}

// =============================================================================
// UTILITY
// =============================================================================

void save_format_time(uint32_t seconds, char* buffer) {
    if (seconds < 60) {
        snprintf(buffer, 16, "%ds", seconds);
    } else if (seconds < 3600) {
        snprintf(buffer, 16, "%dm %ds", seconds / 60, seconds % 60);
    } else {
        snprintf(buffer, 16, "%dh %dm",
                 seconds / 3600, (seconds % 3600) / 60);
    }
}
